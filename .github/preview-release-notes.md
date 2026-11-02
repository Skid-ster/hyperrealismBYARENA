# HyperRealism 512×512 + Gilded Dusk — Minecraft Java 26.3 experimental Preview 26

Preview 26 is a PBR-coverage and validation update. It adds the common storage
blocks players use to judge PBR — **iron, gold, diamond, emerald, lapis,
redstone and coal blocks** — plus **redstone lamps**, and CI now executes the
shader's actual LabPBR decoder against the generated maps in headless OpenGL.
It remains a **partial experimental pack**, not full vanilla coverage. The
resource-pack archive keeps the legacy `HyperRealism128` filename for updater
compatibility; Minecraft displays **HyperRealism 512×512**.

## What changed for PBR

- **New visible metal/gem/lamp targets:** `iron_block` and `gold_block` use
  predefined LabPBR metal IDs (230/231); diamond/emerald/lapis use smooth
  dielectric gem recipes; redstone/coal are rough dielectrics; the powered
  redstone lamp carries an emission map. Target colors are audited against the
  pinned 26.3 vanilla metadata.
- **Strict-driver decode fix:** the new headless test caught a GLSL
  portability bug where LabPBR metal IDs were signed `int` case labels in a
  `switch(uint)`. Mesa rejected it. The IDs are now unsigned literals, so the
  decoder compiles on strict compilers as well as glslang.
- **Headless LabPBR execution test:** `scripts/test_pbr_pipeline.py` checks all
  block/CTM/entity triples, then compiles the real `material.glsl` and renders
  generated `_n`/`_s` maps through it. It asserts metal IDs, emission,
  subsurface, Map Presence colors, normal-driven lighting variation and the
  missing-sidecar fallback. This is software OpenGL validation, not an
  in-game Minecraft/Iris render.
- Block normal slopes, height deviation, smoothness and AO remain at full
  authored LabPBR strength (1.0). Gilded Dusk decodes LabPBR whenever
  **PBR Support = LabPBR 1.3** is selected; the global texture-format flag is
  diagnostic, not a hard gate.
- **Materials → PBR Channel Debug → Map Presence** remains available: green
  means both sidecars reached the shader, yellow one, red neither; a blue tint
  means Iris detected the LabPBR `texture.properties` flag.
- Conservative parallax occlusion mapping remains enabled by default at height
  0.10. It is UV relief, not real geometry.

## Install / update

Use Windows GUI installer 18 or newer: **Install both** selects one published
release containing the resource pack and Gilded Dusk shader, verifies both
ZIPs and checksums, and installs the matched pair together. Or download the two
ZIPs from this release and place them in the active instance's `resourcepacks`
and `shaderpacks` folders. Enable the HyperRealism resource pack above competing
packs, select Gilded Dusk in Iris, set **PBR Support** to **LabPBR 1.3**, and
reload resources with `F3+T`.

## Coverage and honesty

The pack covers 380 block texture targets plus five farm-mob entity paths, not
the entire Minecraft texture inventory. Uncovered blocks remain vanilla or come
from other enabled packs. Models, UVs, cutout silhouettes, block states and
existing animations are unchanged. No Minecraft reference-image pixels are
redistributed. Source scans are CC0 and recorded in `MATERIAL_CREDITS.txt` and
`work/material_manifest.json`.

CI generates the 512px pack, verifies LabPBR triples and the release archive,
executes the headless LabPBR decode test, checks the full-strength build report,
compiles the Iris 26.3 shader matrix, and verifies the shader ZIP. These are
compiler, structural, archive and software-decode checks — not an in-game
Minecraft/Iris GPU render, visual-quality certification, artifact review, or
performance benchmark. Use `docs/PBR_VALIDATION.md` and the Map Presence debug
view to verify channels on your machine.
