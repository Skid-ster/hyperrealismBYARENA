# Bundled material sources

These maps are **CC0-1.0** assets derived from Poly Haven. Their files are
provided solely as a compact, reproducible starter set for the in-progress
resource-pack generator. Poly Haven's asset license permits redistribution:
<https://polyhaven.com/license>. This repository does not use the live Poly
Haven API. Hashes for every bundled image are pinned in
`work/material_manifest.json`.

| Material ID | Poly Haven source | Source author(s) | Included maps |
| --- | --- | --- | --- |
| `brick_red` | [Brick Wall 02](https://polyhaven.com/a/brick_wall_02) | Dimitrios Savva | albedo, OpenGL normal, roughness |
| `concrete_wall` | [Concrete Wall 007](https://polyhaven.com/a/concrete_wall_007) | Charlotte Baglioni, Dario Barresi, Rico Cilliers | albedo, OpenGL normal, roughness |
| `gravel_concrete` | [Gravel Concrete 04](https://polyhaven.com/a/gravel_concrete_04) | Dimitrios Savva | albedo, OpenGL normal, roughness |
| `black_painted_planks` | [Black Painted Planks](https://polyhaven.com/a/black_painted_planks) | Dimitrios Savva | albedo, OpenGL normal, roughness |
| `rusted_shutter` | [Rusted Shutter](https://polyhaven.com/a/rusted_shutter) | Dimitrios Savva | albedo, OpenGL normal, roughness |
| `cotton_jersey` | [Cotton Jersey](https://polyhaven.com/a/cotton_jersey) | Rico Cilliers, colormass | albedo, OpenGL normal, roughness; used as a wool stand-in |
| `walnut` | [Walnut Veneer](https://polyhaven.com/a/walnut_veneer) | Poly Haven | albedo, roughness; no normal map |

All except `walnut` were sourced from a [documented CC0 redistribution](https://github.com/1493-ZA/Lil-Jeff-Simulator/blob/0e2989916fe5da64015e27b67abbe4464e94abf6/public/models/CREDITS.md)
at commit `0e2989916fe5da64015e27b67abbe4464e94abf6`. The building and street maps are 1024px JPEGs; the cotton jersey maps are
512px JPEGs. `walnut` came from the [documented
CC0 redistribution](https://github.com/jasmine314-dev/dm-portal-assets/tree/2cee5cfb6cd7f3e73cfb47259f0106f5fcece65b)
at commit `2cee5cfb6cd7f3e73cfb47259f0106f5fcece65b`, where the diffuse is
1024px JPEG and roughness is a 512px JPEG. Files here are unmodified copies of
those redistributed versions; the generator resizes and transforms them at
build time. They are not the original full-resolution Poly Haven files.

The 96 texture names and approximate sRGB color targets were checked against
an independently maintained [Minecraft 26.3 asset inventory](https://github.com/InventivetalentDev/minecraft-assets/tree/6c088630459d8ca369ef0d6c07636d7d63a3b0f1/assets/minecraft/textures/block),
with source commit and PNG blob IDs recorded in
`catalog/vanilla_26.3_audit.json`. No vanilla image data is bundled. These
metadata checks do not prove in-game accuracy or replace testing with your own
Minecraft installation. Some starter outputs share material sources and their
appearance remains experimental.
