# HyperRealism Java 26.3 — experimental Preview 13

This 512×512 test pack contains **308 distinct block texture paths**. It is a partial resource pack, not a complete game replacement, and it is not benchmarked on the target RTX 5060 Ti.

## This preview

- Redraws 54 door and trapdoor textures with procedural panel/molding layouts and aligned LabPBR maps while leaving all Minecraft models, UVs, cutouts, block states, animations and behavior unchanged.
- Adds a PBR lever texture using the existing vanilla model/UV and preserving powered-on/off model states. Its cobblestone base reuses the existing cobblestone target.
- Sets per-texture `pbr_strength = 0.16` on 54 door/trapdoor paths and the lever, exactly 40% of the previous global `0.4` baseline. In the generator this scales normal slopes, height deviation, smoothness and AO influence; albedo and discrete LabPBR metal/F0 IDs and porosity are preserved.
- Verifies the compact CTM resource paths/method/tile count against Continuity 3.0.1+26.3 source and statically tests all 18 clear/stained/tinted glass rules, all 5 aligned albedo/normal/specular tile maps, and pane-side rules.
- Uses only documented CC0 Poly Haven material sources for the photographic inputs. Procedural masks and layouts are original; no Minecraft images or models are bundled.
- Leaves water unchanged.

## Testing and limitations

Linux and Windows Rust test suites, generated-pack LabPBR structure checks, archive integrity, PNG count, and SHA-256 verification are run by the release workflow before publishing. Continuity compatibility is verified against source code and structural tests, not by launching Minecraft. Door/lever appearance, powered/open state rendering, shader response, CTM seams and target-hardware performance still require in-game validation.

For shaders, use a compatible shader with LabPBR 1.3 and enable normal/specular mapping. Continuity 3.0.1+26.3 and Fabric API are needed for connected glass on Fabric; Iris alone does not provide CTM. Read the [shader setup](https://github.com/Skid-ster/hyperrealismBYARENA/blob/arena/65b59fff-hyperrealismbyarena/docs/SHADER_SETUP.md), [Continuity compatibility notes](https://github.com/Skid-ster/hyperrealismBYARENA/blob/arena/65b59fff-hyperrealismbyarena/docs/CONTINUITY_SUPPORT.md), and [asset credits](https://github.com/Skid-ster/hyperrealismBYARENA/blob/arena/65b59fff-hyperrealismbyarena/work/ASSET_CREDITS.md).
