# HyperRealism Rust — CPU primitives (work in progress)

This repository currently contains **only a small CPU library, not a working
resource-pack generator**. The GitHub `main` revision available during this work
contained just a README; the Python reference, `work/material_manifest.json`,
material assets, vanilla palette data and texture inventory were not present.
They are required to reproduce the original pipeline without inventing its
input format and output recipes.

See [the research and staged rebuild plan](docs/REBUILD_PLAN.md) for the proposed
independent implementation and its acceptance criteria.

## Implemented

- LabPBR-style `_n` pixel encoding (normalized DirectX/Y-down normal XY,
  ambient occlusion, height) and `_s` encoding (1 − sqrt(roughness), raw F0 or
  metal ID, raw porosity and emission). Raw bytes preserve reserved values;
  callers must provide values valid for the desired LabPBR field semantics.
- A periodic central-difference height-to-normal routine.
- Scalar, separable, wrap-aware Lanczos-3 resizing for linear continuous data.
  This routine intentionally does not filter categorical metal IDs; filtering
  such IDs would create other materials. Resize and renormalize normals before
  packing; filter color in the appropriate color space, not as raw sRGB.
- Unit tests for encoding, seams, dimensions and a metal ID of 230.

Run `cargo test`. No third-party dependencies are required. Doctests are disabled
because there are no documentation examples and the sandbox toolchain lacks
`rustdoc`.

## Not implemented / needed before release

There is **no CLI**, manifest parser, PNG I/O, recipe implementation, palette,
packaging, contact sheet, or full texture coverage. There is no wgpu/Vulkan,
FFT, Voronoi, Rayon, SIMD acceleration, or P/E-core affinity. This code has not
been benchmarked against Python or tested on an RTX 5060 Ti or i5-13400.
`--cpu-only` is not available because there is no CLI yet. It cannot generate
the requested ZIP. The scalar reference filter prioritizes clarity over speed.

Please provide the Python sources, a representative material manifest and asset
layout (and the texture/palette inventory if separate). Once available, port
and compare the recipes and output channel semantics against reference fixtures.

## Planned target architecture, not yet implemented

A future implementation should use bounded batches and measure real GPU memory
use against the 8 GiB VRAM budget. Choose CPU/GPU crossover based on benchmarked
workload and transfer cost rather than assuming a universal 512-pixel threshold.
Use explicit hybrid-CPU topology detection and a portable fallback before any
P-core/E-core pinning; CPU IDs are not universally interchangeable across
platforms. SIMD downsampling needs periodic padding or an equivalent wrap-aware
implementation, with proper color-space and normal-map handling. GPU parity,
resource-pack format compatibility, and shader interpretation need validation
on the target hardware and the actual Minecraft/shader versions.
