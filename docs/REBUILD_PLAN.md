# Rust resource-pack generator: research and implementation plan

Status: **work in progress**. An initial CPU-only vertical slice and synthetic
fixtures are implemented; see the README for current capabilities and gaps. This plan describes a new, independent
implementation: there is no Python pipeline, manifest, palette, material set,
or 280-texture inventory in the repository to reproduce or compare against.

## Scope and success criteria

Build a reproducible Rust CLI which creates an original 128×128 Minecraft Java
26.3 **block-texture** pack, with LabPBR 1.3 sidecars, from explicitly selected
locally available materials. Target 280 audited block texture outputs initially;
the names and exceptions must be enumerated and validated against a 26.3 client
JAR supplied by the user, not guessed from another game version. The exact
selection and visual results will not be equivalent to the missing Python pack.
Do not claim all Minecraft textures are covered: items, entities, GUI,
animations and special textures need separate inventories and rules. Minecraft
falls back to its own assets for omitted textures.

Release acceptance: documented command builds the pack and ZIP; all requested
inventory entries have a real, individually recorded output or a build failure
identifies what is missing; every generated PNG and sidecar decodes and has the
correct dimensions; metadata and ZIP paths pass automated validation; fixed
inputs and seed give reproducible results; no silent substitutions. Visual
quality requires contact sheets and in-game review, not a file-count check.

## Research-backed constraints

- Mojang's 26.3 release notes say the resource-pack version is **97.1** [1].
  Use the requested `min_format: [97, 1]` and `max_format: 97` [2]. ZIP the
  *contents* of `pack/`, so `pack.mcmeta` is at the archive root.
- OptiFine documents `assets/minecraft/optifine/texture.properties` and
  `format=lab-pbr/1.3` [3]; a LabPBR-compatible shader is still required [4].
  `_n`: R/G DirectX XY, B linear AO, A height. `_s`: R is
  `1 - sqrt(roughness)`, G dielectric F0 or hardcoded metal, B porosity/SSS,
  A emission. In particular 230 is iron; 230–237 are defined metal IDs, and
  alpha 255 is the no-emission sentinel, not full emission [5]. Keep IDs and
  sentinel values categorical through resampling.
- Poly Haven assets are CC0 [6]. If a future downloader uses its live API,
  comply with its current attribution and User-Agent requirements [7]. A local
  manifest and cache keep generation reproducible and allow offline builds.
- `fast_image_resize` supports AVX2/SSE4.1 and Rayon [8], but periodic padding,
  color-space conversion, alpha treatment, normal renormalization, and discrete
  LabPBR channel handling must be implemented and tested around it; enabling the
  crate alone does not solve those problems.
- The i5-13400 has 6 P-cores, 4 E-cores, 16 threads and AVX2 [9]. CPU IDs are
  not reliable P/E labels. Detect topology and affinity restrictions at runtime;
  permit manual overrides and fall back safely to OS scheduling. Never assume
  the sandbox topology represents the user's machine.
- wgpu supports native Vulkan [10]. GPU FFT libraries expose varying APIs and
  wgpu version dependencies [11]; verify compatibility before choosing one. A
  GPU adapter can be a software implementation, not necessarily the RTX.

## Input and asset contract

1. Specify a **new**, versioned JSON manifest; do not pretend it matches the
   missing `work/material_manifest.json`. Include a stable asset ID, relative
   local paths for albedo and optional normal/roughness/AO/height, normal-map
   convention, color-space/channel interpretation, source attribution/license,
   and a content hash. Reject malformed paths, duplicate IDs, missing required
   files and incorrect hashes before writing output. Resolve paths relative to
   the manifest and prohibit traversal outside its designated root.
2. Keep the 280 selected Minecraft target paths and explicit recipe parameters
   in a separate, versioned catalog. Validate every target against the user's
   own 26.3 client JAR; use the JAR locally to inspect names, palettes, alpha
   masks and `.mcmeta` (without committing or distributing game assets). Define
   how vanilla palette sampling handles tinted foliage and transparent pixels.
   Report excluded animated/non-square/special textures rather than silently
   flattening or mis-sizing them. A generated replacement must be original and
   meaningful, not just an upscaled or renamed vanilla texture.
3. Provide an entirely synthetic, small test manifest with deterministic PNG
   fixtures so tests and CPU smoke runs require no network or game installation.
   Real Poly Haven data is supplied in a user-managed cache; bulk downloading is
   not a prerequisite for a test. Acquisition and attribution must be explicit.

## Pipeline and architecture

1. Validate the manifest, catalog and target paths before creating a temporary
   output tree. Resolve recipe seed by hashing the global seed and target path;
   avoid randomized hash-map iteration. Every stage reports errors with material
   and target name. Finish by atomically replacing output on success only.
2. Decode input maps to a consistent intermediate format; do not decode 280
   materials concurrently. Generate at 512×512 with reusable buffers. Implement
   separately testable procedural families: stone variants and masonry; plank,
   bark and wood ends; soil/moss; leaves with masks; metal and ore; gems;
   glass/transparency. Each recipe needs parameters, bounded variation, seam
   handling, expected material channels and its own fixtures. Compose families
   through the catalog, rather than claiming 280 distinct recipes.
