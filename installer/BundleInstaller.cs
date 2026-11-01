// One-click paired resource-pack + shader installation. Both ZIPs are staged
// and SHA-256 verified before either target is replaced. If a commit fails,
// already-committed files are rolled back from same-folder backups.
using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Net;
using System.Text.RegularExpressions;
using System.Web.Script.Serialization;

namespace HyperRealismInstaller
{
    internal sealed class BundleReleaseInfo
    {
        public string Tag;
        public DateTimeOffset Published;
        public ReleaseInfo Pack;
        public ShaderReleaseInfo Shader;
    }

    internal sealed class BundleInstallPlan
    {
        public string Tag;
        public string ResourcePackTarget;
        public string ShaderTarget;
        public bool ResourcePackChanged;
        public bool ShaderChanged;
        public bool ResourcePackExists;
        public bool ShaderExists;
    }

    internal sealed class BundleCheckResult
    {
        public string Tag;
        public string Pack;
        public string Shader;
        public bool PackNeeds;
        public bool ShaderNeeds;
    }

    internal sealed class BundleEntry
    {
        public string Name;
        public string Target;
        public string Stage;
        public string Backup;
        public string ExpectedHash;
        public string PreviousHash;
        public bool Update;
        public bool Committed;
    }

    internal static class BundleInstaller
    {
        private static string StringValue(Dictionary<string, object> data, string name)
        {
            object value;
            return data.TryGetValue(name, out value) ? value as string : null;
        }

        private static string Checksum(ITransfer transfer, string url, string fileName)
        {
            PackInstaller.ValidateUrl(url);
            string text = transfer.GetText(url).Trim();
            PackInstaller.CheckCancellation(transfer);
            Match match = Regex.Match(text, @"\A([0-9a-fA-F]{64})\s+\*?" + Regex.Escape(fileName) + @"\z", RegexOptions.CultureInvariant);
            if (!match.Success) throw new InvalidDataException("The release checksum for " + fileName + " has an unexpected format.");
            return match.Groups[1].Value;
        }

        // Select one published release containing BOTH complete asset pairs, so
        // a newer pack-only or shader-only release cannot create a mixed pair.
        internal static BundleReleaseInfo Latest(ITransfer transfer)
        {
            ServicePointManager.SecurityProtocol = SecurityProtocolType.Tls12;
            var serializer = new JavaScriptSerializer { MaxJsonLength = 4 * 1024 * 1024 };
            string response = transfer.GetText(PackInstaller.Api);
            PackInstaller.CheckCancellation(transfer);
            var releases = serializer.DeserializeObject(response) as object[];
            if (releases == null) throw new InvalidDataException("Unexpected GitHub release response.");
            var candidates = new List<BundleReleaseInfo>();
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
                var packSums = new List<string>();
                var shaders = new List<string>();
                var shaderSums = new List<string>();
                foreach (object raw in assets)
                {
                    var asset = raw as Dictionary<string, object>;
                    if (asset == null) continue;
                    string name = StringValue(asset, "name");
                    string url = StringValue(asset, "browser_download_url");
                    if (name == PackInstaller.PackName) packs.Add(url);
                    if (name == PackInstaller.PackName + ".sha256") packSums.Add(url);
                    if (name == ShaderPackInstaller.PackName) shaders.Add(url);
                    if (name == ShaderPackInstaller.PackName + ".sha256") shaderSums.Add(url);
                }
                if (packs.Count != 1 || packSums.Count != 1 || shaders.Count != 1 || shaderSums.Count != 1) continue;
                candidates.Add(new BundleReleaseInfo {
                    Tag = tag,
                    Published = published,
                    Pack = new ReleaseInfo { Tag = tag, Published = published, PackUrl = packs[0], ChecksumUrl = packSums[0] },
                    Shader = new ShaderReleaseInfo { Tag = tag, Published = published, PackUrl = shaders[0], ChecksumUrl = shaderSums[0] }
                });
            }
            BundleReleaseInfo latest = candidates.OrderByDescending(r => r.Published).FirstOrDefault();
            if (latest == null) throw new InvalidDataException("No published release contains both the resource pack and shader ZIPs with checksums.");
            PackInstaller.ValidateUrl(latest.Pack.PackUrl);
            PackInstaller.ValidateUrl(latest.Shader.PackUrl);
            latest.Pack.Checksum = Checksum(transfer, latest.Pack.ChecksumUrl, PackInstaller.PackName);
            latest.Shader.Checksum = Checksum(transfer, latest.Shader.ChecksumUrl, ShaderPackInstaller.PackName);
            return latest;
        }

        private static string ExistingHash(string target)
        {
            if (Directory.Exists(target)) throw new InvalidDataException("A folder is blocking the install target: " + target);
            return File.Exists(target) ? PackInstaller.HashFile(target) : null;
        }

