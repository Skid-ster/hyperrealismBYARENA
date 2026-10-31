# Minecraft 26.3 model-to-texture consistency audit

The catalog replaces texture paths; it does not copy or edit vanilla block
models or blockstates. This matters for stairs, slabs, walls and interactive
blocks: if vanilla models use a shared sprite, one generated albedo/normal/
specular triple should serve every state rather than introducing separately
colored copies.

## Cobblestone finding

At the pinned Minecraft-assets mirror commit
`6c088630459d8ca369ef0d6c07636d7d63a3b0f1`, the 26.3 blockstates for
`cobblestone`, `cobblestone_stairs`, `cobblestone_slab` and
`cobblestone_wall` resolve through their model parents to the single path
`assets/minecraft/textures/block/cobblestone.png`. The manifest has exactly one
`cobblestone` output, built from the cobble scan and aligned height map; the
pipeline writes its `_n` and `_s` maps beside it. No stair, slab or wall model
or blockstate needs to be replaced. Their different geometry and rotations
remain vanilla behavior.

## Other checked building families

The same model-reference check confirms these representative shared paths:

- `stone_stairs` and `stone_slab` → `stone`.
- Stone-brick, brick, deepslate-brick and polished-blackstone-brick stairs,
  slabs and walls → their corresponding single `*_bricks` texture.
- Stairs, slabs, fences and fence gates across the 11 non-bamboo vanilla wood
  families (including crimson and warped) → that species' single plank sprite.
  Bamboo stairs/slabs use `bamboo_planks`; bamboo fences and gates each use a
  distinct surface sprite plus a particle sprite. Oak door/trapdoor models also
  retain their separate sprites.
- Cobbled-deepslate stairs, slabs and walls → `cobbled_deepslate`.
- Sandstone and red-sandstone stairs/slabs retain the base side, top and bottom
  texture paths; walls use the side texture.
- Deepslate and blackstone retain separate side and end-cap paths.
- Oak door models use separate top/bottom textures; trapdoors use their own
  texture. These are not incorrectly redirected to oak planks.
- The pinned interactive-state summary records 24 lever variants (both powered
  models share `lever` plus the already-covered `cobblestone` base), 32 oak-door
  variants (top/bottom, open/closed, left/right hinge) and 16 oak-trapdoor
  variants (top/bottom/open). New PBR outputs reuse the vanilla texture paths.

The audit exposed five paths missing from Preview 11: `bamboo_planks`,
`bamboo_fence`, `bamboo_fence_particle`, `bamboo_fence_gate` and
`bamboo_fence_gate_particle`. They are now covered in the source manifest by a
bamboo-specific procedural board recipe over licensed CC0 White Maple Veneer
inputs, with colors guided by the pinned 26.3 inventory. This is an approximate
procedural bamboo material, not a bamboo scan, and needs visual review. The two
bamboo particle sprites have the same pinned vanilla blob hash, so both use a
shared procedural seed and produce byte-identical albedo and PBR maps rather
than drifting apart.

All block texture paths referenced in this sampled model audit are now present
in the 344-entry manifest. Thus these audited families need no duplicate variant
textures or model edits. The distinct texture paths intentionally remain
separate where vanilla UVs distinguish faces, door halves or trapdoor geometry.

## Scope and reproducibility

`catalog/vanilla_26.3_variant_texture_refs.json` stores the resolved **names**
from 160 representative blockstates after following model parents and `#texture`
variables. It contains no Minecraft image pixels. The Rust integration test
checks each reference against the manifest and asserts the key shared-path
families above. The independent `vanilla_26.3_audit.json` records target sprite
names, dimensions, animation-metadata presence and color hints for the catalog.

This is a targeted audit of major building families, not an exhaustive walk of
every one of the 26.3 blockstates or every model orientation/state. It does not
replace local testing with the user's pinned 26.3 client JAR. Model mapping
checks prove which pack paths are selected; they do not prove texture appearance,
normal-map behavior under every UV transform, or shader rendering. The five
bamboo coverage gaps are fixed in the source manifest, and no vanilla behavior
or model was changed as part of this audit.
