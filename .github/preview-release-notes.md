# HyperRealism Java 26.3 — experimental Preview 14

This 512×512 test pack contains **308 distinct block texture paths**. It is a partial resource pack, not a complete game replacement, and it is not benchmarked on the target RTX 5060 Ti.

## This preview

- Applies one global PBR strength of `0.2` (20% of unit strength) to all 308 texture targets. Per-texture overrides have been removed; the PBR maps for connected-glass tiles are generated from the same reduced outputs.
- Softens procedural door/trapdoor panel shading and relief. All 47 previous door/trapdoor/ladder/bar opacity masks remain byte-for-byte unchanged. Minecraft models, UVs, glass pane cap shapes, block states and behavior are unchanged; only the clear/stained glass rim alpha width is intentionally reduced.
- Narrows clear/stained glass rims from 8px to 4px at 512px, and halves compact CTM edge width (2px to 1px at 128px).
- Keeps the Preview 13 lever/PBR textures and Continuity CTM coverage. Water is unchanged.

PBR strength scales normal slopes, height deviation, smoothness and AO influence; it does not scale albedo or discrete LabPBR metal/F0 IDs, porosity or emission. Shader POM depth and sample settings can still make blocks appear displaced; lower/disable POM if bulging remains.

## Testing and limitations

Linux and Windows Rust tests, generated-pack LabPBR structure checks, archive integrity, PNG count and SHA-256 verification are run by the release workflow before publishing. The alpha/cutout masks were compared byte-for-byte before regeneration. Continuity compatibility remains verified against source code and structural tests, not by launching Minecraft. The screenshot's specific shader/settings and Preview 14's appearance, including door/glass, still need in-game validation; target-hardware performance is unmeasured.

For shaders, use a compatible shader with LabPBR 1.3 and enable normal/specular mapping. Continuity 3.0.1+26.3 and Fabric API are needed for connected glass on Fabric; Iris alone does not provide CTM. Read the [shader setup](https://github.com/Skid-ster/hyperrealismBYARENA/blob/arena/65b59fff-hyperrealismbyarena/docs/SHADER_SETUP.md), [Continuity compatibility notes](https://github.com/Skid-ster/hyperrealismBYARENA/blob/arena/65b59fff-hyperrealismbyarena/docs/CONTINUITY_SUPPORT.md), and [asset credits](https://github.com/Skid-ster/hyperrealismBYARENA/blob/arena/65b59fff-hyperrealismbyarena/work/ASSET_CREDITS.md).
