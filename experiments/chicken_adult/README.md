# Adult chicken texture pilot — experimental

This is a standalone Minecraft Java 26.3 resource-pack test for **one adult
variant**: `minecraft:textures/entity/chicken/chicken_temperate.png`. It does not
replace the baby texture, the cold/warm adult variants, chicken item textures,
models, UVs, animation, entity behavior, or any blocks.

The 512×256 RGBA albedo keeps the vanilla adult chicken's 64×32 UV layout at an
exact 8× scale. Its artwork is original procedural plumage: fine feather
barbs/shafts and soft down, with separate small brown eyes, a warm keratin beak,
a red comb/wattle and golden scaly legs. Chickens have **feathers, not mammal
fur**; the soft micro-fuzz is a feather/down finish, not a fur coat. No
Minecraft color pixels are used in the generated artwork. The UV occupancy is
encoded as atlas spans to retain the client's existing mapping and transparency.

Aligned `chicken_temperate_n.png` and `chicken_temperate_s.png` are supplied at
the same 512×256 size. They use LabPBR 1.3 channel conventions and a pack-level
LabPBR declaration. As with all entity PBR, actually seeing those maps depends
on the selected shader's entity-material support; vanilla still displays the
albedo. The pack's color replacement has not yet been tested in a live Minecraft
client, and the PBR maps have not been tested with an Iris/OptiFine shader.

## Build and try it

From the repository root, run:

```sh
python3 scripts/build_chicken_adult_pilot.py
```

This uses only Python's standard library and reproducibly builds this directory
plus `HyperRealism-Chicken-Adult-Pilot.zip`. Drop that ZIP into the 26.3
instance's `resourcepacks` directory, enable it **above HyperRealism 512** (or
other packs that replace the same path), then reload resources with F3+T. In a
temperate biome, inspect an adult chicken; babies and the cold/warm variants
should retain their other textures. Keep the pilot separate from the main pack
until the in-game checks pass.

Check close-up feather detail and normal relief, then check it at ordinary
playing distance, in daylight and shade. Also verify the eye, beak, comb,
wattle, feet, alpha/mip edges, and that only adult temperate chickens change.
For PBR, use a shader that explicitly supports LabPBR entity maps. Do not infer
runtime or performance validation from the image dimensions or structural test.
