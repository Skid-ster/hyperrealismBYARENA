## Experimental 512×512 LabPBR preview — not a complete pack

Preview 12 is a 512×512 test pack for Minecraft Java 26.3. It replaces **307 distinct block texture paths**, not 307 blocks and not the entire game; multiple blocks share a texture and individual blocks can use several textures. Other blocks and entities remain vanilla or come from other packs. It is not measured on the target RTX 5060 Ti.

### Changes since Preview 11

Adds five previously uncovered bamboo-family textures: bamboo planks, fence and fence-gate surfaces, plus their distinct particle sprites. The new textures use a dedicated periodic bamboo-board recipe over a licensed CC0 White Maple Veneer scan, with source colors guided by the pinned 26.3 sprite inventory. This is an original **approximation**, not a bamboo scan, and requires visual review. Vanilla models, UV layouts and fence geometry are unchanged. A pinned model-reference audit now checks 125 representative 26.3 blockstates, including the shared cobblestone stairs/slab/wall mapping and wood families.

PBR remains LabPBR 1.3 with the pack's global 0.4 strength; material IDs and emission channels are not scaled. Water is unchanged. The new source textures have not been visually validated in Minecraft. The model audit verifies texture-path selection, not visual quality or shader rendering.

Connected glass requires a compatible CTM mod such as [Continuity for Fabric 26.3](https://modrinth.com/mod/continuity/version/3.0.1+26.3) plus its required Fabric API; Iris alone does not provide CTM. CTM and glass reflections are not confirmed in game. Clear cutout glass may not show reflections across invisible pixels in every shader.

### Install and check PBR

Use **Install / Update** in the GUI, or put the downloaded ZIP (do not extract it) in the active instance's `minecraft/resourcepacks` folder and enable it above competing packs under **Options → Resource Packs**. The installer checks that the chosen folder exists and is named `resourcepacks`; that does not prove it is the active game instance. Use Iris and a compatible LabPBR shader such as Photon. In Photon enable Normal Mapping and Specular Mapping, choose labPBR in Resource Pack Settings, and optionally enable Parallax Occlusion Mapping. Compare bamboo planks and fence surfaces, cobblestone stairs/slabs/walls, and wood variants with normal mapping on/off; reload with F3+T and inspect different orientations. Shader menus and optional channel support vary.

The downloadable Windows GUI installer and PowerShell updater verify the ZIP's SHA-256 before replacing the previous preview. A checksum from the same release is not an independent signature; the installer is unsigned and may trigger Windows warnings. Pack loading, CTM, shader appearance, the target PC's performance and in-game visual quality remain unverified. GPU acceleration is not implemented.
