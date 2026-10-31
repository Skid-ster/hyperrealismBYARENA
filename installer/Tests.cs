using System;
using System.IO;
using System.Text;
using System.Web.Script.Serialization;
using System.Collections.Generic;
using HyperRealismInstaller;

internal sealed class FakeTransfer : ITransfer
{
    public string Json;
    public string Checksum;
    public byte[] Payload;
    public bool Corrupt;
    public bool Cancel;
    public long LastMaxBytes;
    public bool IsCancellationRequested { get { return Cancel; } }
    public string GetText(string url) { return url == PackInstaller.Api ? Json : Checksum; }
    public void Download(string url, string destination, long maxBytes)
    {
        LastMaxBytes = maxBytes;
        byte[] bytes = Corrupt ? Encoding.UTF8.GetBytes("corrupt") : Payload;
        if (bytes.LongLength > maxBytes) throw new InvalidDataException("Test download exceeds its size limit.");
        File.WriteAllBytes(destination, bytes);
    }
}

internal static class InstallerTests
{
    private static void Assert(bool condition, string message) { if (!condition) throw new Exception(message); }
    private static void Throws(Action action, string message)
    {
        bool caught = false;
        try { action(); } catch { caught = true; }
        Assert(caught, message);
    }
    private static object Release(string tag, string date, bool draft, string host)
    {
        string prefix = host + "/Skid-ster/hyperrealismBYARENA/releases/download/" + tag + "/";
        return new { draft = draft, tag_name = tag, published_at = date, assets = new object[] {
            new { name = PackInstaller.PackName, browser_download_url = prefix + PackInstaller.PackName },
            new { name = PackInstaller.PackName + ".sha256", browser_download_url = prefix + PackInstaller.PackName + ".sha256" }
        } };
    }
    // This runs only on CI push, against the real public GitHub API and assets,
    // in a temporary folder. It never touches a Minecraft installation.
    private static int LiveTest()
    {
        string root = Path.Combine(Path.GetTempPath(), "hr-gui-live-" + Guid.NewGuid().ToString("N"));
        try
        {
            string folder = Path.Combine(root, "resourcepacks");
            Directory.CreateDirectory(folder);
            var web = new WebTransfer();
            InstallerRelease app = SelfUpdater.FindUpdate(web, "v0.1.0-installer.0");
            Assert(app != null, "Live installer release lookup failed.");
            string sumText = web.GetText(app.ChecksumUrl).Trim();
            var sumMatch = System.Text.RegularExpressions.Regex.Match(sumText, @"\A([0-9a-fA-F]{64})\s+\*?HyperRealism128-Installer\.exe\z");
            Assert(sumMatch.Success, "Live installer checksum format was invalid.");
            string downloadedApp = Path.Combine(root, "downloaded-installer.exe");
            web.Download(app.ExeUrl, downloadedApp, SelfUpdater.MaxInstallerBytes);
            Assert(String.Equals(PackInstaller.HashFile(downloadedApp), sumMatch.Groups[1].Value, StringComparison.OrdinalIgnoreCase), "Live installer asset hash mismatch.");
            string result = PackInstaller.Install(web, folder);
            Assert(result.Contains("Installed"), "Live install did not complete.");
            Assert(PackInstaller.Check(web, folder).Contains("Installed file matches"), "Live installed file did not match the published checksum.");
            Console.WriteLine("Live GitHub download and update check passed: " + result);
            return 0;
        }
        finally { if (Directory.Exists(root)) Directory.Delete(root, true); }
    }
    private static object AppRelease(string tag, bool draft, string host)
    {
        string prefix = host + "/Skid-ster/hyperrealismBYARENA/releases/download/" + tag + "/";
        return new { draft = draft, tag_name = tag, assets = new object[] {
            new { name = SelfUpdater.AssetName, browser_download_url = prefix + SelfUpdater.AssetName },
            new { name = SelfUpdater.AssetName + ".sha256", browser_download_url = prefix + SelfUpdater.AssetName + ".sha256" }
        } };
    }

    private static bool WaitForMarker(string marker, string expected)
    {
        for (int i = 0; i < 100; i++)
        {
            try { if (File.Exists(marker) && File.ReadAllText(marker) == expected) return true; }
            catch (IOException) { }
            System.Threading.Thread.Sleep(100);
        }
        return false;
    }

