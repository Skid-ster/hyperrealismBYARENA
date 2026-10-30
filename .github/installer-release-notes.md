## Windows GUI installer 11 — HyperRealism 512×512

Installer 11 fixes a self-update version-reporting defect that could make an
updated installer repeatedly offer the same update. It now identifies itself
as `v0.1.0-installer.11` and can advance from installer 9 or 10 without
repeating the prompt.

The installer UI consistently names the pack **HyperRealism 512×512** and
links to the current shader setup guide. The executable filename, installed
settings directory, and legacy preview ZIP asset name remain `HyperRealism128`
for compatibility with existing updater installations. The Minecraft resource
pack itself displays the accurate 512×512 name.

The app is a separate installer, not PineconeMC. On Windows 11, download
**HyperRealism128-Installer.exe** and double-click it. The GUI defaults to the
ElyPrism `26.3(1)` instance's resourcepacks folder and allows browsing to a
different existing `resourcepacks` directory. A valid folder name does not
prove Minecraft is using that instance; enable the pack in Minecraft after
installing it.

The app checks for newer installer versions at startup and hourly while open,
and can check for a different latest preview when one is installed. It asks
before replacing either file. There is no service, scheduled task, or updater
while the app is closed. Installer version 1 cannot self-update; later versions
can replace themselves using a temporary helper. If the executable is in a
protected folder, move it to a writable folder before updating.

The preview remains experimental and partial. LabPBR effects need a compatible
shader and correct shader/resource-pack settings; no in-game appearance or
target-hardware performance is guaranteed. The installer executable is
unsigned, so Windows may show a SmartScreen warning. Its checksum is published
in the same release and is not an independent signature.
