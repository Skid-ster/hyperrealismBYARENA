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
            Application.EnableVisualStyles();
            Application.SetCompatibleTextRenderingDefault(false);
            if (args.Length == 1 && args[0] == "--smoke-test")
            {
                try { using (var form = new InstallerForm()) form.SmokeTest(); return 0; }
                catch { return 1; }
            }
            Application.Run(new InstallerForm());
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
        private bool busy;

        internal InstallerForm()
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
            header.Controls.Add(title); header.Controls.Add(subtitle); root.Controls.Add(header, 0, 0);

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
            FormClosing += delegate(object sender, FormClosingEventArgs e) { if (busy) e.Cancel = true; };
            pathBox.Text = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "ElyPrismLauncher", "instances", "26.3(1)", "minecraft", "resourcepacks");
            UpdateFolderStatus();
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
            if (!pathBox.Text.EndsWith(Path.Combine("26.3(1)", "minecraft", "resourcepacks"), StringComparison.OrdinalIgnoreCase)) throw new Exception("Incorrect ElyPrism default path.");
            pathBox.Text = Path.GetTempPath();
            if (install.Enabled || check.Enabled) throw new Exception("Invalid folder was accepted.");
            if (String.IsNullOrEmpty(folderStatus.Text) || install.Text != "Install / Update" || check.Text != "Check for updates") throw new Exception("Expected installer controls are missing.");
        }
    }
}
