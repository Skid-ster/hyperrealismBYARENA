import { spawn } from 'node:child_process';
import { mkdtemp, rm } from 'node:fs/promises';
import { createServer } from 'node:net';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { unzipSync, strFromU8 } from 'fflate';
import { DEFAULT_SETTINGS } from '../shared/catalog';

const root = path.resolve(import.meta.dirname, '..');
const directory = await mkdtemp(path.join(tmpdir(), 'hyperrealism-production-'));
const probe = createServer();
await new Promise<void>((resolve) => probe.listen(0, '127.0.0.1', resolve));
const port = (probe.address() as { port: number }).port;
await new Promise<void>((resolve) => probe.close(() => resolve()));
const base = `http://127.0.0.1:${port}`;
const child = spawn(process.execPath, ['--import', 'tsx', 'server/index.ts', '--production'], {
  cwd: root,
  env: { ...process.env, API_PORT: String(port), HYPERREALISM_DATA_DIR: directory },
  stdio: ['ignore', 'pipe', 'pipe'],
});
let logs = '';
child.stdout.on('data', (b) => (logs += b));
child.stderr.on('data', (b) => (logs += b));
const wait = (ms: number) => new Promise((r) => setTimeout(r, ms));
try {
  let healthy = false;
  for (let i = 0; i < 100; i++) {
    try {
      const health = await (await fetch(`${base}/api/health`)).json();
      if (health.engine) {
        healthy = true;
        break;
      }
    } catch {
      /* starting */
    }
    await wait(100);
  }
  if (!healthy) throw new Error(`Production engine did not become healthy.\n${logs}`);
  const page = await fetch(base);
  const html = await page.text();
  if (!page.ok || !html.includes('HyperRealism') || !html.includes('/assets/'))
    throw new Error('Production index was not served.');
  const asset = html.match(/src="(\/assets\/[^"]+\.js)"/)?.[1];
  if (!asset || !(await fetch(base + asset)).ok) throw new Error('Production JS bundle was not served.');
  const created = await (
    await fetch(`${base}/api/builds`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        names: ['stone'],
        settings: { ...DEFAULT_SETTINGS, resolution: 64, packName: 'Production Smoke' },
      }),
    })
  ).json();
  if (!created.id) throw new Error('Production build was not accepted: ' + JSON.stringify(created));
  let job = created;
  for (let i = 0; i < 200 && !['complete', 'failed', 'cancelled'].includes(job.status); i++) {
    await wait(100);
    job = await (await fetch(`${base}/api/builds/${created.id}`)).json();
  }
  if (job.status !== 'complete')
    throw new Error(`Production build failed: ${job.error || job.status}\n${logs}`);
  const download = await fetch(`${base}/api/builds/${job.id}/download`);
  if (!download.ok) throw new Error('Production ZIP download failed.');
  const zip = unzipSync(Buffer.from(await download.arrayBuffer()));
  if (!zip['pack.mcmeta'] || !zip['assets/minecraft/textures/block/stone.png'])
    throw new Error('Production ZIP layout is wrong.');
  const report = await (await fetch(`${base}/api/builds/${job.id}/report`)).json();
  if (report.backend !== 'cpu' || report.texture_count !== 1) throw new Error('Production report is wrong.');
  const crossSite = await fetch(`${base}/api/project`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
      Origin: 'https://evil.example',
      'Sec-Fetch-Site': 'cross-site',
    },
    body: '{}',
  });
  if (crossSite.status !== 403) throw new Error('Cross-site mutation was not rejected.');
  console.log(
    `Production smoke passed: index, bundle, engine, ZIP (${Object.keys(zip).length} files), report, and CSRF guard. ${strFromU8(zip['pack.mcmeta']).length} bytes of metadata.`,
  );
} finally {
  child.kill('SIGTERM');
  await wait(300);
  await rm(directory, { recursive: true, force: true });
}
