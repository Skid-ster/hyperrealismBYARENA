# Bundled material sources

## Original procedural entity pilot maps

The source manifest now also declares five original procedural entity texture
triples for the **adult temperate chicken and cow, adult temperate pig, and the
sheep base plus its tintable wool overlay**. The recipes are implemented in
`scripts/build_chicken_adult_pilot.py` and
`scripts/build_common_mob_pilot.py`; they use no scanned image sources or
Minecraft color pixels. Their alpha/UV atlas spans were recorded to preserve
vanilla 26.3 mapping only. Chicken/cow/pig are 512×512, sheep base/wool are
512×256; albedo, normal and specular maps are aligned. Provenance is marked as
project-created procedural artwork in `work/material_manifest.json`; the
repository does not currently declare a separate license for this original
artwork. Other mob variants, babies, equipment overlays and hostile mobs remain
vanilla. Entity PBR rendering depends on shader support and has not yet been
validated in game.

## Scanned block-material sources

Most block maps are **CC0-1.0** scans from Poly Haven, ambientCG, TextureCan and
other cited providers. Their files are provided solely as a compact,
reproducible starter set for the in-progress
resource-pack generator. Poly Haven's asset license permits redistribution:
<https://polyhaven.com/license>. This repository does not use the live Poly
Haven API. Hashes for every bundled image are pinned in
`work/material_manifest.json`.

