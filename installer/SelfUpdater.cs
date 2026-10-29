// Self-update logic: a verified new EXE is staged beside the running one.
// A temporary copy of the current EXE runs as a helper (started through cmd),
// waits for the application to exit, and uses File.Replace before relaunching.
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Net;
using System.Text;
using System.Text.RegularExpressions;
using System.Threading;
using System.Web.Script.Serialization;
using System.Windows.Forms;

namespace HyperRealismInstaller
{
    internal sealed class InstallerRelease
    {
        public string Tag;
        public Version Version;
        public string ExeUrl;
        public string ChecksumUrl;
    }

    internal static class SelfUpdater
    {
        internal const string CurrentTag = "v0.1.0-installer.6";
        internal const string AssetName = "HyperRealism128-Installer.exe";

        internal static Version ParseVersion(string tag)
        {
            if (tag == null) return null;
            Match match = Regex.Match(tag, @"\Av(\d+)\.(\d+)\.(\d+)-installer\.(\d+)\z", RegexOptions.CultureInvariant);
            Version parsed;
            return match.Success && Version.TryParse(match.Groups[1].Value + "." + match.Groups[2].Value + "." + match.Groups[3].Value + "." + match.Groups[4].Value, out parsed) ? parsed : null;
        }

        private static string Field(Dictionary<string, object> data, string name)
        {
            object value;
            return data.TryGetValue(name, out value) ? value as string : null;
        }

        // Select only a strictly newer, published installer release with both
        // required assets. This does not treat pack releases as app updates.
        internal static InstallerRelease FindUpdate(ITransfer transfer, string currentTag)
        {
            ServicePointManager.SecurityProtocol = SecurityProtocolType.Tls12;
            Version current = ParseVersion(currentTag);
            if (current == null) throw new InvalidDataException("Invalid current installer version.");
            var serializer = new JavaScriptSerializer { MaxJsonLength = 4 * 1024 * 1024 };
            var releases = serializer.DeserializeObject(transfer.GetText(PackInstaller.Api)) as object[];
            if (releases == null) throw new InvalidDataException("Unexpected GitHub release response.");
            var candidates = new List<InstallerRelease>();
            foreach (object item in releases)
            {
                var release = item as Dictionary<string, object>;
                if (release == null) continue;
                object draft;
                if (!release.TryGetValue("draft", out draft) || !(draft is bool) || (bool)draft) continue;
                string tag = Field(release, "tag_name");
                Version version = ParseVersion(tag);
                if (version == null || version.CompareTo(current) <= 0) continue;
                object rawAssets;
                if (!release.TryGetValue("assets", out rawAssets)) continue;
                var assets = rawAssets as object[];
                if (assets == null) continue;
                var exes = new List<string>();
                var sums = new List<string>();
                foreach (object raw in assets)
                {
                    var asset = raw as Dictionary<string, object>;
                    if (asset == null) continue;
                    string name = Field(asset, "name");
                    if (name == AssetName) exes.Add(Field(asset, "browser_download_url"));
                    if (name == AssetName + ".sha256") sums.Add(Field(asset, "browser_download_url"));
                }
                if (exes.Count == 1 && sums.Count == 1)
                    candidates.Add(new InstallerRelease { Tag = tag, Version = version, ExeUrl = exes[0], ChecksumUrl = sums[0] });
            }
            InstallerRelease latest = candidates.OrderByDescending(r => r.Version).FirstOrDefault();
            if (latest != null)
            {
                PackInstaller.ValidateUrl(latest.ExeUrl);
                PackInstaller.ValidateUrl(latest.ChecksumUrl);
            }
            return latest;
        }

        // Download only after user consent. The staged file lives alongside the
        // target so the eventual replace does not cross filesystems.
        internal static string DownloadUpdate(ITransfer transfer, InstallerRelease release, string executable, out string checksum)
        {
            if (release == null || release.Version.CompareTo(ParseVersion(CurrentTag)) <= 0)
                throw new InvalidDataException("The installer update is not newer than this version.");
            PackInstaller.ValidateUrl(release.ExeUrl);
            PackInstaller.ValidateUrl(release.ChecksumUrl);
            string text = transfer.GetText(release.ChecksumUrl).Trim();
            Match match = Regex.Match(text, @"\A([0-9a-fA-F]{64})\s+\*?HyperRealism128-Installer\.exe\z", RegexOptions.CultureInvariant);
            if (!match.Success) throw new InvalidDataException("The installer checksum has an unexpected format.");
            checksum = match.Groups[1].Value;
            string target = Path.GetFullPath(executable);
            string dir = Path.GetDirectoryName(target);
            if (!File.Exists(target) || !target.EndsWith(".exe", StringComparison.OrdinalIgnoreCase))
                throw new InvalidDataException("Could not locate the running installer executable.");
            string stage = Path.Combine(dir, ".hyperrealism-update-" + Guid.NewGuid().ToString("N") + ".tmp");
            try
            {
                transfer.Download(release.ExeUrl, stage);
                if (!String.Equals(PackInstaller.HashFile(stage), checksum, StringComparison.OrdinalIgnoreCase))
                    throw new InvalidDataException("Installer SHA-256 mismatch; the current installer was not changed.");
                return stage;
            }
            catch { try { if (File.Exists(stage)) File.Delete(stage); } catch { } throw; }
        }

