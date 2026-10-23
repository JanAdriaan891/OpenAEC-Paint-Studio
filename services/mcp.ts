import { packageVersion } from '../shared/version.js';
import { McpServer } from '@modelcontextprotocol/server';
import { serveStdio } from '@modelcontextprotocol/server/stdio';
import { z } from 'zod';
import { operationSchema, newDocument } from '../shared/document.js';
import { apiToken } from './config.js';
const base = process.env.PAINT_API_URL || 'http://127.0.0.1:47831';
const target = new URL(base);
if (!['127.0.0.1', 'localhost', '[::1]'].includes(target.hostname) || target.protocol !== 'http:' || target.username || target.password) throw new Error('PAINT_API_URL must be a local HTTP URL');
async function request(path: string, method = 'GET', body?: unknown) {
  return fetch(base + path, { method, headers: { Authorization: `Bearer ${apiToken()}`, ...(body ? { 'Content-Type': 'application/json' } : {}) }, ...(body ? { body: JSON.stringify(body) } : {}), signal: AbortSignal.timeout(30000) });
}
async function jsonRequest(path: string, method = 'GET', body?: unknown) {
  const response = await request(path, method, body), data = await response.json();
  if (!response.ok) throw new Error(data.error || `API ${response.status}`); return data;
}
const textResult = (data: unknown) => ({ content: [{ type: 'text' as const, text: JSON.stringify(data) }] });
const id = z.string().uuid();
function createMcp() {
  const server = new McpServer({ name: 'openaec-paint-studio', version: packageVersion });
  server.registerTool('paint_list_documents', { description: 'List documents stored in the local Paint API workspace.', inputSchema: z.object({}) }, async () => textResult(await jsonRequest('/v1/documents')));
  server.registerTool('paint_create_document', { description: 'Create a blank Paint document. Returns its UUID and revision. The user can load it in the app via Automation.', inputSchema: z.object({ name: z.string().min(1).max(120), width: z.number().int().min(16).max(4096).default(1200), height: z.number().int().min(16).max(4096).default(800) }) }, async ({ name, width, height }) => textResult(await jsonRequest('/v1/documents', 'POST', newDocument(width, height, name))));
  server.registerTool('paint_get_document', { description: 'Read a document and its current revision before modifying it.', inputSchema: z.object({ id }) }, async ({ id }) => textResult(await jsonRequest(`/v1/documents/${id}`)));
  server.registerTool('paint_draw', { description: 'Append brush, eraser, fill, line, rectangle, ellipse or text operations in canvas pixels. Pass the revision just read; conflicts return an error. Colors are #RRGGBB, opacity 0–1, size 1–200.', inputSchema: z.object({ id, expectedRevision: z.number().int().positive(), operations: z.array(operationSchema).min(1).max(200) }) }, async ({ id, expectedRevision, operations }) => textResult(await jsonRequest(`/v1/documents/${id}/operations`, 'POST', { operations, expectedRevision })));
  server.registerTool('paint_undo', { description: 'Remove the last operation. Requires the current revision.', inputSchema: z.object({ id, expectedRevision: z.number().int().positive() }) }, async ({ id, expectedRevision }) => textResult(await jsonRequest(`/v1/documents/${id}/undo`, 'POST', { expectedRevision })));
  server.registerTool('paint_export_png', { description: 'Render a document to a PNG image and return it inline.', inputSchema: z.object({ id }) }, async ({ id }) => {
    const response = await request(`/v1/documents/${id}/export.png`);
    if (!response.ok) throw new Error((await response.json()).error);
    return { content: [{ type: 'image' as const, mimeType: 'image/png', data: Buffer.from(await response.arrayBuffer()).toString('base64') }] };
  });
  return server;
}
void serveStdio(createMcp);
