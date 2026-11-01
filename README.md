# HyperRealism — Resource Pack Studio

A local-first web studio and Rust CPU engine for creating Minecraft Java block
textures with LabPBR 1.3 normal/specular maps. Browse a synthetic demo collection,
import your own PNGs, tune material channels, preview on an interactive 3D block,
and export a resource-pack ZIP with a contact sheet and build report.

> **Scope, honestly:** this is a working studio and CPU generator, not a finished
> photorealistic 280-texture pack. The 24 bundled materials are original
> procedural demos, not Poly Haven scans. The hero image is illustrative. There
> is no GPU/Vulkan backend, no audited 280-target catalog, and no independent
> Minecraft 26.3 or shader validation yet. See [the rebuild plan](docs/REBUILD_PLAN.md).

## Quick start

Requires **Node.js 22.12+** and a current stable **Rust** toolchain
(<https://rustup.rs>; on Windows, install the MSVC build tools if prompted).

```bash
npm ci
npm run dev
```

Open <http://localhost:5173>. The first `npm run dev` builds the Rust engine if
needed, then starts the API on port 3001 and Vite on port 5173. The browser only
talks to relative `/api` paths; Vite proxies them locally.

UI-only development (no Rust required, but generation/import previews need a
built engine):

```bash
npm run dev:web
```

### Production build

```bash
npm run build          # typecheck + Vite build + Rust release build
npm start              # Express serves dist-web and the API on port 3001
```

Open <http://localhost:3001>. Set `API_PORT` to change the port. The server binds
to `0.0.0.0` so it also works in container previews. **This is a single-user
local service with no authentication.** Anyone who can reach the server can read
or modify the workspace. Do not expose it publicly without adding auth/TLS and
appropriate network controls. Browser cross-site mutations are rejected.

## What works

- **Material library:** 24 synthetic demos, search, category filters, favorites,
  grid/list views, sorting, and pack selection.
- **Inspector:** real CPU-generated albedo/normal/specular previews, interactive
  Three.js block with illustrative lighting, tint, roughness scale, normal
  strength, F0/metal ID, porosity, and emission. Edits autosave and survive
  reload; reset restores the demo defaults.
- **Import:** albedo PNG required, optional normal/roughness PNGs, DirectX or
  OpenGL normal convention, recipe/category selection, filename validation, and
  duplicate-name protection. Imports are persisted before the dialog closes.
- **Builds:** 64/128/256 px output, deterministic seed, 1–16 worker threads,
  optional normal/specular maps, queued/running/completed states, cancellation,
  deletion, and a 30-export history.
- **Exports:** ZIP rooted at pack contents (`pack.mcmeta` at the root), a separate
  `contact_sheet.png`, and `build_report.json` with settings, timings, CRC32s,
  backend, synthetic provenance, and unperformed game validation.
- **Responsive UI:** desktop inspector, mobile drawer/navigation, keyboard
  search (Cmd/Ctrl+K), Escape dismissal, focus-trapped dialogs, reduced-motion
  support, and local fonts.

## Output contract

```text
pack/
  pack.mcmeta
  assets/minecraft/textures/block/{name}.png
  assets/minecraft/textures/block/{name}_n.png   # if enabled
  assets/minecraft/textures/block/{name}_s.png   # if enabled
  assets/minecraft/optifine/texture.properties
dist/HyperRealism{resolution}-v1.0-MC26.3.zip
contact_sheet.png
build_report.json
```

The ZIP contains the *contents* of `pack/`, not the `pack/` folder itself.
Metadata targets `min_format: [97, 1]`, `max_format: 97`, and
`format=lab-pbr/1.3`. A LabPBR-compatible shader is required for the PBR maps.
The target version and metadata are implemented as specified but have **not**
been independently validated against a Minecraft 26.3 client JAR.

LabPBR channels: `_n` = DirectX XY / linear AO / height; `_s` =
`1-sqrt(roughness)` / F0 or metal ID / porosity / emission. Emission alpha 255
means *no emission*. F0 values 238–254 are reserved and rejected. Categorical
metal/emission values are preserved through resampling.

## Rust CLI

```bash
cargo run --locked --release -- \
  --input test_assets --output test_output --cpu-only \
  --resolution 128 --seed 128042 --threads 2
```

Options: `--resolution 64|128|256`, `--seed <u32>`, `--name <pack name>`,
`--threads 1..256`, `--no-normal`, `--no-specular`, `--cpu-only`. Every run is
CPU-only; `--cpu-only` is explicit, not a fallback. Existing output is protected:
choose a fresh output directory. The working size is 512×512; inputs are sampled
periodically and continuous channels are downsampled with Lanczos-3. Albedo uses
linear-light premultiplied alpha. The ZIP uses the uncompressed/store method.

`--input` accepts a directory containing `material_manifest.json` or the file
path. Minimal manifest:

```json
{
  "version": 1,
  "materials": [
    {"id":"base", "albedo":"diffuse.png", "normal":"normal.png",
     "normal_y":"opengl", "roughness":"roughness.png"}
  ],
  "textures": [
    {"name":"iron_block", "material":"base", "recipe":"metal",
     "tint":[210,210,210], "f0":230, "porosity":0, "emissive":255,
     "roughness_scale":1.0, "normal_strength":2.5}
  ]
}
```

Relative PNG paths must stay inside the manifest directory. Names use lowercase
ASCII letters, digits, and underscores; `_n`/`_s` suffixes are reserved. Recipes:
`stone`, `brick`, `plank`, `moss`, `glass`, `metal`, `gem`, `leaves`. These are
procedural demonstrations, not material-specific photorealistic recipes.
Animations, items, entities, GUI textures, and per-pixel metal/emission masks are
outside the current scope.

## Storage and cleanup

Runtime data lives in `.workspace/` (override with `HYPERREALISM_DATA_DIR`):
`project.json`, `history.json`, `uploads/`, `previews/`, and `jobs/`. Writes are
atomic and serialized. On startup the server removes orphaned uploads older than
24 hours, preview caches older than 7 days or beyond the newest 200 entries,
stale temporary files, and job directories absent from history. Deleting an
export removes its files. Source images are re-encoded and bounded to 4096×4096
/ 16 megapixels / 24 MB per upload.

## Tests

```bash
npm run test:core       # Rust unit + CLI integration tests
npm test                # shared validation + real local API/engine tests
npm run typecheck
npm run format:check
npx playwright install chromium
npm run test:e2e        # real browser workflows, including ZIP download
npm run build
npm run test:production # production-server smoke test
```

API/E2E tests use the release engine at `target/release/hyperrealism-core`
(or `HYPERREALISM_BIN`). E2E uses isolated `.workspace/e2e` data and ports
5173/3101 by default; set `E2E_WEB_PORT` if 5173 is occupied. CI runs Rust tests
on Linux/Windows and the web/API/browser/production checks on Linux.

## Remaining production gaps

Real asset acquisition/provenance and an audited 280-block catalog; vanilla JAR
validation; animation/alpha edge cases; GPU/Vulkan and hardware benchmarking;
Minecraft/shader in-game review; and multi-user auth/hardening for public hosting.
None of these are claimed as complete.
