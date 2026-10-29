## Experimental 512×512 LabPBR preview — not a complete pack

This is a **512×512** test pack for Minecraft Java 26.3. Its legacy filename is
retained so the existing installer can update it. It replaces **111 block
textures**, not the whole game; all other blocks and entities remain vanilla or
come from other packs. Higher resolution uses substantially more GPU memory and
may affect loading and frame rate. It has not been tested on the RTX 5060 Ti.

The materials are derived from CC0 Poly Haven scans. Many starter inputs are
1K, but some older inputs are 512px or 2K and several blocks reuse the same
material. Five log sides now use the original 1K Bark Willow 02 color, DirectX
normal, roughness and displacement scans, adapted to different colors; log ends
remain vanilla. The existing cobblestone scan includes height but uses default
roughness, and several other materials have approximated height or normals.
Coverage and source hashes are in `build_report.json`; credits are in the ZIP.
These textures and PBR effects have **not yet been visually validated in game**.

### Install and check PBR

Use **Install / Update** in the GUI, or put the downloaded ZIP (do not extract)
in the active instance's `minecraft/resourcepacks` folder, and enable it above
competing packs under **Options → Resource Packs**. Use Iris and a compatible
LabPBR shader such as Photon. In Photon enable Normal Mapping and Specular
Mapping, choose labPBR in Resource Pack Settings, and optionally enable
Parallax Occlusion Mapping. Compare the same oak log or cobblestone with normal
mapping on/off in direct light; rotate the view and reload resources with F3+T.
Shader menus and optional channel support vary. Structural channel tests alone
do not prove the shader is rendering the maps.

The downloadable PowerShell updater and GUI verify the ZIP's SHA-256 before
replacing the previous preview. The checksum is not an independent signature;
the installer is unsigned. GPU acceleration, missing textures, animations,
target-hardware performance measurements and in-game visual validation remain
unfinished. A smaller 128px build is available from source via `--resolution 128`.
