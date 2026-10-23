import { mkdirSync, readdirSync, readFileSync, writeFileSync, renameSync } from 'node:fs';
import { randomUUID } from 'node:crypto';
import { resolve } from 'node:path';
import { z } from 'zod';
import { documentSchema, type PaintDocument, type Operation } from '../shared/document.js';
export const idSchema = z.string().uuid();
export const storedSchema = z.object({ id: idSchema, revision: z.number().int().positive(), updatedAt: z.string(), document: documentSchema });
export type StoredDocument = z.infer<typeof storedSchema>;
export class HttpError extends Error { constructor(public status: number, message: string) { super(message); } }
export class DocumentStore {
  constructor(readonly directory: string) { mkdirSync(directory, { recursive: true, mode: 0o700 }); }
  path(id: string) { return resolve(this.directory, `${idSchema.parse(id)}.json`); }
  get(id: string): StoredDocument {
    try { return storedSchema.parse(JSON.parse(readFileSync(this.path(id), 'utf8'))); }
    catch (e) { if ((e as NodeJS.ErrnoException).code === 'ENOENT') throw new HttpError(404, 'Document not found'); throw e; }
  }
  list() { return readdirSync(this.directory).filter(name => /^[a-f0-9-]{36}\.json$/.test(name)).map(name => {
    const { id, revision, updatedAt, document } = this.get(name.slice(0, -5));
    return { id, revision, updatedAt, name: document.name, width: document.width, height: document.height };
  }); }
  private write(doc: StoredDocument) {
    const path = this.path(doc.id), temp = `${path}.tmp`;
    writeFileSync(temp, JSON.stringify(doc), { mode: 0o600 }); renameSync(temp, path); return doc;
  }
  create(document: PaintDocument) {
    if (this.list().length >= 100) throw new HttpError(409, 'Workspace limit reached (100 documents)');
    return this.write({ id: randomUUID(), revision: 1, updatedAt: new Date().toISOString(), document: documentSchema.parse(document) });
  }
  replace(id: string, document: PaintDocument, expectedRevision: number) {
    const current = this.get(id);
    if (current.revision !== expectedRevision) throw new HttpError(409, 'Document changed. Load the latest revision before saving.');
    return this.write({ ...current, revision: current.revision + 1, updatedAt: new Date().toISOString(), document: documentSchema.parse(document) });
  }
  append(id: string, operations: Operation[], expectedRevision: number) {
    const current = this.get(id); return this.replace(id, { ...current.document, operations: [...current.document.operations, ...operations] }, expectedRevision);
  }
  undo(id: string, expectedRevision: number) {
    const current = this.get(id); return this.replace(id, { ...current.document, operations: current.document.operations.slice(0, -1) }, expectedRevision);
  }
}
