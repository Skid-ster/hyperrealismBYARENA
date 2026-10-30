# Continuity compatibility notes

This pack emits its connected-glass rules as the OptiFine-compatible CTM
properties format that Continuity loads. For the Minecraft 26.3 line, the
checked Continuity source is branch `26.3/dev`, commit
[`fe34da420aa188c92a5d3e325b12742d56c88474`](https://github.com/PepperCode1/Continuity/tree/fe34da420aa188c92a5d3e325b12742d56c88474).
Its Gradle properties identify Continuity `3.0.1` and Minecraft `26.3`.

The compatibility audit checked these behaviors in that exact source revision:

- [`CtmPropertiesLoader`](https://github.com/PepperCode1/Continuity/blob/fe34da420aa188c92a5d3e325b12742d56c88474/src/main/java/me/pepperbell/continuity/client/resource/CtmPropertiesLoader.java)
  scans each resource namespace for `.properties` files under `optifine/ctm`.
- [`ContinuityClient`](https://github.com/PepperCode1/Continuity/blob/fe34da420aa188c92a5d3e325b12742d56c88474/src/main/java/me/pepperbell/continuity/client/ContinuityClient.java)
  registers `ctm_compact` with a minimum of five tiles and texture orientation.
- [`PropertiesParsingHelper`](https://github.com/PepperCode1/Continuity/blob/fe34da420aa188c92a5d3e325b12742d56c88474/src/main/java/me/pepperbell/continuity/client/properties/PropertiesParsingHelper.java)
  resolves unqualified `matchTiles=glass` relative to the properties namespace
  and `matchBlocks=glass` to the default Minecraft namespace. The generated
  properties are under `assets/minecraft/optifine/ctm`, so both resolve to the
  intended Minecraft assets.

For each of the 18 covered glass block sprites (clear, 16 stained colors, and
tinted), the build writes a `ctm_compact` rule and five numbered PNG tile
triples: albedo, LabPBR normal (`_n`) and specular (`_s`). Clear glass and all
stained colors also receive pane-side rules (`faces=sides`); pane caps remain
separate vanilla-path replacement textures. Tinted glass has no vanilla pane
block and therefore no pane rule. This deliberately uses the Minecraft
namespace; duplicating these rules under a `continuity` namespace is unnecessary
and would make relative `matchTiles` references resolve incorrectly.

Rust integration tests generate the pack and check all 18 properties files,
all five tile triples, pane rules, paths, and ZIP inclusion. This verifies the
pack's structure against the inspected Continuity loader contract. It does **not**
launch Minecraft, load Continuity, render connected blocks, or validate a
shader's LabPBR interpretation. Those runtime checks still require the target
Minecraft/Continuity installation. Install Continuity 3.0.1+26.3 and Fabric API,
then test clear/stained glass blocks and panes in world; enable Continuity's
pane-culling fix if seams remain and place this pack above competing glass packs.
Iris alone is not a connected-texture loader.
