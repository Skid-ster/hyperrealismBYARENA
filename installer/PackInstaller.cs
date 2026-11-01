// Standalone Windows GUI installer backend. No third-party dependencies.
// SHA-256 verifies integrity against a same-release checksum, not authenticity.
using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Net;
using System.Security.Cryptography;
using System.Text.RegularExpressions;
using System.Web.Script.Serialization;

namespace HyperRealismInstaller
{
    internal interface ITransfer
    {
        string GetText(string url);
        bool IsCancellationRequested { get; }
        void Download(string url, string destination, long maxBytes);
    }

    internal sealed class WebTransfer : ITransfer
    {
        private readonly Action<long, long> progress;
        private readonly Func<bool> cancellationRequested;

        internal WebTransfer(Action<long, long> progress = null, Func<bool> cancellationRequested = null)
        {
            this.progress = progress;
            this.cancellationRequested = cancellationRequested;
        }

        public bool IsCancellationRequested
        {
            get { return cancellationRequested != null && cancellationRequested(); }
        }

        private sealed class TimedWebClient : WebClient
        {
            protected override WebRequest GetWebRequest(Uri address)
            {
                WebRequest request = base.GetWebRequest(address);
                request.Timeout = 30000;
                var http = request as HttpWebRequest;
                if (http != null) http.ReadWriteTimeout = 30000;
                return request;
            }
        }

        private static WebClient CreateClient()
        {
            var client = new TimedWebClient();
            client.Headers[HttpRequestHeader.UserAgent] = "HyperRealism128-GUI-installer";
            return client;
        }
        public string GetText(string url)
        {
            if (IsCancellationRequested) throw new OperationCanceledException("Update check cancelled.");
            using (var client = CreateClient())
            {
                string text = client.DownloadString(url);
                if (IsCancellationRequested) throw new OperationCanceledException("Update check cancelled.");
                return text;
            }
        }
        public void Download(string url, string destination, long maxBytes)
        {
            if (maxBytes <= 0) throw new ArgumentOutOfRangeException("maxBytes");
            if (IsCancellationRequested) throw new OperationCanceledException("Download cancelled.");
            using (var client = CreateClient())
            using (var source = client.OpenRead(url))
            {
                long total;
                string length = client.ResponseHeaders == null ? null : client.ResponseHeaders[HttpResponseHeader.ContentLength];
                if (!Int64.TryParse(length, NumberStyles.None, CultureInfo.InvariantCulture, out total) || total < 0) total = -1;
                if (total > maxBytes) throw new InvalidDataException("The download exceeds the installer's size limit.");
                using (var output = new FileStream(destination, FileMode.CreateNew, FileAccess.Write, FileShare.None))
                {
                    byte[] buffer = new byte[64 * 1024];
                    long received = 0;
                    DateTime nextProgress = DateTime.UtcNow.AddMilliseconds(200);
                    int count;
                    while ((count = source.Read(buffer, 0, buffer.Length)) > 0)
                    {
                        if (IsCancellationRequested) throw new OperationCanceledException("Download cancelled.");
                        if (received > maxBytes - count) throw new InvalidDataException("The download exceeded the installer's size limit.");
                        output.Write(buffer, 0, count);
                        received += count;
                        DateTime now = DateTime.UtcNow;
                        if (progress != null && now >= nextProgress)
                        {
                            try { progress(received, total); } catch { }
                            nextProgress = now.AddMilliseconds(200);
                        }
                    }
                    if (IsCancellationRequested) throw new OperationCanceledException("Download cancelled.");
                    if (total >= 0 && received != total) throw new EndOfStreamException("The download ended before all bytes arrived.");
                    output.Flush(true);
                    if (progress != null) { try { progress(received, total); } catch { } }
                }
            }
        }
    }

    internal sealed class ReleaseInfo
    {
        public string Tag;
        public string PackUrl;
        public string ChecksumUrl;
        public string Checksum;
        public DateTimeOffset Published;
    }

