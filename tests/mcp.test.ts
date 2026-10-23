import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { resolve, join } from 'node:path';
import { Client } from '@modelcontextprotocol/client';
import { StdioClientTransport } from '@modelcontextprotocol/client/stdio';
import { createApi } from '../services/api.js';
test('MCP stdio handshake, draw, PNG export and undo through the real API', async () => {
  const directory = mkdtempSync(join(tmpdir(), 'paint-mcp-')), token = 'mcp-test-token-with-at-least-32-characters';
  const api = createApi({ directory, token });
  await new Promise<void>(r => api.listen(0, '127.0.0.1', r)); const address = api.address(); assert(address && typeof address !== 'string');
  const transport = new StdioClientTransport({ command: process.execPath, args: ['--import', 'tsx', resolve('services/mcp.ts')], env: { ...process.env as Record<string, string>, PAINT_API_URL: `http://127.0.0.1:${address.port}`, PAINT_API_TOKEN: token }, stderr: 'pipe' });
  const client = new Client({ name: 'paint-test', version: '1.0.0' });
  try {
    await client.connect(transport);
    const tools = await client.listTools(); assert.equal(tools.tools.length, 6);
    const result = await client.callTool({ name: 'paint_create_document', arguments: { name: 'MCP test', width: 64, height: 64 } });
    assert(!result.isError); const block = result.content[0]; assert(block.type === 'text'); const stored = JSON.parse(block.text);
    const drawn = await client.callTool({ name: 'paint_draw', arguments: { id: stored.id, expectedRevision: 1, operations: [{ type: 'line', from: { x: 10, y: 10 }, to: { x: 50, y: 50 }, color: '#d97706', size: 5 }] } }); assert(!drawn.isError);
    const exported = await client.callTool({ name: 'paint_export_png', arguments: { id: stored.id } }); assert.equal(exported.content[0].type, 'image');
    const conflict = await client.callTool({ name: 'paint_undo', arguments: { id: stored.id, expectedRevision: 1 } }); assert.equal(conflict.isError, true);
    const undone = await client.callTool({ name: 'paint_undo', arguments: { id: stored.id, expectedRevision: 2 } }); assert(!undone.isError);
  } finally { await client.close(); await transport.close(); await new Promise<void>(r => api.close(() => r())); rmSync(directory, { recursive: true, force: true }); }
});
