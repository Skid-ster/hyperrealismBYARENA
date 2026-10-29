# HyperRealism Rust — CPU prototype

This is an **early prototype**, not the requested complete production pack.
It generates manifest-defined 128×128 Minecraft Java 26.3 block textures and
LabPBR 1.3 maps using a CPU-only pipeline. The starter catalog contains 96
experimental outputs based on seven CC0 Poly Haven materials; it is **not** the
requested 280-texture catalog or a finished photorealistic pack. A separate
synthetic manifest contains eight test examples. The repository did
not contain the earlier Python code or manifest, so this is a new schema and
implementation, not a port or a claim of matching its results.

See [the research and staged rebuild plan](docs/REBUILD_PLAN.md).

## Run on Windows 11

Install a current stable Rust toolchain from <https://rustup.rs/> (and the
Microsoft C++ Build Tools if Rust prompts for them). From PowerShell in this
repository:

```powershell
cargo test --locked
cargo run --locked --release -- --input work --output pilot_output --cpu-only
```

The output directory must not already contain `pack`, `dist`, or
`contact_sheet.png`, or `build_report.json`; choose a fresh `--output` directory
for another run.
Outputs:

- `pilot_output/pack/` — the starter resource pack, including metadata and
  `assets/minecraft/textures/block/{name}.png`, `{name}_n.png`, `{name}_s.png`.
- `pilot_output/dist/HyperRealism128-v1.0-MC26.3.zip` — pack contents at ZIP root.
- `pilot_output/contact_sheet.png` — the albedo examples in manifest order,
  four tiles per row (outside the pack).
- `pilot_output/build_report.json` — output recipes, material provenance and
  SHA-256 hashes of the manifest and input maps (outside the pack).

The ZIP's name is a target artifact name, **not** an indication that the full
v1.0 pack is finished. The `work` starter manifest uses seven CC0 Poly Haven
materials; `test_assets` contains synthetic test inputs. See
`work/ASSET_CREDITS.md`. A LabPBR-compatible shader is required for PBR maps. Windows CI passes on a GitHub-hosted runner; in-game validation and tests
on your Windows 11 machine remain outstanding.

`--threads N` controls the Rayon pool (default: Rayon determines the thread
count). The current prototype parallelizes scalar output-channel resampling, not
the whole pipeline; the operating system schedules threads. It does not detect or
pin P-cores/E-cores. `--cpu-only` is accepted explicitly; currently **every**
run uses CPU, and there is no GPU backend.

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
these scalar maps. Roughness defaults to 0.8 and AO to 1. If height is absent,
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
`moss`, `glass`, `metal`, `gem`, and `leaves`. These are simple procedural
demonstrations,
not finished material-specific recipes. Names must be lowercase ASCII letters,
digits or underscores. The bundled catalog names and color hints were checked
against a pinned 26.3 asset inventory (see `catalog/vanilla_26.3_audit.json`);
they have not yet been validated in game. Custom names are not checked unless
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
and emission. A generated height byte can be zero; the LabPBR specification
recommends a minimum of 1 to avoid issues with some parallax implementations.
See the [LabPBR 1.3 material standard](https://shaderlabs.org/wiki/LabPBR_Material_Standard).
Shader packs vary in support for AO, height, SSS, porosity and emission; map
encoding alone does not guarantee that any particular shader renders them.

The working size is 512×512. Source maps are periodically resampled using
Lanczos-3 before recipe processing; albedo filtering uses premultiplied alpha
and linear light. Continuous channels are then downsampled to 128×128 using a
scalar periodic Lanczos-3 filter. Normals are filtered as vectors and
renormalized. Alpha cutout coverage preservation, non-tiling edge modes and
animations need further work. Inputs up to 8192×8192 are
supported, but large inputs may use substantial RAM. Only the most recently
used prepared material is cached. Albedo and normal PNGs currently use 8-bit
precision; color profiles and EXR/TIFF are not supported. The ZIP currently
uses the standard uncompressed/store method, not deflate.

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

- Expand the 96-output starter to the requested catalog; acquire appropriate
  materials for each family and review palette/geometry in game. This 96-output
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
