import { drawOperation, type PaintDocument } from '../shared/document';
export async function renderDocument(doc: PaintDocument, target?: HTMLCanvasElement) {
  const ink = document.createElement('canvas'); ink.width = doc.width; ink.height = doc.height;
  const ctx = ink.getContext('2d', { willReadFrequently: true })!;
  for (const op of doc.operations) {
    if (op.type === 'image') {
      const img = new Image(); img.src = op.data; await img.decode();
      ctx.save(); if (op.transform) ctx.transform(...op.transform);
      ctx.drawImage(img, 0, 0, op.sourceSize?.width ?? doc.width, op.sourceSize?.height ?? doc.height); ctx.restore();
    } else drawOperation(ctx, op);
  }
  const output = target ?? document.createElement('canvas'); output.width = doc.width; output.height = doc.height;
  const out = output.getContext('2d')!;
  if (doc.background !== 'transparent') { out.fillStyle = doc.background; out.fillRect(0, 0, doc.width, doc.height); }
  out.drawImage(ink, 0, 0);
  return output;
}
