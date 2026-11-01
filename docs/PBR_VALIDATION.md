# In-game PBR validation checklist

Compiler and archive checks are not a Minecraft render. Use this checklist on
the target machine to confirm that the HyperRealism resource pack’s LabPBR maps
are actually reaching Gilded Dusk.

## Setup

1. Use Minecraft Java 26.3 with the matching Iris/Sodium build and the normal
   OpenGL renderer. Iris does not run with OptiFine or the Minecraft Vulkan
   renderer.
2. Install the HyperRealism resource pack ZIP in `resourcepacks` and enable it
   above competing packs.
3. Install `HyperRealism-Gilded-Dusk-MC-26.3.zip` in the sibling `shaderpacks`
   folder. Installer 17’s **Install shader** button does this automatically.
4. Select **HyperRealism: Gilded Dusk** in Iris.
5. In **Shader Pack Settings → Materials**, set **PBR Support** to **LabPBR 1.3**.
   Leave **Normal Map Strength** and **Material AO Strength** at 1.0 for the
   authored pack response. Preview 25 enables conservative parallax by default;
   disable **Parallax** if you only want normal/specular PBR. Keep **PBR Channel
   Debug** off for normal play.
6. Reload resources with `F3+T` after changing packs or shader settings.

## Channel checks

Use a covered block such as cobblestone, oak log/planks, iron door/bars, copper
block, glass, glowstone, or leaves. Uncovered vanilla textures will not show
HyperRealism PBR. First select **PBR Channel Debug → Map Presence**: green means
both normal and specular sidecars reached the shader, yellow means one, red means
neither. A blue tint means Iris also detected the LabPBR texture-format flag.

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

## Known limits

- The resource pack is partial, not full vanilla coverage.
- Preview 25 uses full authored PBR strength (1.0). Earlier 0.2 packs can look
  almost flat. The shader decodes the maps; it does not invent missing maps.
- No GPU/driver, visual-quality, artifact, or FPS certification is implied by CI.
