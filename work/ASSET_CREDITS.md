# Bundled material sources

These maps are **CC0-1.0** assets derived from Poly Haven. Their files are
provided solely as a compact, reproducible starter set for the in-progress
resource-pack generator. Poly Haven's asset license permits redistribution:
<https://polyhaven.com/license>. This repository does not use the live Poly
Haven API. Hashes for every bundled image are pinned in
`work/material_manifest.json`.

| Material ID | Poly Haven source | Source author(s) | Included maps |
| --- | --- | --- | --- |
| `bark_willow_02` | [Bark Willow 02](https://polyhaven.com/a/bark_willow_02) | Charlotte Baglioni | 1K albedo, DirectX normal, roughness, displacement; approximate bark for five log sides |
| `brick_red` | [Brick Wall 02](https://polyhaven.com/a/brick_wall_02) | Dimitrios Savva | albedo, OpenGL normal, roughness |
| `concrete_wall` | [Concrete Wall 007](https://polyhaven.com/a/concrete_wall_007) | Charlotte Baglioni, Dario Barresi, Rico Cilliers | albedo, OpenGL normal, roughness |
| `gravel_concrete` | [Gravel Concrete 04](https://polyhaven.com/a/gravel_concrete_04) | Dimitrios Savva | albedo, OpenGL normal, roughness |
| `black_painted_planks` | [Black Painted Planks](https://polyhaven.com/a/black_painted_planks) | Dimitrios Savva | albedo, OpenGL normal, roughness |
| `rusted_shutter` | [Rusted Shutter](https://polyhaven.com/a/rusted_shutter) | Dimitrios Savva | albedo, OpenGL normal, roughness |
| `cotton_jersey` | [Cotton Jersey](https://polyhaven.com/a/cotton_jersey) | Rico Cilliers, colormass | albedo, OpenGL normal, roughness; used as a wool stand-in |
| `walnut` | [Walnut Veneer](https://polyhaven.com/a/walnut_veneer) | Poly Haven | albedo, roughness; no normal map |
| `rock_face` | [Rock Face 03](https://polyhaven.com/a/rock_face_03) | Dario Barresi, Rico Cilliers | albedo, OpenGL normal, packed AO/roughness/metallic; netherrack stand-in |
| `forest_ground` | [Forest Ground 04](https://polyhaven.com/a/forest_ground_04) | Rob Tuytel, Rico Cilliers | albedo, OpenGL normal, packed AO/roughness/metallic |
| `sandstone_cracks` | [Sandstone Cracks](https://polyhaven.com/a/sandstone_cracks) | Rob Tuytel | albedo, OpenGL normal, packed AO/roughness/metallic |
| `cobble_photo`, `mossy_cobble_photo` | [Cobblestone Floor 04](https://polyhaven.com/a/cobblestone_floor_04) | Rob Tuytel | modified albedo, displacement; plain version desaturated |
| `aerial_sand` | [Aerial Beach 01](https://polyhaven.com/a/aerial_beach_01) | Rob Tuytel | albedo, OpenGL normal, roughness |

The six materials from `brick_red` through `cotton_jersey` were sourced from a [documented CC0 redistribution](https://github.com/1493-ZA/Lil-Jeff-Simulator/blob/0e2989916fe5da64015e27b67abbe4464e94abf6/public/models/CREDITS.md)
at commit `0e2989916fe5da64015e27b67abbe4464e94abf6`. The building and street maps are 1024px JPEGs; the cotton jersey maps are
512px JPEGs. `walnut` came from the [documented
CC0 redistribution](https://github.com/jasmine314-dev/dm-portal-assets/tree/2cee5cfb6cd7f3e73cfb47259f0106f5fcece65b)
at commit `2cee5cfb6cd7f3e73cfb47259f0106f5fcece65b`, where the diffuse is
1024px JPEG and roughness is a 512px JPEG. Files here are unmodified copies of
those redistributed versions; the generator resizes and transforms them at
build time. They are not the original full-resolution Poly Haven files.

`forest_ground`, `sandstone_cracks` and `rock_face` were sourced from a [documented
CC0 redistribution](https://github.com/gotzawal/morglay/blob/b462748f1e2c2cf2288de615089c79d69ce47b08/editor/library/LICENSES.md)
at commit `b462748f1e2c2cf2288de615089c79d69ce47b08`. The mirror resized and
JPEG-encoded the maps, using an OpenGL normal and packed ARM (AO, roughness,
metallic) channels. The included files are unmodified from that mirror.
`aerial_sand` uses 2K original JPEGs from the [documented CC0 redistribution](https://github.com/IOES-Lab/POSIM-Terrain/blob/df742f903aeced7dd4d9caf9f62b94302f886e03/src/posim_terrain/assets/seabed/manifest.json)
at commit `df742f903aeced7dd4d9caf9f62b94302f886e03`. Its 30-meter aerial scan
is an approximate sand stand-in, and rooted dirt uses the same forest-ground
source as ordinary dirt rather than authentic distinct roots.

The cobblestone inputs derive from [a documented CC0 mirror](https://github.com/VictorZakharov/last-stand/blob/ca08c1817faa703c9a9fb0287b104cf247b8da57/src/core/photos/AGENTS.md)
at commit `ca08c1817faa703c9a9fb0287b104cf247b8da57`: that mirror's 1K
color and displacement scans were stored as lossy WebP with a 5px wrap margin.
We decoded them, removed the margin, selected the middle 512px, blended 64px
at opposite edges to limit seams and saved PNGs. The plain color was converted
to grayscale; the mossy version retains color. The scanned displacement is
used to derive normals; roughness is the generator's default, not a scanned
roughness map. Edge blending can blur individual stones, and these tiles still
need in-game review.

The 111 texture names and approximate sRGB color targets were checked against
an independently maintained [Minecraft 26.3 asset inventory](https://github.com/InventivetalentDev/minecraft-assets/tree/6c088630459d8ca369ef0d6c07636d7d63a3b0f1/assets/minecraft/textures/block),
with source commit and PNG blob IDs recorded in
`catalog/vanilla_26.3_audit.json`. No vanilla image data is bundled. These
metadata checks do not prove in-game accuracy or replace testing with your own
Minecraft installation. Some starter outputs share material sources and their
appearance remains experimental.

`bark_willow_02` is the original 1K JPEG set mirrored by
[texture-explorer](https://github.com/pablo-figueroa-uniandes/texture-explorer/blob/8efed6702632a18f284dcaba5b64cb4b02304fe5/tools/fetch_textures.ps1)
at commit `8efed6702632a18f284dcaba5b64cb4b02304fe5`. The mirror script
retrieves Poly Haven's 1K diffuse, DirectX normal, roughness and displacement
maps. The same willow scan is adapted to oak, spruce, dark oak, acacia and
jungle logs; it is not a species-specific scan. Log end textures remain vanilla.
