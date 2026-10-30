# HyperRealism 512×512 — Minecraft Java 26.3 experimental Preview 19

This 512×512 experimental pack contains **308 distinct block texture paths**. It is a partial resource pack, not a complete game replacement, and it is not benchmarked on the target RTX 5060 Ti. The ZIP keeps its legacy `HyperRealism128` filename so existing installers can update it; the in-game pack name is **HyperRealism 512×512**.

## This preview

- Moves stone, andesite, tuff, and the common overworld ore host surfaces to the existing CC0 Poly Haven **Rock Face 03** scan. Deepslate and Nether ore host surfaces already use that rocky scan. The concrete scan now serves 40 targets instead of 51.
- Retains the global PBR strength `0.2` across all 308 targets, aligned LabPBR 1.3 normal/specular maps, the Preview 15 foliage height/AO work, Preview 16 glass treatment, and the revised Preview 18 door hardware.
- Changes source-material assignments only; Minecraft models, UVs, cutouts, states, and animations remain unchanged.

## Testing and limitations

Linux and Windows Rust tests, 512px generation, generated-pack LabPBR verification, PNG count, archive integrity, and checksums run in GitHub Actions before publishing. These are structural and automated checks, not in-game rendering or a target-hardware benchmark. Shader appearance and performance remain unverified.

For Complementary Reimagined, choose **RP Support → labPBR (RP Required)** and keep **Block Reflection Quality** above OFF to see specular highlights. POM is optional. In BSL, enable **Advanced Materials** and select **LabPBR 1.3** as the Material Format. Continuity 3.0.1+26.3 and Fabric API are needed for connected glass on Fabric; Iris alone does not provide CTM. See the [shader setup](https://github.com/Skid-ster/hyperrealismBYARENA/blob/arena/65b59fff-hyperrealismbyarena/docs/SHADER_SETUP.md), [Continuity compatibility notes](https://github.com/Skid-ster/hyperrealismBYARENA/blob/arena/65b59fff-hyperrealismbyarena/docs/CONTINUITY_SUPPORT.md), and [asset credits](https://github.com/Skid-ster/hyperrealismBYARENA/blob/arena/65b59fff-hyperrealismbyarena/work/ASSET_CREDITS.md).

**Design target:** Windows 11, RTX 5060 Ti 8 GB, i5-13400, and 16 GB RAM. This is a target, not a claim of measured performance or GPU acceleration. The generator is CPU-only; shader rendering uses your GPU.
