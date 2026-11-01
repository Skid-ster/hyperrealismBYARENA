## Windows GUI installer 13 — HyperRealism 512×512 + Gilded Dusk

Installer 13 adds installation and update support for the separate experimental
Iris shaderpack **HyperRealism: Gilded Dusk**:

- **Install shader** downloads the published shader ZIP, verifies its SHA-256,
  and atomically installs it into the `shaderpacks` folder beside the selected
  instance's `resourcepacks` folder. If needed, it creates that standard
  sibling folder. Other files are left untouched.
- **Check updates** checks the installed resource pack and shader, as well as
  the installer. When an installed pack or shader changes, it asks before
  replacing it; automatic update prompts remain deferrable until the next launch.
- Shader downloads are cancellable, use the existing byte-progress UI and
  30-second network timeouts, and are capped at 100 MiB. Pack and installer
  download limits remain 1 GiB and 100 MiB, respectively.
- Downloads are staged and SHA-256 verified before replacement. Checksums are
  published alongside assets; they detect corruption but are not independent
  digital signatures.

The shader is an experimental Iris shaderpack, not a resource pack. Use a
matching Iris/Sodium Minecraft 26.3 instance, enable the HyperRealism resource
pack, then select **HyperRealism: Gilded Dusk** in Iris and keep PBR Support on
LabPBR 1.3. Compiler checks do not establish that it loads or renders correctly
in every Minecraft, mod, GPU, or driver setup; in-game review and performance
validation remain outstanding.

The executable filename and settings directory remain `HyperRealism128` for
updater compatibility. The GUI is a separate app, not PineconeMC; it does not
install Minecraft or manage launcher instances. The executable is unsigned, so
Windows may display a SmartScreen warning.
