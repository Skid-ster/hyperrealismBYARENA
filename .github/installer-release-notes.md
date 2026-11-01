## Windows GUI installer 17 — fixed launcher-style layout

Installer 17 fixes the broken parts of the installer-16 window and keeps the
PineconeMC/Prism-style dark layout.

- Browse is a native button again, so its text/icon cannot disappear.
- The resource-pack path uses a normal bordered dark text box; the gray wrapper
  band under the field is gone.
- Resource-pack and shader summaries no longer clip. The main area now has
  separate Resource pack and Optional shader groups, matching the launcher’s
  group-box style instead of leaving a large empty void.
- Status is top-aligned and includes concise verification/experimental notes.
- Right-side action list remains: Install / Update, Install shader, Launch,
  Open folder, Close.
- The CI capture now renders the normal enabled state and stores the preview
  image in the repository so the actual Windows render can be inspected.

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
