import { spawnSync } from 'node:child_process';
import fs from 'node:fs/promises';
import { existsSync } from 'node:fs';
import path from 'node:path';
import { CATALOG, toTexture, DEFAULT_SETTINGS } from '../shared/catalog';
const root = path.resolve(import.meta.dirname, '..');
const temp = path.join(root, '.workspace/static-previews');
const exe = process.platform === 'win32' ? '.exe' : '';
const binary =
  process.env.HYPERREALISM_BIN ||
  [
    path.join(root, `target/release/hyperrealism-core${exe}`),
    path.join(root, `target/debug/hyperrealism-core${exe}`),
  ].find(existsSync);
if (!binary) throw new Error('Build the Rust engine first: npm run build:core');
await fs.mkdir(temp, { recursive: true });
await fs.mkdir(path.join(root, 'public/previews'), { recursive: true });
const manifest = JSON.parse(await fs.readFile(path.join(root, 'test_assets/material_manifest.json'), 'utf8'));
manifest.textures = CATALOG.map(toTexture);
await fs.writeFile(path.join(temp, 'material_manifest.json'), JSON.stringify(manifest));
for (const name of ['diffuse.png', 'normal.png', 'roughness.png'])
  await fs.copyFile(path.join(root, 'test_assets', name), path.join(temp, name));
const output = path.join(temp, `output-${Date.now()}`);
const result = spawnSync(
  binary,
  [
    '--input',
    temp,
    '--output',
    output,
    '--resolution',
    '128',
    '--seed',
    String(DEFAULT_SETTINGS.seed),
    '--cpu-only',
    '--threads',
    '2',
  ],
  { stdio: 'inherit' },
);
if (result.status !== 0) process.exit(1);
for (const texture of CATALOG)
  for (const suffix of ['', '_n', '_s']) {
    await fs.copyFile(
      path.join(output, `pack/assets/minecraft/textures/block/${texture.name}${suffix}.png`),
      path.join(root, `public/previews/${texture.name}${suffix}.png`),
    );
  }
await fs.rm(output, { recursive: true, force: true });
console.log(`Generated ${CATALOG.length * 3} real CPU previews.`);
