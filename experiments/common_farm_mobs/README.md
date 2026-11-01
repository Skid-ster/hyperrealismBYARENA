# Common farm-mob texture pilot

This 26.3 test pack contains five original entity texture targets:

- adult temperate chicken (`chicken_temperate`)
- adult temperate cow (`cow_temperate`)
- adult temperate pig (`pig_temperate`)
- sheep base texture and tintable sheep wool overlay (`sheep`, `sheep_wool`)

Each has aligned LabPBR 1.3 albedo, normal and specular maps. Native UV layouts
are preserved at 8× linear scale: 512×512 for 64×64 atlases and 512×256 for
64×32 atlases. Animal-specific surface recipes create feather/down, chestnut
hide patches, warm pink skin, sheep face/skin, and a soft wool overlay. These
are procedural prototypes, not scanned photorealistic animal photographs.
The source code and input hashes are in `scripts/build_*_pilot.py` and
`work/material_manifest.json`.

**Scope is deliberately limited.** It does not replace chicken/cow/pig babies,
cold/warm biome variants, other sheep dye/coat behaviors, saddles, collars,
armor, hostile mobs, models or animations. It is not a complete mob pack. The
same five entity paths are now included by the main source-pack generator via
`work/material_manifest.json`; these five targets are included in experimental Preview 22 and subsequent previews.

## Build and try

```sh
python3 scripts/build_chicken_adult_pilot.py
python3 scripts/build_common_mob_pilot.py
```

The second command rebuilds the cow, pig, and sheep source maps under
`work/entity_textures/` and this standalone pack/ZIP; the chicken generator
builds its own first pilot under `experiments/chicken_adult/`. After building
both, the common-mob ZIP includes all five sets. Drop
`HyperRealism-Common-Farm-Mobs-Pilot.zip` into your Minecraft 26.3
`resourcepacks` folder and enable it above competing packs. Check the temperate
adult variants in daylight, shade and at normal play distance. Sheep wool is
tinted by vanilla dye behavior. The color paths and ZIP structure were checked,
but no in-game rendering, shader PBR or performance validation has been done.
PBR entity rendering depends on the chosen shader's entity-material support.
