import { createServer, type IncomingMessage } from 'node:http';
import { timingSafeEqual } from 'node:crypto';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { z, ZodError } from 'zod';
import { documentSchema, operationSchema } from '../shared/document.js';
import { apiToken, dataDir } from './config.js';
import { DocumentStore, HttpError, idSchema } from './store.js';
import { renderPng } from './renderer.js';
const origins = new Set(['http://127.0.0.1:3020', 'http://localhost:3020', 'http://127.0.0.1:4173', 'http://localhost:4173', 'http://tauri.localhost', 'https://tauri.localhost', 'tauri://localhost']);
const revisionSchema = z.number().int().positive();
async function readBody(req: IncomingMessage) {
  if (!req.headers['content-type']?.startsWith('application/json')) throw new HttpError(415, 'Use application/json');
  if (Number(req.headers['content-length'] || 0) > 64_000_000) { req.resume(); throw new HttpError(413, 'Request exceeds 64 MB'); }
  let size = 0; const chunks: Buffer[] = [];
  for await (const chunk of req.iterator({ destroyOnReturn: false })) {
    size += chunk.length; if (size > 64_000_000) { req.resume(); throw new HttpError(413, 'Request exceeds 64 MB'); }
    chunks.push(chunk);
  }
  try { return JSON.parse(Buffer.concat(chunks).toString()); } catch { throw new HttpError(400, 'Invalid JSON'); }
}
export function createApi({ directory = dataDir, token = apiToken(true) }: { directory?: string; token?: string } = {}) {
  const store = new DocumentStore(resolve(directory, 'documents'));
  return createServer(async (req, res) => {
    const json = (value: unknown, status = 200) => { res.writeHead(status, { 'Content-Type': 'application/json' }); res.end(JSON.stringify(value)); };
    res.setHeader('Cache-Control', 'no-store'); res.setHeader('X-Content-Type-Options', 'nosniff');
    try {
      const origin = req.headers.origin;
      if (origin && !origins.has(origin)) throw new HttpError(403, 'Origin not allowed');
      // A fixed loopback Host blocks DNS rebinding, including non-browser clients.
      if (!/^(127\.0\.0\.1|localhost)(:\d+)?$/.test(req.headers.host || '')) throw new HttpError(403, 'Host not allowed');
      if (origin) { res.setHeader('Access-Control-Allow-Origin', origin); res.setHeader('Vary', 'Origin'); }
      if (req.method === 'OPTIONS') { res.writeHead(204, { 'Access-Control-Allow-Methods': 'GET, POST, PUT, OPTIONS', 'Access-Control-Allow-Headers': 'Authorization, Content-Type' }); res.end(); return; }
      const supplied = Buffer.from(req.headers.authorization || ''), expected = Buffer.from(`Bearer ${token}`);
      if (supplied.length !== expected.length || !timingSafeEqual(supplied, expected)) throw new HttpError(401, 'Valid bearer token required');
      const url = new URL(req.url || '/', 'http://127.0.0.1');
      const segments = url.pathname.split('/').filter(Boolean);
      if (url.pathname === '/health' && req.method === 'GET') return json({ name: 'openaec-paint-studio', version: '0.1.0', status: 'ok' });
      if (segments[0] !== 'v1' || segments[1] !== 'documents') throw new HttpError(404, 'Route not found');
      if (segments.length === 2) {
        if (req.method === 'GET') return json(store.list());
        if (req.method === 'POST') return json(store.create(documentSchema.parse(await readBody(req))), 201);
      }
      if (segments.length >= 3) {
        const id = idSchema.parse(segments[2]);
        if (segments.length === 3 && req.method === 'GET') return json(store.get(id));
        if (segments.length === 3 && req.method === 'PUT') {
          const body = z.object({ document: documentSchema, expectedRevision: revisionSchema }).parse(await readBody(req));
          return json(store.replace(id, body.document, body.expectedRevision));
        }
        if (segments.length === 4 && segments[3] === 'operations' && req.method === 'POST') {
          const body = z.object({ operations: z.array(operationSchema).min(1).max(200), expectedRevision: revisionSchema }).parse(await readBody(req));
          return json(store.append(id, body.operations, body.expectedRevision));
        }
        if (segments.length === 4 && segments[3] === 'undo' && req.method === 'POST') {
          const body = z.object({ expectedRevision: revisionSchema }).parse(await readBody(req));
          return json(store.undo(id, body.expectedRevision));
        }
        if (segments.length === 4 && segments[3] === 'export.png' && req.method === 'GET') {
          const png = await renderPng(store.get(id).document); res.writeHead(200, { 'Content-Type': 'image/png', 'Content-Disposition': 'attachment; filename="drawing.png"' }); res.end(png); return;
        }
      }
      throw new HttpError(404, 'Route not found');
    } catch (error) {
      if (error instanceof ZodError) return json({ error: 'Validation failed', issues: error.issues.map(i => ({ path: i.path, message: i.message })) }, 400);
      if (error instanceof HttpError) return json({ error: error.message }, error.status);
      console.error('Paint API request failed:', error instanceof Error ? error.message : 'unknown error');
      json({ error: 'Internal server error' }, 500);
    }
  });
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const port = Number(process.env.PAINT_API_PORT || 47831);
  if (!Number.isInteger(port) || port < 1 || port > 65535) throw new Error('Invalid PAINT_API_PORT');
  const server = createApi();
  server.listen(port, '127.0.0.1', () => console.error(`OpenAEC Paint API: http://127.0.0.1:${port}\nToken: ${resolve(dataDir, 'token')} (or PAINT_API_TOKEN)\nDocuments: ${resolve(dataDir, 'documents')}`));
  server.on('error', error => { console.error(error.message); process.exitCode = 1; });
  for (const signal of ['SIGINT', 'SIGTERM'] as const) process.on(signal, () => server.close(() => process.exit(0)));
}
