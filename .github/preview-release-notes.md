## Experimental 512×512 LabPBR preview — not a complete pack

This is a **512×512** test pack for Minecraft Java 26.3. Its legacy filename is
retained so the existing installer can update it. It replaces **241 block
textures**, not the whole game; all other blocks and entities remain vanilla or
come from other packs. Higher resolution uses substantially more GPU memory and
may affect loading and frame rate. It has not been tested on the RTX 5060 Ti.

Preview 8 reduces PBR intensity to **0.4×**: normal slopes, height variation,
AO contrast and perceptual smoothness are scaled down. Color and discrete
material/emission IDs are unchanged; shader settings affect the perceived result.
Water is not changed.

Preview 9 adds **57 texture files**: ten stripped log sides and ends, clear
and sixteen stained glass blocks and their pane caps, plus spruce, pale oak and
cherry leaves. Glass is procedural, with a cutout rim on clear glass, transparent
stained glass and narrow pane caps. Stripped logs use rotated veneer scans and
approximate procedural rings. The three leaves reuse a broadleaf atlas, not
species-specific scans. These are approximations and need in-game visual testing.

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
