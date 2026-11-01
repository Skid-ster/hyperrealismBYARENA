// Download and install the separate Iris shader ZIP from the latest preview
// release that includes it. SHA-256 detects corruption; it is not a signature.
using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;
using System.Web.Script.Serialization;

namespace HyperRealismInstaller
{
    internal sealed class ShaderReleaseInfo
    {
        public string Tag;
        public string PackUrl;
        public string ChecksumUrl;
        public string Checksum;
        public DateTimeOffset Published;
    }

    internal static class ShaderPackInstaller
    {
        internal const string PackName = "HyperRealism-Gilded-Dusk-MC-26.3.zip";
        internal const long MaxPackBytes = 100L * 1024L * 1024L;
        private const string DownloadPrefix = "/Skid-ster/hyperrealismBYARENA/releases/download/";

        // The selected resourcepacks directory identifies the instance root;
        // shaderpacks is its standard sibling directory.
        internal static string ShaderpacksFolder(string resourcepacksFolder)
        {
            string resources = PackInstaller.ValidateFolder(resourcepacksFolder);
            DirectoryInfo instanceRoot = Directory.GetParent(resources);
            if (instanceRoot == null) throw new InvalidDataException("Could not determine the Minecraft instance folder.");
            return Path.Combine(instanceRoot.FullName, "shaderpacks");
        }

        internal static ShaderReleaseInfo Latest(ITransfer transfer)
        {
            var serializer = new JavaScriptSerializer { MaxJsonLength = 4 * 1024 * 1024 };
            string response = transfer.GetText(PackInstaller.Api);
            PackInstaller.CheckCancellation(transfer);
            var releases = serializer.DeserializeObject(response) as object[];
            if (releases == null) throw new InvalidDataException("Unexpected GitHub release response.");
            var candidates = new List<ShaderReleaseInfo>();
            foreach (object item in releases)
            {
                var release = item as Dictionary<string, object>;
                if (release == null) continue;
                object draft;
                if (!release.TryGetValue("draft", out draft) || !(draft is bool) || (bool)draft) continue;
                string tag = StringValue(release, "tag_name");
                DateTimeOffset published;
                if (String.IsNullOrWhiteSpace(tag) || !DateTimeOffset.TryParse(StringValue(release, "published_at"), CultureInfo.InvariantCulture, DateTimeStyles.AssumeUniversal, out published)) continue;
                object rawAssets;
                if (!release.TryGetValue("assets", out rawAssets)) continue;
                var assets = rawAssets as object[];
                if (assets == null) continue;
                var packs = new List<string>();
                var sums = new List<string>();
                foreach (object raw in assets)
                {
                    var asset = raw as Dictionary<string, object>;
                    if (asset == null) continue;
                    string name = StringValue(asset, "name");
                    if (name == PackName) packs.Add(StringValue(asset, "browser_download_url"));
                    if (name == PackName + ".sha256") sums.Add(StringValue(asset, "browser_download_url"));
                }
                if (packs.Count == 1 && sums.Count == 1)
                    candidates.Add(new ShaderReleaseInfo { Tag = tag, PackUrl = packs[0], ChecksumUrl = sums[0], Published = published });
            }
            ShaderReleaseInfo latest = candidates.OrderByDescending(r => r.Published).FirstOrDefault();
            if (latest == null) throw new InvalidDataException("No published preview with the Iris shader ZIP and checksum was found yet.");
            PackInstaller.ValidateUrl(latest.PackUrl);
            PackInstaller.ValidateUrl(latest.ChecksumUrl);
            string text = transfer.GetText(latest.ChecksumUrl).Trim();
            PackInstaller.CheckCancellation(transfer);
            Match match = Regex.Match(text, @"\A([0-9a-fA-F]{64})\s+\*?HyperRealism-Gilded-Dusk-MC-26\.3\.zip\z", RegexOptions.CultureInvariant);
            if (!match.Success) throw new InvalidDataException("The shader checksum has an unexpected format.");
            latest.Checksum = match.Groups[1].Value;
            return latest;
        }

        private static string StringValue(Dictionary<string, object> data, string name)
        {
            object value;
            return data.TryGetValue(name, out value) ? value as string : null;
        }

        internal static string Check(ITransfer transfer, string resourcepacksFolder)
        {
            ShaderReleaseInfo release = Latest(transfer);
            string path = ShaderpacksFolder(resourcepacksFolder);
            string target = Path.Combine(path, PackName);
            if (!File.Exists(target)) return "Latest shader: " + release.Tag + "  |  Not installed here";
            if (String.Equals(PackInstaller.HashFile(target), release.Checksum, StringComparison.OrdinalIgnoreCase))
                return "Latest shader: " + release.Tag + "  |  Installed file matches";
            return "Latest shader: " + release.Tag + "  |  Installed file differs; update available";
        }

        internal static string Install(ITransfer transfer, string resourcepacksFolder)
        {
            ShaderReleaseInfo release = Latest(transfer);
            PackInstaller.CheckCancellation(transfer);
            string path = ShaderpacksFolder(resourcepacksFolder);
            string temp = Path.Combine(Path.GetTempPath(), "hyperrealism-shader-" + Guid.NewGuid().ToString("N"));
            string staged = Path.Combine(path, ".hyperrealism-shader-" + Guid.NewGuid().ToString("N") + ".tmp");
            string target = Path.Combine(path, PackName);
            try
            {
                Directory.CreateDirectory(path);
                Directory.CreateDirectory(temp);
                string downloaded = Path.Combine(temp, PackName);
                transfer.Download(release.PackUrl, downloaded, MaxPackBytes);
                PackInstaller.CheckCancellation(transfer);
                if (!String.Equals(PackInstaller.HashFile(downloaded), release.Checksum, StringComparison.OrdinalIgnoreCase))
                    throw new InvalidDataException("Shader ZIP SHA-256 mismatch. Your existing shader was not changed.");
                Directory.CreateDirectory(path);
                PackInstaller.CheckCancellation(transfer);
                File.Copy(downloaded, staged, false);
                if (!String.Equals(PackInstaller.HashFile(staged), release.Checksum, StringComparison.OrdinalIgnoreCase))
                    throw new InvalidDataException("Staged shader failed verification. Your existing shader was not changed.");
                PackInstaller.CheckCancellation(transfer);
                if (File.Exists(target)) File.Replace(staged, target, null);
                else File.Move(staged, target);
                return "Installed the Gilded Dusk shader from " + release.Tag + " in shaderpacks. Enable it in Iris; also enable the HyperRealism resource pack.";
            }
            finally
            {
                try { if (File.Exists(staged)) File.Delete(staged); } catch { }
                try { if (Directory.Exists(temp)) Directory.Delete(temp, true); } catch { }
            }
        }
    }
}
