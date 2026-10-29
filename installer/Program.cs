using System;
using System.Drawing;
using System.IO;
using System.Runtime.InteropServices;
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
        private readonly Button appUpdate = new Button();
        private readonly Button toolBrowse = new Button();
        private readonly Button toolOpen = new Button();
        private readonly Button toolCheck = new Button();
        private readonly Button toolApp = new Button();
        private readonly Button sideBrowse = new Button();
        private static readonly Color Background = Color.FromArgb(32, 32, 32);
        private static readonly Color Surface = Color.FromArgb(45, 45, 45);
        private static readonly Color Foreground = Color.FromArgb(224, 224, 224);
        private static readonly Color Green = Color.FromArgb(149, 213, 78);
        private readonly Timer updateTimer = new Timer();
        private bool busy;
        private bool checkingApp;
        private static readonly string SettingsPath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "HyperRealism128", "resourcepacks-path.txt");

        internal InstallerForm() : this("") { }

        internal InstallerForm(string startup)
        {
            Text = "HyperRealism128 Installer";
            StartPosition = FormStartPosition.CenterScreen;
            ClientSize = new Size(805, 502);
            MinimumSize = new Size(760, 535);
            Font = new Font("Segoe UI", 9F);
            AutoScaleMode = AutoScaleMode.Font;
            BackColor = Background;
            ForeColor = Foreground;

            // The dark toolbar, selected pack tile, right-hand actions and
            // double status strip intentionally echo the user's launcher.
            var root = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 1, RowCount = 4, Margin = Padding.Empty, Padding = Padding.Empty, BackColor = Background };
            root.RowStyles.Add(new RowStyle(SizeType.Absolute, 40));
            root.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
            root.RowStyles.Add(new RowStyle(SizeType.Absolute, 27));
            root.RowStyles.Add(new RowStyle(SizeType.Absolute, 24));
            Controls.Add(root);

            var toolbar = new Panel { Dock = DockStyle.Fill, BackColor = Surface, Margin = Padding.Empty };
            var tools = new FlowLayoutPanel { Dock = DockStyle.Fill, WrapContents = false, FlowDirection = FlowDirection.LeftToRight, Padding = new Padding(7, 2, 0, 0), BackColor = Surface };
            StyleButton(toolBrowse, "Choose folder", "folder", 120, 34, true);
            StyleButton(toolOpen, "Open folder", "folder", 110, 34, true);
            StyleButton(toolCheck, "Pack update", "update", 112, 34, true);
            StyleButton(toolApp, "Installer update", "update", 136, 34, true);
            var help = new Button(); StyleButton(help, "Help", "help", 72, 34, true);
            tools.Controls.Add(toolBrowse); tools.Controls.Add(toolOpen); tools.Controls.Add(help); tools.Controls.Add(toolCheck); tools.Controls.Add(toolApp);
            toolbar.Controls.Add(tools); root.Controls.Add(toolbar, 0, 0);

            var main = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 2, RowCount = 1, Margin = Padding.Empty, BackColor = Background };
            main.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            main.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 150));
            root.Controls.Add(main, 0, 1);
            var left = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 1, RowCount = 8, Margin = Padding.Empty, Padding = new Padding(6, 0, 6, 0), BackColor = Background };
            left.RowStyles.Add(new RowStyle(SizeType.Absolute, 28));
            left.RowStyles.Add(new RowStyle(SizeType.Absolute, 117));
            left.RowStyles.Add(new RowStyle(SizeType.Absolute, 24));
            left.RowStyles.Add(new RowStyle(SizeType.Absolute, 33));
            left.RowStyles.Add(new RowStyle(SizeType.Absolute, 30));
            left.RowStyles.Add(new RowStyle(SizeType.Absolute, 42));
            left.RowStyles.Add(new RowStyle(SizeType.Absolute, 15));
            left.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
            main.Controls.Add(left, 0, 0);
            left.Controls.Add(new Label { Dock = DockStyle.Fill, Text = "⌄  Resource packs", TextAlign = ContentAlignment.MiddleLeft, ForeColor = Color.FromArgb(180, 180, 180), Font = new Font("Segoe UI", 9F, FontStyle.Bold) }, 0, 0);
            var tile = new Panel { Dock = DockStyle.Fill, BackColor = Background, Margin = Padding.Empty };
            var block = new PictureBox { Image = BlockIcon(), SizeMode = PictureBoxSizeMode.StretchImage, Size = new Size(48, 48), Location = new Point(29, 12), AccessibleName = "HyperRealism pack icon" };
            var tileCaption = new Label { Text = "HyperRealism128", TextAlign = ContentAlignment.MiddleCenter, ForeColor = Color.FromArgb(30, 30, 30), BackColor = Green, Location = new Point(4, 76), Size = new Size(105, 22), AutoEllipsis = true };
            tile.Controls.Add(block); tile.Controls.Add(tileCaption); left.Controls.Add(tile, 0, 1);
            left.Controls.Add(new Label { Dock = DockStyle.Fill, Text = "Resource pack folder", ForeColor = Foreground, TextAlign = ContentAlignment.MiddleLeft }, 0, 2);
            var pathRow = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 2, RowCount = 1, Margin = Padding.Empty };
            pathRow.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            pathRow.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 88));
            pathBox.Dock = DockStyle.Fill; pathBox.BackColor = Surface; pathBox.ForeColor = Foreground; pathBox.BorderStyle = BorderStyle.FixedSingle; pathBox.Margin = new Padding(0, 3, 7, 0);
            StyleButton(browse, "Browse...", "folder", 85, 28, false); browse.Dock = DockStyle.Fill; browse.Margin = new Padding(0, 0, 0, 3);
            pathRow.Controls.Add(pathBox, 0, 0); pathRow.Controls.Add(browse, 1, 0); left.Controls.Add(pathRow, 0, 3);
            folderStatus.Dock = DockStyle.Fill; folderStatus.Font = new Font("Segoe UI", 9F, FontStyle.Bold); folderStatus.TextAlign = ContentAlignment.MiddleLeft; left.Controls.Add(folderStatus, 0, 4);
            result.Dock = DockStyle.Fill; result.Text = "Ready. Check for updates or install the latest preview."; result.ForeColor = Foreground; left.Controls.Add(result, 0, 5);
            progress.Dock = DockStyle.Fill; progress.Style = ProgressBarStyle.Marquee; progress.MarqueeAnimationSpeed = 25; progress.Visible = false; left.Controls.Add(progress, 0, 6);
            left.Controls.Add(new Label { Dock = DockStyle.Bottom, AutoSize = true, Text = "Folder name is a helpful check, not proof this is the active instance.\r\nChecksums detect corruption but are not an independent signature.", ForeColor = Color.FromArgb(155, 155, 155) }, 0, 7);

            var side = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 1, RowCount = 9, Margin = Padding.Empty, Padding = new Padding(1, 0, 0, 0), BackColor = Surface };
            side.RowStyles.Add(new RowStyle(SizeType.Absolute, 112)); side.RowStyles.Add(new RowStyle(SizeType.Absolute, 34));
            for (int i = 0; i < 6; i++) side.RowStyles.Add(new RowStyle(SizeType.Absolute, 29));
            side.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
            main.Controls.Add(side, 1, 0);
            var sideIcon = new Panel { Dock = DockStyle.Fill, Margin = Padding.Empty, BackColor = Color.FromArgb(36, 36, 36) };
            var bigBlock = new PictureBox { Image = BlockIcon(), SizeMode = PictureBoxSizeMode.StretchImage, Size = new Size(72, 72), Anchor = AnchorStyles.None, Location = new Point(38, 17) };
            sideIcon.Controls.Add(bigBlock); side.Controls.Add(sideIcon, 0, 0);
            side.Controls.Add(new Label { Dock = DockStyle.Fill, Text = "HyperRealism128", TextAlign = ContentAlignment.MiddleCenter, ForeColor = Foreground, BackColor = Color.FromArgb(36, 36, 36) }, 0, 1);
            StyleButton(install, "Install / Update", "install", 148, 27, false);
            StyleButton(check, "Check for updates", "update", 148, 27, false);
            StyleButton(openFolder, "Open folder", "folder", 148, 27, false);
            StyleButton(sideBrowse, "Choose folder", "folder", 148, 27, false);
            StyleButton(appUpdate, "Installer update", "update", 148, 27, false);
            StyleButton(close, "Close", "close", 148, 27, false);
            foreach (var b in new Button[] { install, check, openFolder, sideBrowse, appUpdate, close }) { b.Dock = DockStyle.Fill; b.Margin = Padding.Empty; }
            side.Controls.Add(install, 0, 2); side.Controls.Add(check, 0, 3); side.Controls.Add(openFolder, 0, 4); side.Controls.Add(sideBrowse, 0, 5); side.Controls.Add(appUpdate, 0, 6); side.Controls.Add(close, 0, 7);
            side.Controls.Add(new Panel { Dock = DockStyle.Fill, BackColor = Color.FromArgb(36, 36, 36), Margin = Padding.Empty }, 0, 8);

            var news = new Panel { Dock = DockStyle.Fill, BackColor = Surface, Margin = Padding.Empty };
            news.Controls.Add(new Label { Dock = DockStyle.Fill, Padding = new Padding(12, 0, 0, 0), TextAlign = ContentAlignment.MiddleLeft, ForeColor = Foreground, Text = "▣  HyperRealism128  •  Experimental preview" });
            root.Controls.Add(news, 0, 2);
            var bottom = new Panel { Dock = DockStyle.Fill, BackColor = Surface, Margin = new Padding(0, 1, 0, 0) };
            bottom.Controls.Add(new Label { Dock = DockStyle.Fill, Padding = new Padding(7, 0, 0, 0), TextAlign = ContentAlignment.MiddleLeft, ForeColor = Foreground, Text = "Minecraft Java 26.3   |   128×128 LabPBR preview" });
            bottom.Controls.Add(new Label { Dock = DockStyle.Right, Width = 170, TextAlign = ContentAlignment.MiddleRight, Padding = new Padding(0, 0, 8, 0), ForeColor = Color.FromArgb(180, 180, 180), Text = "Installer " + SelfUpdater.CurrentTag });
            root.Controls.Add(bottom, 0, 3);

            toolBrowse.Click += delegate { browse.PerformClick(); };
            toolOpen.Click += delegate { openFolder.PerformClick(); };
            toolCheck.Click += delegate { check.PerformClick(); };
            toolApp.Click += delegate { appUpdate.PerformClick(); };
            sideBrowse.Click += delegate { browse.PerformClick(); };
            help.Click += delegate { try { System.Diagnostics.Process.Start("https://github.com/Skid-ster/hyperrealismBYARENA#readme"); } catch (Exception ex) { ShowError(ex); } };
            pathBox.TextChanged += delegate { UpdateFolderStatus(); if (!busy) { result.Text = "Ready. Check for updates or install the latest preview."; result.ForeColor = Foreground; } };
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
            appUpdate.Click += async delegate { await CheckInstallerUpdate(false); };
            FormClosing += delegate(object sender, FormClosingEventArgs e) { if (busy) e.Cancel = true; else SavePath(); };
            string defaultPath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "ElyPrismLauncher", "instances", "26.3(1)", "minecraft", "resourcepacks");
            pathBox.Text = LoadPath(defaultPath);
            UpdateFolderStatus();
            if (startup == "--updated") result.Text = "Installer updated and reopened successfully.";
            if (startup == "--update-failed") { result.Text = "Installer update failed; the previous installer was reopened. Close other copies and check folder permissions."; result.ForeColor = Color.FromArgb(255, 142, 142); }
            Shown += async delegate { await CheckInstallerUpdate(true); };
            updateTimer.Interval = 60 * 60 * 1000;
            updateTimer.Tick += async delegate { await CheckInstallerUpdate(true); };
            updateTimer.Start();
            FormClosed += delegate { updateTimer.Stop(); updateTimer.Dispose(); };
        }

        [DllImport("dwmapi.dll", PreserveSig = true)]
        private static extern int DwmSetWindowAttribute(IntPtr hwnd, int attribute, ref int value, int size);
        protected override void OnHandleCreated(EventArgs e)
        {
            base.OnHandleCreated(e);
            try { int dark = 1; if (DwmSetWindowAttribute(Handle, 20, ref dark, sizeof(int)) != 0) DwmSetWindowAttribute(Handle, 19, ref dark, sizeof(int)); }
            catch (DllNotFoundException) { } catch (EntryPointNotFoundException) { }
        }

        private static void StyleButton(Button button, string text, string glyph, int width, int height, bool toolbar)
        {
            button.Text = text; button.Size = new Size(width, height); button.Margin = Padding.Empty;
            button.FlatStyle = FlatStyle.Flat; button.FlatAppearance.BorderSize = 0;
            button.FlatAppearance.MouseOverBackColor = Color.FromArgb(66, 66, 66);
            button.FlatAppearance.MouseDownBackColor = Color.FromArgb(78, 78, 78);
            button.BackColor = toolbar ? Surface : Color.FromArgb(36, 36, 36); button.ForeColor = Foreground;
            button.Image = Glyph(glyph, toolbar ? 24 : 18);
            button.TextImageRelation = TextImageRelation.ImageBeforeText;
            button.ImageAlign = ContentAlignment.MiddleLeft; button.TextAlign = ContentAlignment.MiddleLeft;
            button.Padding = new Padding(toolbar ? 3 : 5, 0, 0, 0);
            button.UseVisualStyleBackColor = false; button.AutoEllipsis = true;
        }

        // Small original pixel-art icons; no launcher assets are redistributed.
        private static Bitmap BlockIcon()
        {
            var image = new Bitmap(48, 48);
            using (var g = Graphics.FromImage(image))
            using (var dirt = new SolidBrush(Color.FromArgb(110, 83, 62)))
            using (var soil = new SolidBrush(Color.FromArgb(127, 100, 76)))
            using (var grass = new SolidBrush(Color.FromArgb(135, 178, 66)))
            using (var top = new SolidBrush(Color.FromArgb(154, 197, 82)))
            {
                g.Clear(Color.Transparent); g.FillRectangle(dirt, 4, 6, 40, 37);
                g.FillRectangle(soil, 8, 28, 7, 6); g.FillRectangle(soil, 24, 34, 7, 6); g.FillRectangle(soil, 35, 24, 5, 5);
                g.FillRectangle(grass, 4, 7, 40, 14); g.FillRectangle(top, 6, 5, 36, 12);
                g.FillRectangle(grass, 9, 18, 7, 8); g.FillRectangle(grass, 22, 17, 6, 11); g.FillRectangle(grass, 33, 17, 7, 7);
            }
            return image;
        }
        private static Bitmap Glyph(string name, int size)
        {
            var image = new Bitmap(size, size);
            using (var g = Graphics.FromImage(image))
            {
                g.Clear(Color.Transparent);
                float k = size / 24f; g.ScaleTransform(k, k);
                using (var white = new SolidBrush(Color.FromArgb(225, 230, 224)))
                using (var green = new SolidBrush(Color.FromArgb(119, 190, 84)))
                using (var dark = new SolidBrush(Color.FromArgb(42, 50, 43)))
                using (var blue = new SolidBrush(Color.FromArgb(70, 148, 188)))
                {
                    if (name == "folder") { g.FillRectangle(green, 2, 5, 20, 16); g.FillRectangle(green, 4, 3, 8, 4); g.FillRectangle(white, 4, 8, 16, 11); }
                    else if (name == "update") { g.FillEllipse(blue, 3, 3, 18, 15); g.FillRectangle(blue, 2, 10, 20, 7); g.FillRectangle(green, 10, 10, 4, 10); g.FillPolygon(green, new Point[] { new Point(6, 17), new Point(18, 17), new Point(12, 23) }); }
                    else if (name == "install") { g.FillRectangle(green, 2, 3, 19, 18); g.FillRectangle(white, 4, 5, 15, 14); g.FillPolygon(green, new Point[] { new Point(9, 7), new Point(17, 12), new Point(9, 17) }); }
                    else if (name == "help") { g.FillEllipse(white, 2, 2, 20, 20); g.FillEllipse(dark, 5, 5, 14, 14); using (var f = new Font("Segoe UI", 12, FontStyle.Bold, GraphicsUnit.Pixel)) g.DrawString("?", f, white, 8, 4); }
                    else { g.FillRectangle(green, 3, 3, 18, 18); g.FillRectangle(dark, 6, 6, 12, 12); using (var f = new Font("Segoe UI", 12, FontStyle.Bold, GraphicsUnit.Pixel)) g.DrawString("×", f, white, 5, 4); }
                }
            }
            return image;
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
            checkingApp = true; appUpdate.Enabled = toolApp.Enabled = false;
            try
            {
                InstallerRelease update = await Task.Run(() => SelfUpdater.FindUpdate(new WebTransfer(), SelfUpdater.CurrentTag));
                if (IsDisposed || busy) return;
                appUpdate.Text = "Installer update";
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
                else if (!IsDisposed) appUpdate.Text = "Retry app update";
            }
            finally { checkingApp = false; if (!IsDisposed) { appUpdate.Enabled = !busy; toolApp.Enabled = !busy; } }
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
                folderStatus.ForeColor = Green;
                valid = true;
            }
            catch (Exception ex)
            {
                folderStatus.Text = ex.Message;
                folderStatus.ForeColor = Color.FromArgb(241, 176, 87);
                valid = false;
            }
            check.Enabled = valid && !busy;
            install.Enabled = valid && !busy;
            openFolder.Enabled = valid && !busy;
            toolOpen.Enabled = valid && !busy;
            toolCheck.Enabled = valid && !busy;
            toolBrowse.Enabled = !busy;
            sideBrowse.Enabled = !busy;
            toolApp.Enabled = !busy && !checkingApp;
        }

        private async Task RunOperation(bool doInstall)
        {
            string path = pathBox.Text;
            SavePath();
            busy = true;
            pathBox.Enabled = browse.Enabled = close.Enabled = appUpdate.Enabled = false;
            UpdateFolderStatus();
            progress.Visible = true;
            result.ForeColor = Foreground;
            result.Text = doInstall ? "Downloading and verifying the latest preview..." : "Checking the latest preview...";
            try
            {
                string message = await Task.Run(() => doInstall ? PackInstaller.Install(new WebTransfer(), path) : PackInstaller.Check(new WebTransfer(), path));
                result.Text = message;
                result.ForeColor = Green;
            }
            catch (Exception ex) { ShowError(ex); }
            finally
            {
                progress.Visible = false;
                busy = false;
                pathBox.Enabled = browse.Enabled = close.Enabled = true;
                appUpdate.Enabled = !checkingApp;
                UpdateFolderStatus();
            }
        }

        private void ShowError(Exception ex)
        {
            result.ForeColor = Color.FromArgb(255, 142, 142);
            result.Text = "Unable to finish. See the error for details.";
            MessageBox.Show(this, ex.Message, "HyperRealism Installer", MessageBoxButtons.OK, MessageBoxIcon.Warning);
        }

        internal void SmokeTest()
        {
            if (String.IsNullOrEmpty(pathBox.Text)) throw new Exception("No default or saved path.");
            pathBox.Text = Path.GetTempPath();
            if (install.Enabled || check.Enabled) throw new Exception("Invalid folder was accepted.");
            if (String.IsNullOrEmpty(folderStatus.Text) || install.Text != "Install / Update" || check.Text != "Check for updates" || toolBrowse.Text != "Choose folder" || BackColor != Background) throw new Exception("Expected launcher-style installer controls are missing.");
        }
    }
}
