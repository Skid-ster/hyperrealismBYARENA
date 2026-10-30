## Experimental 512×512 LabPBR preview — not a complete pack

This is a **512×512** test pack for Minecraft Java 26.3. Its legacy filename is
retained so the existing installer can update it. It replaces **242 block
textures**, not the whole game; all other blocks and entities remain vanilla or
come from other packs. Higher resolution uses substantially more GPU memory and
may affect loading and frame rate. It has not been tested on the RTX 5060 Ti.

Preview 8 reduces PBR intensity to **0.4×**: normal slopes, height variation,
AO contrast and perceptual smoothness are scaled down. Color and discrete
material/emission IDs are unchanged; shader settings affect the perceived result.
Water is not changed.

Preview 10 adds **tinted glass**, thin fallback borders and OptiFine-format
compact connected glass tiles for clear, all 16 stained colors, and tinted
glass, including pane faces. Each tile has matching smooth LabPBR maps. This
requires a compatible CTM implementation: Iris alone is not enough. For a
Fabric 26.3 instance, install [Continuity 3.0.1+26.3](https://modrinth.com/mod/continuity/version/3.0.1+26.3)
and its required Fabric API, restart, and enable this pack. Consider Continuity's
pane culling fix for pane seams. Glass CTM, transparency and shader behavior
are **not yet visually verified in game**. Clear glass uses cutout rendering;
its invisible interior may not reflect, depending on the shader.

Stone already had PBR maps; its per-texture strength is now **0.65×**, up from
0.4×, for slightly stronger depth and shine. The stone color and other blocks'
global PBR strength remain unchanged. Water remains untouched.

This is still **not a complete all-block pack**. Doors, plants, animated
textures and many others remain vanilla, as does water. The 512px textures use
more GPU memory and target-hardware performance has not been measured. Coverage
and input hashes are in `build_report.json`; credits are in the ZIP. This
revision has **not yet been visually validated in game**.

### Install and check PBR

Use **Install / Update** in the GUI, or put the downloaded ZIP (do not extract)
in the active instance's `minecraft/resourcepacks` folder, and enable it above
competing packs under **Options → Resource Packs**. Use Iris and a compatible
LabPBR shader such as Photon. In Photon enable Normal Mapping and Specular
Mapping, choose labPBR in Resource Pack Settings, and optionally enable
Parallax Occlusion Mapping. Compare the same grass side, oak leaves or snow with normal
mapping on/off in direct light; rotate the view and reload resources with F3+T.
Shader menus and optional channel support vary. Structural channel tests alone
do not prove the shader is rendering the maps.

The downloadable PowerShell updater and GUI verify the ZIP's SHA-256 before
replacing the previous preview. The checksum is not an independent signature;
the installer is unsigned. GPU acceleration, missing textures, animations,
target-hardware performance measurements and in-game visual validation remain
unfinished. A smaller 128px build is available from source via `--resolution 128`.
