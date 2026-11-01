import { z } from 'zod';
import { CATALOG, RECIPES } from './catalog';
export const textureName = z
  .string()
  .min(1)
  .max(80)
  .regex(/^[a-z0-9_]+$/, 'Use lowercase letters, numbers, and underscores only.')
  .refine(
    (s) => !s.endsWith('_n') && !s.endsWith('_s'),
    'Names ending in _n or _s are reserved for PBR maps.',
  );
export const textureSchema = z
  .object({
    name: textureName,
    material: z.string().max(80),
    recipe: z.enum(RECIPES),
    tint: z.tuple([
      z.number().int().min(0).max(255),
      z.number().int().min(0).max(255),
      z.number().int().min(0).max(255),
    ]),
    f0: z
      .number()
      .int()
      .min(0)
      .max(255)
      .refine((n) => n < 238 || n === 255, 'F0 values 238–254 are reserved.'),
    porosity: z.number().int().min(0).max(64),
    emissive: z.number().int().min(0).max(255),
    roughness_scale: z.number().min(0).max(1),
    normal_strength: z.number().min(0).max(8),
  })
  .strict();
const assetId = z.string().uuid();
export const entrySchema = textureSchema
  .extend({
    label: z.string().min(1).max(80),
    category: z.enum(['Stone', 'Wood', 'Metal', 'Nature', 'Glass & gems']),
    description: z.string().max(240),
    custom: z.boolean().optional(),
    assets: z
      .object({ albedo: assetId, normal: assetId.optional(), roughness: assetId.optional() })
      .strict()
      .optional(),
    normal_y: z.enum(['directx', 'opengl']).optional(),
  })
  .strict()
  .superRefine((v, ctx) => {
    if (v.custom && v.material !== v.name)
      ctx.addIssue({ code: 'custom', message: 'A custom material ID must match its texture name.' });
    if (v.custom && !v.assets)
      ctx.addIssue({ code: 'custom', message: 'A custom material needs an albedo image.' });
    if (!v.custom && v.material !== 'synthetic')
      ctx.addIssue({ code: 'custom', message: 'Unknown source material.' });
  });
export const settingsSchema = z
  .object({
    packName: z
      .string()
      .trim()
      .min(1, 'Give your pack a name.')
      .max(60)
      .regex(/^[^<>:"/\\|?*\x00-\x1f]+$/, 'The pack name contains unsupported filename characters.'),
    resolution: z.union([z.literal(64), z.literal(128), z.literal(256)]),
    seed: z.number().int().min(0).max(4294967295),
    threads: z.number().int().min(1).max(16),
    includeNormal: z.boolean(),
    includeSpecular: z.boolean(),
  })
  .strict();
export const projectSchema = z
  .object({
    edits: z.record(textureName, textureSchema.partial().strict()),
    custom: z.array(entrySchema).max(256),
    selected: z.array(textureName).max(280),
    favorites: z.array(textureName).max(280),
    settings: settingsSchema,
  })
  .strict()
  .superRefine((p, ctx) => {
    const known = new Set([...CATALOG.map((t) => t.name), ...p.custom.map((t) => t.name)]);
    if ([...p.selected, ...p.favorites, ...Object.keys(p.edits)].some((name) => !known.has(name)))
      ctx.addIssue({ code: 'custom', message: 'The project references an unknown material.' });
    if (new Set(p.custom.map((v) => v.name)).size !== p.custom.length)
      ctx.addIssue({ code: 'custom', message: 'Duplicate material names.' });
    if (p.custom.some((c) => CATALOG.some((original) => original.name === c.name)))
      ctx.addIssue({ code: 'custom', message: 'Imported names must not replace built-in materials.' });
    for (const [name, edit] of Object.entries(p.edits)) {
      if (
        (edit.name && edit.name !== name) ||
        (edit.material && edit.material !== name && edit.material !== 'synthetic')
      )
        ctx.addIssue({ code: 'custom', message: 'Material identity cannot be changed in an edit.' });
    }
    if (p.custom.some((c) => !c.custom))
      ctx.addIssue({ code: 'custom', message: 'Imported materials must be marked custom.' });
  });
export const previewSchema = z
  .object({
    texture: textureSchema,
    seed: settingsSchema.shape.seed,
    normal_y: z.enum(['directx', 'opengl']).default('directx'),
    assets: z
      .object({ albedo: assetId, normal: assetId.optional(), roughness: assetId.optional() })
      .optional(),
  })
  .strict();
export const buildSchema = z
  .object({
    names: z
      .array(textureName)
      .min(1, 'Select at least one material.')
      .max(280)
      .refine((names) => new Set(names).size === names.length, 'Build names must be unique.'),
    settings: settingsSchema,
  })
  .strict();
export type PreviewRequest = z.infer<typeof previewSchema>;
