## Windows GUI installer 6 — experimental pack

This update recreates the reference toolbar with five compact, icon-led buttons,
matching spacing and dropdown indicators. Add Instance selects an existing
resourcepacks folder; Folders and Help open menus; Settings controls automatic
checks; Update checks both installer and pack. It does not create or manage
Minecraft instances. It is a separate HyperRealism app, not
PineconeMC. Download **HyperRealism128-Installer.exe** and double-click it on Windows 11.
The app pre-fills the ElyPrism `26.3(1)` instance path under the current user's
AppData, and lets you browse to a different existing `resourcepacks` folder.
A green message means the folder exists and its name matches; it does **not**
prove Minecraft is using that instance. Use **Check updates** or
**Install / Update** in the right-hand actions, then enable the pack in Minecraft. The
chosen existing folder is remembered across launches.

By default, at startup and hourly while open, the app checks for newer installer versions
and for a different latest pack when the selected folder already has the pack.
If one is found, it asks before updating. For an installer update, it downloads
and verifies the new EXE, uses a temporary CMD/helper process to replace the
old EXE, then reopens it. Use **Update** in the toolbar to check both or **Installer update** in the right-hand actions. No service or scheduled task is installed, and notifications do not
appear while the app is closed. **Version 1 cannot self-update**; download this
version 2 or later manually once. If the EXE is in a protected folder, move it to a
writable folder before updating.

The app downloads the newest published preview ZIP and its SHA-256 checksum
from this project's GitHub releases. It verifies the ZIP before replacing the
same-named installed preview. A checksum from the same release is not an
independent signature. The installer executable is unsigned, so Windows may
show a SmartScreen warning; inspect the source and checksum if desired.

The resource pack remains an experimental 106-texture preview, not the complete
pack. LabPBR effects require a compatible shader; the maps and visual results
have not been validated on the target PC or in-game. The GUI is tested with
mock downloads, a Windows construction smoke test, a live pack download, and
an isolated self-replacement/restart test. It has not been tested through a full
interactive session on the user's machine.
