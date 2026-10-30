# HyperRealism Java 26.3 — experimental Preview 15

This 512×512 test pack contains **308 distinct block texture paths**. It is a partial resource pack, not a complete game replacement, and it is not benchmarked on the target RTX 5060 Ti.

## This preview

- Adds aligned explicit height and AO inputs to the shared CC0 broadleaf atlas used by nine leaf block textures. Height is a periodic reconstruction from the scanned normal map; AO is a subtle local cavity estimate, not a ray-traced bake or captured depth scan.
- Preserves the leaf albedo/cutout silhouettes and all Minecraft models, UVs, geometry, block states and behavior. Height mapping cannot add leaf cards, branches, or canopy volume. Foliage remains one broadleaf source shared across species.
- Retains the Preview 14 global PBR strength of `0.2` on all 308 targets, the softened door/trapdoor relief, and narrowed clear/stained glass rims. No competitor assets were copied.

PBR strength scales normal slopes, height deviation, smoothness and AO influence; it does not scale albedo or discrete LabPBR metal/F0 IDs, porosity or emission. Shader POM depth and sample settings can still make blocks appear displaced; lower/disable POM if bulging remains.

## Testing and limitations

Linux and Windows Rust tests, generated-pack LabPBR structure checks, archive integrity, PNG count and SHA-256 verification run in the release workflow before publishing. Source PNG dimensions, map alignment, and the generated pack are structurally checked. Rival-pack survey is documented in `docs/PBR_TECHNIQUE_RESEARCH.md`; license review and asset provenance are in `work/ASSET_CREDITS.md`. The screenshot's Minecraft 26.3/Fabric/Iris/Solas scene and Preview 15 appearance have **not** been validated in-game; target-hardware performance is unmeasured.

For shaders, use a compatible shader with LabPBR 1.3 and enable normal/specular mapping. POM is optional and requires the shader's parallax setting. Continuity 3.0.1+26.3 and Fabric API are needed for connected glass on Fabric; Iris alone does not provide CTM. Read the [shader setup](https://github.com/Skid-ster/hyperrealismBYARENA/blob/arena/65b59fff-hyperrealismbyarena/docs/SHADER_SETUP.md), [Continuity compatibility notes](https://github.com/Skid-ster/hyperrealismBYARENA/blob/arena/65b59fff-hyperrealismbyarena/docs/CONTINUITY_SUPPORT.md), and [asset credits](https://github.com/Skid-ster/hyperrealismBYARENA/blob/arena/65b59fff-hyperrealismbyarena/work/ASSET_CREDITS.md).
