# HyperRealism Java 26.3 — experimental Preview 17

This 512×512 test pack contains **308 distinct block texture paths**. It is a partial resource pack, not a complete game replacement, and it is not benchmarked on the target RTX 5060 Ti.

## This preview

- Moves procedural door handles to the latch-side edge used by the pinned Java 26.3 door UVs, across the door families; copper doors now use the same edge-aligned hardware treatment. Vanilla model geometry, hinge/open states, UVs and cutout alpha are preserved.
- Keeps aligned LabPBR 1.3 normal (`_n`) and specular (`_s`) sidecars on covered block sprites, including doors, trapdoors and other non-full shapes. BSL must have Advanced Materials enabled and Material Format set to LabPBR 1.3; PBR has not been visually validated in-game.
- Retains the global PBR strength `0.2` across all 308 targets, Preview 15 foliage height/AO, Preview 16 glass treatment and unchanged water.

## Testing and limitations

Linux and Windows Rust tests, 512px generation, generated-pack LabPBR verification, PNG count, archive integrity and checksums run in GitHub Actions before publishing. This is structural/automated validation, not in-game rendering or a target-hardware benchmark. Shader appearance and performance remain unverified.

For shaders, use a LabPBR 1.3-compatible shader and enable normal/specular mapping. In BSL, enable **Advanced Materials** and select **LabPBR 1.3** as the Material Format. Continuity 3.0.1+26.3 and Fabric API are needed for connected glass on Fabric; Iris alone does not provide CTM. Read the [shader setup](https://github.com/Skid-ster/hyperrealismBYARENA/blob/arena/65b59fff-hyperrealismbyarena/docs/SHADER_SETUP.md), [Continuity compatibility notes](https://github.com/Skid-ster/hyperrealismBYARENA/blob/arena/65b59fff-hyperrealismbyarena/docs/CONTINUITY_SUPPORT.md), and [asset credits](https://github.com/Skid-ster/hyperrealismBYARENA/blob/arena/65b59fff-hyperrealismbyarena/work/ASSET_CREDITS.md).