    private static void WaitForRelease(string file)
    {
        for (int i = 0; i < 100; i++)
        {
            try { using (var handle = new FileStream(file, FileMode.Open, FileAccess.ReadWrite, FileShare.None)) return; }
            catch (IOException) { System.Threading.Thread.Sleep(100); }
        }
        throw new Exception("Restarted executable stayed locked.");
    }

    private static void TestSelfUpdate(string root, string gui)
    {
        var fake = new FakeTransfer { Payload = Encoding.UTF8.GetBytes("verified installer bytes") };
        var json = new JavaScriptSerializer();
        fake.Json = json.Serialize(new object[] {
            AppRelease("v0.1.0-installer.3", false, "https://github.com"),
            AppRelease("v0.1.0-installer.10", false, "https://github.com"),
            AppRelease("v0.1.0-installer.11", false, "https://github.com"),
            AppRelease("v0.1.0-installer.12", false, "https://github.com"),
            AppRelease("v0.1.0-installer.13", true, "https://github.com"),
            Release("v9.0.0-preview.1", "2028-01-01T00:00:00Z", false, "https://github.com")
        });
        InstallerRelease update = SelfUpdater.FindUpdate(fake, SelfUpdater.CurrentTag);
        Assert(update == null, "Current installer falsely reported outdated.");
        update = SelfUpdater.FindUpdate(fake, "v0.1.0-installer.9");
        Assert(update != null && update.Tag == "v0.1.0-installer.12", "Installer version ordering or draft filtering failed.");
        update = SelfUpdater.FindUpdate(fake, "v0.1.0-installer.10");
        Assert(update != null && update.Tag == "v0.1.0-installer.12", "Installer 10 should update directly to installer 12.");
        fake.Json = json.Serialize(new object[] { AppRelease("v0.1.0-installer.13", false, "https://github.com") });
        update = SelfUpdater.FindUpdate(fake, SelfUpdater.CurrentTag);
        Assert(update != null && update.Tag == "v0.1.0-installer.13", "Newer update fixture not found.");
        string dir = Path.Combine(root, "self update & 50%! test"); Directory.CreateDirectory(dir);
        string target = Path.Combine(dir, "The Installer.exe"); File.WriteAllText(target, "old executable");
        string payload = Path.Combine(root, "app-payload"); File.WriteAllBytes(payload, fake.Payload);
        fake.Checksum = PackInstaller.HashFile(payload) + "  " + SelfUpdater.AssetName;
        string sum;
        fake.Corrupt = true;
        Throws(() => { string ignored; SelfUpdater.DownloadUpdate(fake, update, target, out ignored); }, "Corrupt app update accepted.");
        Assert(File.ReadAllText(target) == "old executable", "Failed app download damaged original.");
        Assert(Directory.GetFiles(dir, ".hyperrealism-update-*.tmp").Length == 0, "Failed app update left a stage.");
        fake.Corrupt = false;
        string staged = SelfUpdater.DownloadUpdate(fake, update, target, out sum);
        Assert(PackInstaller.HashFile(staged) == sum, "App update not staged correctly.");
        Assert(fake.LastMaxBytes == SelfUpdater.MaxInstallerBytes, "Installer download size cap was not applied.");
        File.Delete(staged);
        fake.Json = json.Serialize(new object[] { AppRelease("v0.1.0-installer.13", false, "https://evil.example") });
        Throws(() => SelfUpdater.FindUpdate(fake, SelfUpdater.CurrentTag), "Untrusted app release host accepted.");

        // End-to-end test of the real cmd -> copied GUI helper -> replace ->
        // restart sequence, without touching the running application or Minecraft.
        string testExe = System.Diagnostics.Process.GetCurrentProcess().MainModule.FileName;
        File.Copy(testExe, target, true); File.AppendAllText(target, "old file marker");
        string oldHash = PackInstaller.HashFile(target);
        staged = Path.Combine(dir, ".hyperrealism-update-test.tmp"); File.Copy(testExe, staged);
        sum = PackInstaller.HashFile(staged);
        Assert(oldHash != sum, "Fixture should differ from replacement.");
        string marker = Path.Combine(root, "restart-marker.txt");
        Environment.SetEnvironmentVariable("HR_UPDATE_TEST_MARKER", marker);
        System.Diagnostics.Process helper;
        // The target is locked briefly to verify that the helper retries until
        // the old application has released it.
        using (var lockFile = new FileStream(target, FileMode.Open, FileAccess.Read, FileShare.None))
        {
            helper = SelfUpdater.LaunchHelper(gui, staged, target, sum);
            System.Threading.Thread.Sleep(1500);
            Assert(!File.Exists(marker), "Restarted while target was locked.");
        }
        Assert(helper.WaitForExit(40000), "Updater helper did not finish.");
        Assert(WaitForMarker(marker, "--updated"), "Updated program did not restart.");
        WaitForRelease(target);
        Assert(PackInstaller.HashFile(target) == sum && !File.Exists(staged), "Replacement failed or stage remains.");

        // If staged bytes are changed while the old executable is locked, the
        // helper must reopen the old file rather than replace it with bad bytes.
        File.Delete(marker);
        File.Copy(testExe, target, true); File.AppendAllText(target, "old file marker");
        oldHash = PackInstaller.HashFile(target);
        staged = Path.Combine(dir, ".hyperrealism-update-mutated.tmp"); File.Copy(testExe, staged);
        sum = PackInstaller.HashFile(staged);
        using (var lockFile = new FileStream(target, FileMode.Open, FileAccess.Read, FileShare.None))
        {
            helper = SelfUpdater.LaunchHelper(gui, staged, target, sum);
            System.Threading.Thread.Sleep(1500);
            File.AppendAllText(staged, "tampered");
        }
        Assert(helper.WaitForExit(40000), "Failure helper did not finish.");
        Assert(WaitForMarker(marker, "--update-failed"), "Old program was not restarted after failure.");
        WaitForRelease(target);
        Assert(PackInstaller.HashFile(target) == oldHash && !File.Exists(staged), "Invalid update damaged old executable or stage remains.");
        Environment.SetEnvironmentVariable("HR_UPDATE_TEST_MARKER", null);
    }

