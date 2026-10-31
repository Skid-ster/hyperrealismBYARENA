# Shader setup and hardware target

**Design target:** Java 26.3 on Windows 11, RTX 5060 Ti 8 GB, i5-13400 and
16 GB RAM. These are the requested specs, not a benchmark or guarantee. The
pack generator is CPU-only; it has not been benchmarked on this hardware. Iris
and the shader render on the GPU. Keep other memory-heavy apps closed on a 16 GB
machine; excessive Java heap allocation can make Windows page memory.

## Recommended free shader: Photon

[Photon Shaders by sixthsurge](https://modrinth.com/shader/photon-shader) is a
free, natural-looking option with full LabPBR support according to its author;
its current distribution lists Java 26.3. Its [source and setup](https://github.com/sixthsurge/photon)
describe Iris support. No shader is objectively best for every user, and this
pack has not yet been visually validated with Photon or on the target machine.
Use the listed 26.3-compatible release. Do not redistribute a shader with this
pack unless its license permits it.

1. Download the Photon shader ZIP from its project page; do not extract it.
2. Put it in your Minecraft instance's `minecraft/shaderpacks` folder. For the
   supplied ElyPrism instance that is
   `%APPDATA%\ElyPrismLauncher\instances\26.3(1)\minecraft\shaderpacks`.
3. The published Preview 23 is named **HyperRealism 512×512**; current development
   builds are named **HyperRealism 256×256**. Enable the version you installed in
   **Options → Resource Packs** and put it above competing packs.
4. With the already-installed Iris, select Photon in **Options → Video
   Settings → Shader Packs** and open **Shader Pack Settings**.
5. In **Materials**, enable **Normal Mapping**, **Specular Mapping**, and
   optionally **Parallax Occlusion Mapping**. In **Resource Pack Settings**, set
   **Texture Format** to **labPBR**. Menu names may vary by Photon version.
   Start with moderate parallax depth; very high depth exaggerates approximated
   height and can create artifacts. Resource-pack height is not a real 3D mesh.
6. Start with the High profile at moderate render/shadow distances; try Ultra
   if performance is acceptable. Adjust resolution, POM samples, reflections,
   and shader profile to taste. There is no measured FPS claim.

Photon's published [option names](https://github.com/sixthsurge/photon/blob/6e67b36e818bacde0304edd49f28e2bb70e33f04/shaders/lang/en_US.lang)
include the settings above. Other good free options include Complementary
Unbound; configure its **RP Support** as labPBR. Individual shaders differ in
handling emission, AO, SSS, porosity and metal IDs.

### Complementary Reimagined (Iris)

The latest supplied screenshot shows **Complementary Reimagined r5.9.3**, not
BSL. In **Options → Video Settings → Shader Packs → Shader Pack Settings**,
set the main **RP Support** option to **labPBR (RP Required)**. Do not leave it
at **Basic** (which disables PBR) or **Integrated PBR+** (which uses generated
material properties rather than fully reading this pack's LabPBR maps). Then
open **Materials → labPBR/seuspbr Materials** and leave **Normal Map Strength**
enabled at a moderate value. For specular highlights, make sure
**Performance Settings → Block Reflection Quality** is not **OFF**; the High
profile shown in the screenshot normally selects High reflection quality, but
check the actual option. Put the installed HyperRealism build (published Preview 23
is 512×512; current development builds are 256×256) above conflicting resource
packs and press F3+T.

**POM is optional**: normal and specular PBR do not depend on it. First verify
mapping with POM off. If you later enable **POM on Cutout Surfaces**, it allows
parallax on cutout regions such as door windows, but the shader warns that it
can be expensive; leave it off unless you specifically want that effect. It is
not required for the door's normal map or reflections. These option names are
confirmed in Complementary Reimagined's [pinned English settings](https://github.com/ComplementaryDevelopment/ComplementaryReimagined/blob/c09950df0650b27dac260c1b0f69afc11387aa6f/shaders/lang/en_US.lang)
and [shader settings](https://github.com/ComplementaryDevelopment/ComplementaryReimagined/blob/c09950df0650b27dac260c1b0f69afc11387aa6f/shaders/shaders.properties).

### BSL Shaders

In BSL's **Shader Pack Settings → Materials**, turn **Advanced Materials** on
and set **Material Format** to **LabPBR 1.3**. BSL is listed as LabPBR 1.3
compatible by the [shaderLABS supported-packs list](https://shaderlabs.org/wiki/LabPBR_Supported_Packs).
Exact menu paths may differ by shader version.

Doors and trapdoors are not excluded because their sprites contain cutout
transparency or because their models are not full cubes. This pack emits
aligned `<texture>_n.png` and `<texture>_s.png` LabPBR sidecars for every
covered block sprite, including those shapes. The color-map alpha continues to
define the door/window cutouts; shader support and correct resource-pack order
are still required for the extra maps to show. A visible before/after
comparison is still needed to confirm your specific shader version and settings.

## Verify that PBR is active

Vanilla Minecraft without a compatible shader only displays the color maps.
In Complementary Reimagined, the pack must be enabled **above** competing packs
and **RP Support** must be **labPBR (RP Required)**; in Photon, set the texture
format to labPBR and enable normal/specular mapping. POM is optional
and requires the shader's parallax setting: height is not visible as geometry
without it. Preview 14 introduced a global PBR strength of `0.2`, with no
per-texture overrides; the current development manifest applies that same global
setting across all 352 texture targets. In this generator that scales
tangent-space normal slopes, height deviations around neutral, and smoothness
to 20% of unit-strength, while AO fades toward white. Albedo and discrete
LabPBR metal/F0 IDs, porosity and emission are preserved. This is defined
channel scaling, not a guarantee of 20% perceived brightness or relief. Preview
15 supplies explicit, normal-derived height and conservative cavity AO for the
shared leaf atlas; use POM if you want to inspect the shallow surface relief.
This does not change leaf block geometry or create a fuller canopy. Roughness
is tuned at the material-map level, not by changing the global strength: finished
wood, metal and glass are kept smoother; sand, mud, bark and oxidized surfaces
stay rough. Cobblestone roughness varies between its stone faces and mortar from
the authored height map. For 26 other scanned materials with no displacement
scan, a conservative periodic height estimate is reconstructed from the normal
map; it is an approximation, not measured geometry. Compare the same sunlit oak
log or cobblestone with Normal Mapping on and off; rotate the view/light to
check for shading changes. For specular, compare a smooth metallic block with
Specular Mapping on/off. If nothing changes, check Iris/Photon versions, pack
order, and shader settings; reload resources with F3+T. This is a diagnostic
procedure, not proof of a test on your PC.

The development build now defaults to 256px: that is one quarter the texel
count of 512px, so it should reduce texture storage and atlas pressure, but no
VRAM, load-time or FPS benchmark has been run. The published Preview 23 is still
512px. Use the optional 128px build for a smaller pack or 512px for more detail;
actual memory and performance depend on Minecraft, shader, driver and hardware.

## How to evaluate

Compare in daylight and in darker light, with and without the resource pack,
then with and without the shader. Try stone, cobblestone stairs/slabs/walls,
oak and bamboo planks/fences, sand, netherrack and Nether gold ore. Ask which
details are visible from a normal playing distance, whether seams repeat, and
whether metals reflect plausibly. The 352 covered block texture paths do not
replace the entire game; animations, most blocks and entities are not supplied. The pack cannot guarantee visual
quality merely because it contains high-resolution images and valid LabPBR channels.

## Connected glass

The pack ships OptiFine-format compact CTM rules and LabPBR maps for all 18
covered clear, stained and tinted glass sprites. Iris is a shader loader, not a
CTM implementation. Continuity 3.0.1+26.3's exact 26.3/dev source loader contract
was inspected and static tests validate its paths, compact method, minimum tile
count and pane side rules; see [the compatibility audit](CONTINUITY_SUPPORT.md).
Install [Continuity 3.0.1+26.3](https://modrinth.com/mod/continuity/version/3.0.1+26.3)
and required Fabric API, restart, and place this pack above conflicting glass
packs. Consider Continuity's glass pane culling fix if panes show seams. With no
compatible CTM implementation the fallback glass remains borderless and uses the same uniform opacity.
Minecraft/Continuity world rendering and shader appearance remain untested. A
clear cutout's invisible interior may not reflect in shaders; tinted glass is
translucent.

Stone had a temporary higher strength in earlier previews, but Preview 14
removes all per-texture overrides: stone and every other covered block now use
the same global 0.2 strength. Normal Mapping must be on to see normal detail;
POM is optional and may still make estimated height look unnatural. If blocks
appear to bulge, disable POM or reduce its depth/samples in the shader settings;
test the same lit face and ensure no other pack overrides it.
