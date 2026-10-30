# HyperRealism Rust — CPU prototype

This is an **early prototype**, not the requested complete production pack.
It generates manifest-defined 512×512 (or optional 128×128) Minecraft Java 26.3 block textures and
LabPBR 1.3 maps using a CPU-only pipeline. Most starter materials use 1K
Poly Haven-derived maps; some older maps are 512px or 2K, and missing channels
are approximated as described below. The current source catalog contains 308
distinct block texture targets based on CC0 Poly Haven maps and original
procedural work; Preview 13 adds the lever texture and PBR coverage for
toggleable doors, trapdoors and levers. This is **not** the requested complete pack or a
finished photorealistic catalog. A separate synthetic manifest contains eight
test examples. The repository did
not contain the earlier Python code or manifest, so this is a new schema and
implementation, not a port or a claim of matching its results.

**Design target:** Windows 11, RTX 5060 Ti 8 GB, i5-13400 and 16 GB RAM.
This is a target, not a claim of measured performance or GPU acceleration.
The generator is currently CPU-only; shader rendering uses your GPU.

See [shader setup and testing](docs/SHADER_SETUP.md), [Continuity compatibility notes](docs/CONTINUITY_SUPPORT.md), and [the research and staged rebuild plan](docs/REBUILD_PLAN.md).

## Download and install the experimental preview

### Windows GUI installer / updater

The launcher's dark interface inspires this installer; it is a separate app,
not PineconeMC. Preview pack releases also include the installer as an asset.

