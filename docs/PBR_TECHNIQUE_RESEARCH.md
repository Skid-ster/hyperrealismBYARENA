# PBR technique survey: foliage, doors and glass

Research snapshot: 2026-10-30. This is a small technique survey, not a claim of
being a complete market comparison or of having rendered the listed packs in
the same game/shader setup. Repository metadata and texture-set structure were
inspected; no competitor textures, screenshots or other copyrighted assets are
included in this project.

## Technique takeaways

- **Keep material channels separate and aligned.** At the pinned JG RTX
  revision, the `azalea_leaves`, `bamboo_door_top`, and `glass` Bedrock texture
  sets each name a color map plus separate normal and MER maps; the repository
  also carries opaque leaf variants. See the [leaf texture set](https://github.com/jasonjgardner/jg-rtx/blob/6c1da08d231ff4c94cf014f780759231756f94bd/bedrock/pack/RP/textures/blocks/azalea_leaves.texture_set.json),
  [bamboo door texture set](https://github.com/jasonjgardner/jg-rtx/blob/6c1da08d231ff4c94cf014f780759231756f94bd/bedrock/pack/RP/textures/blocks/bamboo_door_top.texture_set.json),
  and [glass texture set](https://github.com/jasonjgardner/jg-rtx/blob/6c1da08d231ff4c94cf014f780759231756f94bd/bedrock/pack/RP/textures/blocks/glass.texture_set.json).
  These are Bedrock's channel conventions, not directly portable LabPBR files;
  the general lesson is to keep aligned color, normal, roughness/material and
  height information instead of baking appearance into one color PNG. We use
  LabPBR maps for Java and keep cutout albedo and scanned leaf normal/roughness
  aligned.
- **Build relief at the material scale.** For foliage, use scanned leaf-vein
  normals and a restrained, continuous height field; let the cutout alpha define
  the leaf silhouette. Height/parallax can reveal shallow relief but cannot add
  leaf cards, branches, canopy volume, or replace Minecraft's block geometry.
  AO should remain subtle and localized instead of turning transparent gaps into
  dark halos. A view-dependent shader and in-game validation are still needed.
- **Doors need coherent microdetail, not displaced panels.** Keep the vanilla
  sprite layout, hinges, opening states, UVs and transparency masks; align grain,
  normal, roughness and shallow panel bevel cues. Avoid strong height steps that
  make a flat door look inflated. The pinned [Java 26.3 door model UVs](https://github.com/InventivetalentDev/minecraft-assets/blob/6c088630459d8ca369ef0d6c07636d7d63a3b0f1/assets/minecraft/models/block/door_bottom_left.json)
  and matching right-hinge model mirror the large faces. The fixed procedural
  handle was moved to the latch-side UV edge and raised within the lower-block
  sprite so it sits near mid-height on the assembled two-block door; hinge and
  face flips place it on the correct side. The panel's alpha/cutout remains
  unchanged. The pack's global strength is 0.2, and the
  47 door/trapdoor/ladder/bar opacity masks are unchanged.
- **Use aligned sidecars for partial/cutout sprites too.** The public
  [Vanilla Normals Renewed tree](https://github.com/Poudingue/Vanilla-Normals-Renewed/tree/70d89ee4d401a27c4518ebab3b13897c2a6b6921/assets/minecraft/textures/block)
  includes separate `_n` and `_s` files alongside door top/bottom and trapdoor
  sprites. We adopt that conventional Java shader-pack layout, but generate our
  own maps from licensed/source-controlled material data and construction
  masks; no artwork from that repository is copied. Its repository has no
  declared license, so it is treated only as a path-convention reference.
- **Glass benefits from restraint.** Preserve transparency and the pane-cap
  model/UV silhouettes, but avoid painted frame bands; keep the surface response
  uniform and let shader reflections provide subtle highlights. Connected-texture
  rules are a separate compatibility layer; a resource pack cannot guarantee
  they load without Continuity. Shader behavior and reflections still vary by
  implementation. Preview 16 applies this as borderless texture alpha across
  the glass family.

## Snapshot of inspected public repositories

- [JG RTX](https://github.com/jasonjgardner/jg-rtx), inspected at commit
  `6c1da08d231ff4c94cf014f780759231756f94bd`, declares `CC-BY-SA-4.0` in its
  license file. Its separate normal/MER maps and leaf texture-set variants
  were observed as general workflow examples only. No image assets or data are
  copied or redistributed here.
- [Lytex](https://github.com/AstroX809/Lytex), inspected at commit
  `393fe71023197f926cd9f867c5cd652ac7da7092`, declares MIT. The inspected
  repository tree contained general normal/specular material maps but no leaf
  map paths; it was not used as an art source.
- [VibrantPBR](https://github.com/Sakurapotato/VibrantPBR), inspected at commit
  `cfc9ebf68f1117744716a7c1d701c0475d8d88ef`, has a custom license that limits
  modification and combination to personal use. Its inspected repository tree
  contained only README/license files, so it supplied no texture assets or
  implementation details for this work.

This survey informs general methods only. The foliage source remains the CC0
Poly Haven Tree Small 02 scan by Rico Cilliers, already credited in
[`work/ASSET_CREDITS.md`](../work/ASSET_CREDITS.md). The new height map is a
periodic Poisson reconstruction from its aligned normal map, not a captured
height scan; the new AO is a conservative local cavity estimate, not a baked
ray-traced occlusion pass. The single broadleaf source is still shared across
species and is not botanically species-accurate.
