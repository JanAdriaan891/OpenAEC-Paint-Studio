import type { Operation, Point } from './document.js';
export type Matrix = [number, number, number, number, number, number];
export const identity: Matrix = [1, 0, 0, 1, 0, 0];
export function compose(a: Matrix, b: Matrix = identity): Matrix {
  return [a[0]*b[0]+a[2]*b[1], a[1]*b[0]+a[3]*b[1], a[0]*b[2]+a[2]*b[3], a[1]*b[2]+a[3]*b[3], a[0]*b[4]+a[2]*b[5]+a[4], a[1]*b[4]+a[3]*b[5]+a[5]];
}
export function transformPoint(p: Point, m: Matrix = identity): Point { return { x: m[0]*p.x+m[2]*p.y+m[4], y: m[1]*p.x+m[3]*p.y+m[5] }; }
export type Bounds = { x: number; y: number; width: number; height: number };
export function operationBounds(op: Operation): Bounds | null {
  let points: Point[];
  if (op.type === 'fill' || op.type === 'eraser') return null;
  if ('points' in op) points = op.points;
  else if ('from' in op) points = [op.from, {x:op.from.x,y:op.to.y}, op.to, {x:op.to.x,y:op.from.y}];
  else if (op.type === 'text') {
    const width = Math.max(...op.text.split('\n').map(line=>line.length))*op.size*.65, height = op.text.split('\n').length*op.size*1.3;
    points = [op.at,{x:op.at.x+width,y:op.at.y},{x:op.at.x,y:op.at.y+height},{x:op.at.x+width,y:op.at.y+height}];
  } else { const {width,height}=op.sourceSize || {width:1200,height:800}; points=[{x:0,y:0},{x:width,y:0},{x:0,y:height},{x:width,y:height}]; }
  const transformed=points.map(p=>transformPoint(p,op.transform)), xs=transformed.map(p=>p.x), ys=transformed.map(p=>p.y);
  const x=Math.min(...xs),y=Math.min(...ys);
  return {x,y,width:Math.max(1,Math.max(...xs)-x),height:Math.max(1,Math.max(...ys)-y)};
}
export function transformedOperation(op: Operation, matrix: Matrix, oldSize: {width:number;height:number}): Operation {
  return { ...op, ...(op.type==='image' ? {sourceSize:op.sourceSize || oldSize} : {}), transform: compose(matrix, op.transform) };
}
