import express, { type NextFunction, type Request, type Response } from 'express';
import multer from 'multer';
import { ZodError } from 'zod';
import { existsSync } from 'node:fs';
import path from 'node:path';
import { CATALOG } from '../shared/catalog';
import { previewSchema } from '../shared/validation';
import * as engine from './engine';

const app = express();
app.disable('x-powered-by');
// The studio is a single-user local service, not an authenticated public API.
// Reject browser cross-site mutations; CLI/API clients may omit Origin.
app.use('/api', (req, res, next) => {
  if (!['GET', 'HEAD', 'OPTIONS'].includes(req.method)) {
    const forwardedHost = req.get('x-forwarded-host')?.split(',')[0]?.trim();
    const host = forwardedHost || req.get('host');
    const origin = req.get('origin');
    if (req.get('sec-fetch-site') === 'cross-site' || (origin && new URL(origin).host !== host)) {
      return res.status(403).json({ error: 'Cross-site requests are not allowed.' });
    }
  }
  next();
});
app.use(express.json({ limit: '1mb' }));
app.use('/api', (_req, res, next) => {
  res.set('Cache-Control', 'no-store');
  next();
});
const asyncRoute =
  (fn: (req: Request, res: Response) => Promise<unknown>) =>
  (req: Request, res: Response, next: NextFunction) => {
    Promise.resolve(fn(req, res)).catch(next);
  };
const upload = multer({
  storage: multer.memoryStorage(),
  limits: { fileSize: 24 * 1024 * 1024, files: 1, fields: 0 },
});
app.get(
  '/api/health',
  asyncRoute(async (_req, res) => {
    res.json(await engine.health());
  }),
);
app.get('/api/catalog', (_req, res) => {
  res.json(CATALOG);
});
app.get(
  '/api/project',
  asyncRoute(async (_req, res) => {
    res.json(await engine.getProject());
  }),
);
app.put(
  '/api/project',
  asyncRoute(async (req, res) => {
    res.json(await engine.saveProject(req.body));
  }),
);
app.post(
  '/api/uploads',
  upload.single('image'),
  asyncRoute(async (req, res) => {
    if (!req.file) throw new engine.HttpError(400, 'Choose a PNG image to import.');
    res.status(201).json(await engine.importImage(req.file.buffer));
  }),
);
app.post(
  '/api/previews',
  asyncRoute(async (req, res) => {
    res.json(await engine.getPreview(previewSchema.parse(req.body)));
  }),
);
app.get(
  '/api/previews/:hash/:file',
  asyncRoute(async (req, res) => {
    const file = engine.previewFile(String(req.params.hash), String(req.params.file));
    if (!existsSync(file)) throw new engine.HttpError(404, 'This preview has expired. Generate it again.');
    res.type('png').set('Cache-Control', 'private, max-age=86400').sendFile(file, { dotfiles: 'allow' });
  }),
);
app.get(
  '/api/builds',
  asyncRoute(async (_req, res) => {
    res.json(await engine.listJobs());
  }),
);
app.post(
  '/api/builds',
  asyncRoute(async (req, res) => {
    res.status(202).json(await engine.startBuild(req.body));
  }),
);
app.get(
  '/api/builds/:id',
  asyncRoute(async (req, res) => {
    res.json(await engine.getJob(String(req.params.id)));
  }),
);
app.post(
  '/api/builds/:id/cancel',
  asyncRoute(async (req, res) => {
    res.json(await engine.cancelJob(String(req.params.id)));
  }),
);
app.delete(
  '/api/builds/:id',
  asyncRoute(async (req, res) => {
    await engine.deleteJob(String(req.params.id));
    res.status(204).end();
  }),
);
app.get(
  '/api/builds/:id/download',
  asyncRoute(async (req, res) => {
    const job = await engine.getJob(String(req.params.id));
    const file = await engine.artifact(job.id, 'zip');
    const filename = `${job.name.replace(/[^a-zA-Z0-9 _-]/g, '').trim() || 'HyperRealism'}-${job.settings.resolution}.zip`;
    res.download(file, filename, { dotfiles: 'allow' });
  }),
);
app.get(
  '/api/builds/:id/:kind',
  asyncRoute(async (req, res) => {
    if (req.params.kind !== 'sheet' && req.params.kind !== 'report')
      throw new engine.HttpError(404, 'Artifact not found.');
    const file = await engine.artifact(String(req.params.id), req.params.kind);
    res
      .set(
        'Content-Disposition',
        `inline; filename="${req.params.kind === 'sheet' ? 'contact_sheet.png' : 'build_report.json'}"`,
      )
      .sendFile(file, { dotfiles: 'allow' });
  }),
);
app.use('/api', (_req, res) => {
  res.status(404).json({ error: 'API endpoint not found.' });
});
const production = process.argv.includes('--production') || process.env.NODE_ENV === 'production';
if (production) {
  const dist = path.join(engine.ROOT, 'dist-web');
  app.use(express.static(dist));
  app.get('/{*path}', (_req, res) => {
    if (!existsSync(path.join(dist, 'index.html')))
      return res.status(503).send('Run npm run build:web first.');
    res.sendFile(path.join(dist, 'index.html'));
  });
}
app.use((error: Error, _req: Request, res: Response, _next: NextFunction) => {
  if (error instanceof ZodError)
    return res
      .status(400)
      .json({ error: error.issues.map((i) => `${i.path.join('.')}: ${i.message}`).join(' ') });
  if (error instanceof multer.MulterError)
    return res.status(400).json({
      error: error.code === 'LIMIT_FILE_SIZE' ? 'PNG files must be smaller than 24 MB.' : error.message,
    });
  if (error instanceof SyntaxError && 'body' in error)
    return res.status(400).json({ error: 'The request contains invalid JSON.' });
  const status = error instanceof engine.HttpError ? error.status : 500;
  if (status === 500) console.error(error);
  res.status(status).json({
    error: status === 500 ? 'Something went wrong on the server. Please try again.' : error.message,
  });
});
await engine.initialize();
const port = Number(process.env.API_PORT || 3001);
const server = app.listen(port, '0.0.0.0', () =>
  console.log(`HyperRealism API listening on http://0.0.0.0:${port}`),
);
server.on('error', (error) => {
  console.error(error);
  process.exitCode = 1;
});
