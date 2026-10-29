using System;
using System.Drawing;
using System.IO;
using System.Threading.Tasks;
using System.Windows.Forms;

namespace HyperRealismInstaller
{
    internal static class Program
    {
        [STAThread]
        private static int Main(string[] args)
        {
            if (args.Length == 1 && args[0] == "--apply-update") return SelfUpdater.ApplyUpdateFromEnvironment();
            Application.EnableVisualStyles();
            Application.SetCompatibleTextRenderingDefault(false);
            if (args.Length == 1 && args[0] == "--smoke-test")
            {
                try { using (var form = new InstallerForm()) form.SmokeTest(); return 0; }
                catch { return 1; }
            }
            Application.Run(new InstallerForm(args.Length == 1 ? args[0] : ""));
            return 0;
        }
    }

    internal sealed class InstallerForm : Form
    {
        private readonly TextBox pathBox = new TextBox();
        private readonly Label folderStatus = new Label();
        private readonly Label result = new Label();
        private readonly ProgressBar progress = new ProgressBar();
        private readonly Button browse = new Button();
        private readonly Button openFolder = new Button();
        private readonly Button check = new Button();
        private readonly Button install = new Button();
        private readonly Button close = new Button();
        private readonly LinkLabel appUpdate = new LinkLabel();
        private readonly Timer updateTimer = new Timer();
        private bool busy;
        private bool checkingApp;
        private static readonly string SettingsPath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "HyperRealism128", "resourcepacks-path.txt");

        internal InstallerForm() : this("") { }

        internal InstallerForm(string startup)
        {
            Text = "HyperRealism128 Installer";
            StartPosition = FormStartPosition.CenterScreen;
            ClientSize = new Size(730, 475);
            MinimumSize = new Size(690, 510);
            Font = new Font("Segoe UI", 9F);
            AutoScaleMode = AutoScaleMode.Font;
            BackColor = Color.FromArgb(247, 249, 252);

            var root = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 1, RowCount = 4, Margin = Padding.Empty, Padding = Padding.Empty };
            root.RowStyles.Add(new RowStyle(SizeType.Absolute, 80));
            root.RowStyles.Add(new RowStyle(SizeType.Absolute, 82));
            root.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
            root.RowStyles.Add(new RowStyle(SizeType.Absolute, 70));
            Controls.Add(root);

            var header = new Panel { Dock = DockStyle.Fill, BackColor = Color.FromArgb(22, 31, 48), Padding = new Padding(22, 12, 22, 8) };
            var title = new Label { Text = "HYPERREALISM 128", AutoSize = true, ForeColor = Color.White, Font = new Font("Segoe UI", 16F, FontStyle.Bold), Location = new Point(22, 9) };
            var subtitle = new Label { Text = "Minecraft Java 26.3  /  Experimental preview", AutoSize = true, ForeColor = Color.FromArgb(184, 200, 221), Location = new Point(25, 49) };
            appUpdate.Text = "Check installer updates"; appUpdate.LinkColor = Color.FromArgb(167, 204, 255); appUpdate.ActiveLinkColor = Color.White;
            appUpdate.AutoSize = false; appUpdate.TextAlign = ContentAlignment.MiddleRight; appUpdate.Size = new Size(180, 25);
            appUpdate.Location = new Point(525, 12); appUpdate.Anchor = AnchorStyles.Top | AnchorStyles.Right;
            header.Controls.Add(title); header.Controls.Add(subtitle); header.Controls.Add(appUpdate); root.Controls.Add(header, 0, 0);

            var intro = new Label { Dock = DockStyle.Fill, Padding = new Padding(24, 15, 24, 0), Text = "Install or update the resource pack in your Minecraft instance.\r\nYour configured ElyPrism folder is filled in below; you can change it anytime.", ForeColor = Color.FromArgb(55, 65, 81) };
            root.Controls.Add(intro, 0, 1);

