using System;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.IO;
using System.Net;
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
                try
                {
                    using (var form = new InstallerForm("--capture-ui"))
                    {
                        form.Show(); Application.DoEvents();
                        form.SmokeTest();
                        form.Close();
                    }
                    return 0;
                }
                catch (Exception ex) { try { File.WriteAllText(Path.Combine(Path.GetTempPath(), "hyperrealism-gui-smoke-error.txt"), ex.ToString()); } catch { } return 1; }
            }
            Application.Run(new InstallerForm(args.Length == 1 ? args[0] : ""));
            return 0;
        }
    }

    // Rectangular, flat controls modeled on the launcher’s Qt/Fusion dark UI.
    // No rounded regions: they produced black corner artifacts on Windows.
    internal sealed class LauncherButton : Button
    {
        private bool hovered;
        private bool pressed;
        public bool DropArrow { get; set; }
        public bool Primary { get; set; }
        public bool CenterContent { get; set; }
        public bool DrawBorder { get; set; }
        public Color AccentColor { get; set; }
        public Color AccentTextColor { get; set; }

        public LauncherButton()
        {
            DrawBorder = true;
            AccentColor = Color.FromArgb(150, 219, 89);
            AccentTextColor = Color.Black;
        }
        protected override void OnMouseEnter(EventArgs e) { base.OnMouseEnter(e); hovered = true; Invalidate(); }
        protected override void OnMouseLeave(EventArgs e) { base.OnMouseLeave(e); hovered = false; pressed = false; Invalidate(); }
        protected override void OnMouseDown(MouseEventArgs e) { base.OnMouseDown(e); if (e.Button == MouseButtons.Left) { pressed = true; Invalidate(); } }
        protected override void OnMouseUp(MouseEventArgs e) { base.OnMouseUp(e); pressed = false; Invalidate(); }
        protected override void OnEnabledChanged(EventArgs e) { base.OnEnabledChanged(e); Invalidate(); }

        protected override void OnPaint(PaintEventArgs e)
        {
            Graphics g = e.Graphics;
            g.SmoothingMode = SmoothingMode.AntiAlias;
            Color fill = BackColor;
            Color text = ForeColor;
            if (Primary)
            {
                fill = !Enabled ? Color.FromArgb(58, 58, 58) : pressed ? ControlPaint.Dark(AccentColor, .10f) : hovered ? ControlPaint.Light(AccentColor, .08f) : AccentColor;
                text = Enabled ? AccentTextColor : Color.FromArgb(145, 145, 145);
            }
            else if (Enabled && pressed) fill = FlatAppearance.MouseDownBackColor;
            else if (Enabled && hovered) fill = FlatAppearance.MouseOverBackColor;
            using (var brush = new SolidBrush(fill)) g.FillRectangle(brush, ClientRectangle);
            if (DrawBorder)
            {
                Color border = Enabled && hovered ? Color.FromArgb(120, 120, 120) : FlatAppearance.BorderColor;
                using (var pen = new Pen(border, 1f)) g.DrawRectangle(pen, 0, 0, Width - 1, Height - 1);
            }
            int textX = Padding.Left;
            if (Image != null)
            {
                int imageX = textX;
                int imageY = (ClientSize.Height - Image.Height) / 2;
                if (Enabled) g.DrawImageUnscaled(Image, imageX, imageY);
                else ControlPaint.DrawImageDisabled(g, Image, imageX, imageY, fill);
                textX = imageX + Image.Width + 7;
            }
            int arrowInset = DropArrow ? 16 : 0;
            var textBounds = new Rectangle(textX, 0, Math.Max(0, ClientSize.Width - textX - Padding.Right - arrowInset), ClientSize.Height);
            TextFormatFlags flags = TextFormatFlags.VerticalCenter | TextFormatFlags.SingleLine | TextFormatFlags.EndEllipsis | TextFormatFlags.NoPrefix | TextFormatFlags.NoPadding;
            flags |= CenterContent ? TextFormatFlags.HorizontalCenter : TextFormatFlags.Left;
            TextRenderer.DrawText(g, Text, Font, textBounds, Enabled ? text : Color.FromArgb(145, 145, 145), flags);
            if (DropArrow)
            {
                using (var pen = new Pen(Enabled ? Color.FromArgb(220, 220, 220) : Color.FromArgb(120, 120, 120), 1.3f))
                { g.DrawLine(pen, Width - 15, Height / 2 - 2, Width - 11, Height / 2 + 2); g.DrawLine(pen, Width - 11, Height / 2 + 2, Width - 7, Height / 2 - 2); }
            }
            if (Focused && ShowFocusCues)
            {
                using (var pen = new Pen(AccentColor, 1f)) g.DrawRectangle(pen, 1, 1, Width - 3, Height - 3);
            }
        }
    }

    internal sealed class DarkMenuColors : ProfessionalColorTable
    {
        public override Color ToolStripDropDownBackground { get { return Color.FromArgb(49, 49, 49); } }
        public override Color MenuBorder { get { return Color.FromArgb(85, 85, 85); } }
        public override Color MenuItemBorder { get { return Color.FromArgb(85, 85, 85); } }
        public override Color MenuItemSelected { get { return Color.FromArgb(60, 60, 60); } }
        public override Color MenuItemSelectedGradientBegin { get { return Color.FromArgb(60, 60, 60); } }
        public override Color MenuItemSelectedGradientEnd { get { return Color.FromArgb(60, 60, 60); } }
        public override Color ImageMarginGradientBegin { get { return Color.FromArgb(49, 49, 49); } }
        public override Color ImageMarginGradientMiddle { get { return Color.FromArgb(49, 49, 49); } }
        public override Color ImageMarginGradientEnd { get { return Color.FromArgb(49, 49, 49); } }
    }

    internal sealed class PrismGroup : Panel
    {
        public string HeaderText { get; set; }
        public PrismGroup()
        {
            HeaderText = "";
            DoubleBuffered = true;
            ResizeRedraw = true;
            Padding = new Padding(12, 28, 12, 12);
        }
        protected override void OnPaint(PaintEventArgs e)
        {
            base.OnPaint(e);
            using (var pen = new Pen(Color.FromArgb(85, 85, 85))) e.Graphics.DrawLine(pen, 0, 20, Width, 20);
            TextRenderer.DrawText(e.Graphics, HeaderText, Font, new Rectangle(2, 2, Width - 4, 18), ForeColor,
                TextFormatFlags.VerticalCenter | TextFormatFlags.Left | TextFormatFlags.SingleLine | TextFormatFlags.NoPrefix);
        }
    }

    internal sealed class InstallerForm : Form
    {
        private readonly TextBox pathBox = new TextBox();
        private readonly Label folderStatus = new Label();
        private readonly Label result = new Label();
        private readonly ProgressBar progress = new ProgressBar();
        private readonly LauncherButton cancelDownload = new LauncherButton();
        private readonly Button browse = new Button();
        private readonly LauncherButton openFolder = new LauncherButton();
        private readonly LauncherButton install = new LauncherButton();
        private readonly LauncherButton installShader = new LauncherButton();
        private readonly LauncherButton launch = new LauncherButton();
        private readonly LauncherButton close = new LauncherButton();
        private readonly LauncherButton toolOpen = new LauncherButton();
        private readonly LauncherButton toolCheck = new LauncherButton();
        private readonly LauncherButton toolSettings = new LauncherButton();
        private readonly LauncherButton toolHelp = new LauncherButton();
        private readonly ToolTip tooltips = new ToolTip();
        private static readonly Color Background = Color.FromArgb(49, 49, 49);
        private static readonly Color Surface = Color.FromArgb(49, 49, 49);
        private static readonly Color Sidebar = Color.FromArgb(42, 42, 42);
        private static readonly Color CardSurface = Color.FromArgb(34, 34, 34);
        private static readonly Color ButtonSurface = Color.FromArgb(48, 48, 48);
        private static readonly Color Border = Color.FromArgb(85, 85, 85);
        private static readonly Color Foreground = Color.White;
        private static readonly Color Muted = Color.FromArgb(190, 190, 190);
        private static readonly Color Green = Color.FromArgb(150, 219, 89);
        private static readonly Color GreenSoft = Color.FromArgb(42, 42, 42);
        private static readonly Color Amber = Color.FromArgb(255, 170, 0);
        private static readonly Color AmberSoft = Color.FromArgb(42, 42, 42);
        private readonly Timer updateTimer = new Timer();
        private readonly bool captureMode;
        private bool busy;
        private bool checkingApp;
        private volatile bool cancelTransfer;
        private int transferGeneration;
        private bool automaticUpdates = true;
        private string deferredInstallerTag;
        private string deferredPackUpdate;
        private string deferredShaderUpdate;
        private static readonly string AutoCheckPath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "HyperRealism128", "auto-update-checks.txt");
        private static readonly string LauncherPath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "HyperRealism128", "pineconemc-path.txt");
        private static readonly string SettingsPath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "HyperRealism128", "resourcepacks-path.txt");

        internal InstallerForm() : this("") { }

        internal InstallerForm(string startup)
        {
            captureMode = startup == "--capture-ui";
            Text = "HyperRealism 512×512 Installer";
            using (var bitmap = BlockIcon(32))
            {
                IntPtr handle = bitmap.GetHicon();
                try { Icon = (Icon)System.Drawing.Icon.FromHandle(handle).Clone(); }
                finally { DestroyIcon(handle); }
            }
            StartPosition = FormStartPosition.CenterScreen;
            ClientSize = new Size(920, 600);
            MinimumSize = new Size(860, 540);
            Font = new Font("Segoe UI", 9F);
            AutoScaleMode = AutoScaleMode.Font;
            BackColor = Background;
            ForeColor = Foreground;

            var root = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 1, RowCount = 3, Margin = Padding.Empty, Padding = Padding.Empty, BackColor = Background };
            root.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            root.RowStyles.Add(new RowStyle(SizeType.Absolute, 42));
            root.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
            root.RowStyles.Add(new RowStyle(SizeType.Absolute, 26));
            Controls.Add(root);

            var toolbar = new Panel { Dock = DockStyle.Fill, BackColor = Surface, Padding = new Padding(8, 0, 8, 0), Margin = Padding.Empty };
            var tools = new FlowLayoutPanel { Dock = DockStyle.Fill, WrapContents = false, FlowDirection = FlowDirection.LeftToRight, Padding = new Padding(0, 5, 0, 5), Margin = Padding.Empty, BackColor = Surface };
            StyleButton(toolOpen, "Folders", 104, 32, true); toolOpen.DropArrow = true; toolOpen.Image = GlyphIcon("folders", 16, Foreground); toolOpen.Margin = new Padding(0, 0, 4, 0);
            StyleButton(toolSettings, "Settings", 104, 32, true); toolSettings.Image = GlyphIcon("settings", 16, Foreground); toolSettings.Margin = new Padding(0, 0, 4, 0);
            StyleButton(toolHelp, "Help", 92, 32, true); toolHelp.DropArrow = true; toolHelp.Image = GlyphIcon("help", 16, Foreground); toolHelp.Margin = new Padding(0, 0, 4, 0);
            StyleButton(toolCheck, "Check updates", 144, 32, true); toolCheck.Image = GlyphIcon("update", 16, Foreground);
            tools.Controls.Add(toolOpen); tools.Controls.Add(toolSettings); tools.Controls.Add(toolHelp); tools.Controls.Add(toolCheck);
            toolbar.Controls.Add(tools);
            toolbar.Paint += delegate(object sender, PaintEventArgs e) { using (var pen = new Pen(Border)) e.Graphics.DrawLine(pen, 0, toolbar.Height - 1, toolbar.Width, toolbar.Height - 1); };
            root.Controls.Add(toolbar, 0, 0);
            tooltips.SetToolTip(toolCheck, "Check for installer, resource-pack and installed shader updates.");

            var main = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 2, RowCount = 1, Margin = Padding.Empty, Padding = new Padding(8, 8, 8, 8), BackColor = Background };
            main.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            main.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 230));
            root.Controls.Add(main, 0, 1);

            var content = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 1, RowCount = 4, Margin = new Padding(0, 0, 8, 0), Padding = Padding.Empty, BackColor = Background };
            content.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            content.RowStyles.Add(new RowStyle(SizeType.Absolute, 132));
            content.RowStyles.Add(new RowStyle(SizeType.Absolute, 132));
            content.RowStyles.Add(new RowStyle(SizeType.Absolute, 104));
            content.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
            main.Controls.Add(content, 0, 0);

            var packGroup = new PrismGroup { HeaderText = "Resource pack", Dock = DockStyle.Fill, BackColor = Background, ForeColor = Foreground, Margin = new Padding(0, 0, 0, 8) };
            var packLayout = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 2, RowCount = 1, Margin = Padding.Empty, BackColor = Background };
            packLayout.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 78)); packLayout.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            packLayout.Controls.Add(new PictureBox { Image = BlockIcon(64), Size = new Size(64, 64), SizeMode = PictureBoxSizeMode.Normal, Anchor = AnchorStyles.Left | AnchorStyles.Top, AccessibleName = "HyperRealism resource-pack icon" }, 0, 0);
            var packCopy = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 1, RowCount = 3, Margin = Padding.Empty, Padding = Padding.Empty, BackColor = Background };
            packCopy.RowStyles.Add(new RowStyle(SizeType.Absolute, 28)); packCopy.RowStyles.Add(new RowStyle(SizeType.Absolute, 24)); packCopy.RowStyles.Add(new RowStyle(SizeType.Absolute, 24));
            packCopy.Controls.Add(new Label { Text = "HyperRealism 512×512", Dock = DockStyle.Fill, ForeColor = Foreground, Font = new Font("Segoe UI", 11F, FontStyle.Bold), TextAlign = ContentAlignment.MiddleLeft }, 0, 0);
            packCopy.Controls.Add(new Label { Text = "Experimental LabPBR resource pack for Minecraft Java 26.3.", Dock = DockStyle.Fill, ForeColor = Muted, TextAlign = ContentAlignment.MiddleLeft, AutoEllipsis = true }, 0, 1);
            packCopy.Controls.Add(new Label { Text = "Partial coverage  ·  LabPBR 1.3  ·  CC0 sources", Dock = DockStyle.Fill, ForeColor = Muted, TextAlign = ContentAlignment.MiddleLeft, AutoEllipsis = true }, 0, 2);
            packLayout.Controls.Add(packCopy, 1, 0); packGroup.Controls.Add(packLayout); content.Controls.Add(packGroup, 0, 0);

            var shaderGroup = new PrismGroup { HeaderText = "Optional shader", Dock = DockStyle.Fill, BackColor = Background, ForeColor = Foreground, Margin = new Padding(0, 0, 0, 8) };
            var shaderLayout = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 2, RowCount = 1, Margin = Padding.Empty, BackColor = Background };
            shaderLayout.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 78)); shaderLayout.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            shaderLayout.Controls.Add(new PictureBox { Image = ShaderIcon(64), Size = new Size(64, 64), SizeMode = PictureBoxSizeMode.Normal, Anchor = AnchorStyles.Left | AnchorStyles.Top, AccessibleName = "Gilded Dusk shader icon" }, 0, 0);
            var shaderCopy = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 1, RowCount = 3, Margin = Padding.Empty, Padding = Padding.Empty, BackColor = Background };
            shaderCopy.RowStyles.Add(new RowStyle(SizeType.Absolute, 28)); shaderCopy.RowStyles.Add(new RowStyle(SizeType.Absolute, 24)); shaderCopy.RowStyles.Add(new RowStyle(SizeType.Absolute, 24));
            shaderCopy.Controls.Add(new Label { Text = "Gilded Dusk", Dock = DockStyle.Fill, ForeColor = Foreground, Font = new Font("Segoe UI", 11F, FontStyle.Bold), TextAlign = ContentAlignment.MiddleLeft }, 0, 0);
            shaderCopy.Controls.Add(new Label { Text = "Teal shadows and amber light for Iris. Installs beside the resource pack.", Dock = DockStyle.Fill, ForeColor = Muted, TextAlign = ContentAlignment.MiddleLeft, AutoEllipsis = true }, 0, 1);
            shaderCopy.Controls.Add(new Label { Text = "Iris/Sodium required  ·  LabPBR 1.3  ·  experimental", Dock = DockStyle.Fill, ForeColor = Muted, TextAlign = ContentAlignment.MiddleLeft, AutoEllipsis = true }, 0, 2);
            shaderLayout.Controls.Add(shaderCopy, 1, 0); shaderGroup.Controls.Add(shaderLayout); content.Controls.Add(shaderGroup, 0, 1);

            var folderGroup = new PrismGroup { HeaderText = "Installation folder", Dock = DockStyle.Fill, BackColor = Background, ForeColor = Foreground, Margin = new Padding(0, 0, 0, 8) };
            var folderLayout = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 1, RowCount = 2, Margin = Padding.Empty, BackColor = Background };
            folderLayout.RowStyles.Add(new RowStyle(SizeType.Absolute, 34)); folderLayout.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
            var pathRow = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 2, RowCount = 1, Margin = Padding.Empty, BackColor = Background };
            pathRow.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100)); pathRow.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 110));
            pathBox.Dock = DockStyle.Fill; pathBox.BackColor = CardSurface; pathBox.ForeColor = Foreground; pathBox.BorderStyle = BorderStyle.FixedSingle; pathBox.Font = new Font("Segoe UI", 9F); pathBox.Margin = new Padding(0, 4, 8, 4);
            StyleButton(browse, "Browse…", 110, 28, false); browse.Dock = DockStyle.Fill; browse.Margin = new Padding(0, 3, 0, 3); browse.Image = GlyphIcon("folders", 16, Foreground); browse.TextImageRelation = TextImageRelation.ImageBeforeText; browse.ImageAlign = ContentAlignment.MiddleLeft; browse.TextAlign = ContentAlignment.MiddleLeft;
            pathRow.Controls.Add(pathBox, 0, 0); pathRow.Controls.Add(browse, 1, 0); folderLayout.Controls.Add(pathRow, 0, 0);
            folderStatus.Dock = DockStyle.Fill; folderStatus.Font = new Font("Segoe UI", 9F); folderStatus.TextAlign = ContentAlignment.MiddleLeft; folderStatus.AutoEllipsis = true;
            folderLayout.Controls.Add(folderStatus, 0, 1); folderGroup.Controls.Add(folderLayout); content.Controls.Add(folderGroup, 0, 2);

            var statusGroup = new PrismGroup { HeaderText = "Status", Dock = DockStyle.Fill, BackColor = Background, ForeColor = Foreground, Margin = Padding.Empty };
            var statusLayout = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 1, RowCount = 3, Margin = Padding.Empty, BackColor = Background };
            statusLayout.RowStyles.Add(new RowStyle(SizeType.Absolute, 28)); statusLayout.RowStyles.Add(new RowStyle(SizeType.Percent, 100)); statusLayout.RowStyles.Add(new RowStyle(SizeType.Absolute, 26));
            result.Dock = DockStyle.Fill; result.Text = "Ready — choose a folder, then install the resource pack or optional shader."; result.ForeColor = Muted; result.Font = new Font("Segoe UI", 9F); result.TextAlign = ContentAlignment.TopLeft; result.AutoEllipsis = true;
            statusLayout.Controls.Add(result, 0, 0);
            var notes = new Label { Text = "Downloads are SHA-256 verified and size-capped. Only the named resource-pack or shader ZIP is replaced; other files stay untouched.\r\nThis is an experimental preview, not a full vanilla-coverage or performance-certified pack.", Dock = DockStyle.Fill, ForeColor = Muted, Font = new Font("Segoe UI", 9F), TextAlign = ContentAlignment.TopLeft };
            statusLayout.Controls.Add(notes, 0, 1);
            progress.Dock = DockStyle.Fill; progress.Style = ProgressBarStyle.Marquee; progress.MarqueeAnimationSpeed = 25; progress.Visible = false; progress.AccessibleName = "Download progress"; progress.BackColor = CardSurface; progress.ForeColor = Green; progress.Margin = new Padding(0, 4, 8, 4);
            var progressRow = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 2, RowCount = 1, Margin = Padding.Empty, BackColor = Background };
            progressRow.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100)); progressRow.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 90));
            StyleButton(cancelDownload, "Cancel", 90, 24, false); cancelDownload.Dock = DockStyle.Fill; cancelDownload.Visible = false; cancelDownload.Margin = Padding.Empty;
            cancelDownload.AccessibleName = "Cancel current update check or download";
            tooltips.SetToolTip(cancelDownload, "Cancel the current check or download. Existing installed files are preserved.");
            cancelDownload.Click += delegate { if (busy && !cancelTransfer) { cancelTransfer = true; cancelDownload.Enabled = false; result.ForeColor = Muted; result.Text = "Cancelling; please wait..."; } };
            progressRow.Controls.Add(progress, 0, 0); progressRow.Controls.Add(cancelDownload, 1, 0); statusLayout.Controls.Add(progressRow, 0, 2);
            statusGroup.Controls.Add(statusLayout); content.Controls.Add(statusGroup, 0, 3);

            var sidebar = new Panel { Dock = DockStyle.Fill, BackColor = Sidebar, Padding = new Padding(12, 12, 12, 12), Margin = Padding.Empty };
            var sideLayout = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 1, RowCount = 9, Margin = Padding.Empty, BackColor = Sidebar };
            sideLayout.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            sideLayout.RowStyles.Add(new RowStyle(SizeType.Absolute, 72)); sideLayout.RowStyles.Add(new RowStyle(SizeType.Absolute, 24)); sideLayout.RowStyles.Add(new RowStyle(SizeType.Absolute, 12));
            sideLayout.RowStyles.Add(new RowStyle(SizeType.Absolute, 34)); sideLayout.RowStyles.Add(new RowStyle(SizeType.Absolute, 34)); sideLayout.RowStyles.Add(new RowStyle(SizeType.Absolute, 34)); sideLayout.RowStyles.Add(new RowStyle(SizeType.Absolute, 34)); sideLayout.RowStyles.Add(new RowStyle(SizeType.Absolute, 34)); sideLayout.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
            sideLayout.Controls.Add(new PictureBox { Image = BlockIcon(64), Size = new Size(64, 64), SizeMode = PictureBoxSizeMode.Normal, Anchor = AnchorStyles.None, AccessibleName = "Selected resource pack" }, 0, 0);
            sideLayout.Controls.Add(new Label { Text = "HyperRealism 512", Dock = DockStyle.Fill, ForeColor = Foreground, TextAlign = ContentAlignment.MiddleCenter, AutoEllipsis = true }, 0, 1);
            sideLayout.Controls.Add(new Panel { Dock = DockStyle.Fill, BackColor = Border, Margin = new Padding(0, 5, 0, 5) }, 0, 2);
            StyleButton(install, "Install / Update", 200, 30, false); install.Dock = DockStyle.Fill; install.Margin = new Padding(0, 0, 0, 4); install.Primary = true; install.AccentColor = Green; install.AccentTextColor = Color.Black; install.Image = GlyphIcon("install", 16, Color.Black); install.Font = new Font("Segoe UI", 9F, FontStyle.Bold);
            StyleButton(installShader, "Install shader", 200, 30, false); installShader.Dock = DockStyle.Fill; installShader.Margin = new Padding(0, 0, 0, 4); installShader.Image = GlyphIcon("shader", 16, Foreground);
            StyleButton(launch, "Launch", 200, 30, false); launch.Dock = DockStyle.Fill; launch.Margin = new Padding(0, 0, 0, 4); launch.Image = GlyphIcon("launch", 16, Foreground);
            StyleButton(openFolder, "Open folder", 200, 30, false); openFolder.Dock = DockStyle.Fill; openFolder.Margin = new Padding(0, 0, 0, 4); openFolder.Image = GlyphIcon("folders", 16, Foreground);
            StyleButton(close, "Close", 200, 30, false); close.Dock = DockStyle.Fill; close.Margin = Padding.Empty; close.Image = GlyphIcon("close", 16, Foreground);
            sideLayout.Controls.Add(install, 0, 3); sideLayout.Controls.Add(installShader, 0, 4); sideLayout.Controls.Add(launch, 0, 5); sideLayout.Controls.Add(openFolder, 0, 6); sideLayout.Controls.Add(close, 0, 7);
            sidebar.Controls.Add(sideLayout); main.Controls.Add(sidebar, 1, 0);
            sidebar.Paint += delegate(object sender, PaintEventArgs e) { using (var pen = new Pen(Border)) e.Graphics.DrawLine(pen, 0, 0, 0, sidebar.Height); };
            tooltips.SetToolTip(install, "Install or update the experimental LabPBR resource pack in the selected resourcepacks folder.");
            tooltips.SetToolTip(installShader, "Install Gilded Dusk into the shaderpacks folder beside this instance's resourcepacks folder.");
            tooltips.SetToolTip(launch, "Launch the selected instance through PineconeMC.");
            tooltips.SetToolTip(openFolder, "Open the selected resourcepacks folder.");

            var statusbar = new Panel { Dock = DockStyle.Fill, BackColor = Sidebar, Padding = new Padding(10, 0, 10, 0), Margin = Padding.Empty };
            var statusGrid = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 2, RowCount = 1, Margin = Padding.Empty, BackColor = Sidebar };
            statusGrid.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100)); statusGrid.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 220));
            statusGrid.Controls.Add(new Label { Text = "Minecraft Java 26.3  |  LabPBR preview", Dock = DockStyle.Fill, ForeColor = Muted, TextAlign = ContentAlignment.MiddleLeft }, 0, 0);
            statusGrid.Controls.Add(new Label { Text = "Installer " + SelfUpdater.CurrentTag, Dock = DockStyle.Fill, ForeColor = Muted, TextAlign = ContentAlignment.MiddleRight }, 1, 0);
            statusbar.Controls.Add(statusGrid);
            statusbar.Paint += delegate(object sender, PaintEventArgs e) { using (var pen = new Pen(Border)) e.Graphics.DrawLine(pen, 0, 0, statusbar.Width, 0); };
            root.Controls.Add(statusbar, 0, 2);

            tooltips.SetToolTip(browse, "Choose the resourcepacks folder for the instance you want to update.");
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
            var credits = helpMenu.Items.Add("Icon credits and license");
            var about = helpMenu.Items.Add("About HyperRealism 512×512");
            guide.Click += delegate { OpenWeb("https://github.com/Skid-ster/hyperrealismBYARENA#readme"); };
            shaderGuide.Click += delegate { OpenWeb("https://github.com/Skid-ster/hyperrealismBYARENA/blob/arena/65b59fff-hyperrealismbyarena/docs/SHADER_SETUP.md"); };
            credits.Click += delegate {
                using (var stream = typeof(InstallerForm).Assembly.GetManifestResourceStream("HyperRealismInstaller.IconCredits.txt"))
                using (var reader = new StreamReader(stream)) MessageBox.Show(this, reader.ReadToEnd(), "Icon credits and license", MessageBoxButtons.OK, MessageBoxIcon.Information);
            };
            about.Click += delegate { MessageBox.Show(this, "HyperRealism 512×512 Installer " + SelfUpdater.CurrentTag + "\r\n\r\nIndependent pack and shader installer, not PineconeMC.\r\nThe resource pack and Gilded Dusk Iris shader are experimental. Use Iris with LabPBR 1.3; actual in-game shader compatibility and performance are not certified.", "About", MessageBoxButtons.OK, MessageBoxIcon.Information); };
            toolHelp.Click += delegate { helpMenu.Show(toolHelp, 0, toolHelp.Height); };
            toolCheck.Click += async delegate { await CheckAllUpdates(); };
            installShader.Click += async delegate { await RunShaderOperation(); };
            pathBox.TextChanged += delegate { UpdateFolderStatus(); if (!busy) { result.Text = "Ready. Install the resource pack or Gilded Dusk shader."; result.ForeColor = Foreground; } };
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
            launch.Click += delegate { LaunchGame(); };
            install.Click += async delegate { await RunOperation(true); };
            close.Click += delegate { Close(); };
            AcceptButton = install;
            CancelButton = close;
            FormClosing += delegate(object sender, FormClosingEventArgs e) { if (busy) e.Cancel = true; else if (!captureMode) SavePath(); };
            string defaultPath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "ElyPrismLauncher", "instances", "26.3(1)", "minecraft", "resourcepacks");
            pathBox.Text = LoadPath(defaultPath);
            if (captureMode)
            {
                string previewPath = Path.Combine(Path.GetTempPath(), "HyperRealism UI Preview", "resourcepacks");
                try { Directory.CreateDirectory(previewPath); pathBox.Text = previewPath; } catch { }
            }
            UpdateFolderStatus();
            if (startup == "--updated") result.Text = "Installer updated and reopened successfully.";
            if (startup == "--update-failed") { result.Text = "Installer update failed; the previous installer was reopened. Close other copies and check folder permissions."; result.ForeColor = Color.FromArgb(248, 113, 113); }
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
            try
            {
                int dark = 1;
                if (DwmSetWindowAttribute(Handle, 20, ref dark, sizeof(int)) != 0) DwmSetWindowAttribute(Handle, 19, ref dark, sizeof(int));
                int caption = Background.R | (Background.G << 8) | (Background.B << 16);
                int text = Foreground.R | (Foreground.G << 8) | (Foreground.B << 16);
                DwmSetWindowAttribute(Handle, 35, ref caption, sizeof(int));
                DwmSetWindowAttribute(Handle, 34, ref caption, sizeof(int));
                DwmSetWindowAttribute(Handle, 36, ref text, sizeof(int));
            }
            catch (DllNotFoundException) { } catch (EntryPointNotFoundException) { }
        }

        private static void StyleButton(Button button, string text, int width, int height, bool toolbar)
        {
            button.Text = text; button.Size = new Size(width, height); button.Margin = Padding.Empty;
            button.FlatStyle = FlatStyle.Flat; button.FlatAppearance.BorderSize = 1;
            button.FlatAppearance.BorderColor = Border;
            button.FlatAppearance.MouseOverBackColor = Color.FromArgb(60, 60, 60);
            button.FlatAppearance.MouseDownBackColor = Color.FromArgb(70, 70, 70);
            button.BackColor = toolbar ? Surface : ButtonSurface; button.ForeColor = Foreground;
            button.Image = null;
            button.TextAlign = ContentAlignment.MiddleCenter;
            button.Padding = toolbar ? new Padding(8, 0, 8, 0) : new Padding(10, 0, 10, 0);
            button.Font = new Font("Segoe UI", 9F);
            button.UseVisualStyleBackColor = false; button.AutoEllipsis = true;
            var custom = button as LauncherButton;
            if (custom != null) { custom.Primary = false; custom.CenterContent = false; custom.DrawBorder = !toolbar; custom.AccentColor = Green; custom.AccentTextColor = Color.Black; }
        }

        private static Bitmap GlyphIcon(string kind, int size, Color color)
        {
            var image = new Bitmap(size, size);
            using (var g = Graphics.FromImage(image))
            using (var pen = new Pen(color, 1.5f))
            using (var brush = new SolidBrush(color))
            {
                g.SmoothingMode = SmoothingMode.AntiAlias;
                g.Clear(Color.Transparent);
                if (kind == "folders")
                {
                    g.DrawRectangle(pen, 2, 5, 12, 8);
                    g.DrawLine(pen, 2, 5, 6, 5); g.DrawLine(pen, 6, 5, 8, 3); g.DrawLine(pen, 8, 3, 12, 3);
                }
                else if (kind == "settings")
                {
                    g.DrawEllipse(pen, 5, 5, 6, 6);
                    for (int i = 0; i < 8; i++)
                    {
                        double a = i * Math.PI / 4;
                        int x1 = 8 + (int)Math.Round(Math.Cos(a) * 5);
                        int y1 = 8 + (int)Math.Round(Math.Sin(a) * 5);
                        int x2 = 8 + (int)Math.Round(Math.Cos(a) * 7);
                        int y2 = 8 + (int)Math.Round(Math.Sin(a) * 7);
                        g.DrawLine(pen, x1, y1, x2, y2);
                    }
                }
                else if (kind == "help")
                {
                    g.DrawEllipse(pen, 2, 2, 12, 12);
                    TextRenderer.DrawText(g, "?", new Font("Segoe UI", 8F, FontStyle.Bold), new Rectangle(2, 1, 12, 13), color, TextFormatFlags.HorizontalCenter | TextFormatFlags.VerticalCenter | TextFormatFlags.NoPrefix);
                }
                else if (kind == "update")
                {
                    g.DrawArc(pen, 3, 3, 10, 10, 30, 250);
                    g.DrawLine(pen, 12, 2, 13, 6); g.DrawLine(pen, 13, 6, 9, 5);
                }
                else if (kind == "install")
                {
                    g.DrawLine(pen, 8, 2, 8, 9);
                    g.DrawLine(pen, 5, 7, 8, 10); g.DrawLine(pen, 11, 7, 8, 10);
                    g.DrawLine(pen, 3, 12, 13, 12); g.DrawLine(pen, 3, 12, 3, 14); g.DrawLine(pen, 13, 12, 13, 14);
                }
                else if (kind == "shader")
                {
                    g.DrawEllipse(pen, 3, 3, 10, 10);
                    g.FillEllipse(brush, 8, 4, 4, 4);
                    g.DrawLine(pen, 3, 10, 13, 10);
                }
                else if (kind == "launch")
                {
                    Point[] triangle = new Point[] { new Point(5, 3), new Point(12, 8), new Point(5, 13) };
                    g.FillPolygon(brush, triangle);
                }
                else if (kind == "close")
                {
                    g.DrawLine(pen, 4, 4, 12, 12); g.DrawLine(pen, 12, 4, 4, 12);
                }
            }
            return image;
        }

        // The embedded CC BY-SA grass block is rasterized and cropped from
        // Prism Launcher's instance icon. Attribution is included in Help.
        private static Bitmap BlockIcon(int size)
        {
            var image = new Bitmap(size, size);
            using (var stream = typeof(InstallerForm).Assembly.GetManifestResourceStream("HyperRealismInstaller.Grass.png"))
            {
                if (stream == null) throw new InvalidOperationException("Missing grass icon resource.");
                using (var source = Image.FromStream(stream))
                using (var g = Graphics.FromImage(image))
                {
                    g.Clear(Color.Transparent);
                    g.InterpolationMode = InterpolationMode.HighQualityBicubic;
                    g.PixelOffsetMode = PixelOffsetMode.HighQuality;
                    g.DrawImage(source, new Rectangle(0, 0, size, size));
                }
            }
            return image;
        }

        private static Bitmap ShaderIcon(int size)
        {
            var image = new Bitmap(size, size);
            using (var g = Graphics.FromImage(image))
            using (var gradient = new LinearGradientBrush(new Rectangle(0, 0, size, size), Color.FromArgb(43, 181, 169), Amber, LinearGradientMode.ForwardDiagonal))
            using (var night = new SolidBrush(Color.FromArgb(26, 39, 47)))
            using (var sun = new SolidBrush(Color.FromArgb(255, 214, 143)))
            using (var edge = new Pen(Color.FromArgb(227, 241, 236), Math.Max(1f, size / 18f)))
            {
                g.SmoothingMode = SmoothingMode.AntiAlias;
                g.Clear(Color.Transparent);
                float scale = size / 36f;
                g.ScaleTransform(scale, scale);
                FillRound(g, gradient, 1, 1, 34, 34, 9);
                g.FillEllipse(sun, 19, 6, 10, 10);
                g.FillEllipse(night, 5, 14, 25, 18);
                g.DrawLine(edge, 6, 20, 30, 20);
                g.DrawLine(edge, 9, 25, 27, 25);
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
        private void LaunchGame()
        {
            try
            {
                LaunchTarget target = LauncherIntegration.TargetFromFolder(pathBox.Text);
                string exe;
                // Launcher selection is stored separately from the resourcepacks folder.
                try { exe = File.Exists(LauncherPath) ? File.ReadAllText(LauncherPath).Trim() : ""; } catch { exe = ""; }
                if (!File.Exists(exe)) exe = LauncherIntegration.FindInstalled();
                if (String.IsNullOrEmpty(exe))
                {
                    using (var picker = new OpenFileDialog())
                    {
                        picker.Title = "Select PineconeMC (elyprismlauncher.exe)";
                        picker.Filter = "Programs (*.exe)|*.exe"; picker.CheckFileExists = true; picker.Multiselect = false;
                        if (picker.ShowDialog(this) != DialogResult.OK) return;
                        exe = picker.FileName;
                    }
                }
                using (var process = LauncherIntegration.Launch(exe, target)) { }
                try { Directory.CreateDirectory(Path.GetDirectoryName(LauncherPath)); File.WriteAllText(LauncherPath, exe); } catch { }
                result.ForeColor = Foreground;
                result.Text = "Launch requested for " + target.InstanceId + ". PineconeMC handles accounts and game startup.";
            }
            catch (Exception ex) { ShowError(ex); }
        }

        private static ContextMenuStrip CreateMenu()
        {
            var menu = new ContextMenuStrip { ShowImageMargin = false, BackColor = Surface, ForeColor = Foreground };
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
                dialog.ClientSize = new Size(430, 255); dialog.BackColor = Background; dialog.ForeColor = Foreground; dialog.Font = Font;
                var title = new Label { Text = "Update checks", Font = new Font(Font, FontStyle.Bold), AutoSize = true, Location = new Point(18, 16) };
                var toggle = new CheckBox { Text = "Check for updates at startup and hourly while open", AutoSize = true, Checked = automaticUpdates, Location = new Point(20, 48), ForeColor = Foreground };
                var detail = new Label { Text = "Updates are never installed without your confirmation.", ForeColor = Muted, AutoSize = true, Location = new Point(20, 78) };
                var launcherLabel = new Label { Text = "PineconeMC executable (blank = auto-detect)", AutoSize = true, Location = new Point(20, 116) };
                var launcherBox = new TextBox { Location = new Point(20, 141), Width = 292, BackColor = Surface, ForeColor = Foreground, BorderStyle = BorderStyle.FixedSingle };
                try { if (File.Exists(LauncherPath)) launcherBox.Text = File.ReadAllText(LauncherPath).Trim(); } catch { }
                var choose = new LauncherButton(); StyleButton(choose, "Browse", 88, 27, false); choose.Location = new Point(320, 138);
                choose.Click += delegate { using (var picker = new OpenFileDialog()) { picker.Title = "Select PineconeMC (elyprismlauncher.exe)"; picker.Filter = "Programs (*.exe)|*.exe"; picker.CheckFileExists = true; if (picker.ShowDialog(dialog) == DialogResult.OK) launcherBox.Text = picker.FileName; } };
                var save = new LauncherButton(); StyleButton(save, "Save", 88, 30, false); save.Location = new Point(237, 205);
                var cancel = new LauncherButton(); StyleButton(cancel, "Cancel", 88, 30, false); cancel.Location = new Point(330, 205);
                cancel.Click += delegate { dialog.Close(); };
                save.Click += delegate {
                    try {
                        string selected = launcherBox.Text.Trim().Trim('"');
                        if (selected.Length > 0 && (!Path.IsPathRooted(selected) || !File.Exists(selected) || !String.Equals(Path.GetExtension(selected), ".exe", StringComparison.OrdinalIgnoreCase)))
                            throw new InvalidOperationException("Choose an existing launcher .exe, or leave the field blank to auto-detect.");
                        Directory.CreateDirectory(Path.GetDirectoryName(AutoCheckPath));
                        File.WriteAllText(LauncherPath, selected);
                        File.WriteAllText(AutoCheckPath, toggle.Checked ? "on" : "off");
                        automaticUpdates = toggle.Checked; updateTimer.Enabled = automaticUpdates;
                        dialog.Close();
                    } catch (Exception ex) { MessageBox.Show(dialog, "Could not save settings: " + ex.Message, "Settings", MessageBoxButtons.OK, MessageBoxIcon.Warning); }
                };
                dialog.Controls.Add(title); dialog.Controls.Add(toggle); dialog.Controls.Add(detail); dialog.Controls.Add(launcherLabel); dialog.Controls.Add(launcherBox); dialog.Controls.Add(choose); dialog.Controls.Add(save); dialog.Controls.Add(cancel);
                dialog.AcceptButton = save; dialog.CancelButton = cancel; dialog.ShowDialog(this);
            }
        }

        private sealed class UpdateSummary
        {
            public InstallerRelease Installer;
            public string Pack;
            public string Shader;
        }

        // One toolbar action checks the installer, resource pack and shader.
        // It never installs a changed asset without user confirmation.
        private async Task CheckAllUpdates()
        {
            if (busy || checkingApp) return;
            string folder = null;
            try { folder = PackInstaller.ValidateFolder(pathBox.Text); } catch { }
            UpdateSummary summary = null;
            busy = true; pathBox.Enabled = browse.Enabled = close.Enabled = toolCheck.Enabled = false;
            UpdateFolderStatus(); progress.Visible = true; result.ForeColor = Foreground; result.Text = "Checking GitHub for installer, resource-pack and shader updates...";
            int generation = StartTransferProgress(true);
            try
            {
                summary = await Task.Run(() => {
                    var web = new WebTransfer(null, () => cancelTransfer);
                    string shader;
                    if (folder == null) shader = "Choose an existing resourcepacks folder to locate this instance's shaderpacks folder.";
                    else
                    {
                        try { shader = ShaderPackInstaller.Check(web, folder); }
                        catch (InvalidDataException ex)
                        {
                            if (!ex.Message.StartsWith("No published preview with the Iris shader ZIP", StringComparison.Ordinal)) throw;
                            shader = "Gilded Dusk shader: not published yet.";
                        }
                    }
                    return new UpdateSummary { Installer = SelfUpdater.FindUpdate(web, SelfUpdater.CurrentTag), Pack = folder == null ? "Choose an existing resourcepacks folder to check the pack." : PackInstaller.Check(web, folder), Shader = shader };
                });
            }
            catch (OperationCanceledException)
            {
                result.ForeColor = Foreground;
                result.Text = "Update check cancelled. No files were changed.";
                return;
            }
            catch (Exception ex) { ShowError(ex); return; }
            finally
            {
                busy = false;
                FinishTransferProgress(generation);
                if (!IsDisposed) { pathBox.Enabled = browse.Enabled = close.Enabled = toolCheck.Enabled = true; progress.Visible = false; UpdateFolderStatus(); }
            }
            if (IsDisposed || summary == null) return;
            result.Text = summary.Pack + "\r\n" + summary.Shader; result.ForeColor = Foreground;
            if (summary.Installer != null)
            {
                var choice = MessageBox.Show(this, "Installer " + summary.Installer.Tag + " is available. Download it and restart now?", "Installer update available", MessageBoxButtons.YesNo, MessageBoxIcon.Information);
                if (choice == DialogResult.Yes) { await ApplyInstallerUpdate(summary.Installer); return; }
                deferredInstallerTag = summary.Installer.Tag;
            }
            bool prompted = summary.Installer != null;
            if (summary.Pack.Contains("Installed file differs"))
            {
                prompted = true;
                var choice = MessageBox.Show(this, summary.Pack + "\r\n\r\nInstall the latest resource-pack preview now?", "Resource pack update available", MessageBoxButtons.YesNo, MessageBoxIcon.Information);
                if (choice == DialogResult.Yes)
                {
                    await RunOperation(true);
                    if (result.Text.StartsWith("Download cancelled", StringComparison.Ordinal)) deferredPackUpdate = summary.Pack;
                }
                else deferredPackUpdate = summary.Pack;
            }
            if (summary.Shader != null && summary.Shader.Contains("Installed file differs"))
            {
                prompted = true;
                var choice = MessageBox.Show(this, summary.Shader + "\r\n\r\nInstall the latest Gilded Dusk shader now?", "Shader update available", MessageBoxButtons.YesNo, MessageBoxIcon.Information);
                if (choice == DialogResult.Yes)
                {
                    await RunShaderOperation();
                    if (result.Text.StartsWith("Shader download cancelled", StringComparison.Ordinal)) deferredShaderUpdate = summary.Shader;
                }
                else deferredShaderUpdate = summary.Shader;
            }
            if (!prompted)
                MessageBox.Show(this, (summary.Installer == null ? "Installer: up to date." : "Installer update skipped.") + "\r\n" + summary.Pack + "\r\n" + summary.Shader, "Update check", MessageBoxButtons.OK, MessageBoxIcon.Information);
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
            int generation = 0;
            checkingApp = true; toolCheck.Enabled = false;
            if (!automatic)
            {
                busy = true;
                pathBox.Enabled = browse.Enabled = close.Enabled = false;
                UpdateFolderStatus();
                progress.Visible = true;
                result.ForeColor = Foreground;
                result.Text = "Checking GitHub for installer updates...";
                generation = StartTransferProgress(true);
            }
            try
            {
                InstallerRelease update = await Task.Run(() => SelfUpdater.FindUpdate(new WebTransfer(null, () => !automatic && cancelTransfer), SelfUpdater.CurrentTag));
                if (IsDisposed || busy && automatic || (automatic && !automaticUpdates)) return;
                if (update != null)
                {
                    bool deferred = automatic && String.Equals(deferredInstallerTag, update.Tag, StringComparison.Ordinal);
                    if (!deferred)
                    {
                        var answer = MessageBox.Show(this, "A new installer version (" + update.Tag + ") is available on GitHub.\r\n\r\nDownload it, verify it and restart the installer now?", "Installer update available", MessageBoxButtons.YesNo, MessageBoxIcon.Information);
                        if (answer == DialogResult.Yes) { await ApplyInstallerUpdate(update); return; }
                        deferredInstallerTag = update.Tag;
                        if (!automatic) { result.ForeColor = Foreground; result.Text = "Installer update " + update.Tag + " is available when you're ready."; }
                    }
                }
                else if (!automatic)
                {
                    result.ForeColor = Green;
                    result.Text = "Installer is up to date (" + SelfUpdater.CurrentTag + ").";
                    MessageBox.Show(this, "You have the latest published installer (" + SelfUpdater.CurrentTag + ").", "Installer updates", MessageBoxButtons.OK, MessageBoxIcon.Information);
                }
            }
            catch (OperationCanceledException)
            {
                if (!IsDisposed && !automatic) { result.ForeColor = Foreground; result.Text = "Installer update check cancelled."; }
            }
            catch (Exception ex)
            {
                if (!IsDisposed && !automatic) ShowError(ex);
            }
            finally
            {
                checkingApp = false;
                if (!automatic)
                {
                    busy = false;
                    FinishTransferProgress(generation);
                    if (!IsDisposed) { progress.Visible = false; pathBox.Enabled = browse.Enabled = close.Enabled = true; UpdateFolderStatus(); }
                }
                if (!IsDisposed) { toolCheck.Enabled = !busy; }
            }
            if (automatic && automaticUpdates && !IsDisposed && !busy) await CheckPackOnStartup();
        }

        private async Task CheckPackOnStartup()
        {
            string path = pathBox.Text;
            try
            {
                string folder = PackInstaller.ValidateFolder(path);
                bool hasPack = File.Exists(Path.Combine(folder, PackInstaller.PackName));
                bool hasShader = File.Exists(Path.Combine(ShaderPackInstaller.ShaderpacksFolder(folder), ShaderPackInstaller.PackName));
                if (!hasPack && !hasShader) return;
                string[] states = await Task.Run(() => {
                    var web = new WebTransfer();
                    string pack = hasPack ? PackInstaller.Check(web, folder) : null;
                    string shader = hasShader ? ShaderPackInstaller.Check(web, folder) : null;
                    return new string[] { pack, shader };
                });
                if (IsDisposed || busy || !automaticUpdates || !String.Equals(path, pathBox.Text, StringComparison.Ordinal)) return;
                string packState = states[0];
                string shaderState = states[1];
                if (packState != null && packState.Contains("Installed file differs") && !String.Equals(packState, deferredPackUpdate, StringComparison.Ordinal))
                {
                    var answer = MessageBox.Show(this, packState + "\r\n\r\nInstall the latest resource-pack preview now?", "Resource pack update available", MessageBoxButtons.YesNo, MessageBoxIcon.Information);
                    if (answer == DialogResult.Yes)
                    {
                        await RunOperation(true);
                        if (result.Text.StartsWith("Download cancelled", StringComparison.Ordinal)) deferredPackUpdate = packState;
                    }
                    else deferredPackUpdate = packState;
                }
                if (shaderState != null && shaderState.Contains("Installed file differs") && !String.Equals(shaderState, deferredShaderUpdate, StringComparison.Ordinal))
                {
                    var answer = MessageBox.Show(this, shaderState + "\r\n\r\nInstall the latest Gilded Dusk shader now?", "Shader update available", MessageBoxButtons.YesNo, MessageBoxIcon.Information);
                    if (answer == DialogResult.Yes)
                    {
                        await RunShaderOperation();
                        if (result.Text.StartsWith("Shader download cancelled", StringComparison.Ordinal)) deferredShaderUpdate = shaderState;
                    }
                    else deferredShaderUpdate = shaderState;
                }
            }
            catch { /* Offline checks are silent; the user can retry via the button. */ }
        }

        private async Task ApplyInstallerUpdate(InstallerRelease update)
        {
            string stage = null;
            bool handedOff = false;
            busy = true; pathBox.Enabled = browse.Enabled = close.Enabled = toolCheck.Enabled = false;
            UpdateFolderStatus(); progress.Visible = true;
            int generation = StartTransferProgress(true);
            result.ForeColor = Foreground;
            result.Text = "Downloading and verifying installer " + update.Tag + "...";
            try
            {
                SavePath();
                string target = Application.ExecutablePath;
                string checksum = null;
                stage = await Task.Run(() => SelfUpdater.DownloadUpdate(
                    new WebTransfer((received, total) => ReportTransferProgress(generation, "installer", received, total), () => cancelTransfer),
                    update, target, out checksum));
                if (cancelTransfer) throw new OperationCanceledException("Download cancelled.");
                using (SelfUpdater.LaunchHelper(target, stage, target, checksum)) { }
                handedOff = true;
                busy = false;
                Application.Exit();
            }
            catch (OperationCanceledException)
            {
                deferredInstallerTag = update.Tag;
                result.ForeColor = Foreground;
                result.Text = "Download cancelled. Your current installer was not changed.";
            }
            catch (Exception ex) { deferredInstallerTag = update.Tag; ShowError(ex); }
            finally
            {
                if (!handedOff && stage != null) { try { if (File.Exists(stage)) File.Delete(stage); } catch { } }
                busy = false;
                FinishTransferProgress(generation);
                if (!IsDisposed) { progress.Visible = false; pathBox.Enabled = browse.Enabled = close.Enabled = toolCheck.Enabled = true; UpdateFolderStatus(); }
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
                folderStatus.ForeColor = Amber;
                valid = false;
            }
            install.Enabled = valid && !busy;
            installShader.Enabled = valid && !busy;
            openFolder.Enabled = valid && !busy;
            toolOpen.Enabled = !busy;
            toolCheck.Enabled = !busy && !checkingApp;
            launch.Enabled = !busy;
            toolSettings.Enabled = !busy;
        }

        private async Task RunShaderOperation()
        {
            string path = pathBox.Text;
            SavePath();
            busy = true;
            pathBox.Enabled = browse.Enabled = close.Enabled = toolCheck.Enabled = false;
            UpdateFolderStatus();
            progress.Visible = true;
            int generation = StartTransferProgress(true);
            result.ForeColor = Foreground;
            result.Text = "Downloading and verifying the latest Iris shader...";
            try
            {
                string message = await Task.Run(() => ShaderPackInstaller.Install(
                    new WebTransfer((received, total) => ReportTransferProgress(generation, "shader ZIP", received, total), () => cancelTransfer), path));
                result.Text = message;
                result.ForeColor = Green;
            }
            catch (OperationCanceledException)
            {
                result.ForeColor = Foreground;
                result.Text = "Shader download cancelled. Your current shader was not changed.";
            }
            catch (Exception ex) { ShowError(ex); }
            finally
            {
                FinishTransferProgress(generation);
                progress.Visible = false;
                busy = false;
                pathBox.Enabled = browse.Enabled = close.Enabled = true;
                toolCheck.Enabled = !checkingApp && !busy;
                UpdateFolderStatus();
            }
        }

        private async Task RunOperation(bool doInstall)
        {
            string path = pathBox.Text;
            SavePath();
            busy = true;
            pathBox.Enabled = browse.Enabled = close.Enabled = toolCheck.Enabled = false;
            UpdateFolderStatus();
            progress.Visible = true;
            int generation = StartTransferProgress(doInstall);
            result.ForeColor = Foreground;
            result.Text = doInstall ? "Downloading and verifying the latest preview..." : "Checking the latest preview...";
            try
            {
                string message = await Task.Run(() => doInstall
                    ? PackInstaller.Install(new WebTransfer((received, total) => ReportTransferProgress(generation, "resource pack", received, total), () => cancelTransfer), path)
                    : PackInstaller.Check(new WebTransfer(null, () => cancelTransfer), path));
                result.Text = message;
                result.ForeColor = Green;
            }
            catch (OperationCanceledException)
            {
                result.ForeColor = Foreground;
                result.Text = doInstall ? "Download cancelled. Your current pack was not changed." : "Update check cancelled.";
            }
            catch (Exception ex) { ShowError(ex); }
            finally
            {
                FinishTransferProgress(generation);
                progress.Visible = false;
                busy = false;
                pathBox.Enabled = browse.Enabled = close.Enabled = true;
                toolCheck.Enabled = !checkingApp && !busy;
                UpdateFolderStatus();
            }
        }

        private int StartTransferProgress(bool cancellable)
        {
            transferGeneration++;
            cancelTransfer = false;
            cancelDownload.Visible = cancellable;
            cancelDownload.Enabled = cancellable;
            CancelButton = cancellable ? (IButtonControl)cancelDownload : close;
            progress.Style = ProgressBarStyle.Marquee;
            progress.MarqueeAnimationSpeed = 25;
            progress.Value = 0;
            return transferGeneration;
        }

        private void ReportTransferProgress(int generation, string label, long received, long total)
        {
            if (IsDisposed || !IsHandleCreated) return;
            try
            {
                BeginInvoke(new Action(delegate
                {
                    if (IsDisposed || !busy || generation != transferGeneration) return;
                    if (cancelTransfer) { result.ForeColor = Foreground; result.Text = "Cancelling download; please wait..."; return; }
                    result.ForeColor = Foreground;
                    if (total > 0)
                    {
                        progress.Style = ProgressBarStyle.Blocks;
                        progress.Value = (int)Math.Max(0, Math.Min(100, received * 100.0 / total));
                        result.Text = "Downloading " + label + ": " + FormatBytes(received) + " / " + FormatBytes(total);
                    }
                    else
                    {
                        progress.Style = ProgressBarStyle.Marquee;
                        result.Text = "Downloading " + label + ": " + FormatBytes(received);
                    }
                }));
            }
            catch { /* The form may be closing while a transfer completes. */ }
        }

        private void FinishTransferProgress(int generation)
        {
            if (generation != transferGeneration) return;
            transferGeneration++;
            if (IsDisposed) return;
            cancelDownload.Visible = false;
            cancelDownload.Enabled = false;
            CancelButton = close;
            progress.Style = ProgressBarStyle.Marquee;
            progress.MarqueeAnimationSpeed = 25;
            progress.Value = 0;
        }

        private static string FormatBytes(long bytes)
        {
            string[] units = { "B", "KB", "MB", "GB" };
            double value = Math.Max(0, bytes);
            int unit = 0;
            while (value >= 1024 && unit < units.Length - 1) { value /= 1024; unit++; }
            return unit == 0 ? value.ToString("0") + " " + units[unit] : value.ToString("0.0") + " " + units[unit];
        }

        private void ShowError(Exception ex)
        {
            string guidance;
            var web = ex as WebException;
            if (web == null) web = ex.InnerException as WebException;
            if (web != null) guidance = "Could not complete the request to GitHub. Check your internet connection and try again.";
            else if (ex is UnauthorizedAccessException) guidance = "Windows denied access to a file or folder. Check permissions and close any application using the pack.";
            else if (ex is InvalidDataException) guidance = "The downloaded release did not pass its size or integrity checks. Nothing unverified was installed.";
            else if (ex is DirectoryNotFoundException) guidance = "The selected folder is no longer available. Choose the resourcepacks folder for the intended Minecraft instance.";
            else guidance = "The operation could not be completed. Check the details below, then try again.";
result.ForeColor = Color.FromArgb(248, 113, 113);
            result.Text = guidance;
            MessageBox.Show(this, guidance + "\r\n\r\nDetails: " + ex.Message, "Couldn't complete that action", MessageBoxButtons.OK, MessageBoxIcon.Error);
        }

        internal void SmokeTest()
        {
            if (String.IsNullOrEmpty(pathBox.Text)) throw new Exception("No default or saved path.");
            pathBox.Text = Path.GetTempPath();
            if (install.Enabled || installShader.Enabled) throw new Exception("Invalid folder was accepted.");
            using (var icon = BlockIcon(40)) { if (icon.GetPixel(0, 0).A != 0 || icon.GetPixel(24, 24).A == 0) throw new Exception("Rounded pack icon is invalid."); }
            if (String.IsNullOrEmpty(folderStatus.Text) || install.Text != "Install / Update" || installShader.Text != "Install shader" || toolOpen.Text != "Folders" || toolSettings.Text != "Settings" || toolHelp.Text != "Help" || toolCheck.Text != "Check updates" || launch.Text != "Launch" || BackColor != Background || progress.Visible || progress.AccessibleName != "Download progress" || AcceptButton != install || CancelButton != close || cancelDownload.AccessibleName != "Cancel current update check or download") throw new Exception("Expected installer controls are missing.");
            int progressGeneration = StartTransferProgress(true);
            if (!cancelDownload.Visible || !cancelDownload.Enabled || CancelButton != cancelDownload) throw new Exception("Active operations must expose an accessible cancel action.");
            FinishTransferProgress(progressGeneration);
            if (cancelDownload.Visible || cancelDownload.Enabled || CancelButton != close) throw new Exception("The cancel action did not restore after an operation.");
            foreach (var button in new LauncherButton[] { toolOpen, toolSettings, toolHelp, toolCheck })
            {
                int arrowInset = button.DropArrow ? 16 : 0;
                int imageInset = button.Image == null ? 0 : button.Image.Width + 7;
                int available = button.Width - button.Padding.Left - button.Padding.Right - arrowInset - imageInset;
                int measured = TextRenderer.MeasureText(button.Text, button.Font, new Size(Int32.MaxValue, Int32.MaxValue), TextFormatFlags.NoPadding | TextFormatFlags.SingleLine).Width;
                if (measured > available) throw new Exception("Toolbar button is not cleanly sized: " + button.Text);
            }
            if (close.Image == null || install.Image == null || installShader.Image == null) throw new Exception("Launcher-style action icons are missing.");
            if (!toolOpen.DropArrow || !toolHelp.DropArrow || toolSettings.Text != "Settings" || toolHelp.Text != "Help" || toolCheck.Text != "Check updates") throw new Exception("Toolbar actions are missing.");
        }
    }
}
