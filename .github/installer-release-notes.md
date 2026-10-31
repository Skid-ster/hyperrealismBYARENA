## Windows GUI installer 12 — HyperRealism 512×512

Installer 12 improves update-transfer usability and resilience:

- Pack and installer downloads display byte-level progress and can be cancelled.
  Cancellation removes temporary downloads and leaves the currently installed
  file untouched.
- Automatic prompts for a declined or cancelled version are deferred until the
  next installer launch; manual update checks remain available.
- Pack downloads are capped at 1 GiB and installer downloads at 100 MiB. Network
  requests and reads use 30-second timeouts, and progress callbacks are
  throttled to keep the interface responsive.
- Update files remain SHA-256 checked and are staged before replacement.

The executable filename, settings directory, and legacy preview ZIP asset name
remain `HyperRealism128` for updater compatibility. The Minecraft resource pack
itself is named **HyperRealism 512×512**.

The installer is a separate app, not PineconeMC. On Windows 11, download
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