    private static void TestLaunch(string root)
    {
        string data = Path.Combine(root, "launcher data & 50%");
        string instance = Path.Combine(data, "instances", "26.3 (1) & test");
        string folder = Path.Combine(instance, "minecraft", "resourcepacks");
        Directory.CreateDirectory(folder);
        File.WriteAllText(Path.Combine(instance, "instance.cfg"), "fixture");
        Throws(() => LauncherIntegration.TargetFromFolder(folder), "Unconfigured launcher data directory accepted.");
        File.WriteAllText(Path.Combine(data, "elyprismlauncher.cfg"), "fixture");
        LaunchTarget target = LauncherIntegration.TargetFromFolder(folder);
        Assert(target.DataDirectory == data && target.InstanceId == "26.3 (1) & test", "Wrong launcher target inferred.");
        string bad = Path.Combine(root, "resourcepacks"); Directory.CreateDirectory(bad);
        Throws(() => LauncherIntegration.TargetFromFolder(bad), "Unrelated resourcepacks folder accepted as an instance.");
        File.Delete(Path.Combine(instance, "instance.cfg"));
        Throws(() => LauncherIntegration.TargetFromFolder(folder), "Missing instance.cfg accepted.");
        File.WriteAllText(Path.Combine(instance, "instance.cfg"), "fixture");
        string marker = Path.Combine(root, "launch-probe.txt");
        Environment.SetEnvironmentVariable("HR_LAUNCH_TEST_MARKER", marker);
        try
        {
            string testExe = System.Diagnostics.Process.GetCurrentProcess().MainModule.FileName;
            using (var process = LauncherIntegration.Launch(testExe, target))
                Assert(process.WaitForExit(15000) && process.ExitCode == 0, "Launcher process test failed.");
            var actual = new JavaScriptSerializer().Deserialize<string[]>(File.ReadAllText(marker));
            Assert(actual.Length == 5 && actual[0] == "--dir" && actual[1] == data && actual[2] == "--launch" && actual[3] == target.InstanceId && actual[4] == "--show-window", "Launcher command-line arguments were not preserved.");
            Throws(() => LauncherIntegration.StartInfo(Path.Combine(root, "missing.exe"), target), "Missing launcher accepted.");
        }
        finally { Environment.SetEnvironmentVariable("HR_LAUNCH_TEST_MARKER", null); }
    }

