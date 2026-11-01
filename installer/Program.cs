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

    // Owner-drawn buttons keep the installer polished and readable on dark
    // Windows themes while preserving native keyboard/accessibility behavior.
    internal sealed class LauncherButton : Button
    {
        private bool hovered;
        private bool pressed;
        public bool DropArrow { get; set; }
        public bool Primary { get; set; }
        public bool CenterContent { get; set; }
        public Color AccentColor { get; set; }
        public Color AccentTextColor { get; set; }

        public LauncherButton() { AccentColor = Color.FromArgb(52, 211, 153); AccentTextColor = Color.FromArgb(6, 18, 13); }
        protected override void OnMouseEnter(EventArgs e) { base.OnMouseEnter(e); hovered = true; Invalidate(); }
        protected override void OnMouseLeave(EventArgs e) { base.OnMouseLeave(e); hovered = false; pressed = false; Invalidate(); }
        protected override void OnMouseDown(MouseEventArgs e) { base.OnMouseDown(e); if (e.Button == MouseButtons.Left) { pressed = true; Invalidate(); } }
        protected override void OnMouseUp(MouseEventArgs e) { base.OnMouseUp(e); pressed = false; Invalidate(); }
        protected override void OnEnabledChanged(EventArgs e) { base.OnEnabledChanged(e); Invalidate(); }

        private GraphicsPath RoundedPath(Rectangle bounds, int radius)
        {
            int diameter = Math.Max(2, radius * 2);
            var path = new GraphicsPath();
            path.AddArc(bounds.X, bounds.Y, diameter, diameter, 180, 90);
            path.AddArc(bounds.Right - diameter, bounds.Y, diameter, diameter, 270, 90);
            path.AddArc(bounds.Right - diameter, bounds.Bottom - diameter, diameter, diameter, 0, 90);
            path.AddArc(bounds.X, bounds.Bottom - diameter, diameter, diameter, 90, 90);
            path.CloseFigure();
            return path;
        }

        protected override void OnPaint(PaintEventArgs e)
        {
            Graphics g = e.Graphics;
            g.SmoothingMode = SmoothingMode.AntiAlias;
            Rectangle bounds = Rectangle.Inflate(ClientRectangle, -1, -1);
            Color fill = BackColor;
            Color text = ForeColor;
            if (Primary)
            {
                fill = !Enabled ? Color.FromArgb(36, 48, 58) : pressed ? ControlPaint.Dark(AccentColor, .08f) : hovered ? ControlPaint.Light(AccentColor, .08f) : AccentColor;
                text = Enabled ? AccentTextColor : Color.FromArgb(107, 118, 132);
            }
            else if (Enabled && pressed) fill = FlatAppearance.MouseDownBackColor;
            else if (Enabled && hovered) fill = FlatAppearance.MouseOverBackColor;
            using (var shape = RoundedPath(bounds, 9))
            using (var brush = new SolidBrush(fill)) g.FillPath(brush, shape);
            if (!Primary)
            {
                Color border = Enabled && hovered ? Color.FromArgb(88, 166, 140) : FlatAppearance.BorderColor;
                using (var shape = RoundedPath(bounds, 9))
                using (var pen = new Pen(border, 1f)) g.DrawPath(pen, shape);
            }
            int textX = Padding.Left;
            if (Image != null)
            {
                int imageX = CenterContent ? (ClientSize.Width - Image.Width - TextRenderer.MeasureText(Text, Font).Width - 6) / 2 : textX;
                int imageY = (ClientSize.Height - Image.Height) / 2;
                if (Enabled) g.DrawImageUnscaled(Image, imageX, imageY);
                else ControlPaint.DrawImageDisabled(g, Image, imageX, imageY, fill);
                textX = imageX + Image.Width + 6;
            }
            int arrowInset = DropArrow ? 15 : 0;
            var textBounds = new Rectangle(textX, 0, Math.Max(0, ClientSize.Width - textX - Padding.Right - arrowInset), ClientSize.Height);
            TextFormatFlags flags = TextFormatFlags.VerticalCenter | TextFormatFlags.SingleLine | TextFormatFlags.EndEllipsis | TextFormatFlags.NoPrefix | TextFormatFlags.NoPadding;
            flags |= CenterContent ? TextFormatFlags.HorizontalCenter : TextFormatFlags.Left;
            TextRenderer.DrawText(g, Text, Font, textBounds, Enabled || Primary ? text : Color.FromArgb(127, 142, 151), flags);
            if (DropArrow)
            {
                using (var pen = new Pen(Color.FromArgb(154, 167, 180), 1.4f))
                { g.DrawLine(pen, Width - 15, Height / 2 - 2, Width - 11, Height / 2 + 2); g.DrawLine(pen, Width - 11, Height / 2 + 2, Width - 7, Height / 2 - 2); }
            }
            if (Focused && ShowFocusCues) ControlPaint.DrawFocusRectangle(g, Rectangle.Inflate(ClientRectangle, -4, -4), text, fill);
        }
    }

    internal sealed class ModernCard : Panel
    {
        private int radius = 14;
        public Color BorderColor { get; set; }
        public int Radius { get { return radius; } set { radius = Math.Max(4, value); UpdateRegion(); Invalidate(); } }

        public ModernCard()
        {
            BorderColor = Color.FromArgb(42, 51, 63);
            DoubleBuffered = true;
            ResizeRedraw = true;
            SetStyle(ControlStyles.UserPaint | ControlStyles.AllPaintingInWmPaint | ControlStyles.OptimizedDoubleBuffer, true);
        }

        private GraphicsPath Shape(Rectangle bounds)
        {
            int d = Math.Min(radius * 2, Math.Min(bounds.Width, bounds.Height));
            var path = new GraphicsPath();
            path.AddArc(bounds.X, bounds.Y, d, d, 180, 90);
            path.AddArc(bounds.Right - d, bounds.Y, d, d, 270, 90);
            path.AddArc(bounds.Right - d, bounds.Bottom - d, d, d, 0, 90);
            path.AddArc(bounds.X, bounds.Bottom - d, d, d, 90, 90);
            path.CloseFigure();
            return path;
        }

        private void UpdateRegion()
        {
            if (Width <= 0 || Height <= 0) return;
            using (var path = Shape(new Rectangle(0, 0, Width, Height)))
            {
                var next = new Region(path);
                Region old = Region;
                Region = next;
                if (old != null) old.Dispose();
            }
        }

        protected override void OnResize(EventArgs e) { base.OnResize(e); UpdateRegion(); }
        protected override void OnPaintBackground(PaintEventArgs e)
        {
            e.Graphics.SmoothingMode = SmoothingMode.AntiAlias;
            using (var path = Shape(Rectangle.Inflate(ClientRectangle, -1, -1)))
            using (var brush = new SolidBrush(BackColor)) e.Graphics.FillPath(brush, path);
        }
        protected override void OnPaint(PaintEventArgs e)
        {
            e.Graphics.SmoothingMode = SmoothingMode.AntiAlias;
            using (var path = Shape(Rectangle.Inflate(ClientRectangle, -1, -1)))
            using (var pen = new Pen(BorderColor, 1f)) e.Graphics.DrawPath(pen, path);
            base.OnPaint(e);
        }
    }

    internal sealed class DarkMenuColors : ProfessionalColorTable
    {
        public override Color ToolStripDropDownBackground { get { return Color.FromArgb(18, 23, 30); } }
        public override Color MenuBorder { get { return Color.FromArgb(42, 51, 63); } }
        public override Color MenuItemBorder { get { return Color.FromArgb(42, 51, 63); } }
        public override Color MenuItemSelected { get { return Color.FromArgb(31, 41, 51); } }
        public override Color MenuItemSelectedGradientBegin { get { return Color.FromArgb(31, 41, 51); } }
        public override Color MenuItemSelectedGradientEnd { get { return Color.FromArgb(31, 41, 51); } }
        public override Color ImageMarginGradientBegin { get { return Color.FromArgb(18, 23, 30); } }
        public override Color ImageMarginGradientMiddle { get { return Color.FromArgb(18, 23, 30); } }
        public override Color ImageMarginGradientEnd { get { return Color.FromArgb(18, 23, 30); } }
    }

    internal sealed class InstallerForm : Form
    {
        private readonly TextBox pathBox = new TextBox();
        private readonly Label folderStatus = new Label();
        private readonly Label result = new Label();
        private readonly ProgressBar progress = new ProgressBar();
        private readonly LauncherButton cancelDownload = new LauncherButton();
        private readonly LauncherButton browse = new LauncherButton();
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
        private static readonly Color Background = Color.FromArgb(13, 17, 23);
        private static readonly Color Surface = Color.FromArgb(18, 23, 30);
        private static readonly Color Sidebar = Color.FromArgb(18, 23, 30);
        private static readonly Color CardSurface = Color.FromArgb(23, 29, 37);
        private static readonly Color Border = Color.FromArgb(42, 51, 63);
        private static readonly Color Foreground = Color.FromArgb(230, 237, 243);
        private static readonly Color Muted = Color.FromArgb(154, 167, 180);
        private static readonly Color Green = Color.FromArgb(52, 211, 153);
        private static readonly Color GreenSoft = Color.FromArgb(18, 53, 36);
        private static readonly Color Amber = Color.FromArgb(245, 184, 92);
        private static readonly Color AmberSoft = Color.FromArgb(51, 38, 21);
        private readonly Timer updateTimer = new Timer();
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
            Text = "HyperRealism 512×512 Installer";
            using (var bitmap = BlockIcon(32))
            {
                IntPtr handle = bitmap.GetHicon();
                try { Icon = (Icon)System.Drawing.Icon.FromHandle(handle).Clone(); }
                finally { DestroyIcon(handle); }
            }
            StartPosition = FormStartPosition.CenterScreen;
            ClientSize = new Size(980, 660);
            MinimumSize = new Size(920, 620);
            Font = new Font("Segoe UI", 9.5F);
            AutoScaleMode = AutoScaleMode.Font;
            BackColor = Background;
            ForeColor = Foreground;

            var root = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 1, RowCount = 3, Margin = Padding.Empty, Padding = Padding.Empty, BackColor = Background };
            root.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            root.RowStyles.Add(new RowStyle(SizeType.Absolute, 84));
            root.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
            root.RowStyles.Add(new RowStyle(SizeType.Absolute, 64));
            Controls.Add(root);

            var header = new Panel { Dock = DockStyle.Fill, BackColor = Surface, Padding = new Padding(24, 0, 24, 0), Margin = Padding.Empty };
            var headerGrid = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 2, RowCount = 1, Margin = Padding.Empty, Padding = Padding.Empty, BackColor = Surface };
            headerGrid.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            headerGrid.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 452));
            var brand = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 2, RowCount = 1, Margin = Padding.Empty, Padding = Padding.Empty, BackColor = Surface };
            brand.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 54)); brand.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            var brandIcon = new PictureBox { Image = BlockIcon(40), Size = new Size(40, 40), SizeMode = PictureBoxSizeMode.Normal, Anchor = AnchorStyles.Left };
            var brandCopy = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 1, RowCount = 2, Margin = Padding.Empty, Padding = new Padding(0, 18, 0, 16), BackColor = Surface };
            brandCopy.RowStyles.Add(new RowStyle(SizeType.Absolute, 26)); brandCopy.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
            brandCopy.Controls.Add(new Label { Text = "HyperRealism", Dock = DockStyle.Fill, ForeColor = Foreground, Font = new Font("Segoe UI", 16F, FontStyle.Bold), TextAlign = ContentAlignment.BottomLeft }, 0, 0);
            brandCopy.Controls.Add(new Label { Text = "512×512 LabPBR resource pack  ·  Gilded Dusk Iris shader", Dock = DockStyle.Fill, ForeColor = Muted, Font = new Font("Segoe UI", 8.5F), TextAlign = ContentAlignment.TopLeft }, 0, 1);
            brand.Controls.Add(brandIcon, 0, 0); brand.Controls.Add(brandCopy, 1, 0);
            headerGrid.Controls.Add(brand, 0, 0);

            var tools = new FlowLayoutPanel { Dock = DockStyle.Fill, WrapContents = false, FlowDirection = FlowDirection.LeftToRight, Padding = new Padding(0, 24, 0, 0), Margin = Padding.Empty, BackColor = Surface };
            StyleButton(toolCheck, "Check updates", 132, 36, true); toolCheck.Margin = new Padding(0, 0, 8, 0);
            StyleButton(toolOpen, "Folders", 92, 36, true); toolOpen.DropArrow = true; toolOpen.Margin = new Padding(0, 0, 8, 0);
            StyleButton(toolSettings, "Settings", 96, 36, true); toolSettings.Margin = new Padding(0, 0, 8, 0);
            StyleButton(toolHelp, "Help", 80, 36, true); toolHelp.DropArrow = true;
            tools.Controls.Add(toolCheck); tools.Controls.Add(toolOpen); tools.Controls.Add(toolSettings); tools.Controls.Add(toolHelp);
            headerGrid.Controls.Add(tools, 1, 0);
            header.Controls.Add(headerGrid);
            header.Paint += delegate(object sender, PaintEventArgs e) { using (var pen = new Pen(Border)) e.Graphics.DrawLine(pen, 0, header.Height - 1, header.Width, header.Height - 1); };
            root.Controls.Add(header, 0, 0);
            tooltips.SetToolTip(toolCheck, "Check for installer, resource-pack and installed shader updates.");

            var main = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 1, RowCount = 3, Margin = Padding.Empty, Padding = new Padding(24, 18, 24, 12), BackColor = Background };
            main.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            main.RowStyles.Add(new RowStyle(SizeType.Absolute, 112));
            main.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
            main.RowStyles.Add(new RowStyle(SizeType.Absolute, 96));
            root.Controls.Add(main, 0, 1);

            var location = new ModernCard { Dock = DockStyle.Fill, BackColor = CardSurface, BorderColor = Border, Radius = 14, Padding = new Padding(18, 12, 18, 10), Margin = new Padding(0, 0, 0, 12) };
            var locationGrid = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 1, RowCount = 3, Margin = Padding.Empty, Padding = Padding.Empty, BackColor = CardSurface };
            locationGrid.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            locationGrid.RowStyles.Add(new RowStyle(SizeType.Absolute, 20)); locationGrid.RowStyles.Add(new RowStyle(SizeType.Absolute, 38)); locationGrid.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
            var locationHeading = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 2, RowCount = 1, Margin = Padding.Empty, BackColor = CardSurface };
            locationHeading.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 50)); locationHeading.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 50));
            locationHeading.Controls.Add(new Label { Text = "INSTALL LOCATION", Dock = DockStyle.Fill, ForeColor = Muted, Font = new Font("Segoe UI", 8F, FontStyle.Bold), TextAlign = ContentAlignment.MiddleLeft }, 0, 0);
            locationHeading.Controls.Add(new Label { Text = "Shader installs to the sibling shaderpacks folder", Dock = DockStyle.Fill, ForeColor = Amber, Font = new Font("Segoe UI", 8F, FontStyle.Bold), TextAlign = ContentAlignment.MiddleRight }, 1, 0);
            locationGrid.Controls.Add(locationHeading, 0, 0);
            var pathRow = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 2, RowCount = 1, Margin = Padding.Empty, BackColor = CardSurface };
            pathRow.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100)); pathRow.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 110));
            pathBox.Dock = DockStyle.Fill; pathBox.BackColor = Color.FromArgb(16, 21, 27); pathBox.ForeColor = Foreground; pathBox.BorderStyle = BorderStyle.FixedSingle; pathBox.Font = new Font("Segoe UI", 9.5F); pathBox.Margin = new Padding(0, 3, 10, 3);
            StyleButton(browse, "Browse…", 110, 34, false); browse.Dock = DockStyle.Fill; browse.Margin = new Padding(0, 2, 0, 2);
            pathRow.Controls.Add(pathBox, 0, 0); pathRow.Controls.Add(browse, 1, 0); locationGrid.Controls.Add(pathRow, 0, 1);
            folderStatus.Dock = DockStyle.Fill; folderStatus.Font = new Font("Segoe UI", 8.5F, FontStyle.Bold); folderStatus.TextAlign = ContentAlignment.MiddleLeft;
            locationGrid.Controls.Add(folderStatus, 0, 2); location.Controls.Add(locationGrid); main.Controls.Add(location, 0, 0);

            var products = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 2, RowCount = 1, Margin = new Padding(0, 0, 0, 12), Padding = Padding.Empty, BackColor = Background };
            products.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 50)); products.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 50));
            main.Controls.Add(products, 0, 1);

            var resourceCard = new ModernCard { Dock = DockStyle.Fill, BackColor = CardSurface, BorderColor = Border, Radius = 14, Padding = new Padding(18, 16, 18, 16), Margin = new Padding(0, 0, 8, 0) };
            var resourceGrid = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 1, RowCount = 6, Margin = Padding.Empty, Padding = Padding.Empty, BackColor = CardSurface };
            resourceGrid.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            resourceGrid.RowStyles.Add(new RowStyle(SizeType.Absolute, 36)); resourceGrid.RowStyles.Add(new RowStyle(SizeType.Absolute, 28)); resourceGrid.RowStyles.Add(new RowStyle(SizeType.Absolute, 44)); resourceGrid.RowStyles.Add(new RowStyle(SizeType.Absolute, 20)); resourceGrid.RowStyles.Add(new RowStyle(SizeType.Percent, 100)); resourceGrid.RowStyles.Add(new RowStyle(SizeType.Absolute, 42));
            var resourceHeading = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 2, RowCount = 1, Margin = Padding.Empty, BackColor = CardSurface };
            resourceHeading.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 42)); resourceHeading.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            resourceHeading.Controls.Add(new PictureBox { Image = BlockIcon(32), Size = new Size(32, 32), SizeMode = PictureBoxSizeMode.Normal, AccessibleName = "HyperRealism resource-pack icon" }, 0, 0);
            resourceHeading.Controls.Add(new Label { Text = "RESOURCE PACK", Dock = DockStyle.Fill, ForeColor = Green, Font = new Font("Segoe UI", 8F, FontStyle.Bold), TextAlign = ContentAlignment.MiddleLeft }, 1, 0);
            resourceGrid.Controls.Add(resourceHeading, 0, 0);
            resourceGrid.Controls.Add(new Label { Text = "HyperRealism 512×512", Dock = DockStyle.Fill, ForeColor = Foreground, Font = new Font("Segoe UI", 13F, FontStyle.Bold), TextAlign = ContentAlignment.MiddleLeft }, 0, 1);
            resourceGrid.Controls.Add(new Label { Text = "LabPBR materials for Minecraft Java 26.3. Experimental partial coverage; installs one verified ZIP into resourcepacks.", Dock = DockStyle.Fill, ForeColor = Muted, Font = new Font("Segoe UI", 9F), TextAlign = ContentAlignment.TopLeft }, 0, 2);
            resourceGrid.Controls.Add(new Label { Text = "512×512  ·  LABPBR 1.3  ·  CC0 SOURCES", Dock = DockStyle.Fill, ForeColor = Green, Font = new Font("Segoe UI", 7.5F, FontStyle.Bold), TextAlign = ContentAlignment.MiddleLeft }, 0, 3);
            StyleButton(install, "Install pack", 180, 42, false); install.Primary = true; install.AccentColor = Green; install.AccentTextColor = Color.FromArgb(6, 18, 13); install.CenterContent = true; install.Font = new Font("Segoe UI", 10F, FontStyle.Bold); install.Dock = DockStyle.Fill; install.Margin = Padding.Empty;
            resourceGrid.Controls.Add(install, 0, 5); resourceCard.Controls.Add(resourceGrid); products.Controls.Add(resourceCard, 0, 0);

            var shaderCard = new ModernCard { Dock = DockStyle.Fill, BackColor = CardSurface, BorderColor = Border, Radius = 14, Padding = new Padding(18, 16, 18, 16), Margin = new Padding(8, 0, 0, 0) };
            var shaderGrid = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 1, RowCount = 6, Margin = Padding.Empty, Padding = Padding.Empty, BackColor = CardSurface };
            shaderGrid.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            shaderGrid.RowStyles.Add(new RowStyle(SizeType.Absolute, 36)); shaderGrid.RowStyles.Add(new RowStyle(SizeType.Absolute, 28)); shaderGrid.RowStyles.Add(new RowStyle(SizeType.Absolute, 44)); shaderGrid.RowStyles.Add(new RowStyle(SizeType.Absolute, 20)); shaderGrid.RowStyles.Add(new RowStyle(SizeType.Percent, 100)); shaderGrid.RowStyles.Add(new RowStyle(SizeType.Absolute, 42));
            var shaderHeading = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 2, RowCount = 1, Margin = Padding.Empty, BackColor = CardSurface };
            shaderHeading.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 42)); shaderHeading.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            shaderHeading.Controls.Add(new PictureBox { Image = ShaderIcon(32), Size = new Size(32, 32), SizeMode = PictureBoxSizeMode.Normal, AccessibleName = "Gilded Dusk shader icon" }, 0, 0);
            shaderHeading.Controls.Add(new Label { Text = "OPTIONAL SHADER", Dock = DockStyle.Fill, ForeColor = Amber, Font = new Font("Segoe UI", 8F, FontStyle.Bold), TextAlign = ContentAlignment.MiddleLeft }, 1, 0);
            shaderGrid.Controls.Add(shaderHeading, 0, 0);
            shaderGrid.Controls.Add(new Label { Text = "Gilded Dusk", Dock = DockStyle.Fill, ForeColor = Foreground, Font = new Font("Segoe UI", 13F, FontStyle.Bold), TextAlign = ContentAlignment.MiddleLeft }, 0, 1);
            shaderGrid.Controls.Add(new Label { Text = "Teal shadows and amber light for Iris. Requires Iris/Sodium and LabPBR 1.3; not yet certified in-game.", Dock = DockStyle.Fill, ForeColor = Muted, Font = new Font("Segoe UI", 9F), TextAlign = ContentAlignment.TopLeft }, 0, 2);
            shaderGrid.Controls.Add(new Label { Text = "IRIS  ·  LABPBR 1.3  ·  EXPERIMENTAL", Dock = DockStyle.Fill, ForeColor = Amber, Font = new Font("Segoe UI", 7.5F, FontStyle.Bold), TextAlign = ContentAlignment.MiddleLeft }, 0, 3);
            StyleButton(installShader, "Install shader", 180, 42, false); installShader.Primary = false; installShader.ForeColor = Amber; installShader.FlatAppearance.BorderColor = Color.FromArgb(122, 88, 42); installShader.FlatAppearance.MouseOverBackColor = Color.FromArgb(42, 35, 24); installShader.FlatAppearance.MouseDownBackColor = Color.FromArgb(52, 43, 29); installShader.CenterContent = true; installShader.Font = new Font("Segoe UI", 10F, FontStyle.Bold); installShader.Dock = DockStyle.Fill; installShader.Margin = Padding.Empty;
            shaderGrid.Controls.Add(installShader, 0, 5); shaderCard.Controls.Add(shaderGrid); products.Controls.Add(shaderCard, 1, 0);
            tooltips.SetToolTip(install, "Install the experimental LabPBR resource pack into the selected resourcepacks folder.");
            tooltips.SetToolTip(installShader, "Install Gilded Dusk into the shaderpacks folder beside this instance's resourcepacks folder.");

            var statusCard = new ModernCard { Dock = DockStyle.Fill, BackColor = CardSurface, BorderColor = Border, Radius = 14, Padding = new Padding(18, 10, 18, 10), Margin = Padding.Empty };
            var statusGrid = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 1, RowCount = 2, Margin = Padding.Empty, Padding = Padding.Empty, BackColor = CardSurface };
            statusGrid.RowStyles.Add(new RowStyle(SizeType.Percent, 100)); statusGrid.RowStyles.Add(new RowStyle(SizeType.Absolute, 30));
            result.Dock = DockStyle.Fill; result.Text = "Ready — choose a folder, then install the resource pack or optional shader."; result.ForeColor = Muted; result.Font = new Font("Segoe UI", 9F); result.TextAlign = ContentAlignment.MiddleLeft; result.AutoEllipsis = true;
            statusGrid.Controls.Add(result, 0, 0);
            progress.Dock = DockStyle.Fill; progress.Style = ProgressBarStyle.Marquee; progress.MarqueeAnimationSpeed = 25; progress.Visible = false; progress.AccessibleName = "Download progress"; progress.BackColor = CardSurface; progress.ForeColor = Green; progress.Margin = new Padding(0, 5, 10, 5);
            var progressRow = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 2, RowCount = 1, Margin = Padding.Empty, Padding = Padding.Empty, BackColor = CardSurface };
            progressRow.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100)); progressRow.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 96));
            StyleButton(cancelDownload, "Cancel", 96, 28, false); cancelDownload.Dock = DockStyle.Fill; cancelDownload.Visible = false; cancelDownload.Margin = Padding.Empty;
            cancelDownload.AccessibleName = "Cancel current update check or download";
            tooltips.SetToolTip(cancelDownload, "Cancel the current check or download. Existing installed files are preserved.");
            cancelDownload.Click += delegate { if (busy && !cancelTransfer) { cancelTransfer = true; cancelDownload.Enabled = false; result.ForeColor = Muted; result.Text = "Cancelling; please wait..."; } };
            progressRow.Controls.Add(progress, 0, 0); progressRow.Controls.Add(cancelDownload, 1, 0); statusGrid.Controls.Add(progressRow, 0, 1);
            statusCard.Controls.Add(statusGrid); main.Controls.Add(statusCard, 0, 2);

            var footer = new Panel { Dock = DockStyle.Fill, BackColor = Surface, Padding = new Padding(24, 0, 24, 0), Margin = Padding.Empty };
            var footerGrid = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 2, RowCount = 1, Margin = Padding.Empty, Padding = Padding.Empty, BackColor = Surface };
            footerGrid.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100)); footerGrid.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 360));
            footerGrid.Controls.Add(new Label { Text = "SHA-256 verified downloads  ·  only the selected ZIP is replaced", Dock = DockStyle.Fill, ForeColor = Muted, Font = new Font("Segoe UI", 8.5F), TextAlign = ContentAlignment.MiddleLeft }, 0, 0);
            var footerActions = new TableLayoutPanel { Dock = DockStyle.Fill, ColumnCount = 3, RowCount = 1, Margin = Padding.Empty, Padding = Padding.Empty, BackColor = Surface };
            footerActions.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 34)); footerActions.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 33)); footerActions.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 33));
            StyleButton(openFolder, "Open folder", 120, 36, false); openFolder.Dock = DockStyle.Fill; openFolder.Margin = new Padding(0, 14, 8, 14);
            StyleButton(launch, "Launch", 110, 36, false); launch.Dock = DockStyle.Fill; launch.Margin = new Padding(0, 14, 8, 14);
            StyleButton(close, "Close", 90, 36, false); close.Dock = DockStyle.Fill; close.Margin = new Padding(0, 14, 0, 14);
            footerActions.Controls.Add(openFolder, 0, 0); footerActions.Controls.Add(launch, 1, 0); footerActions.Controls.Add(close, 2, 0);
            footerGrid.Controls.Add(footerActions, 1, 0); footer.Controls.Add(footerGrid); root.Controls.Add(footer, 0, 2);
            footer.Paint += delegate(object sender, PaintEventArgs e) { using (var pen = new Pen(Border)) e.Graphics.DrawLine(pen, 0, 0, footer.Width, 0); };

            tooltips.SetToolTip(browse, "Choose the resourcepacks folder for the instance you want to update.");
            tooltips.SetToolTip(launch, "Launch the selected instance through PineconeMC.");
            tooltips.SetToolTip(openFolder, "Open the selected resourcepacks folder.");
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
            FormClosing += delegate(object sender, FormClosingEventArgs e) { if (busy) e.Cancel = true; else SavePath(); };
            string defaultPath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "ElyPrismLauncher", "instances", "26.3(1)", "minecraft", "resourcepacks");
            pathBox.Text = LoadPath(defaultPath);
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
            button.FlatAppearance.MouseOverBackColor = Color.FromArgb(31, 41, 51);
            button.FlatAppearance.MouseDownBackColor = Color.FromArgb(38, 49, 61);
            button.BackColor = Surface; button.ForeColor = Foreground;
            button.Image = null;
            button.TextAlign = ContentAlignment.MiddleCenter;
            button.Padding = toolbar ? new Padding(10, 0, 10, 0) : new Padding(12, 0, 12, 0);
            button.Font = new Font("Segoe UI", toolbar ? 9F : 9.5F);
            button.UseVisualStyleBackColor = false; button.AutoEllipsis = true;
            var custom = button as LauncherButton;
            if (custom != null) { custom.Primary = false; custom.CenterContent = true; custom.AccentColor = Green; custom.AccentTextColor = Color.FromArgb(6, 18, 13); }
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
            if (String.IsNullOrEmpty(folderStatus.Text) || install.Text != "Install pack" || installShader.Text != "Install shader" || toolOpen.Text != "Folders" || toolSettings.Text != "Settings" || toolHelp.Text != "Help" || toolCheck.Text != "Check updates" || launch.Text != "Launch" || BackColor != Background || progress.Visible || progress.AccessibleName != "Download progress" || AcceptButton != install || CancelButton != close || cancelDownload.AccessibleName != "Cancel current update check or download") throw new Exception("Expected installer controls are missing.");
            int progressGeneration = StartTransferProgress(true);
            if (!cancelDownload.Visible || !cancelDownload.Enabled || CancelButton != cancelDownload) throw new Exception("Active operations must expose an accessible cancel action.");
            FinishTransferProgress(progressGeneration);
            if (cancelDownload.Visible || cancelDownload.Enabled || CancelButton != close) throw new Exception("The cancel action did not restore after an operation.");
            foreach (var button in new LauncherButton[] { toolOpen, toolSettings, toolHelp, toolCheck })
            {
                int arrowInset = button.DropArrow ? 15 : 0;
                int available = button.Width - button.Padding.Left - button.Padding.Right - arrowInset;
                int measured = TextRenderer.MeasureText(button.Text, button.Font, new Size(Int32.MaxValue, Int32.MaxValue), TextFormatFlags.NoPadding | TextFormatFlags.SingleLine).Width;
                if (button.Image != null || measured > available) throw new Exception("Toolbar button is not cleanly sized: " + button.Text);
            }
            if (close.Image != null || install.Image != null || installShader.Image != null) throw new Exception("Action buttons should use clean text-only styling.");
            if (!toolOpen.DropArrow || !toolHelp.DropArrow || toolSettings.Text != "Settings" || toolHelp.Text != "Help" || toolCheck.Text != "Check updates") throw new Exception("Toolbar actions are missing.");
        }
    }
}
