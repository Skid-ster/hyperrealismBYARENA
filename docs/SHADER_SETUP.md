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

## How to evaluate

Compare in daylight and in darker light, with and without the resource pack,
then with and without the shader. Try stone, cobblestone, oak planks, sand,
netherrack and Nether gold ore. Ask which details are visible from a normal
playing distance, whether seams repeat, and whether metals reflect plausibly.
The 106 covered block textures do not replace the entire game; animations,
most blocks and entities are not supplied. The pack cannot guarantee visual
quality merely because it contains 128×128 images and valid LabPBR channels.
