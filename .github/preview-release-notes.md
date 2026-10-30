## Experimental 512×512 LabPBR preview — not a complete pack

This is a **512×512** test pack for Minecraft Java 26.3. Its legacy filename is
retained so the existing installer can update it. It replaces **112 block
textures**, not the whole game; all other blocks and entities remain vanilla or
come from other packs. Higher resolution uses substantially more GPU memory and
may affect loading and frame rate. It has not been tested on the RTX 5060 Ti.

The materials are derived from CC0 Poly Haven scans. Preview 6 adds a biome-tinted
photographic grass top and upgrades twelve plank variants with four matched 1K
wood veneer color, OpenGL normal, roughness and AO sets. Wood board joints are
procedural; height is estimated, not scanned. Grass uses matched normal and
roughness but estimated height and default AO. Grass sides and their transparent
biome-tinted overlays remain vanilla, so the top-to-side transition can differ.
Several blocks reuse sources and not all scans represent the named species.
Other existing materials may have approximated channels. Coverage and hashes are
in `build_report.json`; credits are in the ZIP. This revision has **not yet been
visually validated in game**.

### Install and check PBR

Use **Install / Update** in the GUI, or put the downloaded ZIP (do not extract)
in the active instance's `minecraft/resourcepacks` folder, and enable it above
competing packs under **Options → Resource Packs**. Use Iris and a compatible
LabPBR shader such as Photon. In Photon enable Normal Mapping and Specular
Mapping, choose labPBR in Resource Pack Settings, and optionally enable
Parallax Occlusion Mapping. Compare the same grass top or oak planks with normal
mapping on/off in direct light; rotate the view and reload resources with F3+T.
Shader menus and optional channel support vary. Structural channel tests alone
do not prove the shader is rendering the maps.

The downloadable PowerShell updater and GUI verify the ZIP's SHA-256 before
replacing the previous preview. The checksum is not an independent signature;
the installer is unsigned. GPU acceleration, missing textures, animations,
target-hardware performance measurements and in-game visual validation remain
unfinished. A smaller 128px build is available from source via `--resolution 128`.
