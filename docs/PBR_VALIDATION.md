# In-game PBR validation checklist

Compiler and archive checks are not a Minecraft render. Use this checklist on
the target machine to confirm that the HyperRealism resource pack’s LabPBR maps
are actually reaching Gilded Dusk.

## Setup

1. Use Minecraft Java 26.3 with the matching Iris/Sodium build and the normal
   OpenGL renderer. Iris does not run with OptiFine or the Minecraft Vulkan
   renderer.
2. Use Installer 18’s **Install both** button to install the matching
   HyperRealism resource pack ZIP in `resourcepacks` and
   `HyperRealism-Gilded-Dusk-MC-26.3.zip` in the sibling `shaderpacks` folder,
   then enable the resource pack above competing packs.
3. Select **HyperRealism: Gilded Dusk** in Iris.
4. In **Shader Pack Settings → Materials**, set **PBR Support** to **LabPBR 1.3**.
   Leave **Normal Map Strength** and **Material AO Strength** at 1.0 for the
   authored pack response. Preview 27 enables conservative parallax by default;
   disable **Parallax** if you only want normal/specular PBR. Keep **PBR Channel
   Debug** off for normal play.
5. Reload resources with `F3+T` after changing packs or shader settings.

## Channel checks

Use a covered block such as cobblestone, oak log/planks, iron/gold/diamond/
emerald blocks, copper block, redstone lamp, glass, glowstone, or leaves. Uncovered vanilla textures will not show
HyperRealism PBR. First select **PBR Channel Debug → PBR Activity**: green means
LabPBR specular/material data reached the shader (a flat normal is valid for
glass/gems), yellow means only a non-default normal reached it, red means no PBR
sidecars reached it. A blue tint means Iris also detected the LabPBR texture-format
flag. Use **Normal XY** to inspect normal relief separately.

- **Normal XY**: debug view should show non-flat color variation on covered
  surfaces. A uniform lavender/flat field means the normal map is missing,
  disabled, or the block is uncovered.
- **Roughness**: smoother materials should be darker; rough stone/bark brighter.
- **F0 / Metal**: dielectrics should be dark gray; predefined/other metals should
  show their reflectance/albedo tint.
- **Emission**: glowstone/shroomlight should be bright; ordinary blocks black.
- **Ambient Occlusion**: authored cavity darkening should appear in the blue
  normal-map channel.
- **Height**: should vary where the pack authored relief. Height only displaces
  UVs when **Parallax (POM)** is enabled; it is not real geometry.
- **Porosity / Subsurface**: leaves should show subsurface, not porosity; porous
  stone/dirt should show porosity, not subsurface.

## Lighting checks

- Compare the same sunlit surface with **PBR Support** off and on. Normal mapping
  should change shading as the view/light moves.
- Compare a metal/glass surface with **Rough Sky Reflections** on/off. Specular
  response should change with roughness and viewing angle.
- Enable **Parallax** only after normal/specular checks pass. Start with moderate
  height; excessive height exaggerates approximated maps and can cause artifacts.
- If a channel debug view is flat, check pack order, the `_n.png`/`_s.png`
  sidecars, Iris version, and `logs/latest.log` for resource/shader errors.

## Headless decode test

`python3 scripts/test_pbr_pipeline.py --pack pilot_output/pack --require-gl`
validates every generated block/CTM/entity triple and, when ModernGL/EGL is
available, compiles the actual `lib/common/material.glsl` and sweeps every
generated block triple, one CTM tile per connected-glass directory, and every
entity triple through it. It asserts iron/gold metal IDs, glowstone/redstone-lamp
emission, leaf subsurface, smooth gem/glass dielectrics, PBR Activity colors,
normal-driven lighting variation, and missing/normal-only/specular-only controls.
CI runs this on Linux with Mesa llvmpipe. It already caught and fixed a
strict-GLSL `switch(uint)` metal-ID type mismatch that glslang alone accepted.
This is stronger than a compile-only check, but it is still not Minecraft/Iris
rendering on your GPU.

## Known limits

- The resource pack is partial, not full vanilla coverage.
- Preview 27 uses full authored PBR strength (1.0). Earlier 0.2 packs can look
  almost flat. The shader decodes the maps; it does not invent missing maps.
- No GPU/driver, visual-quality, artifact, or FPS certification is implied by CI.