    internal static class PackInstaller
    {
        internal const string PackName = "HyperRealism128-MC26.3-EXPERIMENTAL.zip";
        internal const long MaxPackBytes = 1024L * 1024L * 1024L;
        internal const string Api = "https://api.github.com/repos/Skid-ster/hyperrealismBYARENA/releases?per_page=100";
        private const string DownloadPrefix = "/Skid-ster/hyperrealismBYARENA/releases/download/";

        // Name matching is a useful safeguard, not proof that Minecraft uses it.
        internal static string ValidateFolder(string value)
        {
            if (String.IsNullOrWhiteSpace(value)) throw new ArgumentException("Choose your resourcepacks folder.");
            string path = value.Trim().Trim('"');
            if (!Regex.IsMatch(path, @"\A[A-Za-z]:[\\/]", RegexOptions.CultureInvariant) && !path.StartsWith(@"\\", StringComparison.Ordinal))
                throw new ArgumentException("Choose an absolute path.");
            path = Path.GetFullPath(path).TrimEnd(Path.DirectorySeparatorChar, Path.AltDirectorySeparatorChar);
            if (!String.Equals(Path.GetFileName(path), "resourcepacks", StringComparison.OrdinalIgnoreCase))
                throw new ArgumentException("The folder must be named resourcepacks.");
            if (!Directory.Exists(path)) throw new DirectoryNotFoundException("This resourcepacks folder does not exist. Browse to an existing folder.");
            return path;
        }

        private static string StringValue(Dictionary<string, object> data, string name)
        {
            object value;
            return data.TryGetValue(name, out value) ? value as string : null;
        }

        internal static void ValidateUrl(string url)
        {
            Uri uri;
            if (!Uri.TryCreate(url, UriKind.Absolute, out uri) || uri.Scheme != Uri.UriSchemeHttps ||
                !String.Equals(uri.Host, "github.com", StringComparison.OrdinalIgnoreCase) ||
                !uri.IsDefaultPort || !String.IsNullOrEmpty(uri.UserInfo) ||
                !uri.AbsolutePath.StartsWith(DownloadPrefix, StringComparison.Ordinal) ||
                !String.IsNullOrEmpty(uri.Query) || !String.IsNullOrEmpty(uri.Fragment))
                throw new InvalidDataException("Refusing a download outside the project's GitHub releases.");
        }

        internal static void CheckCancellation(ITransfer transfer)
        {
            if (transfer.IsCancellationRequested) throw new OperationCanceledException("Operation cancelled.");
        }

