import { afterAll, beforeAll, describe, expect, it } from 'vitest';
import { spawn, type ChildProcess } from 'node:child_process';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { createServer } from 'node:net';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { PNG } from 'pngjs';
import { unzipSync, strFromU8 } from 'fflate';
import { CATALOG, DEFAULT_SETTINGS, toTexture } from '../shared/catalog';

let server: ChildProcess;
let directory: string;
let base: string;
let logs = '';
const initial = {
  edits: {},
  custom: [],
  selected: ['stone'],
  favorites: [],
  settings: { ...DEFAULT_SETTINGS, resolution: 64 },
};
async function request(route: string, method = 'GET', body?: unknown) {
  const response = await fetch(base + route, {
    method,
    headers: body ? { 'Content-Type': 'application/json' } : {},
    body: body ? JSON.stringify(body) : undefined,
  });
  return {
    status: response.status,
    body: response.status === 204 ? null : await response.json(),
    headers: response.headers,
  };
}
async function waitForBuild(id: string) {
  for (let i = 0; i < 150; i++) {
    const { body } = await request(`/builds/${id}`);
    if (['complete', 'failed', 'cancelled'].includes(body.status)) return body;
    await new Promise((r) => setTimeout(r, 100));
  }
  throw new Error('Build did not finish: ' + logs);
}
beforeAll(async () => {
  directory = await mkdtemp(path.join(tmpdir(), 'hyperrealism-api-'));
  const probe = createServer();
  await new Promise<void>((resolve) => probe.listen(0, '127.0.0.1', resolve));
  const port = (probe.address() as { port: number }).port;
  await new Promise<void>((resolve) => probe.close(() => resolve()));
  base = `http://127.0.0.1:${port}/api`;
  const binary =
    process.env.HYPERREALISM_BIN ||
    path.resolve(`target/release/hyperrealism-core${process.platform === 'win32' ? '.exe' : ''}`);
  if (!existsSync(binary))
    throw new Error('Build the Rust engine with npm run build:core before running API tests.');
  server = spawn(process.execPath, ['--import', 'tsx', 'server/index.ts'], {
    cwd: process.cwd(),
    env: {
      ...process.env,
      API_PORT: String(port),
      HYPERREALISM_DATA_DIR: directory,
      HYPERREALISM_BIN: binary,
    },
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  server.stdout?.on('data', (b) => {
    logs += b;
  });
  server.stderr?.on('data', (b) => {
    logs += b;
  });
  for (let i = 0; i < 100; i++) {
    try {
      if ((await request('/health')).body.engine) return;
    } catch {
      /* starting */
    }
    await new Promise((r) => setTimeout(r, 100));
  }
  throw new Error('API did not start: ' + logs);
});
afterAll(async () => {
  server?.kill('SIGTERM');
  await new Promise((r) => setTimeout(r, 250));
  if (directory) await rm(directory, { recursive: true, force: true });
});

describe('real local API and CPU engine', () => {
  it('reports a real engine and validates project writes', async () => {
    expect((await request('/health')).body).toMatchObject({ engine: true, backend: 'CPU', demoTextures: 24 });
    expect((await request('/project', 'PUT', initial)).status).toBe(200);
    expect((await request('/project')).body.settings.resolution).toBe(64);
    expect((await request('/project', 'PUT', { ...initial, selected: ['unknown'] })).status).toBe(400);
    expect(
      (await request('/project', 'PUT', { ...initial, edits: { stone: { label: 'bad' } } })).status,
    ).toBe(400);
    expect(
      (await request('/project', 'PUT', { ...initial, custom: [{ ...CATALOG[0], custom: true }] })).status,
    ).toBe(400);
  });
  it('rejects browser cross-site mutations', async () => {
    const response = await fetch(base + '/project', {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
        Origin: 'https://evil.example',
        'Sec-Fetch-Site': 'cross-site',
      },
      body: JSON.stringify(initial),
    });
    expect(response.status).toBe(403);
  });
  it('rejects invalid uploads and arbitrary asset paths', async () => {
    const invalid = new FormData();
    invalid.append('image', new Blob(['not a png'], { type: 'image/png' }), 'fake.png');
    expect((await fetch(base + '/uploads', { method: 'POST', body: invalid })).status).toBe(400);
    expect(
      (
        await request('/previews', 'POST', {
          texture: toTexture(CATALOG[0]),
          seed: 1,
          assets: { albedo: '../../secret.png' },
        })
      ).status,
    ).toBe(400);
    expect((await request('/previews/not-a-hash/stone.png')).status).toBe(404);
  });
  it('imports a PNG and generates actual albedo, normal and specular maps', async () => {
    const image = new PNG({ width: 32, height: 32 });
    for (let i = 0; i < image.data.length; i += 4) {
      image.data[i] = 130;
      image.data[i + 1] = 155;
      image.data[i + 2] = 140;
      image.data[i + 3] = 255;
    }
    const form = new FormData();
    form.append(
      'image',
      new Blob([new Uint8Array(PNG.sync.write(image))], { type: 'image/png' }),
      'source.png',
    );
    const uploaded = await (await fetch(base + '/uploads', { method: 'POST', body: form })).json();
    expect(uploaded).toMatchObject({ width: 32, height: 32 });
    const entry = {
      ...toTexture(CATALOG[0]),
      name: 'imported_test',
      material: 'imported_test',
      label: 'Imported test',
      category: 'Stone' as const,
      description: 'Test source',
      custom: true,
      assets: { albedo: uploaded.id },
    };
    expect(
      (await request('/project', 'PUT', { ...initial, custom: [entry], selected: [entry.name] })).status,
    ).toBe(200);
    const preview = await request('/previews', 'POST', {
      texture: toTexture(entry),
      seed: 42,
      assets: entry.assets,
      normal_y: 'directx',
    });
    expect(preview.status).toBe(200);
    for (const url of [preview.body.albedo, preview.body.normal, preview.body.specular]) {
      const response = await fetch('http://' + new URL(base).host + url);
      expect(response.status).toBe(200);
      const decoded = PNG.sync.read(Buffer.from(await response.arrayBuffer()));
      expect([decoded.width, decoded.height]).toEqual([128, 128]);
    }
  });
  it('builds a downloadable ZIP with root metadata, exact channels and a report', async () => {
    await request('/project', 'PUT', initial);
    const created = await request('/builds', 'POST', { names: ['iron_block'], settings: initial.settings });
    expect(created.status).toBe(202);
    const job = await waitForBuild(created.body.id);
    expect(job.status).toBe('complete');
    const response = await fetch(`${base}/builds/${job.id}/download`);
    expect(response.status).toBe(200);
    expect(response.headers.get('content-disposition')).toContain('.zip');
    const zip = unzipSync(Buffer.from(await response.arrayBuffer()));
    expect(Object.keys(zip)).toHaveLength(5);
    expect(strFromU8(zip['assets/minecraft/optifine/texture.properties'])).toContain('lab-pbr/1.3');
    expect(JSON.parse(strFromU8(zip['pack.mcmeta'])).pack.min_format).toEqual([97, 1]);
    const spec = PNG.sync.read(Buffer.from(zip['assets/minecraft/textures/block/iron_block_s.png']));
    expect(spec.width).toBe(64);
    for (let i = 0; i < spec.data.length; i += 4) {
      expect(spec.data[i + 1]).toBe(230);
      expect(spec.data[i + 3]).toBe(255);
    }
    const report = await request(`/builds/${job.id}/report`);
    expect(report.body).toMatchObject({ resolution: 64, texture_count: 1, backend: 'cpu' });
    expect((await fetch(`${base}/builds/${job.id}/sheet`)).status).toBe(200);
    expect((await request(`/builds/${job.id}`, 'DELETE')).status).toBe(204);
    expect((await request(`/builds/${job.id}/download`)).status).toBe(404);
  });
  it('rejects empty and duplicate builds', async () => {
    expect((await request('/builds', 'POST', { names: [], settings: initial.settings })).status).toBe(400);
    expect(
      (await request('/builds', 'POST', { names: ['stone', 'stone'], settings: initial.settings })).status,
    ).toBe(400);
  });
  it('can cancel a full-catalog build without producing a ready artifact', async () => {
    const created = await request('/builds', 'POST', {
      names: CATALOG.map((t) => t.name),
      settings: initial.settings,
    });
    expect(created.status).toBe(202);
    expect((await request(`/builds/${created.body.id}/cancel`, 'POST', {})).status).toBe(200);
    const job = await waitForBuild(created.body.id);
    expect(job.status).toBe('cancelled');
    expect((await request(`/builds/${job.id}/download`)).status).toBe(409);
    await request(`/builds/${job.id}`, 'DELETE');
  });
  it('persists the project atomically to disk', async () => {
    const saved = JSON.parse(await readFile(path.join(directory, 'project.json'), 'utf8'));
    expect(saved.custom).toHaveLength(0);
    expect(saved.selected).toEqual(['stone']);
  });
});
