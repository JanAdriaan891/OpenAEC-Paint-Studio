import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { resolve, dirname } from 'node:path';
import { randomBytes } from 'node:crypto';
const sourceParent = fileURLToPath(new URL('../', import.meta.url));
export const projectRoot = existsSync(resolve(sourceParent, 'package.json')) ? sourceParent : resolve(sourceParent, '..');
export const dataDir = resolve(process.env.PAINT_DATA_DIR || resolve(projectRoot, '.paint-data'));
export function apiToken(create = false) {
  if (process.env.PAINT_API_TOKEN) {
    if (process.env.PAINT_API_TOKEN.length < 32) throw new Error('PAINT_API_TOKEN must contain at least 32 characters');
    return process.env.PAINT_API_TOKEN;
  }
  const path = resolve(dataDir, 'token');
  if (!existsSync(path) && create) {
    mkdirSync(dirname(path), { recursive: true, mode: 0o700 });
    writeFileSync(path, randomBytes(32).toString('hex'), { mode: 0o600, flag: 'wx' });
  }
  if (!existsSync(path)) throw new Error('Start the API first, or set PAINT_API_TOKEN and PAINT_API_URL');
  return readFileSync(path, 'utf8').trim();
}