        internal static ReleaseInfo Latest(ITransfer transfer)
        {
            ServicePointManager.SecurityProtocol = SecurityProtocolType.Tls12;
            var serializer = new JavaScriptSerializer();
            serializer.MaxJsonLength = 4 * 1024 * 1024;
            string response = transfer.GetText(Api);
            CheckCancellation(transfer);
            var releases = serializer.DeserializeObject(response) as object[];
            if (releases == null) throw new InvalidDataException("Unexpected GitHub release response.");
            var candidates = new List<ReleaseInfo>();
            foreach (object item in releases)
            {
                var release = item as Dictionary<string, object>;
                if (release == null) continue;
                object draft;
                if (!release.TryGetValue("draft", out draft) || !(draft is bool) || (bool)draft) continue;
                DateTimeOffset published;
                string tag = StringValue(release, "tag_name");
                if (String.IsNullOrWhiteSpace(tag) || !DateTimeOffset.TryParse(StringValue(release, "published_at"), CultureInfo.InvariantCulture, DateTimeStyles.AssumeUniversal, out published)) continue;
                object rawAssets;
                if (!release.TryGetValue("assets", out rawAssets)) continue;
                var assets = rawAssets as object[];
                if (assets == null) continue;
                var packs = new List<string>();
                var checksums = new List<string>();
                foreach (object raw in assets)
                {
                    var asset = raw as Dictionary<string, object>;
                    if (asset == null) continue;
                    string name = StringValue(asset, "name");
                    if (name == PackName) packs.Add(StringValue(asset, "browser_download_url"));
                    if (name == PackName + ".sha256") checksums.Add(StringValue(asset, "browser_download_url"));
                }
                if (packs.Count == 1 && checksums.Count == 1)
                    candidates.Add(new ReleaseInfo { Tag = tag, PackUrl = packs[0], ChecksumUrl = checksums[0], Published = published });
            }
            ReleaseInfo latest = candidates.OrderByDescending(r => r.Published).FirstOrDefault();
            if (latest == null) throw new InvalidDataException("No published preview with a ZIP and checksum was found.");
            ValidateUrl(latest.PackUrl);
            ValidateUrl(latest.ChecksumUrl);
            string text = transfer.GetText(latest.ChecksumUrl).Trim();
            CheckCancellation(transfer);
            Match match = Regex.Match(text, @"\A([0-9a-fA-F]{64})\s+\*?HyperRealism128-MC26\.3-EXPERIMENTAL\.zip\z", RegexOptions.CultureInvariant);
            if (!match.Success) throw new InvalidDataException("The release checksum has an unexpected format.");
            latest.Checksum = match.Groups[1].Value;
            return latest;
        }

        internal static string HashFile(string path)
        {
            using (var sha = SHA256.Create())
            using (var stream = File.OpenRead(path))
                return BitConverter.ToString(sha.ComputeHash(stream)).Replace("-", "");
        }

        internal static string Check(ITransfer transfer, string path)
        {
            path = ValidateFolder(path);
            ReleaseInfo release = Latest(transfer);
            string target = Path.Combine(path, PackName);
            if (!File.Exists(target)) return "Latest: " + release.Tag + "  |  Not installed here";
            if (String.Equals(HashFile(target), release.Checksum, StringComparison.OrdinalIgnoreCase))
                return "Latest: " + release.Tag + "  |  Installed file matches";
            return "Latest: " + release.Tag + "  |  Installed file differs; update available";
        }

        internal static string Install(ITransfer transfer, string path)
        {
            path = ValidateFolder(path);
            ReleaseInfo release = Latest(transfer);
            CheckCancellation(transfer);
            string temp = Path.Combine(Path.GetTempPath(), "hyperrealism-" + Guid.NewGuid().ToString("N"));
            string staged = Path.Combine(path, ".hyperrealism-" + Guid.NewGuid().ToString("N") + ".tmp");
            string target = Path.Combine(path, PackName);
            try
            {
                Directory.CreateDirectory(temp);
                string downloaded = Path.Combine(temp, PackName);
                transfer.Download(release.PackUrl, downloaded, MaxPackBytes);
                CheckCancellation(transfer);
                if (!String.Equals(HashFile(downloaded), release.Checksum, StringComparison.OrdinalIgnoreCase))
                    throw new InvalidDataException("SHA-256 mismatch. Your existing pack was not changed.");
                // Copy to the same volume, then atomically replace/move. If replacing is
                // unsupported or denied, fail rather than damage the existing pack.
                ValidateFolder(path);
                CheckCancellation(transfer);
                File.Copy(downloaded, staged, false);
                if (!String.Equals(HashFile(staged), release.Checksum, StringComparison.OrdinalIgnoreCase))
                    throw new InvalidDataException("Staged file failed verification. Your existing pack was not changed.");
                CheckCancellation(transfer);
                if (File.Exists(target)) File.Replace(staged, target, null);
                else File.Move(staged, target);
                return "Installed " + release.Tag + ". Enable the pack in Minecraft.";
            }
            finally
            {
                try { if (File.Exists(staged)) File.Delete(staged); } catch { }
                try { if (Directory.Exists(temp)) Directory.Delete(temp, true); } catch { }
            }
        }
    }
}
