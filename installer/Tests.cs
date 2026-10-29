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
    public string GetText(string url) { return url == PackInstaller.Api ? Json : Checksum; }
    public void Download(string url, string destination) { File.WriteAllBytes(destination, Corrupt ? Encoding.UTF8.GetBytes("corrupt") : Payload); }
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
    private static int Main()
    {
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
            Console.WriteLine("GUI installer backend tests passed.");
            return 0;
        }
        finally { if (Directory.Exists(root)) Directory.Delete(root, true); }
    }
}
