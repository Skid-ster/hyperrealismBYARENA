using System;
using System.Drawing;
using System.Drawing.Drawing2D;
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
            if (args.Length == 2 && args[0] == "--capture-ui")
            {
                try
                {
                    using (var form = new InstallerForm("--capture-ui"))
                    {
                        form.Show(); Application.DoEvents();
                        using (var bitmap = new Bitmap(form.Width, form.Height)) { form.DrawToBitmap(bitmap, new Rectangle(0, 0, bitmap.Width, bitmap.Height)); bitmap.Save(args[1], System.Drawing.Imaging.ImageFormat.Png); }
                        form.Close();
                    }
                    return 0;
                }
                catch { return 1; }
            }
            if (args.Length == 1 && args[0] == "--smoke-test")
            {
                try { using (var form = new InstallerForm()) form.SmokeTest(); return 0; }
                catch (Exception ex) { try { File.WriteAllText(Path.Combine(Path.GetTempPath(), "hyperrealism-gui-smoke-error.txt"), ex.ToString()); } catch { } return 1; }
            }
            Application.Run(new InstallerForm(args.Length == 1 ? args[0] : ""));
            return 0;
        }
    }

    // WinForms' default disabled-button renderer makes text almost black on a
    // dark surface. Render the compact launcher-style actions ourselves while
    // retaining Button's keyboard and accessibility behavior.
    internal sealed class LauncherButton : Button
    {
        private bool hovered;
        private bool pressed;
        public bool DropArrow { get; set; }
        protected override void OnMouseEnter(EventArgs e) { base.OnMouseEnter(e); hovered = true; Invalidate(); }
        protected override void OnMouseLeave(EventArgs e) { base.OnMouseLeave(e); hovered = false; pressed = false; Invalidate(); }
        protected override void OnMouseDown(MouseEventArgs e) { base.OnMouseDown(e); if (e.Button == MouseButtons.Left) { pressed = true; Invalidate(); } }
        protected override void OnMouseUp(MouseEventArgs e) { base.OnMouseUp(e); pressed = false; Invalidate(); }
        protected override void OnEnabledChanged(EventArgs e) { base.OnEnabledChanged(e); Invalidate(); }
        protected override void OnPaint(PaintEventArgs e)
        {
            Color surface = Enabled && pressed ? FlatAppearance.MouseDownBackColor : Enabled && hovered ? FlatAppearance.MouseOverBackColor : BackColor;
            using (var brush = new SolidBrush(surface)) e.Graphics.FillRectangle(brush, ClientRectangle);
            int x = Padding.Left;
            if (Image != null)
            {
                int y = (ClientSize.Height - Image.Height) / 2;
                if (Enabled) e.Graphics.DrawImageUnscaled(Image, x, y);
                else ControlPaint.DrawImageDisabled(e.Graphics, Image, x, y, surface);
                x += Image.Width + 3;
            }
            var bounds = new Rectangle(x, 0, Math.Max(0, ClientSize.Width - x), ClientSize.Height);
            TextRenderer.DrawText(e.Graphics, Text, Font, bounds, Enabled ? ForeColor : Color.FromArgb(145, 145, 145),
                TextFormatFlags.Left | TextFormatFlags.VerticalCenter | TextFormatFlags.SingleLine | TextFormatFlags.EndEllipsis | TextFormatFlags.NoPrefix | TextFormatFlags.NoPadding);
            if (DropArrow)
            {
                using (var brush = new SolidBrush(Color.FromArgb(155, 155, 155)))
                    e.Graphics.FillPolygon(brush, new Point[] { new Point(Width - 8, Height - 8), new Point(Width - 4, Height - 8), new Point(Width - 6, Height - 6) });
            }
            if (Focused && ShowFocusCues) ControlPaint.DrawFocusRectangle(e.Graphics, Rectangle.Inflate(ClientRectangle, -2, -2), ForeColor, surface);
        }
    }

    internal sealed class ToolbarGrip : Control
    {
        public ToolbarGrip() { Width = 11; Height = 34; Margin = Padding.Empty; }
        protected override void OnPaint(PaintEventArgs e)
        {
            e.Graphics.Clear(BackColor);
            using (var brush = new SolidBrush(Color.FromArgb(110, 110, 110)))
                for (int y = 8; y <= 24; y += 3) for (int x = 3; x <= 6; x += 3) e.Graphics.FillRectangle(brush, x, y, 1, 1);
        }
    }
    internal sealed class DarkMenuColors : ProfessionalColorTable
    {
        public override Color ToolStripDropDownBackground { get { return Color.FromArgb(49, 49, 49); } }
        public override Color MenuBorder { get { return Color.FromArgb(78, 78, 78); } }
        public override Color MenuItemBorder { get { return Color.FromArgb(83, 83, 83); } }
        public override Color MenuItemSelected { get { return Color.FromArgb(65, 65, 65); } }
        public override Color MenuItemSelectedGradientBegin { get { return Color.FromArgb(65, 65, 65); } }
        public override Color MenuItemSelectedGradientEnd { get { return Color.FromArgb(65, 65, 65); } }
        public override Color ImageMarginGradientBegin { get { return Color.FromArgb(49, 49, 49); } }
        public override Color ImageMarginGradientMiddle { get { return Color.FromArgb(49, 49, 49); } }
        public override Color ImageMarginGradientEnd { get { return Color.FromArgb(49, 49, 49); } }
    }

    internal sealed class InstallerForm : Form
    {
        private readonly TextBox pathBox = new TextBox();
        private readonly Label folderStatus = new Label();
        private readonly Label result = new Label();
        private readonly ProgressBar progress = new ProgressBar();
        private readonly LauncherButton browse = new LauncherButton();
        private readonly LauncherButton openFolder = new LauncherButton();
        private readonly LauncherButton check = new LauncherButton();
        private readonly LauncherButton install = new LauncherButton();
        private readonly LauncherButton close = new LauncherButton();
        private readonly LauncherButton appUpdate = new LauncherButton();
        private readonly LauncherButton toolBrowse = new LauncherButton();
        private readonly LauncherButton toolOpen = new LauncherButton();
        private readonly LauncherButton toolCheck = new LauncherButton();
        private readonly LauncherButton toolSettings = new LauncherButton();
        private readonly LauncherButton toolHelp = new LauncherButton();
        private readonly ToolTip tooltips = new ToolTip();
        private readonly LauncherButton sideBrowse = new LauncherButton();
        private static readonly Color Background = Color.FromArgb(34, 34, 34);
        private static readonly Color Surface = Color.FromArgb(50, 50, 50);
        private static readonly Color Sidebar = Color.FromArgb(49, 49, 49);
        private static readonly Color Foreground = Color.FromArgb(224, 224, 224);
        private static readonly Color Green = Color.FromArgb(150, 219, 89);
        private readonly Timer updateTimer = new Timer();
        private bool busy;
        private bool checkingApp;
        private bool automaticUpdates = true;
        private static readonly string AutoCheckPath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "HyperRealism128", "auto-update-checks.txt");
        private static readonly string SettingsPath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "HyperRealism128", "resourcepacks-path.txt");

        internal InstallerForm() : this("") { }

        internal InstallerForm(string startup)
        {
            Text = "HyperRealism128 Installer";
            using (var bitmap = BlockIcon(32))
            {
                IntPtr handle = bitmap.GetHicon();
                try { Icon = (Icon)System.Drawing.Icon.FromHandle(handle).Clone(); }
                finally { DestroyIcon(handle); }
            }
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
            root.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            root.RowStyles.Add(new RowStyle(SizeType.Absolute, 37));
            root.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
            root.RowStyles.Add(new RowStyle(SizeType.Absolute, 27));
            root.RowStyles.Add(new RowStyle(SizeType.Absolute, 24));
            Controls.Add(root);

            var toolbar = new Panel { Dock = DockStyle.Fill, BackColor = Surface, Margin = Padding.Empty };
            var tools = new FlowLayoutPanel { Dock = DockStyle.Fill, WrapContents = false, FlowDirection = FlowDirection.LeftToRight, Padding = Padding.Empty, BackColor = Surface };
            // Match the reference's five compact buttons; each performs an
            // installer action rather than pretending to manage launcher data.
            StyleButton(toolBrowse, "Add Instance", "folder-add", 112, 34, true);
            StyleButton(toolOpen, "Folders", "folder", 82, 34, true); toolOpen.DropArrow = true;
            StyleButton(toolSettings, "Settings", "settings", 85, 34, true);
            StyleButton(toolHelp, "Help", "help", 67, 34, true); toolHelp.DropArrow = true;
            StyleButton(toolCheck, "Update", "update", 76, 34, true);
            tools.Controls.Add(new ToolbarGrip { BackColor = Surface });
            tools.Controls.Add(toolBrowse); tools.Controls.Add(toolOpen); tools.Controls.Add(toolSettings); tools.Controls.Add(toolHelp); tools.Controls.Add(toolCheck);
            tooltips.SetToolTip(toolBrowse, "Choose an existing instance's resourcepacks folder; this does not create a Minecraft instance.");
            tooltips.SetToolTip(toolCheck, "Check for installer and resource-pack updates.");
            var brand = new Panel { Dock = DockStyle.Right, Width = 150, BackColor = Surface };
            var brandIcon = new PictureBox { Image = BlockIcon(22), Size = new Size(22, 22), Location = new Point(5, 7), SizeMode = PictureBoxSizeMode.Normal };
            var brandLabel = new Label { Text = "HyperRealism128", ForeColor = Foreground, Location = new Point(31, 2), Size = new Size(115, 32), TextAlign = ContentAlignment.MiddleLeft, AutoEllipsis = true };
            brand.Controls.Add(brandIcon); brand.Controls.Add(brandLabel);
            var toolbarLayout = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 2, RowCount = 1, Margin = Padding.Empty, Padding = Padding.Empty };
            toolbarLayout.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100)); toolbarLayout.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 150));
            tools.Margin = Padding.Empty; brand.Margin = Padding.Empty; brand.Dock = DockStyle.Fill;
            toolbarLayout.Controls.Add(tools, 0, 0); toolbarLayout.Controls.Add(brand, 1, 0); toolbar.Controls.Add(toolbarLayout);
            toolbar.Paint += delegate(object sender, PaintEventArgs e) { using (var pen = new Pen(Color.FromArgb(72, 72, 72))) e.Graphics.DrawLine(pen, 0, toolbar.Height - 1, toolbar.Width, toolbar.Height - 1); };
            root.Controls.Add(toolbar, 0, 0);

            var main = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 2, RowCount = 1, Margin = Padding.Empty, BackColor = Background };
            main.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            main.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 125));
            root.Controls.Add(main, 0, 1);
            var left = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 1, RowCount = 8, Margin = Padding.Empty, Padding = new Padding(6, 0, 6, 0), BackColor = Background };
            left.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            left.RowStyles.Add(new RowStyle(SizeType.Absolute, 28));
            left.RowStyles.Add(new RowStyle(SizeType.Absolute, 117));
            left.RowStyles.Add(new RowStyle(SizeType.Absolute, 24));
            left.RowStyles.Add(new RowStyle(SizeType.Absolute, 33));
            left.RowStyles.Add(new RowStyle(SizeType.Absolute, 30));
            left.RowStyles.Add(new RowStyle(SizeType.Absolute, 42));
            left.RowStyles.Add(new RowStyle(SizeType.Absolute, 15));
            left.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
            main.Controls.Add(left, 0, 0);
            var groupHeading = new Panel { Dock = DockStyle.Fill, Margin = Padding.Empty, BackColor = Background };
            var groupLabel = new Label { Text = "⌄  Resource packs", TextAlign = ContentAlignment.MiddleLeft, ForeColor = Color.FromArgb(175, 175, 175), Font = new Font("Segoe UI", 9F, FontStyle.Bold), Dock = DockStyle.Left, Width = 124 };
            groupHeading.Controls.Add(groupLabel);
            groupHeading.Paint += delegate(object sender, PaintEventArgs e) { using (var pen = new Pen(Color.FromArgb(54, 54, 54))) e.Graphics.DrawLine(pen, 126, 14, Math.Max(126, groupHeading.Width - 4), 14); };
            left.Controls.Add(groupHeading, 0, 0);
            var tile = new Panel { Dock = DockStyle.Fill, BackColor = Background, Margin = Padding.Empty };
            var block = new PictureBox { Image = BlockIcon(40), SizeMode = PictureBoxSizeMode.Normal, Size = new Size(40, 40), Location = new Point(29, 12), AccessibleName = "HyperRealism pack icon" };
            var tileCaption = new Label { Text = "HyperRealism128", TextAlign = ContentAlignment.MiddleCenter, ForeColor = Color.FromArgb(30, 30, 30), BackColor = Green, Location = new Point(0, 63), Size = new Size(105, 22), AutoEllipsis = true };
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

            var side = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 1, RowCount = 10, Margin = Padding.Empty, Padding = new Padding(1, 0, 0, 0), BackColor = Color.FromArgb(77, 77, 77) };
            side.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            side.RowStyles.Add(new RowStyle(SizeType.Absolute, 101)); side.RowStyles.Add(new RowStyle(SizeType.Absolute, 28));
            side.RowStyles.Add(new RowStyle(SizeType.Absolute, 27)); side.RowStyles.Add(new RowStyle(SizeType.Absolute, 27));
            side.RowStyles.Add(new RowStyle(SizeType.Absolute, 7));
            for (int i = 0; i < 4; i++) side.RowStyles.Add(new RowStyle(SizeType.Absolute, 27));
            side.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
            main.Controls.Add(side, 1, 0);
            var sideIcon = new Panel { Dock = DockStyle.Fill, Margin = Padding.Empty, BackColor = Sidebar };
            var bigBlock = new PictureBox { Image = BlockIcon(67), SizeMode = PictureBoxSizeMode.Normal, Size = new Size(67, 67), Location = new Point(28, 23) };
            sideIcon.Resize += delegate { bigBlock.Left = (sideIcon.ClientSize.Width - bigBlock.Width) / 2; };
            sideIcon.Controls.Add(bigBlock);
            sideIcon.Paint += delegate(object sender, PaintEventArgs e) {
                using (var pen = new SolidBrush(Color.FromArgb(111, 111, 111)))
                    for (int x = -7; x <= 7; x += 3) for (int y = 0; y < 3; y++) e.Graphics.FillRectangle(pen, sideIcon.Width / 2 + x, 5 + y * 3, 1, 1);
            };
            side.Controls.Add(sideIcon, 0, 0);
            side.Controls.Add(new Label { Dock = DockStyle.Fill, Text = "HyperRealism128", TextAlign = ContentAlignment.MiddleCenter, ForeColor = Foreground, BackColor = Sidebar, AutoEllipsis = true }, 0, 1);
            StyleButton(install, "Install / Update", "install", 123, 27, false);
            StyleButton(check, "Check updates", "update", 123, 27, false);
            StyleButton(openFolder, "Open folder", "folder", 123, 27, false);
            StyleButton(sideBrowse, "Choose folder", "folder", 123, 27, false);
            StyleButton(appUpdate, "Installer update", "update", 123, 27, false);
            StyleButton(close, "Close", "close", 123, 27, false);
            foreach (var b in new Button[] { install, check, openFolder, sideBrowse, appUpdate, close }) { b.Dock = DockStyle.Fill; b.Margin = Padding.Empty; }
            side.Controls.Add(install, 0, 2); side.Controls.Add(check, 0, 3);
            var divider = new Panel { Dock = DockStyle.Fill, Margin = Padding.Empty, BackColor = Sidebar };
            divider.Paint += delegate(object sender, PaintEventArgs e) { using (var pen = new Pen(Color.FromArgb(69, 69, 69))) e.Graphics.DrawLine(pen, 3, 3, divider.Width - 4, 3); };
            side.Controls.Add(divider, 0, 4);
            side.Controls.Add(openFolder, 0, 5); side.Controls.Add(sideBrowse, 0, 6); side.Controls.Add(appUpdate, 0, 7); side.Controls.Add(close, 0, 8);
            side.Controls.Add(new Panel { Dock = DockStyle.Fill, BackColor = Sidebar, Margin = Padding.Empty }, 0, 9);

            var news = new Panel { Dock = DockStyle.Fill, BackColor = Surface, Margin = Padding.Empty };
            var newsLabel = new Label { Dock = DockStyle.Fill, Padding = new Padding(29, 0, 0, 0), TextAlign = ContentAlignment.MiddleLeft, ForeColor = Foreground, Text = "HyperRealism128  •  Experimental preview" };
            var newsIcon = new PictureBox { Image = Glyph("news", 18), Size = new Size(18, 18), Location = new Point(12, 4), SizeMode = PictureBoxSizeMode.CenterImage };
            var more = new LinkLabel { Text = "More info", LinkBehavior = LinkBehavior.HoverUnderline, LinkColor = Foreground, ActiveLinkColor = Green, TextAlign = ContentAlignment.MiddleRight, Dock = DockStyle.Right, Width = 90, Padding = new Padding(0, 0, 9, 0) };
            more.LinkClicked += delegate { try { System.Diagnostics.Process.Start("https://github.com/Skid-ster/hyperrealismBYARENA/releases"); } catch (Exception ex) { ShowError(ex); } };
            news.Controls.Add(newsLabel); news.Controls.Add(more); news.Controls.Add(newsIcon); newsIcon.BringToFront();
            news.Paint += delegate(object sender, PaintEventArgs e) { using (var pen = new Pen(Color.FromArgb(75, 75, 75))) e.Graphics.DrawLine(pen, 0, 0, news.Width, 0); };
            root.Controls.Add(news, 0, 2);
            var bottom = new Panel { Dock = DockStyle.Fill, BackColor = Color.FromArgb(37, 37, 37), Margin = new Padding(0, 1, 0, 0) };
            bottom.Controls.Add(new Label { Dock = DockStyle.Fill, Padding = new Padding(7, 0, 0, 0), TextAlign = ContentAlignment.MiddleLeft, ForeColor = Foreground, Text = "Minecraft Java 26.3   |   LabPBR preview" });
            bottom.Controls.Add(new Label { Dock = DockStyle.Right, Width = 170, TextAlign = ContentAlignment.MiddleRight, Padding = new Padding(0, 0, 8, 0), ForeColor = Color.FromArgb(180, 180, 180), Text = "Installer " + SelfUpdater.CurrentTag });
            root.Controls.Add(bottom, 0, 3);

            toolBrowse.Click += delegate { browse.PerformClick(); };
            sideBrowse.Click += delegate { browse.PerformClick(); };
            var folders = CreateMenu();
            var chooseFolder = folders.Items.Add("Choose resourcepacks folder...");
            var openSelected = folders.Items.Add("Open selected folder");
            var openInstances = folders.Items.Add("Open ElyPrism instances folder");
            chooseFolder.Click += delegate { browse.PerformClick(); };
            openSelected.Click += delegate { openFolder.PerformClick(); };
            openInstances.Click += delegate {
                string dir = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "ElyPrismLauncher", "instances");
                try { if (!Directory.Exists(dir)) throw new DirectoryNotFoundException("The ElyPrism instances folder was not found."); System.Diagnostics.Process.Start("explorer.exe", "\"" + dir + "\""); }
                catch (Exception ex) { ShowError(ex); }
            };
            toolOpen.Click += delegate { openSelected.Enabled = openFolder.Enabled; folders.Show(toolOpen, 0, toolOpen.Height); };
            toolSettings.Click += delegate { ShowSettings(); };
            var helpMenu = CreateMenu();
            var guide = helpMenu.Items.Add("Installer guide");
            var shaderGuide = helpMenu.Items.Add("Shader setup guide");
            var about = helpMenu.Items.Add("About HyperRealism128");
            guide.Click += delegate { OpenWeb("https://github.com/Skid-ster/hyperrealismBYARENA#readme"); };
            shaderGuide.Click += delegate { OpenWeb("https://github.com/Skid-ster/hyperrealismBYARENA/blob/arena/e000bf82-hyperrealismbyarena/docs/SHADER_SETUP.md"); };
            about.Click += delegate { MessageBox.Show(this, "HyperRealism128 Installer " + SelfUpdater.CurrentTag + "\r\n\r\nIndependent resource-pack installer, not PineconeMC.\r\nThe preview pack is experimental and requires a compatible shader for PBR.", "About", MessageBoxButtons.OK, MessageBoxIcon.Information); };
            toolHelp.Click += delegate { helpMenu.Show(toolHelp, 0, toolHelp.Height); };
            toolCheck.Click += async delegate { await CheckAllUpdates(); };
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
            automaticUpdates = LoadAutoChecks();
            if (startup != "--capture-ui" && automaticUpdates) Shown += async delegate { await CheckInstallerUpdate(true); };
            updateTimer.Interval = 60 * 60 * 1000;
            updateTimer.Tick += async delegate { if (automaticUpdates) await CheckInstallerUpdate(true); };
            updateTimer.Enabled = automaticUpdates;
            FormClosed += delegate { updateTimer.Stop(); updateTimer.Dispose(); tooltips.Dispose(); folders.Dispose(); helpMenu.Dispose(); };
        }

        [DllImport("user32.dll", SetLastError = true)]
        private static extern bool DestroyIcon(IntPtr handle);

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
            button.BackColor = toolbar ? Surface : Sidebar; button.ForeColor = Foreground;
            button.Image = Glyph(glyph, toolbar ? 24 : 18);
            button.TextImageRelation = TextImageRelation.ImageBeforeText;
            button.ImageAlign = ContentAlignment.MiddleLeft; button.TextAlign = ContentAlignment.MiddleLeft;
            button.Padding = new Padding(toolbar ? 3 : 5, 0, 0, 0);
            button.UseVisualStyleBackColor = false; button.AutoEllipsis = true;
        }

        // Independently drawn, native-size icons with crisp geometric details.
        // The curves are antialiased, but interior pixels are not resampled.
        private static Bitmap BlockIcon(int size)
        {
            var image = new Bitmap(size, size);
            using (var g = Graphics.FromImage(image))
            using (var outline = new GraphicsPath())
            using (var dirt = new LinearGradientBrush(new PointF(0, 0), new PointF(0, 48), Color.FromArgb(134, 103, 79), Color.FromArgb(112, 82, 58)))
            using (var soil = new SolidBrush(Color.FromArgb(145, 112, 81)))
            using (var soilDark = new SolidBrush(Color.FromArgb(103, 78, 61)))
            using (var grass = new LinearGradientBrush(new PointF(0, 0), new PointF(48, 12), Color.FromArgb(185, 205, 99), Color.FromArgb(153, 205, 97)))
            using (var shade = new SolidBrush(Color.FromArgb(141, 165, 79)))
            {
                g.Clear(Color.Transparent); g.ScaleTransform(size / 48f, size / 48f); g.SmoothingMode = SmoothingMode.AntiAlias;
                outline.AddArc(0, 0, 12, 12, 180, 90); outline.AddArc(36, 0, 12, 12, 270, 90);
                outline.AddArc(36, 36, 12, 12, 0, 90); outline.AddArc(0, 36, 12, 12, 90, 90); outline.CloseFigure();
                g.FillPath(dirt, outline); g.SetClip(outline); g.SmoothingMode = SmoothingMode.None;
                g.FillRectangle(soil, 4, 29, 6, 6); g.FillRectangle(soil, 24, 37, 7, 7); g.FillRectangle(soil, 38, 25, 6, 6);
                g.FillRectangle(soilDark, 15, 28, 5, 5); g.FillRectangle(soilDark, 33, 41, 4, 4);
                g.FillRectangle(shade, 0, 0, 48, 19); g.FillRectangle(grass, 0, 0, 48, 15);
                g.FillRectangle(grass, 0, 14, 10, 5); g.FillRectangle(grass, 11, 14, 6, 10);
                g.FillRectangle(grass, 25, 14, 7, 6); g.FillRectangle(grass, 37, 14, 7, 11); g.FillRectangle(grass, 44, 14, 4, 6);
                g.ResetClip();
            }
            return image;
        }
        private static void FillRound(Graphics g, Brush brush, float x, float y, float width, float height, float radius)
        {
            using (var path = new GraphicsPath())
            {
                float d = radius * 2;
                path.AddArc(x, y, d, d, 180, 90); path.AddArc(x + width - d, y, d, d, 270, 90);
                path.AddArc(x + width - d, y + height - d, d, d, 0, 90); path.AddArc(x, y + height - d, d, d, 90, 90); path.CloseFigure();
                g.FillPath(brush, path);
            }
        }
        private static Bitmap Glyph(string name, int size)
        {
            var image = new Bitmap(size, size);
            using (var g = Graphics.FromImage(image))
            {
                g.Clear(Color.Transparent); g.ScaleTransform(size / 24f, size / 24f); g.SmoothingMode = SmoothingMode.AntiAlias;
                using (var white = new SolidBrush(Color.FromArgb(242, 242, 242)))
                using (var green = new SolidBrush(Color.FromArgb(57, 181, 74)))
                using (var darkGreen = new SolidBrush(Color.FromArgb(0, 146, 69)))
                using (var dark = new SolidBrush(Color.FromArgb(36, 36, 36)))
                using (var grey = new SolidBrush(Color.FromArgb(112, 112, 112)))
                using (var blue = new SolidBrush(Color.FromArgb(102, 153, 204)))
                using (var tan = new SolidBrush(Color.FromArgb(140, 98, 57)))
                {
                    if (name == "folder" || name == "folder-add" || name == "settings")
                    {
                        // A compact green header, warm backing and light inset,
                        // drawn at the display size instead of scaled afterwards.
                        FillRound(g, tan, 1, 2, 22, 21, 3);
                        FillRound(g, green, 1, 1, 22, 8, 3);
                        if (name == "folder")
                        {
                            FillRound(g, tan, 3, 4, 10, 5, 2);
                            FillRound(g, white, 3, 6, 9, 6, 1.5f);
                            FillRound(g, white, 3, 8, 19, 13, 2);
                        }
                        else
                        {
                            FillRound(g, white, 4, 6, 16, 14, 1.5f);
                            if (name == "folder-add")
                            {
                                using (var plus = new Pen(darkGreen, 3) { StartCap = LineCap.Round, EndCap = LineCap.Round })
                                { g.DrawLine(plus, 12, 10, 12, 17); g.DrawLine(plus, 8.5f, 13.5f, 15.5f, 13.5f); }
                            }
                            else
                            {
                                g.FillRectangle(dark, 6, 8, 5, 5); g.FillRectangle(dark, 6, 15, 5, 4);
                                g.FillRectangle(white, 7.5f, 9.5f, 2, 2); g.FillRectangle(white, 7.5f, 16, 2, 2);
                                g.FillRectangle(grey, 13, 9, 5, 2); g.FillRectangle(grey, 13, 16, 5, 2);
                            }
                        }
                    }
                    else if (name == "update")
                    {
                        g.FillEllipse(blue, 0, 6, 15, 13); g.FillEllipse(blue, 6, 1, 13, 16); g.FillEllipse(blue, 14, 7, 10, 12);
                        FillRound(g, dark, 9, 10, 6, 10, 3); FillRound(g, green, 10, 12, 4, 7, 2);
                        g.FillPolygon(green, new Point[] { new Point(7, 18), new Point(17, 18), new Point(12, 23) });
                    }
                    else if (name == "install")
                    {
                        FillRound(g, green, 1, 2, 22, 20, 3); FillRound(g, white, 4, 5, 16, 14, 1.5f);
                        g.FillPolygon(darkGreen, new Point[] { new Point(9, 7), new Point(17, 12), new Point(9, 17) });
                    }
                    else if (name == "help")
                    {
                        g.FillEllipse(green, 1, 1, 22, 22); g.FillEllipse(tan, 2, 2, 20, 20); g.FillEllipse(white, 5, 5, 14, 14);
                        using (var f = new Font("Segoe UI", 14, FontStyle.Bold, GraphicsUnit.Pixel)) g.DrawString("?", f, grey, 7, 3);
                    }
                    else if (name == "news")
                    {
                        FillRound(g, green, 1, 2, 22, 16, 2); g.FillPolygon(green, new Point[] { new Point(5, 17), new Point(5, 21), new Point(10, 17) });
                        using (var line = new Pen(dark, 1.5f)) { g.DrawLine(line, 6, 7, 18, 7); g.DrawLine(line, 6, 10, 18, 10); g.DrawLine(line, 6, 13, 15, 13); }
                    }
                    else
                    {
                        FillRound(g, green, 3, 3, 18, 18, 2); g.FillRectangle(dark, 6, 6, 12, 12);
                        using (var f = new Font("Segoe UI", 12, FontStyle.Bold, GraphicsUnit.Pixel)) g.DrawString("×", f, white, 5, 4);
                    }
                }
            }
            return image;
        }

        private static ContextMenuStrip CreateMenu()
        {
            var menu = new ContextMenuStrip { ShowImageMargin = false, BackColor = Sidebar, ForeColor = Foreground };
            menu.Renderer = new ToolStripProfessionalRenderer(new DarkMenuColors());
            return menu;
        }

        private void OpenWeb(string url)
        {
            try { System.Diagnostics.Process.Start(url); }
            catch (Exception ex) { ShowError(ex); }
        }

        private static bool LoadAutoChecks()
        {
            try { if (File.Exists(AutoCheckPath)) return !String.Equals(File.ReadAllText(AutoCheckPath).Trim(), "off", StringComparison.OrdinalIgnoreCase); }
            catch { }
            return true;
        }

        private void ShowSettings()
        {
            using (var dialog = new Form())
            {
                dialog.Text = "Installer Settings"; dialog.StartPosition = FormStartPosition.CenterParent;
                dialog.FormBorderStyle = FormBorderStyle.FixedDialog; dialog.MaximizeBox = false; dialog.MinimizeBox = false;
                dialog.ClientSize = new Size(430, 175); dialog.BackColor = Background; dialog.ForeColor = Foreground; dialog.Font = Font;
                var title = new Label { Text = "Update checks", Font = new Font(Font, FontStyle.Bold), AutoSize = true, Location = new Point(18, 16) };
                var toggle = new CheckBox { Text = "Check for updates at startup and hourly while open", AutoSize = true, Checked = automaticUpdates, Location = new Point(20, 48), ForeColor = Foreground };
                var detail = new Label { Text = "Updates are never installed without your confirmation.", ForeColor = Color.FromArgb(170, 170, 170), AutoSize = true, Location = new Point(20, 78) };
                var save = new LauncherButton(); StyleButton(save, "Save", "install", 88, 30, false); save.Location = new Point(237, 125);
                var cancel = new LauncherButton(); StyleButton(cancel, "Cancel", "close", 88, 30, false); cancel.Location = new Point(330, 125);
                cancel.Click += delegate { dialog.Close(); };
                save.Click += delegate {
                    try {
                        Directory.CreateDirectory(Path.GetDirectoryName(AutoCheckPath));
                        File.WriteAllText(AutoCheckPath, toggle.Checked ? "on" : "off");
                        automaticUpdates = toggle.Checked; updateTimer.Enabled = automaticUpdates;
                        dialog.Close();
                    } catch (Exception ex) { MessageBox.Show(dialog, "Could not save settings: " + ex.Message, "Settings", MessageBoxButtons.OK, MessageBoxIcon.Warning); }
                };
                dialog.Controls.Add(title); dialog.Controls.Add(toggle); dialog.Controls.Add(detail); dialog.Controls.Add(save); dialog.Controls.Add(cancel);
                dialog.AcceptButton = save; dialog.CancelButton = cancel; dialog.ShowDialog(this);
            }
        }

        private sealed class UpdateSummary
        {
            public InstallerRelease Installer;
            public string Pack;
        }

        // One toolbar action checks both products. It never installs either one
        // without the same confirmation used by the automatic update prompts.
        private async Task CheckAllUpdates()
        {
            if (busy || checkingApp) return;
            string folder = null;
            try { folder = PackInstaller.ValidateFolder(pathBox.Text); } catch { }
            UpdateSummary summary = null;
            busy = true; pathBox.Enabled = browse.Enabled = close.Enabled = appUpdate.Enabled = false;
            UpdateFolderStatus(); progress.Visible = true; result.ForeColor = Foreground; result.Text = "Checking GitHub for installer and pack updates...";
            try
            {
                summary = await Task.Run(() => {
                    var web = new WebTransfer();
                    return new UpdateSummary { Installer = SelfUpdater.FindUpdate(web, SelfUpdater.CurrentTag), Pack = folder == null ? "Choose an existing resourcepacks folder to check the pack." : PackInstaller.Check(web, folder) };
                });
            }
            catch (Exception ex) { ShowError(ex); return; }
            finally
            {
                busy = false;
                if (!IsDisposed) { pathBox.Enabled = browse.Enabled = close.Enabled = appUpdate.Enabled = true; progress.Visible = false; UpdateFolderStatus(); }
            }
            if (IsDisposed || summary == null) return;
            result.Text = summary.Pack; result.ForeColor = Foreground;
            if (summary.Installer != null)
            {
                var choice = MessageBox.Show(this, "Installer " + summary.Installer.Tag + " is available. Download it and restart now?", "Installer update available", MessageBoxButtons.YesNo, MessageBoxIcon.Information);
                if (choice == DialogResult.Yes) { await ApplyInstallerUpdate(summary.Installer); return; }
            }
            if (summary.Pack.Contains("Installed file differs"))
            {
                var choice = MessageBox.Show(this, summary.Pack + "\r\n\r\nInstall the latest resource-pack preview now?", "Resource pack update available", MessageBoxButtons.YesNo, MessageBoxIcon.Information);
                if (choice == DialogResult.Yes) await RunOperation(true);
            }
            else MessageBox.Show(this, (summary.Installer == null ? "Installer: up to date." : "Installer update skipped.") + "\r\n" + summary.Pack, "Update check", MessageBoxButtons.OK, MessageBoxIcon.Information);
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
            if (checkingApp || busy || (automatic && !automaticUpdates)) return;
            checkingApp = true; appUpdate.Enabled = false; toolCheck.Enabled = false;
            try
            {
                InstallerRelease update = await Task.Run(() => SelfUpdater.FindUpdate(new WebTransfer(), SelfUpdater.CurrentTag));
                if (IsDisposed || busy || (automatic && !automaticUpdates)) return;
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
            finally { checkingApp = false; if (!IsDisposed) { appUpdate.Enabled = !busy; toolCheck.Enabled = !busy; } }
            if (automatic && automaticUpdates && !IsDisposed && !busy) await CheckPackOnStartup();
        }

        private async Task CheckPackOnStartup()
        {
            string path = pathBox.Text;
            try
            {
                string folder = PackInstaller.ValidateFolder(path);
                if (!File.Exists(Path.Combine(folder, PackInstaller.PackName))) return;
                string state = await Task.Run(() => PackInstaller.Check(new WebTransfer(), folder));
                if (IsDisposed || busy || !automaticUpdates || !String.Equals(path, pathBox.Text, StringComparison.Ordinal)) return;
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
            toolOpen.Enabled = !busy;
            toolCheck.Enabled = !busy && !checkingApp;
            toolBrowse.Enabled = !busy;
            toolSettings.Enabled = !busy;
            sideBrowse.Enabled = !busy;
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
            using (var icon = BlockIcon(40)) { if (icon.GetPixel(0, 0).A != 0 || icon.GetPixel(24, 24).A == 0) throw new Exception("Rounded pack icon is invalid."); }
            if (String.IsNullOrEmpty(folderStatus.Text) || install.Text != "Install / Update" || check.Text != "Check updates" || toolBrowse.Text != "Add Instance" || BackColor != Background) throw new Exception("Expected launcher-style installer controls are missing.");
            foreach (var button in new LauncherButton[] { toolBrowse, toolOpen, toolSettings, toolHelp, toolCheck })
                if (button.Image == null || TextRenderer.MeasureText(button.Text, button.Font, new Size(Int32.MaxValue, Int32.MaxValue), TextFormatFlags.NoPadding | TextFormatFlags.SingleLine).Width > button.Width - button.Padding.Left - button.Image.Width - 3)
                    throw new Exception("Toolbar button is missing an icon or has clipped text: " + button.Text);
            if (!toolOpen.DropArrow || !toolHelp.DropArrow || toolSettings.Text != "Settings" || toolHelp.Text != "Help" || toolCheck.Text != "Update") throw new Exception("Toolbar actions are missing.");
        }
    }
}
