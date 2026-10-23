import type { PaintDocument } from '../shared/document';
import { documentSchema } from '../shared/document';
export async function draftStore(value?: PaintDocument): Promise<PaintDocument | undefined> {
  const db = await new Promise<IDBDatabase>((resolve, reject) => {
    const request = indexedDB.open('openaec-paint', 1);
    request.onupgradeneeded = () => request.result.createObjectStore('drafts');
    request.onsuccess = () => resolve(request.result); request.onerror = () => reject(request.error);
  });
  try {
    return await new Promise((resolve, reject) => {
      const tx = db.transaction('drafts', value ? 'readwrite' : 'readonly');
      const request = value ? tx.objectStore('drafts').put(value, 'current') : tx.objectStore('drafts').get('current');
      tx.oncomplete = () => { try { resolve(value ? value : request.result ? documentSchema.parse(request.result) : undefined); } catch (error) { reject(error); } };
      tx.onerror = () => reject(tx.error);
    });
  } finally { db.close(); }
}
export const isDesktop = () => '__TAURI_INTERNALS__' in window;
export async function saveFile(name: string, data: Blob, extension: string) {
  if (isDesktop()) {
    const { save } = await import('@tauri-apps/plugin-dialog');
    const { writeFile } = await import('@tauri-apps/plugin-fs');
    const path = await save({ defaultPath: name, filters: [{ name: extension.toUpperCase(), extensions: [extension] }] });
    if (!path) return false;
    await writeFile(path, new Uint8Array(await data.arrayBuffer()));
  } else {
    const url = URL.createObjectURL(data), a = document.createElement('a');
    a.href = url; a.download = name; a.click(); setTimeout(() => URL.revokeObjectURL(url), 1000);
  }
  return true;
}
