import { encode, decode } from 'cborg';
import { z } from 'zod';
import { packageVersion } from './version.js';

export const rasterAttribute = 'openaec::paint::rasterImage';
const maxBytes = 64_000_000;
const rasterSchema = z.object({
  version: z.literal(1), name: z.string().min(1).max(120),
  width: z.number().int().min(1).max(4096), height: z.number().int().min(1).max(4096),
  mimeType: z.literal('image/png'), uri: z.string().startsWith('data:image/png;base64,').max(48_000_000),
}).refine(image => image.width * image.height <= 8_388_608, 'Image exceeds 8 megapixels');
const containerSchema = z.object({
  header: z.object({ ifcxVersion: z.literal('ifcx_alpha') }),
  data: z.array(z.object({ attributes: z.record(z.string(), z.unknown()).optional() })).max(20000),
});

function pngBytes(uri: string, width: number, height: number) {
  const binary = atob(uri.slice('data:image/png;base64,'.length));
  const bytes = Uint8Array.from(binary, character => character.charCodeAt(0));
  const signature = [137,80,78,71,13,10,26,10];
  if (bytes.length < 33 || signature.some((value,index) => bytes[index] !== value) ||
      String.fromCharCode(...bytes.slice(12,16)) !== 'IHDR') throw new Error('Invalid embedded PNG');
  const view = new DataView(bytes.buffer);
  if (view.getUint32(16) !== width || view.getUint32(20) !== height) throw new Error('Embedded PNG dimensions do not match IFCX metadata');
  return bytes;
}

export function rasterIfcx(name: string, width: number, height: number, uri: string) {
  const image = rasterSchema.parse({version:1,name,width,height,mimeType:'image/png',uri});
  pngBytes(uri,width,height);
  return {
    header: { id: crypto.randomUUID(), ifcxVersion:'ifcx_alpha', dataVersion:'1.0.0', author:`OpenAEC Paint Studio ${packageVersion}`, timestamp:new Date().toISOString() },
    imports: [],
    schemas: { [rasterAttribute]: { value: { dataType:'Object' } } },
    data: [{ path:'/paint/image', attributes: { [rasterAttribute]:image } }],
  };
}

async function boundedBytes(stream: ReadableStream<Uint8Array>) {
  const reader = stream.getReader(), chunks: Uint8Array[] = [];
  let length = 0;
  try {
    while (true) {
      const {done,value} = await reader.read(); if (done) break;
      length += value.length;
      if (length > maxBytes) { await reader.cancel(); throw new Error('IFCX exceeds 64 MB'); }
      chunks.push(value);
    }
  } finally { reader.releaseLock(); }
  const bytes = new Uint8Array(length); let offset = 0;
  for (const chunk of chunks) { bytes.set(chunk,offset); offset += chunk.length; }
  return bytes;
}

export async function encodeRasterIfcx(document: ReturnType<typeof rasterIfcx>, binary = false) {
  if (!binary) return new Blob([JSON.stringify(document,null,2)], {type:'application/json'});
  const body = encode(document), bytes = new Uint8Array(body.length+3);
  bytes.set([0xd9,0xd9,0xf7]); bytes.set(body,3); // RFC 8949 self-described CBOR.
  if (bytes.length > maxBytes) throw new Error('IFCX exceeds 64 MB');
  const compressed = await boundedBytes(new Blob([bytes]).stream().pipeThrough(new CompressionStream('gzip')));
  return new Blob([compressed as BlobPart],{type:'application/octet-stream'});
}

export async function readRasterIfcx(input: Uint8Array) {
  if (input.length > maxBytes) throw new Error('IFCX exceeds 64 MB');
  let bytes = input;
  if (bytes[0]===0x1f && bytes[1]===0x8b) bytes = await boundedBytes(new Blob([bytes as BlobPart]).stream().pipeThrough(new DecompressionStream('gzip')));
  const value: unknown = bytes[0]===0xd9 && bytes[1]===0xd9 && bytes[2]===0xf7
    ? decode(bytes.subarray(3), {rejectDuplicateMapKeys:true})
    : JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(bytes));
  const container = containerSchema.parse(value);
  const matches = container.data.filter(node=>node.attributes?.[rasterAttribute]!==undefined);
  if (matches.length!==1) throw new Error('This IFCX file must contain exactly one Paint Studio raster image');
  const image = rasterSchema.parse(matches[0].attributes![rasterAttribute]);
  return {...image,bytes:pngBytes(image.uri,image.width,image.height)};
}
