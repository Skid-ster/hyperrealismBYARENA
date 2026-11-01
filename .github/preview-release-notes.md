# HyperRealism 512×512 + Gilded Dusk — Minecraft Java 26.3 experimental Preview 25

Preview 25 is a PBR-focused update: the resource pack is rebuilt at **full
authored LabPBR strength (1.0)** and the Gilded Dusk shader no longer waits for a
possibly stale/global Iris texture-format flag before decoding LabPBR sidecars.
It remains a **partial experimental pack**, not full vanilla coverage. The
resource-pack archive keeps the legacy `HyperRealism128` filename for updater
compatibility; Minecraft displays **HyperRealism 512×512**.

## What changed for PBR

- Block normal slopes, height deviation, smoothness and AO are no longer
  compressed to 20%. The manifest now uses `pbr_strength: 1.0`, preserving the
  authored material response. Discrete metal/F0, porosity, subsurface and
  emission fields remain unchanged.
- Gilded Dusk decodes LabPBR whenever **PBR Support = LabPBR 1.3** is selected.
  Iris still supplies safe default normal/specular textures for missing sidecars.
  The `MC_TEXTURE_FORMAT_LAB_PBR` flag is used for diagnostics, not as a hard
  gate that can disable PBR after pack-order/resource reloads.
- Added **Materials → PBR Channel Debug → Map Presence**: green means both
  normal and specular sidecars are present, yellow means one, red means neither;
  a blue tint means Iris detected the LabPBR `texture.properties` flag.
- Conservative parallax occlusion mapping is enabled by default at height 0.10
  so authored height can produce visible relief. Low-end profiles disable it.
  It is still UV relief, not real geometry.
- The shader retains the corrected LabPBR channel decode: normal RG/AO B/height
  A, perceptual smoothness, dielectric/predefined/other metal F0 ranges,
  mutually exclusive porosity/SSS, and the no-emission sentinel.

## Install / update

Use Windows GUI installer 18 or newer: **Install both** selects one published
release containing the resource pack and Gilded Dusk shader, verifies both
ZIPs and checksums, and installs the matched pair together. Or download the two ZIPs from
this release and place them in the active instance's `resourcepacks` and
`shaderpacks` folders. Enable the HyperRealism resource pack above competing
packs, select Gilded Dusk in Iris, set **PBR Support** to **LabPBR 1.3**, and
reload resources with `F3+T`.

## Coverage and honesty

The pack covers 371 block texture targets plus five farm-mob entity paths, not
the entire Minecraft texture inventory. Uncovered blocks remain vanilla or come
from other enabled packs. Models, UVs, cutout silhouettes, block states and
existing animations are unchanged. No Minecraft reference-image pixels are
redistributed. Source scans are CC0 and recorded in `MATERIAL_CREDITS.txt` and
`work/material_manifest.json`.

CI generates the 512px pack, verifies LabPBR triples and the release archive,
checks the full-strength build report, compiles the Iris 26.3 shader matrix and
LabPBR decode harness, and verifies the shader ZIP. These are compiler,
structural and archive checks—not an in-game Minecraft/Iris GPU render,
visual-quality certification, artifact review, or performance benchmark. Use
`docs/PBR_VALIDATION.md` and the Map Presence debug view to verify channels on
your machine.
