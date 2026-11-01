import { spawn, type ChildProcess } from 'node:child_process';
import { createHash, randomUUID } from 'node:crypto';
import { existsSync } from 'node:fs';
import fs from 'node:fs/promises';
import path from 'node:path';
import { PNG } from 'pngjs';
import {
  CATALOG,
  DEFAULT_SETTINGS,
  resolveEntry,
  toTexture,
  type MaterialEntry,
  type Project,
  type Settings,
  type Texture,
} from '../shared/catalog';
import {
  buildSchema,
  previewSchema,
  projectSchema,
  settingsSchema,
  type PreviewRequest,
} from '../shared/validation';

export const ROOT = path.resolve(import.meta.dirname, '..');
export const WORKSPACE = path.resolve(process.env.HYPERREALISM_DATA_DIR || path.join(ROOT, '.workspace'));
export const UPLOADS = path.join(WORKSPACE, 'uploads');
const JOBS = path.join(WORKSPACE, 'jobs');
const PREVIEWS = path.join(WORKSPACE, 'previews');
const DEFAULT_PROJECT: Project = {
  edits: {},
  custom: [],
  selected: CATALOG.map((t) => t.name),
  favorites: [],
  settings: DEFAULT_SETTINGS,
};
export class HttpError extends Error {
  constructor(
    public status: number,
    message: string,
  ) {
    super(message);
  }
}
export interface Job {
  id: string;
  name: string;
  status: 'queued' | 'running' | 'complete' | 'failed' | 'cancelled';
  createdAt: string;
  completedAt?: string;
  total: number;
  completed: number;
  current: string;
  settings: Settings;
  error?: string;
  bytes?: number;
  archive?: string;
  logs: string[];
}
const active = new Map<string, ChildProcess>();
const cancelled = new Set<string>();
let queue: Promise<unknown> = Promise.resolve();
let projectQueue: Promise<unknown> = Promise.resolve();
let historyQueue: Promise<unknown> = Promise.resolve();
let engineFingerprint = 'unbuilt';
let project: Project = structuredClone(DEFAULT_PROJECT);
let jobs: Job[] = [];
let ready: Promise<void>;
export function initialize() {
  ready = (async () => {
    await Promise.all([UPLOADS, JOBS, PREVIEWS].map((p) => fs.mkdir(p, { recursive: true })));
    try {
      const hash = createHash('sha256');
      hash.update(await fs.readFile(findEngine()));
      for (const file of ['diffuse.png', 'normal.png', 'roughness.png'])
        hash.update(await fs.readFile(path.join(ROOT, 'test_assets', file)));
      engineFingerprint = hash.digest('hex').slice(0, 16);
    } catch {
      /* health reports an unbuilt engine */
    }
    try {
      project = projectSchema.parse(
        JSON.parse(await fs.readFile(path.join(WORKSPACE, 'project.json'), 'utf8')),
      );
      const known = new Set([...CATALOG, ...project.custom].map((t) => t.name));
      project.selected = [...new Set(project.selected)].filter((n) => known.has(n));
      project.favorites = [...new Set(project.favorites)].filter((n) => known.has(n));
    } catch (e) {
      if ((e as NodeJS.ErrnoException).code !== 'ENOENT')
        console.warn('Saved project could not be loaded:', e);
    }
    try {
      const saved = JSON.parse(await fs.readFile(path.join(WORKSPACE, 'history.json'), 'utf8')) as Job[];
      jobs = saved
        .filter(
          (j) =>
            /^[a-f0-9-]{36}$/.test(j.id) && Array.isArray(j.logs) && j.settings && Number.isInteger(j.total),
        )
        .slice(0, 30)
        .map((j) =>
          ['queued', 'running'].includes(j.status)
            ? {
                ...j,
                status: 'failed',
                error: 'The server stopped before this build finished. Start a new build to try again.',
              }
            : j,
        );
    } catch (e) {
      if ((e as NodeJS.ErrnoException).code !== 'ENOENT')
        console.warn('Build history could not be loaded:', e);
    }
    await cleanupWorkspace();
  })();
  return ready;
}
// Startup-only garbage collection: never touches a running build or a recent draft upload.
async function cleanupWorkspace() {
  const referenced = new Set(project.custom.flatMap((entry) => Object.values(entry.assets || {})));
  const now = Date.now();
  const age = async (file: string) => now - (await fs.stat(file)).mtimeMs;
  for (const id of await fs.readdir(UPLOADS)) {
    const file = path.join(UPLOADS, id);
    if (!referenced.has(id.replace(/\.png$/, '')) && (await age(file)) > 24 * 60 * 60 * 1000)
      await fs.rm(file, { recursive: true, force: true });
  }
  const knownJobs = new Set(jobs.map((job) => job.id));
  for (const id of await fs.readdir(JOBS))
    if (!knownJobs.has(id)) await fs.rm(path.join(JOBS, id), { recursive: true, force: true });
  const previews = await Promise.all(
    (await fs.readdir(PREVIEWS)).map(async (id) => ({
      id,
      mtime: (await fs.stat(path.join(PREVIEWS, id))).mtimeMs,
    })),
  );
  previews.sort((a, b) => b.mtime - a.mtime);
  for (const [index, entry] of previews.entries()) {
    if (index >= 200 || now - entry.mtime > 7 * 24 * 60 * 60 * 1000)
      await fs.rm(path.join(PREVIEWS, entry.id), { recursive: true, force: true });
  }
  for (const file of await fs.readdir(WORKSPACE)) {
    if (file.endsWith('.tmp') && (await age(path.join(WORKSPACE, file))) > 60 * 60 * 1000)
      await fs.rm(path.join(WORKSPACE, file), { force: true });
  }
}
export async function atomicJson(file: string, value: unknown) {
  const tmp = `${file}.${randomUUID()}.tmp`;
  await fs.writeFile(tmp, JSON.stringify(value, null, 2));
  await fs.rename(tmp, file);
}
export async function getProject() {
  await ready;
  return project;
}
export async function saveProject(input: unknown) {
  await ready;
  const next = projectSchema.parse(input);
  const task = projectQueue
    .catch(() => {})
    .then(async () => {
      for (const entry of next.custom) {
        for (const id of Object.values(entry.assets || {})) {
          if (!id) continue;
          try {
            await fs.access(path.join(UPLOADS, `${id}.png`));
          } catch {
            throw new HttpError(400, `The source image for ${entry.label} is missing. Import it again.`);
          }
        }
      }
      for (const [name, edit] of Object.entries(next.edits)) {
        const original = [...CATALOG, ...next.custom].find((t) => t.name === name);
        if (edit.material && edit.material !== original?.material)
          throw new HttpError(400, 'An edit cannot change the source material.');
      }
      const known = new Set([...CATALOG, ...next.custom].map((t) => t.name));
      if ([...next.selected, ...next.favorites, ...Object.keys(next.edits)].some((n) => !known.has(n)))
        throw new HttpError(400, 'The project references an unknown material.');
      const saved: Project = {
        ...next,
        selected: [...new Set(next.selected)],
        favorites: [...new Set(next.favorites)],
      };
      await atomicJson(path.join(WORKSPACE, 'project.json'), saved);
      project = saved;
      return saved;
    });
  projectQueue = task.catch(() => {});
  return task;
}
export async function importImage(buffer: Buffer) {
  await ready;
  if (buffer.length < 8 || !buffer.subarray(0, 8).equals(Buffer.from([137, 80, 78, 71, 13, 10, 26, 10])))
    throw new HttpError(400, 'Choose a valid PNG image. JPEG and WebP are not supported.');
  // Inspect IHDR before allocating a decoder buffer, avoiding decompression bombs.
  if (buffer.length < 33 || buffer.toString('ascii', 12, 16) !== 'IHDR')
    throw new HttpError(400, 'This PNG has no valid image header.');
  const width = buffer.readUInt32BE(16),
    height = buffer.readUInt32BE(20);
  if (width < 1 || height < 1 || width > 4096 || height > 4096 || width * height > 16777216)
    throw new HttpError(400, 'Source images must be at most 4096 × 4096 and 16 megapixels.');
  let png: PNG;
  try {
    png = PNG.sync.read(buffer);
  } catch {
    throw new HttpError(400, 'This PNG could not be decoded. Try exporting it again.');
  }
  if (
    png.width < 1 ||
    png.height < 1 ||
    png.width > 4096 ||
    png.height > 4096 ||
    png.width * png.height > 16777216
  )
    throw new HttpError(400, 'Source images must be at most 4096 × 4096 and 16 megapixels.');
  const id = randomUUID();
  await fs.writeFile(path.join(UPLOADS, `${id}.png`), PNG.sync.write(png));
  return { id, width: png.width, height: png.height };
}
export function findEngine() {
  const custom = process.env.HYPERREALISM_BIN;
  const suffix = process.platform === 'win32' ? '.exe' : '';
  const candidates = [
    custom,
    path.join(ROOT, `target/release/hyperrealism-core${suffix}`),
    path.join(ROOT, `target/debug/hyperrealism-core${suffix}`),
  ].filter(Boolean) as string[];
  const found = candidates.find((p) => existsSync(p));
  if (!found)
    throw new HttpError(503, 'The Rust engine is not built yet. Run npm run build:core, then try again.');
  return found;
}
interface RunResult {
  code: number;
  stdout: string;
  stderr: string;
}
function runEngine(
  args: string[],
  onLine?: (line: string) => void,
  register?: (child: ChildProcess) => void,
): Promise<RunResult> {
  return new Promise((resolve, reject) => {
    const child = spawn(findEngine(), args, {
      cwd: ROOT,
      windowsHide: true,
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    register?.(child);
    let stdout = '';
    let stderr = '';
    let pending = '';
    let timedOut = false;
    const timeout = setTimeout(
      () => {
        timedOut = true;
        child.kill('SIGTERM');
        setTimeout(() => child.kill('SIGKILL'), 5000).unref();
        reject(new HttpError(504, 'The engine timed out. Try fewer materials or smaller source images.'));
      },
      5 * 60 * 1000,
    );
    child.stdout.on('data', (chunk: Buffer) => {
      const text = chunk.toString();
      stdout = (stdout + text).slice(-32000);
      pending = (pending + text).slice(-4096);
      const lines = pending.split('\n');
      pending = lines.pop() || '';
      for (const line of lines) if (line.trim()) onLine?.(line.trim());
    });
    child.stderr.on('data', (chunk: Buffer) => {
      stderr = (stderr + chunk.toString()).slice(-12000);
    });
    child.on('error', (e) => {
      clearTimeout(timeout);
      if (!timedOut) reject(new HttpError(503, `Could not start the Rust engine: ${e.message}`));
    });
    child.on('close', (code) => {
      clearTimeout(timeout);
      if (timedOut) return;
      if (pending.trim()) onLine?.(pending.trim());
      resolve({ code: code ?? 1, stdout, stderr });
    });
  });
}
// Material preparation is separate from texture selection; custom assets never use client paths.
async function prepare(
  directory: string,
  textures: Texture[],
  entries: MaterialEntry[],
  normalY = 'directx',
) {
  await fs.mkdir(directory, { recursive: true });
  const used = entries.filter((m) => textures.some((t) => t.material === m.name));
  const materials: object[] = [];
  if (textures.some((t) => t.material === 'synthetic')) {
    const source = path.join(ROOT, 'test_assets');
    for (const file of ['diffuse.png', 'normal.png', 'roughness.png'])
      await fs.copyFile(path.join(source, file), path.join(directory, file));
    materials.push({
      id: 'synthetic',
      albedo: 'diffuse.png',
      normal: 'normal.png',
      roughness: 'roughness.png',
      normal_y: 'directx',
    });
  }
  for (const entry of used) {
    if (!entry.assets) throw new HttpError(400, `${entry.label} is missing source images.`);
    const m: Record<string, string> = { id: entry.name, normal_y: entry.normal_y || normalY };
    for (const [kind, id] of Object.entries(entry.assets)) {
      if (!id) continue;
      const file = `${entry.name}_${kind}.png`;
      try {
        await fs.copyFile(path.join(UPLOADS, `${id}.png`), path.join(directory, file));
      } catch {
        throw new HttpError(400, `The ${kind} image for ${entry.label} is missing. Import it again.`);
      }
      m[kind] = file;
    }
    materials.push(m);
  }
  const manifest = { version: 1, materials, textures };
  const content = JSON.stringify(manifest, null, 2);
  await fs.writeFile(path.join(directory, 'material_manifest.json'), content);
  return createHash('sha256').update(content).digest('hex');
}
const previewRequests = new Map<
  string,
  Promise<{ albedo: string; normal: string; specular: string; cached: boolean }>
>();
let previewQueue: Promise<unknown> = Promise.resolve();
export async function getPreview(input: PreviewRequest) {
  await ready;
  const parsed = previewSchema.parse(input);
  const key = createHash('sha256')
    .update(JSON.stringify({ ...parsed, engine: engineFingerprint }))
    .digest('hex')
    .slice(0, 24);
  const existing = previewRequests.get(key);
  if (existing) return existing;
  const task = previewQueue.catch(() => {}).then(() => generatePreview(parsed));
  previewRequests.set(key, task);
  try {
    return await task;
  } finally {
    previewRequests.delete(key);
  }
}
async function generatePreview({ texture, seed, assets, normal_y }: PreviewRequest) {
  let entry: MaterialEntry | undefined;
  if (assets)
    entry = {
      ...texture,
      custom: true,
      assets,
      label: texture.name,
      category: 'Stone',
      description: '',
      material: texture.name,
    };
  const hash = createHash('sha256')
    .update(JSON.stringify({ texture, seed, assets, normal_y, engine: engineFingerprint }))
    .digest('hex')
    .slice(0, 24);
  const folder = path.join(PREVIEWS, hash);
  const block = path.join(folder, 'output/pack/assets/minecraft/textures/block');
  if (!['', '_n', '_s'].every((suffix) => existsSync(path.join(block, `${texture.name}${suffix}.png`)))) {
    await fs.rm(folder, { recursive: true, force: true });
    await fs.mkdir(folder, { recursive: true });
    const inputDir = path.join(folder, 'input');
    await prepare(inputDir, [texture], entry ? [entry] : [], normal_y);
    const result = await runEngine([
      '--input',
      inputDir,
      '--output',
      path.join(folder, 'output'),
      '--cpu-only',
      '--threads',
      '2',
      '--resolution',
      '128',
      '--seed',
      String(seed),
    ]);
    if (result.code !== 0) {
      await fs.rm(folder, { recursive: true, force: true });
      throw new HttpError(400, result.stderr || 'Preview generation failed.');
    }
  }
  const base = `/api/previews/${hash}/${texture.name}`;
  return { albedo: `${base}.png`, normal: `${base}_n.png`, specular: `${base}_s.png`, cached: true };
}
export function previewFile(hash: string, file: string) {
  if (!/^[a-f0-9]{24}$/.test(hash) || !/^[a-z0-9_]{1,80}(_n|_s)?\.png$/.test(file))
    throw new HttpError(404, 'Preview not found.');
  return path.join(PREVIEWS, hash, 'output/pack/assets/minecraft/textures/block', file);
}
export async function listJobs() {
  await ready;
  return jobs;
}
function persistJobs() {
  const snapshot = structuredClone(jobs);
  const task = historyQueue
    .catch(() => {})
    .then(() => atomicJson(path.join(WORKSPACE, 'history.json'), snapshot));
  historyQueue = task.catch(() => {});
  return task;
}
export async function startBuild(input: unknown) {
  await ready;
  findEngine();
  const { names, settings } = buildSchema.parse(input);
  if (new Set(names).size !== names.length) throw new HttpError(400, 'Duplicate texture names in build.');
  const all = [...CATALOG, ...project.custom].map((t) => resolveEntry(t, project.edits));
  const entries = names.map((n) => all.find((t) => t.name === n));
  if (entries.some((t) => !t))
    throw new HttpError(400, 'A selected material no longer exists. Save your project and try again.');
  if (jobs.filter((j) => j.status === 'queued' || j.status === 'running').length >= 3)
    throw new HttpError(429, 'Three builds are already queued. Wait for one to finish.');
  const job: Job = {
    id: randomUUID(),
    name: settings.packName,
    status: 'queued',
    createdAt: new Date().toISOString(),
    total: names.length,
    completed: 0,
    current: 'Waiting for the CPU engine',
    settings,
    logs: ['Build queued. Your saved materials will be used.'],
  };
  const previous = jobs;
  const removed = previous.slice(29);
  jobs = [job, ...previous].slice(0, 30);
  try {
    await persistJobs();
  } catch (e) {
    jobs = previous;
    throw e;
  }
  for (const old of removed) await fs.rm(path.join(JOBS, old.id), { recursive: true, force: true });
  const textures = (entries as MaterialEntry[]).map(toTexture);
  const task = queue.then(async () => {
    if (cancelled.has(job.id)) {
      job.status = 'cancelled';
      job.current = 'Cancelled';
      await persistJobs();
      return;
    }
    job.status = 'running';
    job.current = 'Preparing source maps';
    await persistJobs();
    const directory = path.join(JOBS, job.id);
    try {
      await prepare(path.join(directory, 'input'), textures, entries as MaterialEntry[]);
      const result = await runEngine(
        [
          '--input',
          path.join(directory, 'input'),
          '--output',
          path.join(directory, 'output'),
          '--cpu-only',
          '--threads',
          String(settings.threads),
          '--resolution',
          String(settings.resolution),
          '--seed',
          String(settings.seed),
          '--name',
          settings.packName,
          ...(settings.includeNormal ? [] : ['--no-normal']),
          ...(settings.includeSpecular ? [] : ['--no-specular']),
        ],
        (line) => {
          job.logs.push(line);
          job.logs = job.logs.slice(-100);
          const progress = line.match(/^PROGRESS (\d+)\/(\d+) (.+)$/);
          if (progress) {
            job.completed = Number(progress[1]);
            job.current = progress[3];
          }
        },
        (child) => active.set(job.id, child),
      );
      if (cancelled.has(job.id)) {
        job.status = 'cancelled';
        job.current = 'Cancelled';
      } else if (result.code !== 0) {
        job.status = 'failed';
        job.error = result.stderr || `Engine exited with code ${result.code}`;
        job.current = 'Build failed';
      } else {
        const archive = path.join(
          directory,
          `output/dist/HyperRealism${settings.resolution}-v1.0-MC26.3.zip`,
        );
        const stat = await fs.stat(archive);
        job.status = 'complete';
        job.bytes = stat.size;
        job.archive = path.basename(archive);
        job.current = 'Ready to download';
      }
    } catch (e) {
      job.status = cancelled.has(job.id) ? 'cancelled' : 'failed';
      job.error = (e as Error).message;
      job.current = job.status === 'cancelled' ? 'Cancelled' : 'Build failed';
    } finally {
      active.delete(job.id);
      cancelled.delete(job.id);
      job.completedAt = new Date().toISOString();
      await persistJobs();
    }
  });
  queue = task.catch((e) => console.error('Build queue failure', e));
  return job;
}
export async function getJob(id: string) {
  await ready;
  const job = jobs.find((j) => j.id === id);
  if (!job) throw new HttpError(404, 'Build not found.');
  return job;
}
export async function cancelJob(id: string) {
  const job = await getJob(id);
  if (job.status === 'queued') {
    cancelled.add(id);
    job.status = 'cancelled';
    job.current = 'Cancelled';
    await persistJobs();
  } else if (job.status === 'running') {
    cancelled.add(id);
    job.current = 'Cancelling…';
    active.get(id)?.kill('SIGTERM');
    await persistJobs();
  } else throw new HttpError(409, 'This build has already finished.');
  return job;
}
export async function artifact(id: string, kind: 'zip' | 'sheet' | 'report') {
  const job = await getJob(id);
  if (job.status !== 'complete') throw new HttpError(409, 'This build is not ready.');
  const file =
    kind === 'zip'
      ? `dist/HyperRealism${job.settings.resolution}-v1.0-MC26.3.zip`
      : kind === 'sheet'
        ? 'contact_sheet.png'
        : 'build_report.json';
  return path.join(JOBS, id, 'output', file);
}
export async function deleteJob(id: string) {
  const job = await getJob(id);
  if (['running', 'queued'].includes(job.status))
    throw new HttpError(409, 'Cancel this build before deleting it.');
  jobs = jobs.filter((j) => j.id !== id);
  await persistJobs();
  await fs.rm(path.join(JOBS, id), { recursive: true, force: true });
}
export async function health() {
  await ready;
  let engine = false;
  let version = '';
  try {
    const result = await runEngine(['--help']);
    engine = result.code === 0;
    version = result.stdout.split('\n')[0];
  } catch {
    /* reported as unavailable */
  }
  return {
    status: 'ok',
    engine,
    version,
    backend: 'CPU',
    storage: 'local',
    labpbr: '1.3',
    target: 'Minecraft Java 26.3',
    demoTextures: CATALOG.length,
  };
}