        private static void VerifySnapshot(BundleEntry entry)
        {
            if (Directory.Exists(entry.Target)) throw new InvalidDataException("A folder is blocking the install target: " + entry.Target);
            if (entry.PreviousHash == null)
            {
                if (File.Exists(entry.Target)) throw new IOException("The install target appeared after staging: " + entry.Target);
                return;
            }
            if (!File.Exists(entry.Target)) throw new IOException("The existing file disappeared after staging: " + entry.Target);
            // Fail early if another application holds the ZIP open.
            using (new FileStream(entry.Target, FileMode.Open, FileAccess.ReadWrite, FileShare.None)) { }
            if (!String.Equals(PackInstaller.HashFile(entry.Target), entry.PreviousHash, StringComparison.OrdinalIgnoreCase))
                throw new IOException("The existing file changed after staging: " + entry.Target);
        }

        // Internal for failure-injection tests. Backups are intentionally kept
        // until the caller confirms the whole transaction succeeded.
        internal static void CommitEntries(BundleEntry[] entries, ITransfer transfer)
        {
            try
            {
                foreach (BundleEntry entry in entries) VerifySnapshot(entry);
                foreach (BundleEntry entry in entries)
                {
                    PackInstaller.CheckCancellation(transfer);
                    if (!entry.Update) continue;
                    if (entry.PreviousHash != null)
                    {
                        entry.Backup = Path.Combine(Path.GetDirectoryName(entry.Target), ".hyperrealism-backup-" + Guid.NewGuid().ToString("N") + ".tmp");
                        File.Replace(entry.Stage, entry.Target, entry.Backup);
                    }
                    else
                    {
                        File.Move(entry.Stage, entry.Target);
                    }
                    entry.Committed = true;
                }
                foreach (BundleEntry entry in entries)
                {
                    if (!File.Exists(entry.Target) || !String.Equals(PackInstaller.HashFile(entry.Target), entry.ExpectedHash, StringComparison.OrdinalIgnoreCase))
                        throw new InvalidDataException("Installed file failed final verification: " + entry.Target);
                }
            }
            catch
            {
                Exception rollbackError = null;
                for (int i = entries.Length - 1; i >= 0; i--)
                {
                    BundleEntry entry = entries[i];
                    if (!entry.Committed) continue;
                    try
                    {
                        if (entry.PreviousHash != null)
                        {
                            if (entry.Backup != null && File.Exists(entry.Backup))
                            {
                                if (File.Exists(entry.Target)) File.Replace(entry.Backup, entry.Target, null);
                                else File.Move(entry.Backup, entry.Target);
                            }
                            else
                            {
                                rollbackError = new IOException("A rollback backup is missing for " + entry.Target);
                            }
                        }
                        else if (File.Exists(entry.Target))
                        {
                            File.Delete(entry.Target);
                        }
                    }
                    catch (Exception ex) { rollbackError = ex; }
                }
                if (rollbackError != null)
                {
                    var preserved = entries.Where(e => e.Backup != null && File.Exists(e.Backup)).Select(e => e.Backup).ToArray();
                    throw new InvalidDataException("Installation failed and automatic rollback could not finish. Preserved backups: " + String.Join("; ", preserved), rollbackError);
                }
                throw;
            }
        }

        // Compare the installed pair against one matched release, not against
        // separate pack-only/shader-only latest releases.
        internal static BundleCheckResult Check(ITransfer transfer, string resourcepacksFolder)
        {
            string resources = PackInstaller.ValidateFolder(resourcepacksFolder);
            string shaders = ShaderPackInstaller.ShaderpacksFolder(resources);
            BundleReleaseInfo release = Latest(transfer);
            string packTarget = Path.Combine(resources, PackInstaller.PackName);
            string shaderTarget = Path.Combine(shaders, ShaderPackInstaller.PackName);
            string packHash = File.Exists(packTarget) ? PackInstaller.HashFile(packTarget) : null;
            string shaderHash = File.Exists(shaderTarget) ? PackInstaller.HashFile(shaderTarget) : null;
            bool packNeeds = !String.Equals(packHash, release.Pack.Checksum, StringComparison.OrdinalIgnoreCase);
            bool shaderNeeds = !String.Equals(shaderHash, release.Shader.Checksum, StringComparison.OrdinalIgnoreCase);
            return new BundleCheckResult {
                Tag = release.Tag,
                Pack = "Latest matched pack: " + release.Tag + "  |  " + (packHash == null ? "Not installed here" : packNeeds ? "Installed file differs; update available" : "Installed file matches"),
                Shader = "Latest matched shader: " + release.Tag + "  |  " + (shaderHash == null ? "Not installed here" : shaderNeeds ? "Installed file differs; update available" : "Installed file matches"),
                PackNeeds = packNeeds,
                ShaderNeeds = shaderNeeds
            };
        }

