## Windows GUI installer 16 — matches the PineconeMC/Prism launcher

Installer 16 replaces the generic dark dashboard with a window modeled on the
launcher’s actual dark theme. It is still a standalone installer, not PineconeMC,
but it should no longer look like a different product.

- Uses the launcher-style gray palette: window/button/base grays, white text,
  green highlight, and rectangular flat controls. No rounded neon cards.
- Top toolbar with icon + text actions: Folders, Settings, Help, Check updates.
- Main area uses launcher-style group boxes: Resource pack, Installation folder,
  and Status/progress.
- Right-side action list matches the launcher pattern: Install / Update,
  Install shader, Launch, Open folder, Close.
- Bottom status bar shows Minecraft/LabPBR context and the installer version.
- Fixes the black corner/focus artifacts from the previous custom rounded
  buttons by using rectangular Qt/Fusion-like chrome.

The existing functionality remains: verified and capped downloads, atomic
installation into the selected resourcepacks folder and sibling shaderpacks
folder, cancellable transfers, explicit confirmation before replacing changed
files, deferred automatic update prompts, launcher integration, settings, and
self-updates. Only the named resource-pack or shader ZIP is replaced; other
files are left untouched.

The resource pack and Gilded Dusk shader are experimental. Use a matching Iris
and Minecraft 26.3 setup, and keep PBR Support on LabPBR 1.3. Shader compiler
checks passed, but the shader has not been certified by in-game, GPU/driver,
visual-quality, or performance testing. Checksums detect corruption; they are
not independent digital signatures. The executable is unsigned, so Windows may
display a SmartScreen warning.

The executable filename and settings directory remain `HyperRealism128` for
updater compatibility. This is a separate installer app, not PineconeMC; it does
not install Minecraft or manage launcher instances.
