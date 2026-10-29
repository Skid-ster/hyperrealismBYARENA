## Windows GUI installer — experimental pack

Download **HyperRealism128-Installer.exe** and double-click it on Windows 11.
The app pre-fills the ElyPrism `26.3(1)` instance path under the current user's
AppData, and lets you browse to a different existing `resourcepacks` folder.
A green message means the folder exists and its name matches; it does **not**
prove Minecraft is using that instance. Use **Check for updates** or
**Install / Update** at the bottom, then enable the pack in Minecraft.

The app downloads the newest published preview ZIP and its SHA-256 checksum
from this project's GitHub releases. It verifies the ZIP before replacing the
same-named installed preview. A checksum from the same release is not an
independent signature. The installer executable is unsigned, so Windows may
show a SmartScreen warning; inspect the source and checksum if desired.

The resource pack remains an experimental 106-texture preview, not the complete
pack. LabPBR effects require a compatible shader; the maps and visual results
have not been validated on the target PC or in-game. The GUI is tested with
mock downloads and a Windows construction smoke test, not through a full
interactive session on the user's machine.
