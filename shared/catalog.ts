export type Recipe = 'stone' | 'brick' | 'plank' | 'moss' | 'glass' | 'metal' | 'gem' | 'leaves';
export type Category = 'Stone' | 'Wood' | 'Metal' | 'Nature' | 'Glass & gems';
export interface Texture {
  name: string;
  material: string;
  recipe: Recipe;
  tint: [number, number, number];
  f0: number;
  porosity: number;
  emissive: number;
  roughness_scale: number;
  normal_strength: number;
}
export interface MaterialEntry extends Texture {
  label: string;
  category: Category;
  description: string;
  custom?: boolean;
  assets?: { albedo: string; normal?: string; roughness?: string };
  normal_y?: 'directx' | 'opengl';
}
export interface Settings {
  packName: string;
  resolution: 64 | 128 | 256;
  seed: number;
  threads: number;
  includeNormal: boolean;
  includeSpecular: boolean;
}
export interface Project {
  edits: Record<string, Partial<Texture>>;
  custom: MaterialEntry[];
  selected: string[];
  favorites: string[];
  settings: Settings;
}
export const DEFAULT_SETTINGS: Settings = {
  packName: 'HyperRealism Demo',
  resolution: 128,
  seed: 128042,
  threads: 2,
  includeNormal: true,
  includeSpecular: true,
};
const make = (
  name: string,
  label: string,
  category: Category,
  recipe: Recipe,
  tint: [number, number, number],
  description: string,
  extra: Partial<Texture> = {},
): MaterialEntry => ({
  name,
  label,
  category,
  recipe,
  tint,
  description,
  material: 'synthetic',
  f0: 10,
  porosity: 20,
  emissive: 255,
  roughness_scale: 0.78,
  normal_strength: 2.5,
  ...extra,
});
export const CATALOG: MaterialEntry[] = [
  make(
    'stone',
    'Stone',
    'Stone',
    'stone',
    [170, 175, 166],
    'Cool, mineral-rich stone with a naturally weathered surface.',
    { porosity: 28 },
  ),
  make(
    'cobblestone',
    'Cobblestone',
    'Stone',
    'stone',
    [155, 160, 149],
    'Rugged, interlocking stones with deep, irregular joints.',
    { porosity: 35, normal_strength: 3.4 },
  ),
  make(
    'stone_bricks',
    'Stone bricks',
    'Stone',
    'brick',
    [164, 169, 155],
    'Time-worn masonry with soft edges and recessed mortar.',
    { porosity: 30 },
  ),
  make(
    'granite',
    'Granite',
    'Stone',
    'stone',
    [192, 143, 121],
    'Warm granite with a fine crystalline grain.',
    { roughness_scale: 0.65 },
  ),
  make(
    'deepslate',
    'Deepslate',
    'Stone',
    'stone',
    [91, 101, 107],
    'Dark, layered stone from the depths of the overworld.',
    { porosity: 15 },
  ),
  make(
    'andesite',
    'Andesite',
    'Stone',
    'stone',
    [171, 177, 170],
    'A fine-grained volcanic rock with subtle mineral flecks.',
  ),
  make(
    'bricks',
    'Clay bricks',
    'Stone',
    'brick',
    [205, 122, 105],
    'Kiln-fired clay, uneven color, and beautifully aged mortar.',
    { porosity: 35 },
  ),
  make(
    'sandstone',
    'Sandstone',
    'Stone',
    'stone',
    [221, 204, 154],
    'Soft sedimentary stone with a warm, sandy finish.',
    { porosity: 45 },
  ),
  make(
    'oak_planks',
    'Oak planks',
    'Wood',
    'plank',
    [205, 165, 105],
    'Warm oak boards with expressive grain and tight seams.',
    { porosity: 40 },
  ),
  make(
    'spruce_planks',
    'Spruce planks',
    'Wood',
    'plank',
    [140, 105, 70],
    'Rich, dark timber with a rustic character.',
    { porosity: 40 },
  ),
  make(
    'birch_planks',
    'Birch planks',
    'Wood',
    'plank',
    [222, 205, 154],
    'Pale, clean boards with a delicate natural grain.',
    { porosity: 35 },
  ),
  make(
    'dark_oak_planks',
    'Dark oak planks',
    'Wood',
    'plank',
    [111, 79, 49],
    'Deep chocolate wood for rich, atmospheric interiors.',
    { porosity: 40 },
  ),
  make(
    'iron_block',
    'Iron block',
    'Metal',
    'metal',
    [212, 216, 215],
    'Brushed iron with a soft metallic response.',
    { f0: 230, porosity: 0, roughness_scale: 0.32, normal_strength: 1.3 },
  ),
  make(
    'gold_block',
    'Gold block',
    'Metal',
    'metal',
    [247, 204, 88],
    'Warm gold with fine brushing and restrained highlights.',
    { f0: 231, porosity: 0, roughness_scale: 0.27, normal_strength: 1.2 },
  ),
  make(
    'copper_block',
    'Copper block',
    'Metal',
    'metal',
    [211, 133, 100],
    'Fresh copper with a warm, softly reflective finish.',
    { f0: 232, porosity: 0, roughness_scale: 0.4, normal_strength: 1.4 },
  ),
  make(
    'coal_block',
    'Coal block',
    'Stone',
    'stone',
    [61, 65, 66],
    'Dense, fractured coal with a subtle mineral sheen.',
    { porosity: 20, roughness_scale: 0.55 },
  ),
  make(
    'moss_block',
    'Moss block',
    'Nature',
    'moss',
    [112, 166, 76],
    'A dense carpet of moss with soft, organic relief.',
    { porosity: 50, roughness_scale: 0.95 },
  ),
  make(
    'oak_leaves',
    'Oak leaves',
    'Nature',
    'leaves',
    [100, 163, 78],
    'Layered foliage with carefully preserved transparent gaps.',
    { porosity: 25, roughness_scale: 0.85 },
  ),
  make(
    'dirt',
    'Dirt',
    'Nature',
    'moss',
    [148, 111, 79],
    'Loose, granular earth with natural color variation.',
    { porosity: 55, roughness_scale: 1 },
  ),
  make(
    'grass_block_side',
    'Grass block side',
    'Nature',
    'moss',
    [143, 151, 83],
    'An earthy, moss-like study for grass block sides.',
    { porosity: 40 },
  ),
  make(
    'glass',
    'Glass',
    'Glass & gems',
    'glass',
    [205, 230, 226],
    'Lightly tinted glass with transparent panes and defined edges.',
    { porosity: 0, roughness_scale: 0.12, normal_strength: 0.8 },
  ),
  make(
    'diamond_block',
    'Diamond block',
    'Glass & gems',
    'gem',
    [108, 224, 211],
    'Cool crystalline facets with a polished surface.',
    { f0: 43, porosity: 0, roughness_scale: 0.2 },
  ),
  make(
    'emerald_block',
    'Emerald block',
    'Glass & gems',
    'gem',
    [75, 205, 128],
    'Vivid green crystal with a clean, glossy response.',
    { f0: 43, porosity: 0, roughness_scale: 0.23 },
  ),
  make(
    'amethyst_block',
    'Amethyst block',
    'Glass & gems',
    'gem',
    [172, 137, 218],
    'Soft violet crystal with a luminous mineral character.',
    { f0: 43, porosity: 0, roughness_scale: 0.3 },
  ),
];
export const CATEGORIES: Category[] = ['Stone', 'Wood', 'Metal', 'Nature', 'Glass & gems'];
export const RECIPES: Recipe[] = ['stone', 'brick', 'plank', 'moss', 'glass', 'metal', 'gem', 'leaves'];
export const tintHex = (tint: number[]) =>
  '#' + tint.map((v) => Math.round(v).toString(16).padStart(2, '0')).join('');
export const hexTint = (hex: string): [number, number, number] => [
  parseInt(hex.slice(1, 3), 16),
  parseInt(hex.slice(3, 5), 16),
  parseInt(hex.slice(5, 7), 16),
];
export const prettyName = (name: string) => name.replaceAll('_', ' ').replace(/^./, (s) => s.toUpperCase());
export function resolveEntry(entry: MaterialEntry, edits: Project['edits']): MaterialEntry {
  return { ...entry, ...edits[entry.name] };
}
export function toTexture(entry: MaterialEntry): Texture {
  const { name, material, recipe, tint, f0, porosity, emissive, roughness_scale, normal_strength } = entry;
  return { name, material, recipe, tint, f0, porosity, emissive, roughness_scale, normal_strength };
}
