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
3. Enable HyperRealism128 in **Options → Resource Packs**; put it above any
   competing pack, or disable other packs for an isolated test.
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

## Verify that PBR is active

Vanilla Minecraft without a compatible shader only displays the color maps.
In Photon, the pack must be enabled **above** competing packs, with normal and
specular mapping enabled and the texture format set to labPBR. POM is optional
and requires the shader's parallax setting: height is not visible as geometry
without it. Preview 13 assigns a per-texture PBR strength override of `0.16`
to the lever and 54 door/trapdoor paths, 40% of the previous global `0.4`
baseline. In this generator, the override scales tangent-space normal
slopes, height deviations around neutral, and smoothness; AO fades toward white.
Albedo and discrete LabPBR metal/F0 IDs, porosity and emission are preserved.
This is a defined output-channel scaling, not a measured 40% reduction in
perceived visual intensity. Compare the same sunlit oak log or cobblestone with
Normal Mapping on and off; rotate the view/light to check for changes in shading.
For specular, compare a smooth metallic block in the manifest, with Specular Mapping on/off.
If nothing changes, check Iris/Photon versions, pack order, and shader settings;
reload resources with F3+T. Scan-driven height exists on the new bark and
cobblestone; many older materials still use approximate height, so parallax
can look wrong. This is a diagnostic procedure, not proof of a test on your PC.

At 512px, shader atlas memory and load times can increase substantially; if you
see stutter or texture-load errors, reduce shader settings and try the optional
128px build rather than assuming 8 GB is sufficient.

## How to evaluate

Compare in daylight and in darker light, with and without the resource pack,
then with and without the shader. Try stone, cobblestone stairs/slabs/walls,
oak and bamboo planks/fences, sand, netherrack and Nether gold ore. Ask which
details are visible from a normal playing distance, whether seams repeat, and
whether metals reflect plausibly. The 308 covered block texture paths do not
replace the entire game; animations, most blocks and entities are not supplied. The pack cannot guarantee visual
quality merely because it contains 512×512 images and valid LabPBR channels.

## Connected glass

The pack ships OptiFine-format compact CTM rules and LabPBR maps for all 18
covered clear, stained and tinted glass sprites. Iris is a shader loader, not a
CTM implementation. Continuity 3.0.1+26.3's exact 26.3/dev source loader contract
was inspected and static tests validate its paths, compact method, minimum tile
count and pane side rules; see [the compatibility audit](CONTINUITY_SUPPORT.md).
Install [Continuity 3.0.1+26.3](https://modrinth.com/mod/continuity/version/3.0.1+26.3)
and required Fabric API, restart, and place this pack above conflicting glass
packs. Consider Continuity's glass pane culling fix if panes show seams. With no
compatible CTM implementation the fallback has thin borders on every block.
Minecraft/Continuity world rendering and shader appearance remain untested. A
clear cutout's invisible interior may not reflect in shaders; tinted glass is
translucent.

Stone already had LabPBR maps. Preview 10 increases the stone-only PBR strength
from 0.4 to 0.65 while keeping its albedo unchanged. Normal Mapping must be on
to see normal detail; POM is optional and may make the estimated height look
unnatural. Test toggling normal mapping on the same lit stone face, and ensure
no other resource pack overrides it.
