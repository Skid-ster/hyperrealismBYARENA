## Windows GUI installer 15 — dark premium setup screen

Installer 15 is a full UI rebuild after the cramped installer-14 light layout.
It keeps the existing verified install/update behavior, but presents it in a
larger, readable, dark game-installer layout.

- 980×660 default window with a dark charcoal palette, emerald primary accent,
  and amber shader accent. The Windows title bar is dark where supported.
- Full-width install-location card with a readable path field, Browse button,
  and clear folder validation. No clipped right-hand action rail.
- Two large product cards: HyperRealism 512×512 resource pack and optional
  Gilded Dusk Iris shader. The resource-pack install is the obvious primary
  action; shader install is secondary.
- Larger toolbar buttons with text-fit smoke checks, dark menus, and a dedicated
  status/progress card with an accessible Cancel action.
- Footer keeps Open folder, Launch, and Close available without squeezing the
  main install flow.

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