    private static int Main(string[] args)
    {
        string launchMarker = Environment.GetEnvironmentVariable("HR_LAUNCH_TEST_MARKER");
        if (!String.IsNullOrEmpty(launchMarker) && args.Length > 0 && args[0] == "--dir")
        {
            File.WriteAllText(launchMarker, new JavaScriptSerializer().Serialize(args));
            return 0;
        }
        if (args.Length == 1 && (args[0] == "--updated" || args[0] == "--update-failed"))
        {
            string marker = Environment.GetEnvironmentVariable("HR_UPDATE_TEST_MARKER");
            if (!String.IsNullOrEmpty(marker)) File.WriteAllText(marker, args[0]);
            return 0;
        }
        if (args.Length == 1 && args[0] == "--live-test") return LiveTest();
        string root = Path.Combine(Path.GetTempPath(), "hr-gui-test-" + Guid.NewGuid().ToString("N"));
        try
        {
            string folder = Path.Combine(root, "resourcepacks");
            Directory.CreateDirectory(folder);
            string wrong = Path.Combine(root, "not-resourcepacks"); Directory.CreateDirectory(wrong);
            Assert(PackInstaller.ValidateFolder(folder + "\\") == folder, "Trailing slash should work.");
            Throws(() => PackInstaller.ValidateFolder(wrong), "Wrong folder name accepted.");
            Throws(() => PackInstaller.ValidateFolder(Path.Combine(root, "missing", "resourcepacks")), "Missing folder accepted.");
            Throws(() => PackInstaller.ValidateFolder("resourcepacks"), "Relative path accepted.");
            var fake = new FakeTransfer();
            fake.Cancel = true;
            Throws(() => PackInstaller.Check(fake, folder), "Cancelled update check continued processing.");
            fake.Cancel = false;
            var cancelledWeb = new WebTransfer(null, () => true);
            Throws(() => cancelledWeb.Download("https://github.com/", Path.Combine(root, "cancelled-download"), 1024), "Cancelled transfer started a network download.");
            fake.Payload = Encoding.UTF8.GetBytes("verified pack bytes");
            string source = Path.Combine(root, "fixture"); File.WriteAllBytes(source, fake.Payload);
            fake.Checksum = PackInstaller.HashFile(source) + "  " + PackInstaller.PackName;
            var serializer = new JavaScriptSerializer();
            fake.Json = serializer.Serialize(new object[] {
                Release("old", "2025-01-01T00:00:00Z", false, "https://github.com"),
                Release("new", "2026-01-01T00:00:00Z", false, "https://github.com"),
                Release("draft", "2027-01-01T00:00:00Z", true, "https://github.com"),
                new { draft = false, tag_name = "installer-only", published_at = "2028-01-01T00:00:00Z", assets = new object[0] }
            });
            Assert(PackInstaller.Check(fake, folder).Contains("new") && PackInstaller.Check(fake, folder).Contains("Not installed"), "Latest release not selected.");
            Assert(PackInstaller.Install(fake, folder).Contains("new"), "Install did not report version.");
            Assert(fake.LastMaxBytes == PackInstaller.MaxPackBytes, "Pack download size cap was not applied.");
            string target = Path.Combine(folder, PackInstaller.PackName);
            Assert(PackInstaller.Check(fake, folder).Contains("Installed file matches"), "Installed hash not recognized.");
            File.WriteAllText(target, "old pack");
            Assert(PackInstaller.Check(fake, folder).Contains("differs"), "Different installed file not detected.");
            fake.Corrupt = true;
            Throws(() => PackInstaller.Install(fake, folder), "Corrupt download accepted.");
            Assert(File.ReadAllText(target) == "old pack", "Failed download damaged old pack.");
            fake.Corrupt = false;
            PackInstaller.Install(fake, folder);
            Assert(PackInstaller.HashFile(target) == PackInstaller.HashFile(source), "Atomic update failed.");
            Assert(Directory.GetFiles(folder, ".hyperrealism-*.tmp").Length == 0, "Temporary staging file was left behind.");
            fake.Checksum = "not a checksum";
            Throws(() => PackInstaller.Install(fake, folder), "Invalid checksum accepted.");
            Assert(PackInstaller.HashFile(target) == PackInstaller.HashFile(source), "Bad checksum damaged existing pack.");
            fake.Checksum = PackInstaller.HashFile(source) + "  " + PackInstaller.PackName;
            fake.Json = serializer.Serialize(new object[] { Release("bad", "2026-01-01T00:00:00Z", false, "https://evil.example") });
            Throws(() => PackInstaller.Install(fake, folder), "Untrusted host accepted.");
            if (args.Length != 1) throw new Exception("Expected path to the built GUI installer.");
            TestLaunch(root);
            TestSelfUpdate(root, args[0]);
            Console.WriteLine("GUI installer backend and self-update tests passed.");
            return 0;
        }
        finally { if (Directory.Exists(root)) Directory.Delete(root, true); }
    }
}