            var body = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 1, RowCount = 6, Padding = new Padding(24, 0, 24, 0) };
            body.RowStyles.Add(new RowStyle(SizeType.Absolute, 30));
            body.RowStyles.Add(new RowStyle(SizeType.Absolute, 37));
            body.RowStyles.Add(new RowStyle(SizeType.Absolute, 38));
            body.RowStyles.Add(new RowStyle(SizeType.Absolute, 44));
            body.RowStyles.Add(new RowStyle(SizeType.Absolute, 22));
            body.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
            root.Controls.Add(body, 0, 2);
            body.Controls.Add(new Label { Text = "RESOURCE PACKS FOLDER", Dock = DockStyle.Fill, Font = new Font("Segoe UI", 9F, FontStyle.Bold), ForeColor = Color.FromArgb(39, 52, 71) }, 0, 0);
            var pathRow = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 2, RowCount = 1, Margin = Padding.Empty };
            pathRow.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            pathRow.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 106));
            pathBox.Dock = DockStyle.Fill; pathBox.Margin = new Padding(0, 1, 9, 0);
            browse.Text = "Browse..."; browse.Dock = DockStyle.Fill; browse.Margin = new Padding(0, 0, 0, 5);
            pathRow.Controls.Add(pathBox, 0, 0); pathRow.Controls.Add(browse, 1, 0); body.Controls.Add(pathRow, 0, 1);
            folderStatus.Dock = DockStyle.Fill; folderStatus.Font = new Font("Segoe UI", 9F, FontStyle.Bold); body.Controls.Add(folderStatus, 0, 2);
            result.Dock = DockStyle.Fill; result.Text = "Ready. Check for updates or install the latest preview."; result.ForeColor = Color.FromArgb(55, 65, 81); body.Controls.Add(result, 0, 3);
            progress.Dock = DockStyle.Fill; progress.Style = ProgressBarStyle.Marquee; progress.MarqueeAnimationSpeed = 25; progress.Visible = false; body.Controls.Add(progress, 0, 4);
            body.Controls.Add(new Label { Dock = DockStyle.Bottom, AutoSize = true, Text = "Folder name is a helpful check, not proof this is the active Minecraft instance.\r\nDownloads are SHA-256 checked; the checksum is not an independent signature.", ForeColor = Color.FromArgb(107, 114, 128) }, 0, 5);

            var footer = new FlowLayoutPanel { Dock = DockStyle.Fill, FlowDirection = FlowDirection.RightToLeft, WrapContents = false, Padding = new Padding(20, 16, 20, 8), BackColor = Color.White };
            close.Text = "Close"; close.Size = new Size(78, 33);
            install.Text = "Install / Update"; install.Size = new Size(133, 33); install.BackColor = Color.FromArgb(31, 100, 194); install.ForeColor = Color.White; install.FlatStyle = FlatStyle.Flat; install.FlatAppearance.BorderSize = 0;
            check.Text = "Check for updates"; check.Size = new Size(140, 33);
            openFolder.Text = "Open folder"; openFolder.Size = new Size(110, 33);
            footer.Controls.Add(close); footer.Controls.Add(install); footer.Controls.Add(check); footer.Controls.Add(openFolder); root.Controls.Add(footer, 0, 3);

            pathBox.TextChanged += delegate { UpdateFolderStatus(); if (!busy) result.Text = "Ready. Check for updates or install the latest preview."; };
            browse.Click += delegate {
                using (var picker = new FolderBrowserDialog())
                {
                    picker.Description = "Select your Minecraft instance's resourcepacks folder";
                    picker.ShowNewFolderButton = false;
                    if (Directory.Exists(pathBox.Text)) picker.SelectedPath = pathBox.Text;
                    if (picker.ShowDialog(this) == DialogResult.OK) pathBox.Text = picker.SelectedPath;
                }
            };
            openFolder.Click += delegate {
                try { System.Diagnostics.Process.Start("explorer.exe", "\"" + PackInstaller.ValidateFolder(pathBox.Text) + "\""); }
                catch (Exception ex) { ShowError(ex); }
            };
            check.Click += async delegate { await RunOperation(false); };
            install.Click += async delegate { await RunOperation(true); };
            close.Click += delegate { Close(); };
            appUpdate.LinkClicked += async delegate { await CheckInstallerUpdate(false); };
            FormClosing += delegate(object sender, FormClosingEventArgs e) { if (busy) e.Cancel = true; else SavePath(); };
            string defaultPath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "ElyPrismLauncher", "instances", "26.3(1)", "minecraft", "resourcepacks");
            pathBox.Text = LoadPath(defaultPath);
            UpdateFolderStatus();
            if (startup == "--updated") result.Text = "Installer updated and reopened successfully.";
            if (startup == "--update-failed") { result.Text = "Installer update failed; the previous installer was reopened. Close other copies and check folder permissions."; result.ForeColor = Color.FromArgb(185, 28, 28); }
            Shown += async delegate { await CheckInstallerUpdate(true); };
            updateTimer.Interval = 60 * 60 * 1000;
            updateTimer.Tick += async delegate { await CheckInstallerUpdate(true); };
            updateTimer.Start();
            FormClosed += delegate { updateTimer.Stop(); updateTimer.Dispose(); };
        }

        private static string LoadPath(string fallback)
        {
            try { if (File.Exists(SettingsPath)) { string text = File.ReadAllText(SettingsPath).Trim(); if (text.Length > 0 && text.Length < 32768) return text; } }
            catch { }
            return fallback;
        }

        private void SavePath()
        {
            try
            {
                string path = PackInstaller.ValidateFolder(pathBox.Text);
                Directory.CreateDirectory(Path.GetDirectoryName(SettingsPath));
                File.WriteAllText(SettingsPath, path);
            }
            catch { /* Preferences are optional; the path is always editable. */ }
        }

        private async Task CheckInstallerUpdate(bool automatic)
        {
            if (checkingApp || busy) return;
            checkingApp = true; appUpdate.Enabled = false;
            try
            {
                InstallerRelease update = await Task.Run(() => SelfUpdater.FindUpdate(new WebTransfer(), SelfUpdater.CurrentTag));
                if (IsDisposed || busy) return;
                appUpdate.Text = "Check installer updates";
                if (update != null)
                {
                    var answer = MessageBox.Show(this, "A new installer version (" + update.Tag + ") is available on GitHub.\r\n\r\nDownload it, verify it and restart the installer now?", "Installer update available", MessageBoxButtons.YesNo, MessageBoxIcon.Information);
                    if (answer == DialogResult.Yes) { await ApplyInstallerUpdate(update); return; }
                }
                else if (!automatic) MessageBox.Show(this, "You have the latest published installer (" + SelfUpdater.CurrentTag + ").", "Installer updates", MessageBoxButtons.OK, MessageBoxIcon.Information);
            }
            catch (Exception ex)
            {
                if (!IsDisposed && !automatic) ShowError(ex);
                else if (!IsDisposed) appUpdate.Text = "Update check unavailable";
            }
            finally { checkingApp = false; if (!IsDisposed) appUpdate.Enabled = true; }
            if (automatic && !IsDisposed && !busy) await CheckPackOnStartup();
        }

        private async Task CheckPackOnStartup()
        {
            string path = pathBox.Text;
            try
            {
                string folder = PackInstaller.ValidateFolder(path);
                if (!File.Exists(Path.Combine(folder, PackInstaller.PackName))) return;
                string state = await Task.Run(() => PackInstaller.Check(new WebTransfer(), folder));
                if (IsDisposed || busy || !String.Equals(path, pathBox.Text, StringComparison.Ordinal)) return;
                if (state.Contains("Installed file differs"))
                {
                    var answer = MessageBox.Show(this, state + "\r\n\r\nInstall the latest resource-pack preview now?", "Resource pack update available", MessageBoxButtons.YesNo, MessageBoxIcon.Information);
                    if (answer == DialogResult.Yes) await RunOperation(true);
                }
            }
            catch { /* Offline checks are silent; the user can retry via the button. */ }
        }

        private async Task ApplyInstallerUpdate(InstallerRelease update)
        {
            string stage = null;
            bool handedOff = false;
            busy = true; pathBox.Enabled = browse.Enabled = close.Enabled = appUpdate.Enabled = false;
            UpdateFolderStatus(); progress.Visible = true;
            result.Text = "Downloading and verifying installer " + update.Tag + "...";
            try
            {
                SavePath();
                string target = Application.ExecutablePath;
                string checksum = null;
                stage = await Task.Run(() => SelfUpdater.DownloadUpdate(new WebTransfer(), update, target, out checksum));
                SelfUpdater.LaunchHelper(target, stage, target, checksum);
                handedOff = true;
                busy = false;
                Application.Exit();
            }
            catch (Exception ex) { ShowError(ex); }
            finally
            {
                if (!handedOff && stage != null) { try { if (File.Exists(stage)) File.Delete(stage); } catch { } }
                busy = false;
                if (!IsDisposed) { progress.Visible = false; pathBox.Enabled = browse.Enabled = close.Enabled = appUpdate.Enabled = true; UpdateFolderStatus(); }
            }
        }

        private void UpdateFolderStatus()
        {
            bool valid;
            try
            {
                PackInstaller.ValidateFolder(pathBox.Text);
                folderStatus.Text = "Correct folder name  •  Folder exists";
                folderStatus.ForeColor = Color.FromArgb(21, 128, 61);
                valid = true;
            }
            catch (Exception ex)
            {
                folderStatus.Text = ex.Message;
                folderStatus.ForeColor = Color.FromArgb(185, 83, 15);
                valid = false;
            }
            check.Enabled = valid && !busy;
            install.Enabled = valid && !busy;
            openFolder.Enabled = valid && !busy;
        }

        private async Task RunOperation(bool doInstall)
        {
            string path = pathBox.Text;
            SavePath();
            busy = true;
            pathBox.Enabled = browse.Enabled = close.Enabled = false;
            UpdateFolderStatus();
            progress.Visible = true;
            result.ForeColor = Color.FromArgb(55, 65, 81);
            result.Text = doInstall ? "Downloading and verifying the latest preview..." : "Checking the latest preview...";
            try
            {
                string message = await Task.Run(() => doInstall ? PackInstaller.Install(new WebTransfer(), path) : PackInstaller.Check(new WebTransfer(), path));
                result.Text = message;
                result.ForeColor = Color.FromArgb(21, 128, 61);
            }
            catch (Exception ex) { ShowError(ex); }
            finally
            {
                progress.Visible = false;
                busy = false;
                pathBox.Enabled = browse.Enabled = close.Enabled = true;
                UpdateFolderStatus();
            }
        }

        private void ShowError(Exception ex)
        {
            result.ForeColor = Color.FromArgb(185, 28, 28);
            result.Text = "Unable to finish. See the error for details.";
            MessageBox.Show(this, ex.Message, "HyperRealism Installer", MessageBoxButtons.OK, MessageBoxIcon.Warning);
        }

        internal void SmokeTest()
        {
            if (String.IsNullOrEmpty(pathBox.Text)) throw new Exception("No default or saved path.");
            pathBox.Text = Path.GetTempPath();
            if (install.Enabled || check.Enabled) throw new Exception("Invalid folder was accepted.");
            if (String.IsNullOrEmpty(folderStatus.Text) || install.Text != "Install / Update" || check.Text != "Check for updates") throw new Exception("Expected installer controls are missing.");
        }
    }
}
