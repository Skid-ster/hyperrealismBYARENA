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

**Target:** Minecraft Java 26.3, the matching Iris/Sodium build, Fabric or
NeoForge, and an OpenGL 4.3-capable GPU. Iris does not run with OptiFine or the
Minecraft Vulkan renderer; use the normal OpenGL renderer. The upstream
Glimmer README also excludes macOS and Raspberry Pi because they do not provide
the required OpenGL feature level. Use the [official Iris installer](https://irisshaders.dev/download)
for your Minecraft version; it installs Iris and Sodium together.

1. Run `python3 scripts/package_stylized_shader.py` from the project root, or
   use the supplied `HyperRealism-Gilded-Dusk-MC-26.3.zip`.
2. Put the ZIP itself in the active instance's `shaderpacks` directory. Do not
   unzip it. The archive is laid out with `shaders/` at its root, as Iris
   expects; it is not a resource pack.
3. Enable the HyperRealism resource pack in the resource-pack screen, then
   select **HyperRealism: Gilded Dusk** under **Options → Video Settings →
   Shader Packs**.
4. In shader settings, leave **PBR Support** on **LabPBR 1.3** for this pack.
   Keep parallax mapping off initially; enable it only if you want the
   height/depth effect. Adjust **Post-Processing → Teal–Amber Style** from 0
   (neutral) to 100 (strong split tone); the default is 70.

If the shader does not appear, check that the ZIP is in the launcher's actual
instance directory, not `resourcepacks`. If Iris reports a compile problem,
check that instance's `logs/latest.log` and capture the first shader error.

## Validation and compatibility limits

The pack is pinned to Glimmer's MIT-licensed upstream source commit shown above.
Its source has now been compiler-checked with glslang against the Iris 26.3 macro interface (from
Iris's [`StandardMacros`](https://github.com/IrisShaders/Iris/blob/26.3/common/src/main/java/net/irisshaders/iris/gl/shader/StandardMacros.java)
and [`WorldRenderingPhase`](https://github.com/IrisShaders/Iris/blob/26.3/common/src/main/java/net/irisshaders/iris/pipeline/WorldRenderingPhase.java)):
all 227 default stages, 227 PBR-off/style-maximum stages, and a 239-stage
all-features/Distant-Horizons matrix passed (693 stage compilations). A
source/archive check also verifies the LabPBR default, style setting, root ZIP
layout, and retained notices.

This is not the same as running Minecraft: **in-game loading, actual GPU/driver
compatibility, image quality, artifacts, and performance have not been
verified here.** Glimmer's own README states Iris 1.6+ and OpenGL 4.3+, while
its latest published [v1.5.2 version metadata](https://modrinth.com/shader/glimmer-shaders/version/1dD3KC4h)
lists Minecraft through 1.21.11; that is not an upstream certification for
26.3. The 26.3 compile matrix is our additional
check, not a promise that every modded instance or GPU will render it
identically. No hardware-performance claim is made.
