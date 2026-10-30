# HyperRealism 512×512 — Minecraft Java 26.3 experimental Preview 21

This 512×512 experimental resource pack now generates **344 block texture paths**. It remains a partial pack, not a replacement for every vanilla texture; animated and cutout Nether sprites that cannot yet be safely regenerated with their exact frame/alpha data remain vanilla. The archive retains its legacy `HyperRealism128` filename for updater compatibility. Minecraft displays **HyperRealism 512×512**.

## Nether material pass

- Adds 36 static Nether texture paths: netherrack/ore hosts, basalt and polished basalt faces, blackstone/obsidian/ancient debris families, Nether and polished-blackstone masonry variants, quartz faces, soul sand/soil, crimson/warped nylium and wart blocks, stripped stems, glowstone and shroomlight.
- Replaces the generic stone stand-in on netherrack and its ore hosts with ambientCG's CC0 **Lava 003** scan; the Nether ore recipes keep their distinct gold/quartz inclusions.
- Uses ambientCG's CC0 **Rock 058** maps for dark basalt, blackstone, obsidian and related Nether stone, with its aligned OpenGL normal and packed AO/roughness/metal channels.
- Uses the existing scanned red-brick source for Nether masonry, keeps bright quartz surfaces on a neutral light-stone scan, and separates soul sand/soil and nylium palettes using their existing ground scans.
- Gives crying obsidian an aligned CC0 Volcanic Lava Flow fissure/emission mask; glowstone and shroomlight receive restrained LabPBR emission. Gilded blackstone uses sparse procedural metallic inclusions.
- Recolors crimson and warped wood with wood sources selected for the respective palettes. The crimson and warped stem-side animations, magma animation, Nether portal, roots, vines, fungus sprites and other cutouts remain untouched so their original frames, alpha shapes and block behavior are preserved.

All three new scans are CC0 and their creators, provenance and SHA-256 hashes are recorded in `work/ASSET_CREDITS.md` and `work/material_manifest.json`. The current catalog covers 344 targets, not the entire 26.3 block-texture inventory. Source choices are material-informed approximations rather than exact scans of Minecraft's fictional blocks.

The global PBR strength remains `0.2`; every generated target uses the same strength. Models, UVs, cutout silhouettes, block states and existing animations are unchanged. No Minecraft reference image pixels are redistributed.

## Automated checks and limitations

Linux and Windows CI generate the 512px pack, run tests, verify LabPBR structure, check the target count and validate the archive. Static source-image hashes and target-to-material assignments are checked. These are automated and structural checks—not an in-game Complementary Reimagined render, CTM runtime test, or performance benchmark on the target RTX 5060 Ti. The new maps still need in-game review for scale, repetition, color and visual fit.

For Complementary, choose **RP Support → labPBR (RP Required)** and keep **Block Reflection Quality** above OFF; POM is optional. See the [shader guide](https://github.com/Skid-ster/hyperrealismBYARENA/blob/arena/65b59fff-hyperrealismbyarena/docs/SHADER_SETUP.md), [Continuity compatibility notes](https://github.com/Skid-ster/hyperrealismBYARENA/blob/arena/65b59fff-hyperrealismbyarena/docs/CONTINUITY_SUPPORT.md), and [asset credits](https://github.com/Skid-ster/hyperrealismBYARENA/blob/arena/65b59fff-hyperrealismbyarena/work/ASSET_CREDITS.md).
