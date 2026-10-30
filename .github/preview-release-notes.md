# HyperRealism Java 26.3 — experimental Preview 16

This 512×512 test pack contains **308 distinct block texture paths**. It is a partial resource pack, not a complete game replacement, and it is not benchmarked on the target RTX 5060 Ti.

## This preview

- Removes decorative alpha rims from clear, stained, tinted and connected glass. Clear glass now has uniform 12% opacity; stained/tinted glass stays at uniform 40% opacity. Compact CTM tile maps retain their five-tile structure but no longer draw borders.
- Slightly darkens the target color by 4% on all **35** glass block and pane-cap sprites. The pane-cap alpha strip, original models/UVs/states and pane geometry remain unchanged; that strip defines the vanilla pane surface and is not a decorative outline.
- Retains the Preview 15 foliage height/AO maps, global PBR strength `0.2` across all 308 targets, softened door relief, and unchanged water.

## Testing and limitations

Linux and Windows Rust tests, 512px generation, generated-pack LabPBR verification, PNG count, archive integrity and checksums run in CI before publishing. Tests assert uniform alpha at glass texture edges and centers, and validate every glass color target against the pinned Minecraft 26.3 audit. No in-game rendering or target-hardware benchmark was performed; the user's Minecraft/Fabric/Iris/Solas appearance remains unverified.

For shaders, use a compatible shader with LabPBR 1.3 and enable normal/specular mapping. Continuity 3.0.1+26.3 and Fabric API are needed for connected glass on Fabric; Iris alone does not provide CTM. Read the [shader setup](https://github.com/Skid-ster/hyperrealismBYARENA/blob/arena/65b59fff-hyperrealismbyarena/docs/SHADER_SETUP.md), [Continuity compatibility notes](https://github.com/Skid-ster/hyperrealismBYARENA/blob/arena/65b59fff-hyperrealismbyarena/docs/CONTINUITY_SUPPORT.md), and [asset credits](https://github.com/Skid-ster/hyperrealismBYARENA/blob/arena/65b59fff-hyperrealismbyarena/work/ASSET_CREDITS.md).