Download [HyperRealism128-Installer.exe](https://github.com/Skid-ster/hyperrealismBYARENA/releases/download/v0.1.0-installer.9/HyperRealism128-Installer.exe) and double-click it. The app defaults to the current user's
`%APPDATA%\ElyPrismLauncher\instances\26.3(1)\minecraft\resourcepacks`;
browse or type another **existing** `resourcepacks` folder if needed. Green
means the folder exists and has the expected name—not that the game actually
uses that folder. Use **Check updates** to compare the installed file's
hash with the latest preview, or **Install / Update** to download and install
it. The toolbar mirrors the reference: **Add Instance** selects an existing
instance's resourcepacks folder (it does not create an instance), **Folders**
provides folder actions, **Settings** controls automatic update checks,
**Help** opens project guidance, and **Update** checks both the pack and installer. The sidebar **Launch** asks PineconeMC to launch the instance inferred from the selected standard `instances/<instance>/minecraft/resourcepacks` folder. It uses PineconeMC's own account and game settings; it does not log in or install the game. The first time, if the launcher is not found in common install locations, select `elyprismlauncher.exe` (or the PineconeMC executable); the selection is remembered under `%LOCALAPPDATA%\HyperRealism128\pineconemc-path.txt` and can be changed in **Settings**. For custom instance directories launch directly from PineconeMC. The request starts the launcher but cannot verify Minecraft actually started. The grass block artwork is used under CC BY-SA 4.0 with [attribution](installer/assets/ATTRIBUTION.txt); other glyphs are independently drawn. Only the named preview ZIP is replaced. By default, at startup and hourly while the
installer is open, it checks for newer installer releases and, if a pack is
already installed in the chosen folder, for a different latest pack. It asks
before updating. An installer update is downloaded and checksum-verified,
then a short-lived CMD helper replaces the executable and reopens it. No checks
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

No Rust installation is needed to use the prebuilt pack. Preview 13 uses 512×512
textures (despite the legacy filename), which requires substantially more GPU
memory than the previous 128×128 preview; performance on the RTX 5060 Ti has
not yet been tested. Enable a compatible shader and LabPBR settings as described
in the [shader guide](docs/SHADER_SETUP.md). Download
[HyperRealism128-MC26.3-EXPERIMENTAL.zip](https://github.com/Skid-ster/hyperrealismBYARENA/releases/download/v0.1.0-preview.13/HyperRealism128-MC26.3-EXPERIMENTAL.zip),
place the ZIP (do **not** extract it) in `%APPDATA%\.minecraft\resourcepacks`,
and enable it in Minecraft Java 26.3 under **Options → Resource Packs**. Its
308 distinct texture paths replace only a subset of the game; the rest remain
vanilla or come from other enabled packs. A compatible shader is required for PBR, and optional
LabPBR features vary by shader. The previous preview was tried in-game;
this revision has not yet been validated there. Grass sides now have a
separate biome-tinted cutout fringe; six common broadleaf types share a
photographic foliage atlas with transparency and static mipmap metadata.
Dirt, gravel, snow, mud, dirt paths and podzol use new scanned ground maps.
Preview 8 also adds stone and deepslate families, terracotta, clay,
other building blocks and ten log ends with approximate procedural rings. Preview 9 adds
stripped wood, clear and stained glass/pane caps, and three leaf variants. Preview 10 adds tinted glass, connected-glass support and a modest stone PBR boost.
Preview 11 adds doors and trapdoors, ladders and iron/copper bars with custom
cutout layouts, photographic materials and PBR maps; their geometry remains
Minecraft’s original block models. Preview 12 added bamboo planks, fences and gates plus matching particle
sprites. Preview 13 redraws the procedural door and trapdoor construction masks,
adds a custom PBR lever texture while preserving Minecraft models and powered
states, and sets a per-texture PBR strength of 0.16 (40% of the previous 0.4 baseline) on 55
door, trapdoor and lever targets. All use aligned LabPBR maps; wood grain reuses
CC0 Poly Haven veneer scans and metal details use the documented CC0 steel scan.
Water is unchanged. It replaces only specified
blocks: try `cobblestone`, `mossy_cobblestone`, `dirt`, `sand`,
`stone_bricks`, `oak_planks`, `white_wool`, `gold_ore` and `diamond_ore` to
compare. See the manifest for the full list. Other textures stay unchanged.

To update manually, check the [releases page](https://github.com/Skid-ster/hyperrealismBYARENA/releases)
for a newer preview, download its ZIP, and replace the older preview ZIP in the
resourcepacks folder. Keep only one preview version enabled.

Alternatively, Windows users can download and review the
[preview installer script](https://github.com/Skid-ster/hyperrealismBYARENA/releases/download/v0.1.0-preview.13/Install-HyperRealismPreview.ps1).
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

Preview 13 includes compact CTM data for clear, all 16 stained colors and tinted
glass, including pane sides. The files were checked against Continuity 3.0.1+26.3's
source loader contract; the generated namespace, compact method, minimum tile
count, pane-face rule and all tile maps are covered by automated structural tests.
See [Continuity compatibility notes](docs/CONTINUITY_SUPPORT.md). Runtime loading
and visual rendering still need a Minecraft 26.3 + Continuity installation. Without
CTM support the pack falls back to thin-bordered individual glass. **Iris alone
does not supply CTM**. For Fabric 26.3, add [Continuity 3.0.1+26.3](https://modrinth.com/mod/continuity/version/3.0.1+26.3)
and its required Fabric API, then restart. Enable its glass-pane culling fix if pane
seams remain, and put HyperRealism above competing glass packs. OptiFine with
connected textures enabled is another possible route when available for the version. Clear glass uses a cutout
layer: transparent interiors cannot be guaranteed to show reflections in every
shader; stained glass is translucent. CTM tiles include flat normals and smooth
LabPBR specular maps, but shader/mod support varies.

## Build from source on Windows 11

Install a current stable Rust toolchain from <https://rustup.rs/> (and the
Microsoft C++ Build Tools if Rust prompts for them). From PowerShell in this
repository:

```powershell
cargo test --locked
cargo run --locked --release -- --input work --output pilot_output --cpu-only
# Optional smaller 128×128 pack, to a separate fresh directory:
cargo run --locked --release -- --input work --output pilot_128 --cpu-only --resolution 128
```

The output directory must not already contain `pack`, `dist`, or
`contact_sheet.png`, or `build_report.json`; choose a fresh `--output` directory
for another run.
Outputs:

- `pilot_output/pack/` — the starter resource pack, including metadata, material credits and
  `assets/minecraft/textures/block/{name}.png`, `{name}_n.png`, `{name}_s.png`.
- `pilot_output/dist/HyperRealism128-v1.0-MC26.3.zip` — pack contents at ZIP root.
- `pilot_output/contact_sheet.png` — the albedo examples in manifest order,
  four tiles per row (outside the pack).
- `pilot_output/build_report.json` — output recipes, material provenance and
  SHA-256 hashes of the manifest and input maps (outside the pack).

The legacy ZIP filename is retained for compatibility with the updater; the
pack generated by default is **512×512**, not 128×128. The v1.0 filename is
**not** an indication that the full pack is finished. The `work` starter manifest uses twenty-five CC0 Poly Haven
material sources; `test_assets` contains synthetic test inputs. See
`work/ASSET_CREDITS.md`. A LabPBR-compatible shader is required for PBR maps. Windows CI passes on a GitHub-hosted runner; in-game validation and tests
on your Windows 11 machine remain outstanding.

`--threads N` controls the Rayon pool (default: Rayon determines the thread
count). The current prototype parallelizes scalar output-channel resampling, not
the whole pipeline; the operating system schedules threads. It does not detect or
pin P-cores/E-cores. `--cpu-only` is accepted explicitly; currently **every**
run uses CPU, and there is no GPU backend.

## PBR strength

The bundled manifest's global `pbr_strength` remains **0.4**. Preview 13
sets an explicit **0.16** override on 54 door/trapdoor targets and the lever—
40% of that previous 0.4 baseline—while other textures retain the global
setting. This scales normal slope, height deviation and smoothness, and fades
AO toward white; base colors and discrete metal/SSS/emission fields are not
scaled. Actual appearance and reflection brightness depend on the shader; this
is not a guarantee of exactly 40% perceived intensity. Source builders
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
  ]
}
```

`normal`, `roughness`, `height`, and `ao` are optional PNG or JPEG maps;
`normal_y` is `directx` (default) or `opengl`. Roughness, height and AO use the
red/gray channel as linear 0–1 data; 16-bit PNG precision is preserved for
these scalar maps. Alternatively, `arm` is a packed PNG/JPEG with linear AO
in red and roughness in green; its metallic blue channel is ignored and LabPBR
metal IDs must be chosen separately. `arm` cannot be combined with separate
`ao` or `roughness`. Roughness defaults to 0.8 and AO to 1. If height is absent,
a luminance-based approximation is used; without a normal map, normals are
derived from height. When a normal map is present, it is combined with recipe
relief without adding the height-map gradients twice. JPEG alpha is opaque;
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

LabPBR `_n` channels are DirectX tangent-space X/Y, ambient occlusion and
height; `_s` channels are `1 - sqrt(roughness)`, F0/metal ID, porosity/SSS,
and emission. Height output is clamped to a minimum byte value of 1 to avoid
issues with some parallax implementations.
See the [LabPBR 1.3 material standard](https://shaderlabs.org/wiki/LabPBR_Material_Standard).
Shader packs vary in support for AO, height, SSS, porosity and emission; map
encoding alone does not guarantee that any particular shader renders them.

The working size is 512×512. Source maps are periodically resampled using
Lanczos-3 before recipe processing; albedo filtering uses premultiplied alpha
and linear light. Continuous channels are then kept at 512×512 by default, or downsampled to 128×128 with `--resolution 128` using a
scalar periodic Lanczos-3 filter. Normals are filtered as vectors and
renormalized. Alpha cutout coverage preservation, non-tiling edge modes and
animations need further work. Inputs up to 8192×8192 are
supported, but large inputs may use substantial RAM. Only the most recently
used prepared material is cached. Albedo and normal PNGs currently use 8-bit
precision; color profiles and EXR/TIFF are not supported. The ZIP currently
uses the standard uncompressed/store method, not deflate.

Each generated pack is checked before packaging for consistently sized 128×128 or 512×512 RGBA texture
triples, version metadata, plausible normal XY, nonzero POM height, reserved
material IDs, and metal blue-channel rules. You can rerun these structural
checks on an unpacked generated pack:

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

- Expand the 308-output starter to broader block coverage; acquire appropriate
  materials for each family and review palette/geometry in game. This 308-output
  set is not guaranteed to overlap the missing Python implementation.
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
