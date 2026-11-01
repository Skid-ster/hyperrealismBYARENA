import { describe, expect, it } from 'vitest';
import { CATALOG, DEFAULT_SETTINGS, toTexture } from '../shared/catalog';
import {
  buildSchema,
  previewSchema,
  projectSchema,
  settingsSchema,
  textureSchema,
} from '../shared/validation';

const project = () => ({
  edits: {},
  custom: [],
  selected: ['stone'],
  favorites: [],
  settings: { ...DEFAULT_SETTINGS },
});
describe('material contracts', () => {
  it('accepts every original demo material', () => {
    for (const entry of CATALOG) expect(textureSchema.safeParse(toTexture(entry)).success).toBe(true);
  });
  it('has 24 unique, collision-free demo names', () => {
    expect(CATALOG).toHaveLength(24);
    expect(new Set(CATALOG.map((t) => t.name)).size).toBe(24);
  });
  it.each(['../stone', 'Oak Planks', 'stone_n', 'stone_s', '', 'a'.repeat(81), 'stone.png'])(
    'rejects unsafe filename %s',
    (name) => {
      expect(textureSchema.safeParse({ ...toTexture(CATALOG[0]), name }).success).toBe(false);
    },
  );
  it('preserves categorical metal and emission values', () => {
    const texture = { ...toTexture(CATALOG[0]), f0: 230, emissive: 255 };
    expect(textureSchema.parse(texture)).toMatchObject({ f0: 230, emissive: 255 });
    expect(textureSchema.safeParse({ ...texture, f0: 238 }).success).toBe(false);
    expect(textureSchema.safeParse({ ...texture, f0: 255 }).success).toBe(true);
  });
  it.each([
    { porosity: 65 },
    { roughness_scale: -0.1 },
    { roughness_scale: 1.1 },
    { normal_strength: 9 },
    { normal_strength: NaN },
    { tint: [256, 0, 0] },
  ])('rejects out-of-range channels %#', (patch) => {
    expect(textureSchema.safeParse({ ...toTexture(CATALOG[0]), ...patch }).success).toBe(false);
  });
  it('does not allow metadata in a texture edit', () => {
    const p = project();
    expect(
      projectSchema.safeParse({ ...p, edits: { stone: { label: 'Not a texture field' } } }).success,
    ).toBe(false);
    expect(projectSchema.safeParse({ ...p, edits: { stone: { roughness_scale: 0.4 } } }).success).toBe(true);
  });
  it('rejects unknown materials and identity-changing edits', () => {
    expect(projectSchema.safeParse({ ...project(), selected: ['unknown'] }).success).toBe(false);
    expect(projectSchema.safeParse({ ...project(), edits: { stone: { name: 'bricks' } } }).success).toBe(
      false,
    );
  });
  it('rejects duplicate or built-in-shadowing custom names', () => {
    const custom = {
      ...CATALOG[0],
      custom: true,
      material: 'stone',
      assets: { albedo: 'd3a31ce2-889a-4ad1-993c-886191b9228b' },
    };
    expect(projectSchema.safeParse({ ...project(), custom: [custom] }).success).toBe(false);
    expect(
      projectSchema.safeParse({
        ...project(),
        custom: [
          { ...custom, name: 'my_stone' },
          { ...custom, name: 'my_stone' },
        ],
      }).success,
    ).toBe(false);
  });
  it('requires consistent IDs and source maps for custom textures', () => {
    const custom = { ...CATALOG[0], name: 'custom_stone', custom: true };
    expect(projectSchema.safeParse({ ...project(), custom: [custom] }).success).toBe(false);
    expect(
      projectSchema.safeParse({
        ...project(),
        custom: [{ ...custom, assets: { albedo: 'd3a31ce2-889a-4ad1-993c-886191b9228b' } }],
      }).success,
    ).toBe(false);
  });
  it('accepts a matching custom material and opaque asset IDs', () => {
    const custom = {
      ...CATALOG[0],
      name: 'custom_stone',
      material: 'custom_stone',
      custom: true,
      assets: { albedo: 'd3a31ce2-889a-4ad1-993c-886191b9228b' },
    };
    expect(
      projectSchema.safeParse({ ...project(), custom: [custom], selected: ['custom_stone'] }).success,
    ).toBe(true);
  });
  it('rejects client-controlled filesystem paths in preview requests', () => {
    expect(
      previewSchema.safeParse({
        texture: toTexture(CATALOG[0]),
        seed: 1,
        assets: { albedo: '../../secret.png' },
      }).success,
    ).toBe(false);
  });
});
describe('build contracts', () => {
  it.each([64, 128, 256])('accepts supported resolution %i', (resolution) => {
    expect(settingsSchema.safeParse({ ...DEFAULT_SETTINGS, resolution }).success).toBe(true);
  });
  it.each([
    { resolution: 512 },
    { threads: 0 },
    { threads: 17 },
    { seed: -1 },
    { seed: 4294967296 },
    { packName: '' },
    { packName: '../../pack' },
  ])('rejects invalid settings %#', (patch) => {
    expect(settingsSchema.safeParse({ ...DEFAULT_SETTINGS, ...patch }).success).toBe(false);
  });
  it('requires a nonempty, unique, bounded selection', () => {
    expect(buildSchema.safeParse({ names: [], settings: DEFAULT_SETTINGS }).success).toBe(false);
    expect(buildSchema.safeParse({ names: ['stone', 'stone'], settings: DEFAULT_SETTINGS }).success).toBe(
      false,
    );
    expect(buildSchema.safeParse({ names: ['stone'], settings: DEFAULT_SETTINGS }).success).toBe(true);
  });
});
