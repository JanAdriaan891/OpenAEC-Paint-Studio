import { createCanvas, loadImage } from '@napi-rs/canvas';
import { documentSchema, drawOperation, type PaintDocument } from '../shared/document.js';

export async function renderPng(document: PaintDocument): Promise<Buffer> {
  const doc = documentSchema.parse(document);
  const ink = createCanvas(doc.width, doc.height), ctx = ink.getContext('2d');
  for (const op of doc.operations) {
    if (op.type === 'image') {
      const bytes = Buffer.from(op.data.slice('data:image/png;base64,'.length), 'base64');
      if (bytes.length < 24 || bytes.subarray(0, 8).toString('hex') !== '89504e470d0a1a0a') throw new Error('Invalid PNG');
      const w = bytes.readUInt32BE(16), h = bytes.readUInt32BE(20);
      if (w < 1 || h < 1 || w > 4096 || h > 4096 || w * h > 8_388_608) throw new Error('Embedded image exceeds canvas limits');
      const img = await loadImage(bytes); ctx.save(); if(op.transform) ctx.transform(...op.transform);
      ctx.drawImage(img, 0, 0, op.sourceSize?.width ?? doc.width, op.sourceSize?.height ?? doc.height); ctx.restore();
    } else drawOperation(ctx as unknown as CanvasRenderingContext2D, op);
  }
  const out = createCanvas(doc.width, doc.height), output = out.getContext('2d');
  if (doc.background !== 'transparent') { output.fillStyle = doc.background; output.fillRect(0, 0, doc.width, doc.height); }
  output.drawImage(ink, 0, 0); return out.toBuffer('image/png');
}
