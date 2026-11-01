## Windows GUI installer 14 — a modern HyperRealism setup experience

Installer 14 refreshes the Windows installer with a larger, polished dark UI
while preserving existing install, update, cancellation, settings, folder,
launcher, and self-update behavior.

- A responsive two-column layout presents the resource pack and optional Gilded
  Dusk Iris shader as separate products, with clear experimental and LabPBR
  requirements.
- Instance destination selection, folder validation, quick-start guidance,
  download status/progress, and instance actions are visible together.
- Custom rounded cards and standard owner-drawn buttons use accessible contrast,
  keyboard behavior, tooltips, and familiar controls; the previous Add Instance
  action remains absent.
- The Windows CI build renders the full window to a PNG smoke artifact in
  addition to compiling the app, running backend tests, and checking UI
  construction. This image verifies that the UI renders, not a substitute for
  in-game visual or performance validation.

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