3. Resample albedo in linear light with premultiplied alpha; re-encode to sRGB.
   Resample normals as vectors, normalize and encode DirectX XY. Resample height,
   AO and roughness as continuous linear data before packing. Apply discrete
   metal/emission fields after filtering where applicable. Use wrap-aware
   Lanczos-3 for periodic tiles and explicit edge modes for non-tiling masks.
4. CPU execution must work independently of Vulkan (`--cpu-only`), including
   FFT/noise and Voronoi where recipes use them. Use bounded Rayon compute and
   I/O queues and avoid nested oversubscription with `fast_image_resize`'s
   Rayon mode. On hybrid systems use topology-aware scheduling only when
   detection is trustworthy; pinning must be configurable and benchmarked
   against the OS scheduler. Separate I/O and compute pools do not by themselves
   guarantee one task per physical core.
5. Add a feature-gated wgpu Vulkan backend, sharing recipe parameters and
   producing testable CPU-equivalent outputs for the same seed. GPU kernels:
   Voronoi/noise, height normals, and LabPBR packing; choose and validate a
   compatible 2D FFT implementation before introducing spectral noise. Use
   workgroup bounds checks and adapter limit checks. Create/reuse pipelines,
   pool buffers and staging readbacks, and overlap CPU image I/O with GPU work.
   Batch a configurable 16–32 512² textures **only when useful**, subject to a
   conservative configurable VRAM budget and all actual in-flight buffers,
   scratch FFT storage and staging allocations; 8 GiB is a device ceiling, not
   a safe allocation target. Never queue the full catalog at once.
6. Default 512² single textures to CPU; consider GPU for larger operations or
   sufficiently large batches. Use per-stage timing and an explicit backend
   override to determine crossover on the RTX 5060 Ti, not a hard-coded claim
   about dispatch time. `auto` must report its selection and fail over cleanly
   when Vulkan is absent; `--gpu` should fail clearly instead of silently using
   software. Avoid promising GPU/CPU bit identity for transcendental floats;
   specify channel tolerances and categorical exactness.
7. Write `pack.mcmeta`, `assets/minecraft/optifine/texture.properties`, PNGs,
   any required animation sidecars, and a sorted deterministic ZIP. Include a
   build report with input hashes, generated/omitted/error lists, seed, backend,
   timings and licensing provenance. Produce a labeled contact sheet separately
   from the pack so it is not accidentally loaded as a game texture.

## Implementation milestones and checks

- **M1 – Contract and CPU vertical slice:** The basic CPU CLI, manifest validation,
  PNG I/O, example recipes and packaging now exist. Remaining work includes Clap,
  a separately audited catalog and more complete safety and integration tests.
  Original target: Clap CLI, schema/catalog validation,
  PNG load/write, one synthetic recipe, normal/specular encoding, wrap resize,
  pack metadata and ZIP. Integration test extracts the ZIP and verifies files,
  channels, dimensions, reproducibility, path safety and error handling.
- **M2 – Material families:** Implement and visually inspect each recipe with
  small fixtures; then enumerate and audit the target catalog using the supplied
  26.3 client JAR. Add coverage and omission reports; preserve alpha and animation
  semantics. Expand to the agreed 280 targets only when supported by fixtures.
- **M3 – Performance:** Rayon, SIMD resize with correct wrap padding and a
  bounded pipeline. Bench decode, recipe, resize, encode and total separately;
  measure peak RAM on representative materials. Test one- and multi-threaded
  results for determinism.
- **M4 – Vulkan:** GPU kernels and FFT integration; unit and integration tests
  compare against CPU with documented tolerances. Validate no GPU is needed for
  `--cpu-only`; record hardware, driver, device limits, peak VRAM, batch size
  and throughput on the actual RTX 5060 Ti. Sandbox GPU results cannot replace
  this target-machine test.
- **M5 – Release review:** Generate visual contact sheets, inspect seams and
  alpha, and test the ZIP in Minecraft 26.3 with an explicitly named compatible
  shader. Confirm the 280-entry report and 128×128 outputs. Document remaining
  unsupported textures, hardware/OS caveats and real performance figures.

## References

1. https://www.minecraft.net/en-us/article/minecraft-java-edition-26-3
2. https://texturepackmaker.com/guides/minecraft-pack-format-versions/
3. https://github.com/sp614x/optifine/blob/master/OptiFineDoc/doc/texture.properties
4. https://shaders.properties/current/how-to/pbr_standards/
5. https://bonemeal.io/labpbr
6. https://polyhaven.com/license
7. https://polyhaven.com/our-api
8. https://docs.rs/fast_image_resize
9. https://www.intel.com/content/www/us/en/products/sku/230495/intel-core-i513400-processor-20m-cache-up-to-4-60-ghz/specifications.html
10. https://github.com/gfx-rs/wgpu
11. https://docs.rs/wgsl-fft/latest/wgsl_fft/
