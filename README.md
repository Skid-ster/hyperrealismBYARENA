# HyperRealism 256×256 — Minecraft Java LabPBR pack

This is an **early prototype**, not the requested complete production pack.
The development build defaults to manifest-defined 256×256 Minecraft Java 26.3
block textures and aligned LabPBR 1.3 maps using a CPU-only pipeline; optional
outputs are 128×128 or 512×512. The currently published Preview 23 remains the
older 512×512 build until a new pack release. Most starter materials use 1K
Poly Haven-derived maps; some older maps are 512px or 2K, and missing channels
are approximated as described below. The current source catalog contains 371
distinct block texture targets based on CC0 scans from Poly Haven, ambientCG,
TextureCan and other credited providers, plus original procedural work; Preview 13 added the lever texture, Preview 14 unified PBR
strength at 20% and softened door/glass treatments, Preview 15 added
normal-derived foliage height/AO, Preview 16 refined glass, and Preview 17 fixed
door-handle UV placement and documented LabPBR setup. Preview 18 raises door
hardware to the correct full-door height, clarifies Complementary Reimagined
LabPBR settings and identifies the in-game pack as HyperRealism 512×512. Preview
19 moves stone, andesite, tuff and overworld ore host surfaces to a rocky CC0
scan. Preview 20 adds a woven wool scan, distinct CC0 bark scans for oak,
dark-oak and jungle logs, and full-PBR Walnut Veneer maps for dark-oak wood.
Preview 21 expands coverage to 344 block texture paths, adding 36 static Nether
surfaces with volcanic rock and aligned emission maps; animated and cutout Nether
sprites remain unchanged. Preview 22 adds five adult temperate farm-mob texture
paths. Preview 23 calibrates roughness by material and reconstructs subtle height
from 26 aligned normal maps; these remain experimental and need in-game review.
Pale oak keeps the color-matched maple source. These
are targeted source improvements, not a complete or visually validated
photorealistic catalog. A separate synthetic manifest contains eight
test examples. The repository did
not contain the earlier Python code or manifest, so this is a new schema and
implementation, not a port or a claim of matching its results.

**Design target:** Windows 11, RTX 5060 Ti 8 GB, i5-13400 and 16 GB RAM.
This is a target, not a claim of measured performance or GPU acceleration.
The generator is currently CPU-only; shader rendering uses your GPU. This
repository also includes an experimental Iris/LabPBR shader prototype at
[`shaderpacks/HyperRealism-Stylized-PBR`](shaderpacks/HyperRealism-Stylized-PBR/README.md),
based on MIT-licensed Glimmer. It has not yet been shader-compiled or validated
in Minecraft; Photon remains the documented practical option for now.

See [shader setup and testing](docs/SHADER_SETUP.md), [full block coverage and the custom-shader assessment](docs/FULL_BLOCK_SUPPORT_PLAN.md), [Continuity compatibility notes](docs/CONTINUITY_SUPPORT.md), and [the research and staged rebuild plan](docs/REBUILD_PLAN.md). The pinned 26.3 metadata audit currently finds 1,272 distinct block texture references; the block catalog covers 371, leaving 901 uncovered, so complete vanilla block coverage remains unfinished. The current source build additionally supports five experimental entity texture paths (adult temperate chicken, cow and pig, plus sheep base and wool layer); baby, other-biome, equipment and hostile-mob textures remain vanilla. Experimental Preview 22 includes these five adult temperate entity paths; all other mob variants remain vanilla. See the [farm-mob pilot notes](experiments/common_farm_mobs/README.md).

## Download and install the experimental preview

### Windows GUI installer / updater

The launcher's dark interface inspires this installer; it is a separate app,
not PineconeMC. Preview pack releases also include the installer as an asset.

