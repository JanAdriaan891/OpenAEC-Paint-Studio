import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { ImageMagick, MagickColors, MagickFormat } from '@imagemagick/magick-wasm';
import { initializeDecoder,decodeImage,decoderFormats } from '../shared/image-decoder.js';
test('local decoder imports common raster and layered formats',async()=>{
  await initializeDecoder(readFileSync(new URL(import.meta.resolve('@imagemagick/magick-wasm/magick.wasm'))));
  for(const format of [MagickFormat.Png,MagickFormat.Jpeg,MagickFormat.WebP,MagickFormat.Bmp,MagickFormat.Tiff,MagickFormat.Gif,MagickFormat.Psd,MagickFormat.Ico,MagickFormat.Tga,MagickFormat.Avif,MagickFormat.Jp2]){
    const bytes=ImageMagick.read(MagickColors.Red,64,48,image=>image.write(format,data=>data.slice()));
    const decoded=decodeImage(bytes,format);assert.equal(decoded.width,64,format);assert.equal(decoded.height,48,format);assert.equal(Buffer.from(decoded.bytes).subarray(0,8).toString('hex'),'89504e470d0a1a0a');
  }
  assert(decoderFormats().some(f=>f.format==='HEIC'));assert(decoderFormats().some(f=>f.format==='DNG'));
  assert.throws(()=>decodeImage(new Uint8Array([1,2,3,4])));
});
