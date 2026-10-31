# HyperRealism: Gilded Dusk (experimental shader)

An Iris shaderpack prototype for the HyperRealism resource pack. It keeps a full
LabPBR 1.3 material pipeline and adds a deliberately stylized filmic finish:
AgX tone mapping, slightly lifted contrast and saturation, and adjustable
cool-teal shadows / warm-amber highlights. The grade is a display transform;
it does not replace, rewrite or bypass the PBR material maps.

## Open-source foundation and license

This shader starts from **Glimmer by Josh Britain (jbritain)**, copied from
[`jbritain/glimmer-shaders`](https://github.com/jbritain/glimmer-shaders) at
commit [`6a591097b6348155a4ba5a1aab92b79a2c9896d3`](https://github.com/jbritain/glimmer-shaders/commit/6a591097b6348155a4ba5a1aab92b79a2c9896d3).
The upstream project advertises complete LabPBR 1.3 support and lists Iris as
its supported loader. The upstream MIT license is retained in `LICENSE`; its
README and development notes are retained in `UPSTREAM_GLIMMER_README.md` and
`UPSTREAM_GLIMMER_DEVELOPMENT.md`, including original acknowledgements.

HyperRealism modifications: a final teal-shadow/amber-highlight grade, an
adjustable **Teal–Amber Style** control, stronger contrast and saturation
defaults, an AgX tonemapper default, and pack-specific settings labels. The
copyright and attribution for the upstream shader remain with its original
author and contributors. See the MIT notice before copying or redistributing.

## Install and configure

1. Use Minecraft Java 26.3 with Iris and Sodium. This prototype does not claim
   OptiFine compatibility.
2. Copy the `HyperRealism-Stylized-PBR` folder into that instance's
   `shaderpacks` directory (or zip the folder with `shaders/` at the archive
   root).
3. Select **HyperRealism: Gilded Dusk** in Iris. For this resource pack, set
   **PBR Support** to **LabPBR 1.3** and start with parallax mapping off; raise
   parallax only after checking that the texture's estimated height looks
   plausible.
4. Adjust **Post-Processing → Teal–Amber Style** from 0 (neutral) to 100
   (strong split tone). The default is 70.

## Validation status

This is an early source integration, not a release-ready or visually validated
shader. The source is pinned, the upstream license is included, and the new
color-grade control is wired into Iris settings. Shader compilation, Minecraft
26.3/Iris runtime, LabPBR appearance, visual artifacts and performance have not
yet been tested in-game. No hardware-performance claim is made.
