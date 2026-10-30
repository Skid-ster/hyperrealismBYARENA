# HyperRealism 512×512 — Minecraft Java 26.3 experimental Preview 18

This 512×512 test pack contains **308 distinct block texture paths**. It is a partial resource pack, not a complete game replacement, and it is not benchmarked on the target RTX 5060 Ti. The ZIP keeps its legacy `HyperRealism128` filename so existing installers can continue to update it; the pack itself is named **HyperRealism 512×512** in Minecraft.

## This preview

- Repositions procedural door hardware to the latch-side UV edge and raises it within the lower-block sprite, placing it near mid-height on the assembled two-block door. Removes the pale-oak decorative band that overlapped and obscured that handle. Vanilla geometry, hinge/open states, UV mapping and cutout alpha remain unchanged.
- Corrects the in-game resource-pack name and installer branding to **HyperRealism 512×512**.
- Retains aligned LabPBR 1.3 normal (`_n`) and specular (`_s`) sidecars on covered block sprites, including doors, trapdoors and other non-full shapes. Complementary Reimagined must use **RP Support → labPBR (RP Required)**; selecting the shader or High profile alone does not guarantee the pack's LabPBR channels are active.
- Retains the global PBR strength `0.2` across all 308 targets, Preview 15 foliage height/AO, Preview 16 glass treatment and unchanged water.

## Testing and limitations

Linux and Windows Rust tests, 512px generation, generated-pack LabPBR verification, PNG count, archive integrity and checksums run in GitHub Actions before publishing. This is structural/automated validation, not in-game rendering or a target-hardware benchmark. Shader appearance and performance remain unverified.

For Complementary Reimagined, choose **RP Support → labPBR (RP Required)**; keep **Block Reflection Quality** above OFF to see specular highlights. POM is optional, and POM on cutout surfaces is not required for normal/specular mapping. In BSL, enable **Advanced Materials** and select **LabPBR 1.3** as the Material Format. Continuity 3.0.1+26.3 and Fabric API are needed for connected glass on Fabric; Iris alone does not provide CTM. Read the [shader setup](https://github.com/Skid-ster/hyperrealismBYARENA/blob/arena/65b59fff-hyperrealismbyarena/docs/SHADER_SETUP.md), [Continuity compatibility notes](https://github.com/Skid-ster/hyperrealismBYARENA/blob/arena/65b59fff-hyperrealismbyarena/docs/CONTINUITY_SUPPORT.md), and [asset credits](https://github.com/Skid-ster/hyperrealismBYARENA/blob/arena/65b59fff-hyperrealismbyarena/work/ASSET_CREDITS.md).
