import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createCanvas,loadImage } from '@napi-rs/canvas';
import { newDocument,type Operation } from '../shared/document.js';
import { transformedOperation,operationBounds } from '../shared/geometry.js';
import { documentSvg } from '../shared/svg.js';
import { renderPng } from '../services/renderer.js';
import { adjustPixels } from '../shared/adjustments.js';
test('vector objects survive rotation and remain vector objects in SVG',async()=>{
  const op:Operation={type:'rectangle',from:{x:10,y:10},to:{x:40,y:30},color:'#d97706',size:2,opacity:1,filled:true};
  const rotated=transformedOperation(op,[0,1,-1,0,64,0],{width:64,height:48});
  assert.equal(rotated.type,'rectangle');assert.deepEqual(operationBounds(rotated),{x:34,y:10,width:20,height:30});
  const doc={...newDocument(64,64,'<Sketch>'),operations:[rotated]};
  const svg=documentSvg(doc);assert(svg.includes('<rect x="10"'));assert(svg.includes('matrix(0 1 -1 0 64 0)'));assert(svg.includes('&lt;Sketch&gt;'));assert(!svg.includes('<image'));
  const canvas=createCanvas(64,64),ctx=canvas.getContext('2d');ctx.drawImage(await loadImage(await renderPng(doc)),0,0);assert.deepEqual(Array.from(ctx.getImageData(40,20,1,1).data),[217,119,6,255]);
});
test('color corrections preserve alpha and invert colors',()=>{
  const pixels=new Uint8ClampedArray([10,20,30,120]);adjustPixels(pixels,{brightness:0,contrast:0,saturation:100},'invert');assert.deepEqual(Array.from(pixels),[245,235,225,120]);
});