        internal static Process LaunchHelper(string helperExecutable, string stage, string target, string checksum)
        {
            target = Path.GetFullPath(target);
            stage = Path.GetFullPath(stage);
            if (!String.Equals(Path.GetDirectoryName(stage), Path.GetDirectoryName(target), StringComparison.OrdinalIgnoreCase) ||
                !Path.GetFileName(stage).StartsWith(".hyperrealism-update-", StringComparison.Ordinal) ||
                !stage.EndsWith(".tmp", StringComparison.OrdinalIgnoreCase))
                throw new InvalidDataException("Update must be staged beside the installer.");
            if (!File.Exists(target) || !String.Equals(PackInstaller.HashFile(stage), checksum, StringComparison.OrdinalIgnoreCase))
                throw new InvalidDataException("The staged installer failed verification.");
            string prefix = Path.Combine(Path.GetTempPath(), "hyperrealism-helper-" + Guid.NewGuid().ToString("N"));
            string helper = prefix + ".exe";
            string script = prefix + ".cmd";
            try
            {
                File.Copy(helperExecutable, helper, false);
                // Paths are passed only through environment variables, not
                // interpolated into batch source. Delayed expansion is disabled.
                File.WriteAllText(script, "@echo off\r\nsetlocal DisableDelayedExpansion\r\nstart \"\" /wait \"%HR_UPDATE_HELPER%\" --apply-update\r\ndel /q \"%HR_UPDATE_HELPER%\" >nul 2>&1\r\ndel /q \"%HR_UPDATE_SCRIPT%\" >nul 2>&1\r\n", Encoding.ASCII);
                var start = new ProcessStartInfo(Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.System), "cmd.exe"));
                start.UseShellExecute = false;
                start.CreateNoWindow = true;
                start.Arguments = "/D /Q /C \"\"%HR_UPDATE_SCRIPT%\"\"";
                start.EnvironmentVariables["HR_UPDATE_HELPER"] = helper;
                start.EnvironmentVariables["HR_UPDATE_SCRIPT"] = script;
                start.EnvironmentVariables["HR_UPDATE_STAGE"] = stage;
                start.EnvironmentVariables["HR_UPDATE_TARGET"] = target;
                start.EnvironmentVariables["HR_UPDATE_HASH"] = checksum;
                Process process = Process.Start(start);
                if (process == null) throw new IOException("Could not start the updater helper.");
                return process;
            }
            catch
            {
                try { if (File.Exists(helper)) File.Delete(helper); } catch { }
                try { if (File.Exists(script)) File.Delete(script); } catch { }
                throw;
            }
        }

        internal static int ApplyUpdateFromEnvironment()
        {
            string stage = Environment.GetEnvironmentVariable("HR_UPDATE_STAGE");
            string target = Environment.GetEnvironmentVariable("HR_UPDATE_TARGET");
            string hash = Environment.GetEnvironmentVariable("HR_UPDATE_HASH");
            string reason = null;
            bool replaced = false;
            bool safeStage = false;
            try
            {
                if (String.IsNullOrEmpty(stage) || String.IsNullOrEmpty(target) || String.IsNullOrEmpty(hash)) throw new InvalidDataException("Updater is missing required paths.");
                stage = Path.GetFullPath(stage); target = Path.GetFullPath(target);
                safeStage = String.Equals(Path.GetDirectoryName(stage), Path.GetDirectoryName(target), StringComparison.OrdinalIgnoreCase) &&
                    Path.GetFileName(stage).StartsWith(".hyperrealism-update-", StringComparison.Ordinal) && stage.EndsWith(".tmp", StringComparison.OrdinalIgnoreCase);
                if (!safeStage || !target.EndsWith(".exe", StringComparison.OrdinalIgnoreCase) || !File.Exists(target) ||
                    !String.Equals(PackInstaller.HashFile(stage), hash, StringComparison.OrdinalIgnoreCase))
                    throw new InvalidDataException("Staged update verification failed.");
                Exception last = null;
                for (int attempt = 0; attempt < 60; attempt++)
                {
                    // Recheck after every wait: another process may have modified
                    // the staged file while the old executable was locked.
                    if (!String.Equals(PackInstaller.HashFile(stage), hash, StringComparison.OrdinalIgnoreCase))
                        throw new InvalidDataException("Staged installer changed while waiting; the current installer was not replaced.");
                    try { File.Replace(stage, target, null); replaced = true; break; }
                    catch (IOException ex) { last = ex; }
                    catch (UnauthorizedAccessException ex) { last = ex; }
                    Thread.Sleep(500);
                }
                if (!replaced) throw new IOException("Could not replace the installer. Close all other copies and check folder permissions.", last);
            }
            catch (Exception ex) { reason = ex.Message; }
            finally { try { if (safeStage && File.Exists(stage)) File.Delete(stage); } catch { } }
            Exception launchError = null;
            for (int attempt = 0; attempt < 20; attempt++)
            {
                try
                {
                    var start = new ProcessStartInfo(target, replaced ? "--updated" : "--update-failed");
                    start.UseShellExecute = true;
                    start.WorkingDirectory = Path.GetDirectoryName(target);
                    if (Process.Start(start) == null) throw new IOException("Windows did not start the installer.");
                    return replaced ? 0 : 1;
                }
                catch (Exception ex) { launchError = ex; Thread.Sleep(250); }
            }
            MessageBox.Show("The installer could not reopen automatically. " + (reason ?? "") + "\r\n" + (launchError == null ? "" : launchError.Message) + "\r\nPlease open the installer again manually.", "HyperRealism Installer", MessageBoxButtons.OK, MessageBoxIcon.Warning);
            return 1;
        }
    }
}
