import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createCanvas } from '@napi-rs/canvas';
import { gunzipSync } from 'node:zlib';
import { rasterIfcx, rasterAttribute, encodeRasterIfcx, readRasterIfcx } from '../shared/ifcx.js';

test('IFCX and IFCXB embed and restore the exact raster PNG including alpha', async () => {
  const image=createCanvas(2,2);image.getContext('2d').fillRect(0,0,1,1);
  const png=image.toBuffer('image/png');
  const document=rasterIfcx('Raster ✓',2,2,`data:image/png;base64,${png.toString('base64')}`);
  assert.equal(document.schemas[rasterAttribute].value.dataType,'Object');
  for(const binary of [false,true]){
    const blob=await encodeRasterIfcx(document,binary),bytes=new Uint8Array(await blob.arrayBuffer());
    if(binary){assert.equal(bytes[0],0x1f);assert.deepEqual([...gunzipSync(bytes).subarray(0,3)],[0xd9,0xd9,0xf7]);}
    const decoded=await readRasterIfcx(bytes);
    assert.equal(decoded.name,'Raster ✓');assert.equal(decoded.width,2);assert.equal(decoded.height,2);
    assert.deepEqual(Buffer.from(decoded.bytes),png);
  }
});

test('IFCX rejects missing rasters and PNG dimension mismatches',async()=>{
  const png=createCanvas(2,2).toBuffer('image/png');
  assert.throws(()=>rasterIfcx('Wrong size',3,2,`data:image/png;base64,${png.toString('base64')}`),/dimensions/);
  await assert.rejects(readRasterIfcx(new TextEncoder().encode(JSON.stringify({header:{ifcxVersion:'ifcx_alpha'},data:[]}))),/exactly one/);
  await assert.rejects(readRasterIfcx(new Uint8Array([0x1f,0x8b,0,1])),/./);
});
