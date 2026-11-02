# HyperRealism 512×512 + Gilded Dusk — Minecraft Java 26.3 experimental Preview 27

Preview 27 is a PBR-validation and diagnostic update. It keeps the Preview 26
storage-block coverage and strict-driver decoder fix, then makes the in-shader
PBR check honest for flat-normal materials and expands CI from a handful of
sample blocks to an exhaustive headless decode sweep. It remains a **partial
experimental pack**, not full vanilla coverage. The resource-pack archive keeps
the legacy `HyperRealism128` filename for updater compatibility; Minecraft
displays **HyperRealism 512×512**.

## What changed for PBR

- **PBR Activity diagnostic replaces “Map Presence”.** The old check treated a
  valid flat normal map (glass/gems) as “missing” because it matched Iris’
  default normal. The new check reports green when LabPBR specular/material data
  is active, yellow when only a non-default normal is active, and red when no
  PBR sidecars are active. Use **Normal XY** to inspect relief separately.
- **Exhaustive headless LabPBR sweep:** CI now renders every generated block
  triple, one CTM tile per connected-glass directory, and every entity triple
  through the actual `material.glsl` decoder. It checks PBR Activity, roughness,
  metal IDs, emission, AO, SSS/porosity ranges and normal bounds, plus
  missing/normal-only/specular-only controls.
- **Strict-driver decode fix retained:** LabPBR metal IDs are unsigned literals
  so `switch(uint)` compiles on strict GLSL compilers (Mesa/AMD), not just
  glslang.
- **Storage/lamp coverage retained:** iron, gold, diamond, emerald, lapis,
  redstone and coal blocks plus redstone lamps use audited 26.3 target colors and
  explicit LabPBR metal/F0/emission IDs.
- Full authored LabPBR strength (1.0), explicit `PBR_MODE == 1` decoding, and
  conservative parallax at height 0.10 remain enabled by default.

## Install / update

Use Windows GUI installer 18 or newer: **Install both** selects one published
release containing the resource pack and Gilded Dusk shader, verifies both ZIPs
and checksums, and installs the matched pair together. Or download the two ZIPs
from this release and place them in the active instance's `resourcepacks` and
`shaderpacks` folders. Enable the HyperRealism resource pack above competing
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
executes the exhaustive headless LabPBR decode sweep, checks the full-strength
build report, compiles the Iris 26.3 shader matrix, and verifies the shader ZIP.
These are compiler, structural, archive and software-decode checks — not an
in-game Minecraft/Iris GPU render, visual-quality certification, artifact review,
or performance benchmark. Use `docs/PBR_VALIDATION.md` and the PBR Activity
debug view to verify channels on your machine.
