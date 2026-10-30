# HyperRealism 512×512 — Minecraft Java 26.3 experimental Preview 20

This 512×512 experimental pack contains **308 distinct block texture paths**. It is partial, not a complete game replacement, and is not benchmarked on the target RTX 5060 Ti. The archive keeps its legacy `HyperRealism128` filename for updater compatibility; Minecraft displays **HyperRealism 512×512**.

## Texture-quality pass

- Replaces the flat Cotton Jersey wool stand-in with Poly Haven's CC0 **Poly Wool Herringbone** scan across all 16 recolored wool blocks. Albedo, OpenGL normal and roughness are aligned.
- Diversifies bark surfaces: oak logs use **Bark Brown 01**, dark-oak and jungle logs use **Bark Platanus**, and spruce/acacia retain the existing willow scan. These are better-informed material choices, not proof of exact botanical matches.
- Gives dark-oak planks, stripped sides, doors and trapdoors a dedicated **Walnut Veneer** source with aligned albedo, OpenGL normal, roughness and AO. Plank grain is rotated horizontally with the normal vectors transformed consistently.
- Retains the color-matched White Maple source for pale oak; the darker White Oak scan was rejected because its color adaptation would require excessive gain against the audited pale-oak palette.
- Retains Preview 19's geological host-surface remap, the global PBR strength `0.2` across all 308 targets, aligned LabPBR 1.3 maps, Preview 15 foliage height/AO, Preview 16 glass and Preview 18 door hardware. Minecraft models, UVs, cutouts, states and animations are unchanged.
- Removes the unused Black Painted Planks and Cotton Jersey source maps from the active library.

## Testing and limitations

Linux and Windows tests generate the 512px pack and verify its LabPBR structure, texture count, archive integrity and checksums. Source-image hashes and material assignments are pinned and tested. This is automated/structural validation—not an in-game shader render, CTM runtime test, or target-hardware performance benchmark.

Several approximations remain: spruce/acacia share one bark scan; broadleaf species still share a foliage atlas; several wood species use veneer stand-ins; and some stone/ground families reuse scans. Their scale and appearance still need review in Minecraft Java 26.3 with Complementary Reimagined r5.9.3. For Complementary, choose **RP Support → labPBR (RP Required)** and keep **Block Reflection Quality** above OFF. POM is optional. Water is unchanged.

Sources are CC0 and credited in the pack. See the [shader guide](https://github.com/Skid-ster/hyperrealismBYARENA/blob/arena/65b59fff-hyperrealismbyarena/docs/SHADER_SETUP.md), [Continuity compatibility notes](https://github.com/Skid-ster/hyperrealismBYARENA/blob/arena/65b59fff-hyperrealismbyarena/docs/CONTINUITY_SUPPORT.md), and [asset credits](https://github.com/Skid-ster/hyperrealismBYARENA/blob/arena/65b59fff-hyperrealismbyarena/work/ASSET_CREDITS.md).
