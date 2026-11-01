## Windows GUI installer 18 — one-click pack + shader install

Installer 18 adds a primary **Install both** button that installs the matching
HyperRealism resource pack and Gilded Dusk shader into the selected instance in
one action.

- One release is selected only when it contains all four required assets:
  resource-pack ZIP + checksum and shader ZIP + checksum. Newer pack-only or
  shader-only releases cannot create a mismatched pair.
- Both ZIPs are downloaded to hidden same-folder staging files and SHA-256
  verified before either installed file is replaced.
- Files already matching the release checksum are skipped. Replacement uses
  atomic `File.Replace` / `File.Move`; if the second commit fails, the first is
  rolled back from a same-folder backup. Cancellation before commit leaves the
  existing pair untouched.
- The right-side action list is now **Install both**, **Resource pack only**,
  **Shader only**, **Launch**, **Open folder**, **Close**. The individual
  actions remain available for advanced/manual use.
- Automatic update checks now offer one combined pack + shader update prompt.

The PineconeMC/Prism-style dark layout from installer 17 is retained, with the
new primary action render-verified by CI. Existing functionality remains:
verified/capped downloads, cancellable transfers, confirm-before-replace,
deferred update prompts, launcher integration, settings, and self-update. Only
the named resource-pack and shader ZIPs are changed; other files stay untouched.

The resource pack and Gilded Dusk shader are experimental. Use a matching Iris
and Minecraft 26.3 setup, and keep PBR Support on LabPBR 1.3. Shader compiler
checks passed, but the shader has not been certified by in-game, GPU/driver,
visual-quality, or performance testing. Checksums detect corruption; they are
not independent digital signatures. The executable is unsigned, so Windows may
display a SmartScreen warning.

The executable filename and settings directory remain `HyperRealism128` for
updater compatibility. This is a separate installer app, not PineconeMC; it does
not install Minecraft or manage launcher instances.