| Material ID | Source asset | Source author(s) | Included maps |
| --- | --- | --- | --- |
| `bark_willow_02` | [Bark Willow 02](https://polyhaven.com/a/bark_willow_02) | Charlotte Baglioni | 1K albedo, DirectX normal, roughness, displacement; used on spruce and acacia log sides |
| `bark_brown_01` | [Bark Brown 01](https://polyhaven.com/a/bark_brown_01) | Rob Tuytel | 1K albedo, OpenGL normal and roughness; oak log bark |
| `bark_platanus` | [Bark Platanus](https://polyhaven.com/a/bark_platanus) | Dimitrios Savva | 1K albedo, OpenGL normal and roughness; dark-oak and jungle log bark |
| `brick_red` | [Brick Wall 02](https://polyhaven.com/a/brick_wall_02) | Dimitrios Savva | albedo, OpenGL normal, roughness |
| `concrete_wall` | [Concrete Wall 007](https://polyhaven.com/a/concrete_wall_007) | Charlotte Baglioni, Dario Barresi, Rico Cilliers | albedo, OpenGL normal, roughness |
| `gravel_concrete` | [Gravel Concrete 04](https://polyhaven.com/a/gravel_concrete_04) | Dimitrios Savva | albedo, OpenGL normal, roughness |
| `rusted_shutter` | [Rusted Shutter](https://polyhaven.com/a/rusted_shutter) | Dimitrios Savva | albedo, OpenGL normal, roughness |
| `poly_wool_herringbone` | [Poly Wool Herringbone](https://polyhaven.com/a/poly_wool_herringbone) | colormass, Rico Cilliers | 1K albedo, OpenGL normal and roughness; shared neutral woven-wool source for recolored wool targets |
| `walnut`, `walnut_horizontal` | [Walnut Veneer](https://polyhaven.com/a/walnut_veneer) | Jenelle van Heerden | 1K albedo, OpenGL normal, roughness and AO; horizontal alias rotates all aligned maps and normal vectors |
| `rock_face` | [Rock Face 03](https://polyhaven.com/a/rock_face_03) | Dario Barresi, Rico Cilliers | albedo, OpenGL normal, packed AO/roughness/metallic; shared rocky host surface for overworld stone, andesite, tuff and ore variants |
| `nether_lava_003` | [Lava 003](https://ambientcg.com/view?id=Lava003) | ambientCG, Lennart Demes | 1K albedo, OpenGL normal and packed AO/roughness/metallic; CC0 volcanic crust for netherrack and ore hosts |
| `nether_dark_rock` | [Rock 058](https://ambientcg.com/view?id=Rock058) | ambientCG, Lennart Demes | 1K albedo, OpenGL normal and packed AO/roughness/metallic; CC0 dark rock for basalt, blackstone and obsidian families |
| `nether_lava_flow` | [Volcanic Lava Flow, ground_0027](https://www.texturecan.com/terms/) | TextureCan | 1K albedo, OpenGL normal, packed AO/roughness/metallic and aligned emission mask; CC0 source for crying-obsidian fissures |
| `forest_ground` | [Forest Ground 04](https://polyhaven.com/a/forest_ground_04) | Rob Tuytel, Rico Cilliers | albedo, OpenGL normal, packed AO/roughness/metallic |
| `sandstone_cracks` | [Sandstone Cracks](https://polyhaven.com/a/sandstone_cracks) | Rob Tuytel | albedo, OpenGL normal, packed AO/roughness/metallic |
| `cobble_photo`, `mossy_cobble_photo` | [Cobblestone Floor 04](https://polyhaven.com/a/cobblestone_floor_04) | Rob Tuytel | modified albedo, displacement; plain version desaturated |
| `aerial_sand` | [Aerial Beach 01](https://polyhaven.com/a/aerial_beach_01) | Rob Tuytel | albedo, OpenGL normal, roughness |

The active `brick_red` and `rusted_shutter` sources were sourced from a [documented CC0 redistribution](https://github.com/1493-ZA/Lil-Jeff-Simulator/blob/0e2989916fe5da64015e27b67abbe4464e94abf6/public/models/CREDITS.md)
at commit `0e2989916fe5da64015e27b67abbe4464e94abf6`; their 1K maps are
redistributed as JPEGs. The former Cotton Jersey wool stand-in and unused
Black Painted Planks maps were removed from the active source set. `walnut`
uses matched, unmodified provider 1K JPEG maps from the pinned [H2 material
mirror](https://github.com/noahpeters/H2/tree/768413f5e3bf1f9572208566aab138c957374bc6/public/studio/materials).
Its provenance identifies Poly Haven, the CC0 license, creator, map URLs,
dimensions and original SHA-256 values; the bundled copies are verified
against those hashes in `work/material_manifest.json`. Walnut now includes
aligned OpenGL normals, roughness and AO rather than the former partial
albedo/roughness-only copy. The `bark_brown_01` and `bark_platanus` 1K maps
are similarly documented Poly Haven CC0 redistributions in the pinned
[Abbott-game harvest](https://github.com/wmehobbs/Abbott-game/tree/0796b0a859aded9629cd802a0c132993b0c6e091/tools/content_factory/raw_harvest);
their source manifests record the provider URLs, authors and licenses. Input
hashes are pinned in the manifest.

Preview 21's `nether_lava_003` and `nether_dark_rock` are original CC0 ambientCG
Lava003 and Rock058 scans (Lennart Demes), with aligned OpenGL normals and
packed AO/roughness/metal channels. `nether_lava_flow` is TextureCan's CC0
Volcanic Lava Flow (ground_0027); its aligned emission mask drives the glowing
fissures on crying obsidian. Exact redistributed source bytes and source
provenance are pinned to the documented [ReneHack CC0 art mirror](https://github.com/g0ddest/renethack/tree/7cba95ae442dc4ccb174c965c3de91e0b2093bcd/client/godot/art/cc0),
commit `7cba95ae442dc4ccb174c965c3de91e0b2093bcd`. Only the upstream CC0 scan
maps are reused; no original game, competitor or proprietary texture pixels are
bundled. Source SHA-256 values are recorded in the manifest.

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

Twelve new chiseled targets—eight stone/tuff textures and four copper oxidation
stages—use a shared, original procedural shallow-diamond carving treatment.
They reuse the already documented CC0 `rock_face`, `sandstone_cracks`,
`metal_plate_02` and `rusted_shutter` source scans for material detail; they do
not copy the corresponding vanilla glyphs or pixels. The relief pattern is
generated by the project and remains an approximation pending in-game review.

Seven crystalline block targets—amethyst, cinnabar and sulfur in raw, brick
and/or chiseled forms—reuse the documented CC0 `rock_face` scan, with original
`gem`, `brick` and `carved` procedural treatments and color means rounded from
the local 26.3 metadata references. They do not reuse vanilla pixels. This is a
shared rocky approximation, not a scan of those specific minerals, and still
needs in-game art review.

The 371 texture names and approximate sRGB color targets were checked against
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
maps. It remains an approximate shared source for spruce and acacia only;
oak now uses Bark Brown 01, while dark oak and jungle use Bark Platanus. These
are species-informed source choices rather than scans proven to match the
exact Minecraft species. Log ends remain separate from bark-side scans.

### Preview 6 wood and grass

`oak_veneer_02`, `natural_walnut_veneer`, `white_maple_veneer`,
`cherry_veneer`, and `walnut` are 1K CC0 Poly Haven veneer scans by **Jenelle
van Heerden**: [Oak Veneer 02](https://polyhaven.com/a/oak_veneer_02),
[Natural Walnut Veneer](https://polyhaven.com/a/natural_walnut_veneer),
[White Maple Veneer](https://polyhaven.com/a/white_maple_veneer),
[Cherry Veneer](https://polyhaven.com/a/cherry_veneer), and
[Walnut Veneer](https://polyhaven.com/a/walnut_veneer). All provide aligned
albedo, OpenGL normals, roughness and ambient occlusion from the pinned
[H2 material mirror](https://github.com/noahpeters/H2/tree/768413f5e3bf1f9572208566aab138c957374bc6/public/studio/materials),
with provider URLs and original checksums. The Walnut Veneer source now
provides a distinctly dark-oak grain, while pale oak deliberately retains
White Maple Veneer because the white-oak scan's dark albedo would require
extreme color gain against Minecraft's very pale palette. Veneers remain
approximations for solid lumber, but species families now separate dark oak,
oak, maple/birch, cherry and walnut more deliberately. Rotated aliases
transform all continuous maps and normal vectors together. Bamboo still uses
White Maple Veneer with an original periodic bamboo-board/node recipe; it is
not a sourced bamboo photograph. Generated height combines procedural board
relief with a luminance estimate, not measured displacement.

The 16 recolored wool blocks now use [Poly Wool Herringbone](https://polyhaven.com/a/poly_wool_herringbone)
by colormass (photography) and Rico Cilliers (processing), CC0. The aligned
1K albedo, OpenGL normal and roughness maps were sourced from the documented
[Abbott-game harvest](https://github.com/wmehobbs/Abbott-game/tree/0796b0a859aded9629cd802a0c132993b0c6e091/tools/content_factory/raw_harvest/poly_wool_herringbone),
whose source manifest identifies the exact Poly Haven asset and CC0 license.
The maps are slightly rectangular (1024×1045) and are periodically resampled
by the generator to the square working tile; this small aspect correction
still needs shader/game review. This replaces a flat Cotton Jersey stand-in.

`leafy_grass` uses [Leafy Grass](https://polyhaven.com/a/leafy_grass) by
Charlotte Baglioni, CC0. The 1K color, matched OpenGL normal and roughness came
from the [RAINY portfolio](https://github.com/ssboroo/Rainy-portfolio/tree/9bb0a831e79574565686571557c3272558a52e1a/public/assets),
which identifies the maps as Leafy Grass in its README. Their color JPEG SHA-256
is `cfa40bc9d9417d1852db8753a8d5917f110c40101179f63543c382e39bc05e4a`;
`scripts/prepare-neutral-grass.py` converts it to grayscale because Minecraft
applies the biome color to the top face. Matched normal and roughness remain
unchanged; no scanned height or AO is supplied, so height is estimated and AO
defaults to white. Only the grass top is replaced: side and transparent tinted
side overlay remain vanilla, so their visual transition may be imperfect.

### Preview 7: everyday overworld surfaces

The five 1K sets [Snow 02](https://polyhaven.com/a/snow_02),
[Gravel Ground 01](https://polyhaven.com/a/gravel_ground_01),
[Brown Mud Dry](https://polyhaven.com/a/brown_mud_dry),
[Brown Mud 03](https://polyhaven.com/a/brown_mud_03) and
[Forest Leaves 04](https://polyhaven.com/a/forest_leaves_04) are CC0 scans by
Rob Tuytel. Each includes albedo, OpenGL normal, packed AO/roughness/metallic
(blue ignored) and displacement. They are original 1K JPEGs from the
[documented CrumpleZone redistribution](https://github.com/awittmannTech/CrumpleZone/tree/1c62ca1e91051b6d42b9910a31a371837537c827/assets/ground)
at commit `1c62ca1e91051b6d42b9910a31a371837537c827`; input SHA-256 hashes
are in the manifest. Snow, mud, gravel, dirt/path and podzol are approximate
Minecraft adaptations, not voxel-perfect reconstructions.

The broadleaf cutout atlas is composed from three fragments of the 1K leaves
of [Tree Small 02](https://polyhaven.com/a/tree_small_02) by Rico Cilliers,
CC0. It is derived from the WebP-compressed GLB in the
[documented London redistribution](https://github.com/take44444/london/tree/ede3b8850b307f5ba95eacd587a0663446212f61/dist/assets)
at commit `ede3b8850b307f5ba95eacd587a0663446212f61`, SHA-256
`9eb662fd339645584a69f176643dca8f7d2a16e2daff6ca1b7049b0b9ed6b747`.
The fragments and their OpenGL normals/roughness are rotated and layered with
periodic wrapping; their albedo is neutralized for biome tint. This is one
shared broadleaf scan, not species-accurate oak, birch, jungle, acacia, mangrove
or dark-oak foliage. `scripts/prepare-leaf-depth.py` derives a continuous height
map by periodic Poisson integration of the aligned scanned normal slopes, then
creates a conservative cavity-AO estimate from that height. These are derived
maps, not captured displacement or ray-traced AO; the normal, roughness, height
and AO are aligned. Subsurface remains a chosen constant. The audit's static
`dark_cutout` mip strategy is included. The grass side now has a soil base and
a separately generated, transparent, biome-tinted fringe from Leafy Grass;
the fringe shape is procedural. `scripts/prepare-overworld-materials.py`
recreates both derivatives with Pillow and numpy from the pinned GLB and
previously bundled neutral grass. The leaves and grass side need in-game review,
including seams and shader rendering; the source photos alone do not establish
visual quality in Minecraft. Log ends were still vanilla in preview 7.

### Preview 8 clay and common materials

`clay_floor` is derived from [Clay Floor 001](https://polyhaven.com/a/clay_floor_001)
by Dimitrios Savva and Rico Cilliers, CC0. The 1K color and normal and 512px
ARM files are decoded lossless PNGs of the WebP derivatives from
[Plinth](https://github.com/x0finch/Plinth/tree/382c1dbc17008b97c9d34badd685f6785426f56c/libraries/textures/clay)
at commit `382c1dbc17008b97c9d34badd685f6785426f56c`. The same clay scan is
recolored as terracotta variants and clay, an approximation. Stone, deepslate,
Nether bricks and sandstone variants reuse existing scans and procedural patterns;
they are not scans of the named blocks. Log ends combine wood veneer with
procedural rings and a bark rim, rather than scanned cross sections. The PBR
strength is reduced; height remains estimated for materials without displacement.
Water is deliberately not replaced.

### Preview 9 glass, stripped logs and foliage

Glass is procedurally generated from a neutral, flat color and a constant
roughness map created for this project (CC0); it is not a photographic scan.
Clear glass uses cutout geometry, stained glass is translucent, and pane caps
are narrow strips. Real shader and game testing is still needed. Stripped log
sides use the existing veneer scans, rotated with all aligned maps and normal
vectors. Their ends use approximate procedural rings without a bark rim. Three
new leaf variants reuse the neutral broadleaf atlas and may not resemble their
real species; cherry is tinted pink rather than biome-neutral. None is a
species-specific scan. Water is unchanged.

### Preview 10 connected glass

Compact CTM glass is generated procedurally using the OptiFine documented
five-tile layout, and adds no external image assets. Plain glass uses a thin
cutout rim; stained and tinted glass use translucent flat color. Each CTM tile
has a flat normal and smooth specular map. Actual connected-texture rendering,
pane culling and shader behavior have not yet been tested in Minecraft. Stone's
existing concrete scan has a moderately stronger 0.65 PBR strength; its albedo
is unchanged. Water is not replaced.

### Preview 11 shaped building blocks

Wood door/trapdoor designs, ladder and bars are independently drawn procedural
construction masks generated by `scripts/prepare-shaped-layouts.py`. The
masks encode shade, relief, metallic hardware and transparency. Their material
surfaces reuse the CC0 veneer scans described above; species matching, inset
panels and window layouts are approximations, not scans of real doors. No
Minecraft image data is bundled. Metal parts use the 1K albedo/normal/roughness
of [Metal Plate 02](https://polyhaven.com/a/metal_plate_02) by Rob Tuytel, CC0,
from [Daggie](https://github.com/89502147777r-wq/Daggie/pull/5), pinned commit
`f7728045722ebe9cd62e00c25f69fa4f4268dfab`; the mirror recompressed the maps
and desaturated/brightened the albedo. Normal convention is treated as OpenGL.
The source is worn steel, reused as an approximate stand-in for copper. Door
knobs use a procedural iron mask; iron and less-oxidized copper receive selected
LabPBR metal IDs, while heavily oxidized variants use dielectric IDs as an
approximation. No photorealistic representation or game compatibility is
guaranteed. This revision has not been visually tested in Minecraft. Water is
unchanged.

### Preview 13: interactive blocks and connected-texture audit

Door and trapdoor masks were redrawn as original, supersampled multistep
moldings, recessed/open panels, slats, and hardware. The 47 pre-existing door,
trapdoor, ladder and bar opacity masks were compared pixel-for-pixel with the
prior generated masks and are unchanged; shade, relief and hardware-channel
drawings were revised without changing those silhouettes. The shapes and
transparency are procedural; CC0 Poly Haven veneer and Metal Plate 02 scans
provide matched
albedo/normal/roughness inputs. Missing height is estimated and combined with
procedural panel relief. Fifty-four door/trapdoor texture targets
plus the new vanilla-path `lever` target use per-texture PBR strength `0.16`,
40% of the previous global `0.4` baseline, independently of the global setting. The lever's
wood-grip/iron-collar layout aligns with the vanilla 2×10 UV region; the base
continues using the already-covered cobblestone target. The powered/unpowered
lever, door hinge/open/half, and trapdoor open/top/bottom model-state families
were audited from the pinned 26.3 asset tree. No Minecraft images or models are
bundled or changed. PBR channels are structurally checked, but the revised art,
shader response, CTM rendering, and in-game state appearance still need runtime
validation.

Continuity 3.0.1+26.3 compatibility was checked against the exact upstream
`26.3/dev` source revision in `docs/CONTINUITY_SUPPORT.md`: the loader scans
`optifine/ctm`, registers `ctm_compact` with a five-tile minimum, and resolves
the generated match properties. Automated tests check all 18 covered glass
sprites, their tile-map triples, and pane rules. This is a source-contract and
structural verification, not a Minecraft/Continuity runtime render test.

### Preview 14: reduced PBR and softer door/glass edges

The global PBR strength is now `0.2` for all 344 generated texture targets, with
all per-texture overrides removed. This scales normal slope, height deviation
and smoothness to 20% of their unit-strength values and fades AO toward white;
albedo and discrete material-ID, porosity and emission channels remain intact.
The door/trapdoor layout shade and relief steps were softened. Their opacity
masks remain byte-for-byte unchanged from the prior revision, so original
window/cutout silhouettes are preserved. Clear/stained glass rims were narrowed
from 8px to 4px in the 512px recipe and CTM outputs (2px to 1px at 128px). The
source palette and glass/pane model UVs are unchanged. These are intended to
reduce harsh bevels, bright outlines and shader-parallax bulging; visual quality
still needs an in-game check with the user's shader and settings. Reducing POM
depth/samples in the shader can further reduce apparent block displacement.

### Preview 15: reconstructed foliage height and cavity AO

The shared CC0 broadleaf atlas now supplies explicit LabPBR height and AO
inputs for its nine leaf block variants, instead of relying on luminance-based
height and flat AO. Height is reconstructed periodically from the scanned
OpenGL normal slopes; AO is a restrained local cavity estimate. This adds
shallow leaf-surface relief only. It does not add canopy geometry, and the one
broadleaf source is still shared across species. The tree-leaf alpha cutouts are
unchanged. Map structure and checksums are validated in the generated pack; the
user's Minecraft 26.3/Fabric/Iris/Solas scene still needs live in-game review.
Technique research and license notes are in
[`docs/PBR_TECHNIQUE_RESEARCH.md`](../docs/PBR_TECHNIQUE_RESEARCH.md); no
competitor assets were copied.

### Preview 16: uniform, rim-free glass

The generated clear, 16 stained, tinted, and 17 pane-cap textures use target
colors reduced by 4% from the pinned Minecraft 26.3 audit. The change is applied
through manifest target colors; the source material remains the original
procedural neutral glass. Clear glass now has uniform 12% opacity, while stained
and tinted glass retain uniform 40% opacity. All five compact CTM albedo tiles
use the center color/alpha uniformly, so the previous edge frames are removed
rather than only thinned. Pane-cap alpha silhouettes are kept as vanilla UV
shape data; those are not decorative borders. Models, UVs, states, pane-cap silhouettes, door masks, glass PBR channels and
water are otherwise unchanged. This revision has structural tests but has not
been rendered in Minecraft.

### Preview 21: Nether solid-surface materials

The 36 new static Nether targets use source-informed CC0 scans for volcanic
crust, dark rock, lava fissures, red brick, pale quartz-like stone, soul soils,
nylium and fungal blocks. Existing door/plank assignments now keep crimson
wood on the cherry palette and warped wood on the pale-maple palette. Crying
obsidian uses the aligned TextureCan lava-fissure emission mask; glowstone and
shroomlight have constant emissive strengths. Gilded blackstone uses the
existing deterministic ore inclusion recipe with the gold LabPBR ID.

Minecraft `magma`, Nether portal, animated stems, roots/vines, fungus and other
cutout/animated sprites were not regenerated because the current pipeline does
not preserve their exact original animation frames and alpha silhouettes.
They remain vanilla. All targeted block models, UVs and block states remain
unchanged. The 344 target paths are validated against the pinned 26.3 metadata;
source licensing, sizes, hashes and channel structure are documented here and
in the manifest. This is not visual shader/game validation, and species/material
fit still needs in-game review.

### Current development branch: copper block coverage

Five static copper paths were added: `copper_block`, `cut_copper`, and its
exposed, weathered and oxidized variants. Three raw-ore blocks (`raw_copper_block`,
`raw_gold_block`, `raw_iron_block`) use the existing CC0 Poly Haven Rock Face 03
scan and deterministic ore recipe. Copper reuses the project's CC0 Poly Haven
`Metal Plate 02` and `Rusted Shutter` scans; no new source art was copied. Target
RGB averages and Git blob IDs are recorded in the pinned 26.3 audit catalog.
The `metal_cut` recipe adds periodic 4×4 panel seams and restrained recessed
relief without changing models, UVs or states. Vanilla pixels were used only
locally to calculate average-color hints; no game art is packed or committed.
The current development build defaults to 256×256 block maps and 256px-wide
entity atlases with each mob's original UV aspect ratio preserved (for example,
256×128 chicken and 256×256 cow maps). Automated tests verify the mapping,
source licensing, seam periodicity, atlas aspect ratios and coverage counts.
Coverage is 371 of 1,272 paths (901 remain); Minecraft/shader appearance has not
been visually validated.
