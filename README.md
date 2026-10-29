# HyperRealism Rust — CPU prototype

This is an **early prototype**, not the requested complete production pack.
It generates a small, manifest-defined set of 128×128 Minecraft Java 26.3 block
textures and LabPBR 1.3 maps using a CPU-only pipeline. The included synthetic
manifest contains eight examples; it does **not** contain the 280-texture catalog,
real Poly Haven assets, or a photorealistic finished pack. The repository did
not contain the earlier Python code or manifest, so this is a new schema and
implementation, not a port or a claim of matching its results.

See [the research and staged rebuild plan](docs/REBUILD_PLAN.md).

## Run on Windows 11

Install a current stable Rust toolchain from <https://rustup.rs/> (and the
Microsoft C++ Build Tools if Rust prompts for them). From PowerShell in this
repository:

```powershell
cargo test --locked
cargo run --locked --release -- --input test_assets --output test_output --cpu-only
```

The output directory must not already contain `pack`, `dist`, or
`contact_sheet.png`; choose a fresh `--output` directory for another run.
Outputs:

- `test_output/pack/` — the example resource pack, including metadata and
  `assets/minecraft/textures/block/{name}.png`, `{name}_n.png`, `{name}_s.png`.
- `test_output/dist/HyperRealism128-v1.0-MC26.3.zip` — pack contents at ZIP root.
- `test_output/contact_sheet.png` — the albedo examples in manifest order,
  four tiles per row (outside the pack).

The ZIP's name is a target artifact name, **not** an indication that the full
v1.0 pack is finished. The bundled PNGs are deliberately synthetic for testing,
not Poly Haven materials. A LabPBR-compatible shader is required to use the PBR
maps. Windows CI passes on a GitHub-hosted runner; in-game validation and tests
on your Windows 11 machine remain outstanding.

`--threads N` controls the Rayon pool (default: Rayon determines the thread
count). The current prototype parallelizes scalar channel resampling, not the
whole pipeline; the operating system schedules threads. It does not detect or
pin P-cores/E-cores. `--cpu-only` is accepted explicitly; currently **every**
run uses CPU, and there is no GPU backend.

## Custom input

`--input` accepts a directory containing `material_manifest.json` or the path to
that file. The following is a minimal version-1 manifest; relative PNG paths are
resolved inside the manifest's directory and must not escape it:

```json
{
  "version": 1,
  "materials": [
    {"id":"base", "albedo":"diffuse.png", "normal":"normal.png",
     "normal_y":"opengl", "roughness":"rough.png"}
  ],
  "textures": [
    {"name":"iron_block", "material":"base", "recipe":"metal",
     "tint":[210,210,210], "f0":230, "porosity":0, "emissive":255}
  ]
}
```

`normal` and `roughness` are optional PNGs; `normal_y` is `directx` (default) or
`opengl`. Roughness is read from the red channel as a linear 0–1 value; if
omitted, it defaults to 0.8. `tint` is an sRGB multiplier, not a target palette
match. Supported experimental recipes are `stone`, `brick`, `plank`, `moss`,
`glass`, `metal`, `gem`, and `leaves`. These are simple procedural demonstrations,
not finished material-specific recipes. Names must be lowercase ASCII letters,
digits or underscores; output names are not validated against a game JAR.
F0 defaults to 10; 230–237 are hardcoded metal IDs, 255 is custom metal, and
238–254 are rejected. Porosity defaults to 0 and must be 0–64. Emissive defaults
to the LabPBR no-emission sentinel 255; values 0–254 are passed through.

The working size is 512×512. Input images of other sizes are currently sampled
with nearest-neighbour scaling before generation; use 512×512 input for best
results in this prototype. Continuous channels are downsampled with a scalar,
periodic Lanczos-3 filter. Albedo uses linear-light premultiplied alpha, and
normals are renormalized before packing. Alpha cutout coverage, non-tiling edge
modes, animations and per-pixel metal/emission masks need further work. Inputs
are decoded one material at a time, with no persistent cache. The ZIP currently
uses the standard uncompressed/store method, not deflate.

## Still needed

- Real material acquisition/provenance and an audited 280-texture catalog,
  vanilla-informed palettes and robust recipe definitions.
- wgpu/Vulkan, GPU FFT, Voronoi, VRAM budgeting/batching, benchmarking and
  CPU/GPU parity testing on the RTX 5060 Ti.
- `fast_image_resize` SIMD, benchmarked CPU/GPU crossover, and safe Windows 11
  P/E-core detection/scheduling. No hardware-optimization or speed claim has
  been measured for the target machine.
- Explicit texture animation handling, broader color/alpha correctness tests,
  production error handling, and Minecraft/shader visual validation.

Tests include channel values, periodic filtering, output layout, metal ID 230,
no-overwrite behavior and matching ZIP bytes across one and two Rayon threads.
GitHub Actions passed the suite and CPU smoke run on Linux and Windows runners;
this is not a substitute for testing on the specified local hardware.