Download [HyperRealism128-Installer.exe](https://github.com/Skid-ster/hyperrealismBYARENA/releases/download/v0.1.0-installer.12/HyperRealism128-Installer.exe)
and double-click it. Installer 12 adds cancellable pack and installer downloads,
byte-level progress, 30-second network timeouts, and 1 GiB / 100 MiB size caps.
Declined or cancelled automatic update prompts are deferred until the next launch.
The app defaults to the current user's
`%APPDATA%\ElyPrismLauncher\instances\26.3(1)\minecraft\resourcepacks`;
browse or type another **existing** `resourcepacks` folder if needed. Green
means the folder exists and has the expected name—not that the game actually
uses that folder. Use **Check updates** to compare the installed file's
hash with the latest preview, or **Install / Update** to download and install
it. The toolbar keeps to standard installer actions: **Folders** provides folder
shortcuts, **Settings** controls automatic update checks, **Help** opens project
guidance, and **Check updates** checks both the pack and installer. Use the
resource-pack path's **Browse...** button to select an existing folder; the app
does not create or manage launcher instances. The sidebar **Launch** asks
PineconeMC to launch the instance inferred from the selected standard
`instances/<instance>/minecraft/resourcepacks` folder. It uses PineconeMC's own account and game settings; it does not log in or install the game. The first time, if the launcher is not found in common install locations, select `elyprismlauncher.exe` (or the PineconeMC executable); the selection is remembered under `%LOCALAPPDATA%\HyperRealism128\pineconemc-path.txt` and can be changed in **Settings**. For custom instance directories launch directly from PineconeMC. The request starts the launcher but cannot verify Minecraft actually started. The grass block artwork is used under CC BY-SA 4.0 with [attribution](installer/assets/ATTRIBUTION.txt); other glyphs are independently drawn. Only the named preview ZIP is replaced. By default, at startup and hourly while the
installer is open, it checks for newer installer releases and, if a pack is
already installed in the chosen folder, for a different latest pack. It asks
before updating. A declined automatic prompt is deferred for that version until
the next installer launch. Pack and installer downloads can be cancelled from
the progress row without replacing the current file. Downloads show byte progress,
use 30-second network
timeouts, and are capped at 1 GiB for a pack and 100 MiB for the installer. An
installer update is
checksum-verified, then a short-lived CMD helper replaces the executable and
reopens it. No checks
or notifications run while the app is closed; there is no background service or
Windows startup entry. Installer version 1 cannot update itself, so download
version 2 once manually. The selected existing resourcepacks path is remembered
in `%LOCALAPPDATA%\HyperRealism128\resourcepacks-path.txt`. The pack is not
enabled automatically; enable it in Minecraft and reload resources after an
update. The executable is unsigned, so Windows
may display a SmartScreen warning. The download checksum comes from the same
release, not an independent signature. GUI source and offline tests are under
`installer/`; build them on Windows with `powershell -File installer/build.ps1`.

### Manual installation

No Rust installation is needed to use the prebuilt pack. Preview 23 uses 512×512
textures (the legacy `HyperRealism128` archive name is retained for updater
compatibility), which requires substantially more GPU
memory than the previous 128×128 preview; performance on the RTX 5060 Ti has
not yet been tested. Enable a compatible shader and LabPBR settings as described
in the [shader guide](docs/SHADER_SETUP.md). Download
[HyperRealism 512×512 Preview 23 ZIP](https://github.com/Skid-ster/hyperrealismBYARENA/releases/download/v0.1.0-preview.23/HyperRealism128-MC26.3-EXPERIMENTAL.zip),
place the ZIP (do **not** extract it) in `%APPDATA%\.minecraft\resourcepacks`,
and enable it in Minecraft Java 26.3 under **Options → Resource Packs**. Preview 23
contains 344 distinct block texture paths at 512×512; the current development
branch adds five copper and three raw-ore paths, defaults to 256×256, and has not
yet been published. That default has one quarter as many block texels as the
512×512 preview, which should reduce texture storage, but no in-game VRAM/FPS
benchmark has been run. Both remain subsets of the game; all other textures stay
vanilla or come from other enabled packs. A compatible shader is required for PBR, and optional
LabPBR features vary by shader. The previous preview was tried in-game;
the new roughness calibration and normal-derived height estimates have not yet been validated there. Grass sides now have a
separate biome-tinted cutout fringe; six common broadleaf types share a
photographic foliage atlas with transparency and static mipmap metadata.
Dirt, gravel, snow, mud, dirt paths and podzol use new scanned ground maps.
Preview 8 also adds stone and deepslate families, terracotta, clay,
other building blocks and ten log ends with approximate procedural rings. Preview 9 adds
stripped wood, clear and stained glass/pane caps, and three leaf variants. Preview 10 adds tinted glass, connected-glass support and a modest stone PBR boost (later unified to 0.2 for all targets in Preview 14).
Preview 11 adds doors and trapdoors, ladders and iron/copper bars with custom
cutout layouts, photographic materials and PBR maps; their geometry remains
Minecraft’s original block models. Preview 12 added bamboo planks, fences and gates plus matching particle
sprites. Preview 13 redraws the procedural door and trapdoor construction masks,
adds a custom PBR lever texture while preserving Minecraft models and powered
states, and sets a per-texture PBR strength of 0.16 (40% of the previous 0.4 baseline) on 55
door, trapdoor and lever targets. All use aligned LabPBR maps; wood grain reuses
CC0 Poly Haven veneer scans and metal details use the documented CC0 steel scan.
Preview 14 softens the procedural door-panel shade/relief contrast, narrows
glass rims, and applies the global 0.2 PBR strength uniformly with no overrides.
Preview 15 adds normal-derived height and subtle cavity AO to the shared CC0
broadleaf atlas; it does not add canopy geometry. Preview 16 removes decorative
edge bands from clear, stained, tinted and connected glass, and lowers the
average target color by 4% for all 35 glass-block and pane-cap sprites. Clear
glass has uniform 12% opacity; stained/tinted glass retains uniform 40% opacity.
Pane-cap alpha silhouettes, models and UVs remain unchanged because they define
the vanilla pane shape, not a decorative rim. Preview 17 moved the door handle
to the latch-side UV edge; Preview 18 raises it within the lower-block sprite
so it sits near full-door mid-height and removes a pale-oak band that obscured
its handle. Door/trapdoor cutout alpha, Minecraft
models, UVs and states remain unchanged. The resource-pack menu name is now
**HyperRealism 512×512**. For Complementary Reimagined, set **RP Support** to
**labPBR (RP Required)**; the shader's Integrated PBR+ mode does not fully read
this pack's LabPBR maps. POM is optional. Water is unchanged. Foliage remains a
single broadleaf source shared among species. Research notes:
[competing PBR techniques and licensing](docs/PBR_TECHNIQUE_RESEARCH.md). It
replaces only specified
blocks: try `cobblestone`, `mossy_cobblestone`, `dirt`, `sand`,
`stone_bricks`, `oak_planks`, `white_wool`, `gold_ore` and `diamond_ore` to
compare. See the manifest for the full list. Other textures stay unchanged.

To update manually, check the [releases page](https://github.com/Skid-ster/hyperrealismBYARENA/releases)
for a newer preview, download its ZIP, and replace the older preview ZIP in the
resourcepacks folder. Keep only one preview version enabled.

Alternatively, Windows users can download and review the
[preview installer script](https://github.com/Skid-ster/hyperrealismBYARENA/releases/download/v0.1.0-preview.23/Install-HyperRealismPreview.ps1).
In PowerShell, from the folder where it was downloaded, run:

```powershell
powershell -ExecutionPolicy Bypass -File .\Install-HyperRealismPreview.ps1
# To update or replace an existing installed preview:
powershell -ExecutionPolicy Bypass -File .\Install-HyperRealismPreview.ps1 -Force
```

The script selects the newest published release containing the preview ZIP and
its SHA-256 file, verifies the download before replacing the existing preview,
and does not run the downloaded ZIP. The checksum detects corruption; it is not
an independent signature. `-Destination` chooses a different resourcepacks
folder. GitHub API rate limits can affect unauthenticated requests.

## Connected glass

Preview 14 includes compact CTM data for clear, all 16 stained colors and tinted
glass, including pane sides. The files were checked against Continuity 3.0.1+26.3's
source loader contract; the generated namespace, compact method, minimum tile
count, pane-face rule and all tile maps are covered by automated structural tests.
See [Continuity compatibility notes](docs/CONTINUITY_SUPPORT.md). Runtime loading
and visual rendering still need a Minecraft 26.3 + Continuity installation. The
fallback textures and CTM tiles now use uniform opacity without drawn edge rims;
CTM still preserves compatibility, but has no visible frame differences. **Iris
alone does not supply CTM**. For Fabric 26.3, add [Continuity 3.0.1+26.3](https://modrinth.com/mod/continuity/version/3.0.1+26.3)
and its required Fabric API, then restart. Enable its glass-pane culling fix if pane
seams remain, and put HyperRealism above competing glass packs. OptiFine with
connected textures enabled is another possible route when available for the version. Clear glass is uniformly low-opacity; reflections still vary by shader. Stained glass remains translucent. Pane-cap alpha keeps its vanilla narrow-strip silhouette. CTM tiles include flat normals and smooth
LabPBR specular maps, but shader/mod support varies.

## Build from source on Windows 11

Install a current stable Rust toolchain from <https://rustup.rs/> (and the
Microsoft C++ Build Tools if Rust prompts for them). From PowerShell in this
repository:

```powershell
cargo test --locked
cargo run --locked --release -- --input work --output pilot_output --cpu-only
# Optional lower-memory 128×128 pack:
cargo run --locked --release -- --input work --output pilot_128 --cpu-only --resolution 128
# Optional higher-detail 512×512 pack:
cargo run --locked --release -- --input work --output pilot_512 --cpu-only --resolution 512
```

The output directory must not already contain `pack`, `dist`, or
`contact_sheet.png`, or `build_report.json`; choose a fresh `--output` directory
for another run.
Outputs:

- `pilot_output/pack/` — the starter resource pack, including metadata,
  `MATERIAL_CREDITS.txt`, `ENTITY_TEXTURES.txt`, block-map triples under
  `assets/minecraft/textures/block/`, and declared entity albedo/`_n`/`_s`
  triples under `assets/minecraft/textures/entity/`.
- `pilot_output/dist/HyperRealism128-v1.0-MC26.3.zip` — pack contents at ZIP root.
- `pilot_output/contact_sheet.png` — the albedo examples in manifest order,
  four tiles per row (outside the pack).
- `pilot_output/build_report.json` — output recipes, material provenance and
  SHA-256 hashes of the manifest and input maps (outside the pack).

The ZIP retains its legacy `HyperRealism128` filename for updater compatibility;
the development build now defaults to **256×256**. The 512×512 option remains
available, and the filename/v1.0 marker does not imply full texture coverage. The `work` starter manifest uses twenty-five CC0 Poly Haven
material sources; `test_assets` contains synthetic test inputs. See
`work/ASSET_CREDITS.md`. A LabPBR-compatible shader is required for PBR maps. Windows CI passes on a GitHub-hosted runner; in-game validation and tests
on your Windows 11 machine remain outstanding.

`--threads N` controls the Rayon pool (default: Rayon determines the thread
count). The current prototype parallelizes scalar output-channel resampling, not
the whole pipeline; the operating system schedules threads. It does not detect or
pin P-cores/E-cores. `--cpu-only` is accepted explicitly; currently **every**
run uses CPU, and there is no GPU backend.

## PBR strength

The bundled manifest uses a single global `pbr_strength` of **0.2** for all 371
texture targets; per-texture overrides have been removed. This scales normal
slope, height deviation and smoothness to 20% of their unit-strength values,
and fades AO toward white. Base colors and discrete metal/SSS/emission fields
are not scaled. Actual appearance and reflection brightness depend on the
shader; 20% channel strength is not a guarantee of exactly 20% perceived
intensity. Preview 16 removes the drawn clear/stained-glass rim entirely and
slightly darkens all glass swatches; softened door relief remains a separate
layout change. Source builders
can set `pbr_strength` between 0 and 1 in the manifest (omitting it defaults to
1 for backwards compatibility). This does not change water.

## Custom input

`--input` accepts a directory containing `material_manifest.json` or the path to
that file. The following is a version-1 manifest; relative PNG/JPEG paths are
resolved inside the manifest's directory and must not escape it:

```json
{
  "version": 1,
  "materials": [
    {"id":"base", "albedo":"diffuse.jpg", "normal":"normal.png",
     "normal_y":"opengl", "roughness":"rough.png", "height":"height.png",
     "ao":"ao.png", "source":"Poly Haven: material ID", "license":"CC0"}
  ],
  "textures": [
    {"name":"iron_block", "material":"base", "recipe":"metal",
     "tint":[210,210,210], "f0":230, "porosity":0, "emissive":255}
  ],
  "entity_textures": [
    {"path":"entity/chicken/chicken_temperate",
     "albedo":"entity_textures/chicken_temperate.png",
     "normal":"entity_textures/chicken_temperate_n.png",
     "specular":"entity_textures/chicken_temperate_s.png",
     "source":"original procedural artwork", "license":"not separately licensed"}
  ]
}
```

`normal`, `roughness`, `height`, and `ao` are optional PNG or JPEG maps;
`normal_y` is `directx` (default) or `opengl`. Roughness, height and AO use the
red/gray channel as linear 0–1 data; 16-bit PNG precision is preserved for
these scalar maps. Alternatively, `arm` is a packed PNG/JPEG with linear AO
in red and roughness in green; its metallic blue channel is ignored and LabPBR
metal IDs must be chosen separately. `arm` cannot be combined with separate
`ao` or `roughness`. Roughness defaults to 0.8 and AO to 1. Material-level
`roughness_bias` adds a bounded calibration to source roughness; it does not
change the global PBR strength. `roughness_from_height` can derive a rougher
stone-joint response from an authored height map. If height is absent, a
luminance-based approximation is used unless `height_from_normal` is enabled;
that option estimates a periodic height field from the normal map for surfaces
with meaningful relief. These reconstructions are conservative approximations,
not measured displacement. Without a normal map, normals are derived from
height. When a normal map is present, it is combined with recipe relief without
adding the height-map gradients twice. JPEG alpha is opaque;
CMYK JPEG is not supported. Source and license are optional, user-provided
provenance fields and are recorded, not verified. `tint` is an sRGB multiplier,
not a target palette match. An optional `target_color: [R,G,B]` applies a bounded gain to move the
source's average linear color toward an approximate sRGB target; clipping,
material variation, and recipe layers can change the final average. This is not
a pixel-level palette match. Supported experimental recipes are `material`
(preserve the source), `stone`,
`ore`, `brick`, `cracked_brick`, `cobble`, `moss_cobble`, `moss_brick`, `plank`,
`bamboo`, `moss`, `glass`, `metal`, `gem`, and `leaves`. The bamboo recipe adds
periodic board joints, subtle longitudinal fibers and node relief over the CC0
veneer input; it is an approximation rather than a bamboo scan. These are
experimental procedural recipes, not all fully validated material-specific
recipes. An optional lowercase `pattern_seed` lets texture aliases with matching
inputs and parameters generate identical maps. Names must be lowercase ASCII
letters, digits or underscores. The bundled catalog names and color hints were checked
against a pinned 26.3 asset inventory (see `catalog/vanilla_26.3_audit.json`).
A separate targeted model-reference audit checks how 125 common building-family
blockstates resolve shared stair/slab/wall, wood and multi-face textures; see
[the consistency audit](docs/TEXTURE_VARIANT_AUDIT.md). These checks have not
yet been validated in game. Custom names are not checked unless
`--minecraft-jar` is supplied.
F0 defaults to 10; 230–237 are hardcoded metal IDs, 255 is custom metal, and
238–254 are rejected. Porosity defaults to 0 and must be 0–64. Emissive defaults
to the LabPBR no-emission sentinel 255; values 0–254 are passed through. For
`ore`, supply `ore_color` (sRGB RGB bytes) and `ore_f0`; an irregular periodic
inclusion mask mixes the color and selects the discrete F0/metal ID. Bundled
ore inclusions and concrete-based host rocks are experimental approximations,
not scanned ore materials; redstone ore is not emissive by default.

For mixed materials, `f0_mask` specifies a grayscale PNG/JPEG path and
`masked_f0` supplies the other encoded F0/metal ID. After linear filtering a
mask value ≥0.5 selects `masked_f0`, otherwise `f0`; IDs are never interpolated.
These two fields must be supplied together and cannot be used with `ore`.
`subsurface` is an optional **encoded byte** 65–255 in the B channel, mutually
exclusive with nonzero `porosity`; the B channel is forced to 0 on metallic
pixels. `emissive_map` is a grayscale linear map, filtered to the output and
encoded as 0–254 (black means zero emission); it cannot be combined with a
non-default `emissive`. The 255 sentinel means emission is disabled. Per-pixel
map paths use the same validation and SHA-256 reporting as material maps.

`entity_textures` is optional. Each entry copies an existing entity atlas and
its LabPBR normal/specular maps without resampling or changing the model or UVs.
The `path` is relative to `assets/minecraft/textures/`, has no `.png` suffix,
and must stay under `entity/`; the builder writes `_n` and `_s` siblings. All
three source PNGs must be aligned 8-bit RGBA images with identical dimensions.
This path is a direct-map integration, not generated from block materials.
Entity `source` and `license` strings are recorded, not independently verified;
entity animation metadata and in-game/shader appearance are not validated.

LabPBR `_n` channels are DirectX tangent-space X/Y, ambient occlusion and
height; `_s` channels are `1 - sqrt(roughness)`, F0/metal ID, porosity/SSS,
and emission. Height output is clamped to a minimum byte value of 1 to avoid
issues with some parallax implementations.
See the [LabPBR 1.3 material standard](https://shaderlabs.org/wiki/LabPBR_Material_Standard).
Shader packs vary in support for AO, height, SSS, porosity and emission; map
encoding alone does not guarantee that any particular shader renders them.

The material working size remains 512×512. Source maps are periodically
resampled using Lanczos-3 before recipe processing; albedo filtering uses
premultiplied alpha and linear light. Block maps default to 256×256 and can be
set to 128×128 or 512×512 with `--resolution`; continuous channels use a
periodic Lanczos-3 filter and normals are filtered as vectors then renormalized.
Entity maps default to 256px wide while preserving each vanilla atlas ratio
(chicken/sheep 256×128; cow/pig 256×256). Their non-tiling edges are clamped,
albedo is filtered in linear premultiplied space, and LabPBR ID/sentinel
channels are kept discrete. Animated sources remain
unsupported. Inputs up to 8192×8192 are
supported, but large inputs may use substantial RAM. Only the most recently
used prepared material is cached. Albedo and normal PNGs currently use 8-bit
precision; color profiles and EXR/TIFF are not supported. The ZIP currently
uses the standard uncompressed/store method, not deflate.

Each generated pack is checked before packaging for consistently sized
128×128, 256×256 or 512×512 block-map triples and aligned 8-bit RGBA entity atlas
triples at 128, 256 or 512px wide, preserving vanilla entity aspect ratios, version metadata, plausible
normal XY, nonzero POM height, reserved material IDs, and metal blue-channel
rules. The standalone `--verify-pack` check also scans nested entity paths and
rejects missing or mismatched sidecars. You can rerun these structural checks
on an unpacked generated pack:

```powershell
cargo run --locked --release -- --verify-pack pilot_output/pack
```

This is **not** an in-game, appearance, license, or shader compatibility test.
Normals, height and roughness are approximated where the input lacks those maps;
the verifier cannot determine whether those approximations look realistic.

Optionally add a top-level `"checksums"` object mapping input-relative paths to
expected SHA-256 hex strings. Mismatches and unknown paths fail before output
is created. All map hashes are recorded even if no expected checksum is given;
those hashes alone do not establish a source's license or authenticity.

To check texture **names** against a locally installed Minecraft client JAR:

```powershell
cargo run --locked --release -- --input work --output pilot_output2 --cpu-only --minecraft-jar "$env:APPDATA\.minecraft\versions\26.3\26.3.jar"
```

This fails on missing target paths, animation metadata or unknown `.png.mcmeta`
sections. Static `texture` metadata (such as mipmap strategy) is copied to the
corresponding generated albedo's sidecar and ZIP. It does not check the JAR's
version, validate palettes, or redistribute game images. If the flag is omitted,
target names are not checked against Minecraft. You must supply your own client
JAR and appropriate, lawfully obtained input materials.

## Still needed

- Expand beyond the current 371-output subset, especially animated/cutout
  Nether vegetation; acquire suitable materials for each family and review
  palettes/geometry in game. It is not guaranteed to overlap the missing Python
  implementation.
- wgpu/Vulkan, GPU FFT, Voronoi, VRAM budgeting/batching, benchmarking and
  CPU/GPU parity testing on the RTX 5060 Ti.
- `fast_image_resize` SIMD, benchmarked CPU/GPU crossover, and safe Windows 11
  P/E-core detection/scheduling. No hardware-optimization or speed claim has
  been measured for the target machine.
- Explicit texture animation handling, broader color/alpha correctness tests,
  production error handling, and Minecraft/shader visual validation.

Tests include PNG/JPEG decoding, 16-bit scalar inputs, alpha-aware periodic
filtering, input hashes, JAR preflight, channel values, output layout, mixed metal IDs, emission masks, SSS, alpha filtering, static metadata,
no-overwrite behavior and matching ZIP bytes across one and two Rayon threads.
GitHub Actions passed the suite and CPU smoke run on Linux and Windows runners;
this is not a substitute for testing on the specified local hardware.