        internal static string Install(ITransfer transfer, string resourcepacksFolder, Action<string> status, Func<BundleInstallPlan, bool> confirm)
        {
            string resources = PackInstaller.ValidateFolder(resourcepacksFolder);
            string shaders = ShaderPackInstaller.ShaderpacksFolder(resources);
            Report(status, "Finding a published release with both the resource pack and shader...");
            BundleReleaseInfo release = Latest(transfer);
            string packTarget = Path.Combine(resources, PackInstaller.PackName);
            string shaderTarget = Path.Combine(shaders, ShaderPackInstaller.PackName);
            string packOld = ExistingHash(packTarget);
            string shaderOld = ExistingHash(shaderTarget);
            bool packUpdate = !String.Equals(packOld, release.Pack.Checksum, StringComparison.OrdinalIgnoreCase);
            bool shaderUpdate = !String.Equals(shaderOld, release.Shader.Checksum, StringComparison.OrdinalIgnoreCase);
            if (!packUpdate && !shaderUpdate)
                return "Resource pack and shader are already installed and verified from " + release.Tag + ".";

            var entries = new[] {
                new BundleEntry { Name = "resource pack", Target = packTarget, ExpectedHash = release.Pack.Checksum, PreviousHash = packOld, Update = packUpdate },
                new BundleEntry { Name = "shader", Target = shaderTarget, ExpectedHash = release.Shader.Checksum, PreviousHash = shaderOld, Update = shaderUpdate }
            };
            bool createdShaderFolder = !Directory.Exists(shaders);
            bool success = false;
            try
            {
                if (shaderUpdate) Directory.CreateDirectory(shaders);
                foreach (BundleEntry entry in entries)
                {
                    if (!entry.Update) continue;
                    PackInstaller.CheckCancellation(transfer);
                    Report(status, "Downloading " + entry.Name + "...");
                    entry.Stage = Path.Combine(Path.GetDirectoryName(entry.Target), ".hyperrealism-bundle-" + Guid.NewGuid().ToString("N") + ".tmp");
                    bool isPack = entry.Name == "resource pack";
                    transfer.Download(isPack ? release.Pack.PackUrl : release.Shader.PackUrl, entry.Stage, isPack ? PackInstaller.MaxPackBytes : ShaderPackInstaller.MaxPackBytes);
                    PackInstaller.CheckCancellation(transfer);
                    Report(status, "Verifying " + entry.Name + "...");
                    if (!String.Equals(PackInstaller.HashFile(entry.Stage), entry.ExpectedHash, StringComparison.OrdinalIgnoreCase))
                        throw new InvalidDataException("The " + entry.Name + " SHA-256 did not match. Neither installed file was changed.");
                }
                PackInstaller.CheckCancellation(transfer);
                bool replacing = entries.Any(e => e.Update && e.PreviousHash != null);
                if (replacing)
                {
                    var plan = new BundleInstallPlan {
                        Tag = release.Tag,
                        ResourcePackTarget = packTarget,
                        ShaderTarget = shaderTarget,
                        ResourcePackChanged = packUpdate,
                        ShaderChanged = shaderUpdate,
                        ResourcePackExists = packOld != null,
                        ShaderExists = shaderOld != null
                    };
                    Report(status, "Both downloads are verified. Waiting for replacement confirmation...");
                    if (confirm == null || !confirm(plan)) throw new OperationCanceledException("Installation cancelled before replacing files.");
                }
                PackInstaller.CheckCancellation(transfer);
                Report(status, "Installing verified resource pack and shader...");
                CommitEntries(entries, transfer);
                success = true;
                return "Installed resource pack + shader from " + release.Tag + ". Both ZIPs verified. Enable HyperRealism in Resource Packs and Gilded Dusk in Iris.";
            }
            finally
            {
                foreach (BundleEntry entry in entries)
                {
                    try { if (entry.Stage != null && File.Exists(entry.Stage)) File.Delete(entry.Stage); } catch { }
                    if (success)
                    {
                        try { if (entry.Backup != null && File.Exists(entry.Backup)) File.Delete(entry.Backup); } catch { }
                    }
                }
                if (!success && createdShaderFolder)
                {
                    try { if (Directory.Exists(shaders) && !Directory.EnumerateFiles(shaders).Any() && !Directory.EnumerateDirectories(shaders).Any()) Directory.Delete(shaders); } catch { }
                }
            }
        }

        private static void Report(Action<string> status, string text)
        {
            if (status != null) status(text);
        }
    }
}
