// Launch an existing PineconeMC/Prism-derived instance through the launcher,
// never by reconstructing Minecraft credentials or Java arguments ourselves.
using System;
using System.Diagnostics;
using System.IO;
using System.Text;

namespace HyperRealismInstaller
{
    internal sealed class LaunchTarget
    {
        public string DataDirectory;
        public string InstanceId;
    }

    internal static class LauncherIntegration
    {
        // Only infer an ID when the selected pack folder has the standard
        // <data>/instances/<id>/{minecraft,.minecraft}/resourcepacks layout.
        // Non-standard custom instance locations must be launched in PineconeMC.
        internal static LaunchTarget TargetFromFolder(string resourcepacks)
        {
            string folder = PackInstaller.ValidateFolder(resourcepacks);
            DirectoryInfo game = Directory.GetParent(folder);
            if (game == null || !(String.Equals(game.Name, "minecraft", StringComparison.OrdinalIgnoreCase) || String.Equals(game.Name, ".minecraft", StringComparison.OrdinalIgnoreCase)))
                throw new InvalidOperationException("Launch needs an instance's minecraft\\resourcepacks folder. For custom locations, launch the instance from PineconeMC.");
            DirectoryInfo instance = game.Parent;
            DirectoryInfo instances = instance == null ? null : instance.Parent;
            DirectoryInfo data = instances == null ? null : instances.Parent;
            if (instance == null || instances == null || data == null || !String.Equals(instances.Name, "instances", StringComparison.OrdinalIgnoreCase) || !File.Exists(Path.Combine(instance.FullName, "instance.cfg")))
                throw new InvalidOperationException("Could not identify a PineconeMC instance from this folder. Select a standard instances\\<instance>\\minecraft\\resourcepacks folder, or launch from PineconeMC.");
            // Refuse to create a fresh launcher data directory by accident. A
            // configured install has one of these launcher config filenames.
            bool configured = false;
            foreach (string config in new string[] { "elyprismlauncher.cfg", "pineconemc.cfg", "prismlauncher.cfg", "polymc.cfg", "multimc.cfg" })
                if (File.Exists(Path.Combine(data.FullName, config))) { configured = true; break; }
            if (!configured) throw new InvalidOperationException("No launcher configuration was found alongside this instances folder. Launch this instance from PineconeMC if you use a custom data layout.");
            return new LaunchTarget { DataDirectory = data.FullName, InstanceId = instance.Name };
        }

        // Quote according to Windows CommandLineToArgvW rules; never invoke a shell.
        internal static string QuoteArgument(string value)
        {
            if (value == null) throw new ArgumentNullException("value");
            var text = new StringBuilder("\"");
            int slashes = 0;
            foreach (char c in value)
            {
                if (c == '\\') { slashes++; continue; }
                if (c == '"') { text.Append('\\', slashes * 2 + 1); text.Append('"'); }
                else { text.Append('\\', slashes); text.Append(c); }
                slashes = 0;
            }
            text.Append('\\', slashes * 2); text.Append('"');
            return text.ToString();
        }

        internal static ProcessStartInfo StartInfo(string executable, LaunchTarget target)
        {
            if (String.IsNullOrWhiteSpace(executable) || !Path.IsPathRooted(executable) || !File.Exists(executable) || !String.Equals(Path.GetExtension(executable), ".exe", StringComparison.OrdinalIgnoreCase))
                throw new FileNotFoundException("Choose an existing PineconeMC executable (.exe).", executable);
            if (target == null || String.IsNullOrWhiteSpace(target.DataDirectory) || String.IsNullOrWhiteSpace(target.InstanceId))
                throw new ArgumentException("No instance selected.");
            return new ProcessStartInfo {
                FileName = executable,
                Arguments = "--dir " + QuoteArgument(target.DataDirectory) + " --launch " + QuoteArgument(target.InstanceId) + " --show-window",
                WorkingDirectory = Path.GetDirectoryName(executable),
                UseShellExecute = false
            };
        }

        internal static Process Launch(string executable, LaunchTarget target)
        {
            Process result = Process.Start(StartInfo(executable, target));
            if (result == null) throw new InvalidOperationException("Could not start PineconeMC.");
            return result;
        }

        internal static string FindInstalled()
        {
            string[] roots = {
                Environment.GetFolderPath(Environment.SpecialFolder.ProgramFiles),
                Environment.GetFolderPath(Environment.SpecialFolder.ProgramFilesX86),
                Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Programs")
            };
            string[] folders = { "PineconeMC", "ElyPrismLauncher", "ElyPrism Launcher" };
            string[] binaries = { "elyprismlauncher.exe", "PineconeMC.exe", "ElyPrismLauncher.exe" };
            foreach (string root in roots)
                if (!String.IsNullOrWhiteSpace(root)) foreach (string folder in folders) foreach (string binary in binaries)
                {
                    string candidate = Path.Combine(root, folder, binary);
                    if (File.Exists(candidate)) return candidate;
                }
            return null;
        }
    }
}
