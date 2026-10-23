import type { Operation, PaintDocument } from './document.js';
const xml=(value:string)=>value.replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&apos;'}[c]!));
function element(op:Operation,erase=false):string {
  const transform=op.transform?` transform="matrix(${op.transform.join(' ')})"`:'';
  if(op.type==='image')return `<image width="${op.sourceSize?.width}" height="${op.sourceSize?.height}" href="${op.data}"${transform}/>`;
  if(op.type==='fill')throw new Error('Rasterize the prefix through the last fill before SVG export');
  const color=erase?'black':op.color, style=`${transform} opacity="${op.opacity}" stroke-linecap="round" stroke-linejoin="round"`;
  if(op.type==='text')return `<text${style} fill="${color}" font-family="${op.fontFamily||'sans-serif'}" font-weight="${op.bold?'bold':'normal'}" font-size="${op.size}" dominant-baseline="text-before-edge">${op.text.split('\n').map((line,i)=>`<tspan x="${op.at.x}" y="${op.at.y+i*op.size*1.3}">${xml(line)}</tspan>`).join('')}</text>`;
  if('points' in op) {
    if(op.points.length===1)return `<circle cx="${op.points[0].x}" cy="${op.points[0].y}" r="${op.size/2}" fill="${color}"${style}/>`;
    let d=`M ${op.points[0].x} ${op.points[0].y}`;
    for(let i=1;i<op.points.length-1;i++){const p=op.points[i],n=op.points[i+1];d+=` Q ${p.x} ${p.y} ${(p.x+n.x)/2} ${(p.y+n.y)/2}`;}
    const end=op.points[op.points.length-1];d+=` L ${end.x} ${end.y}`;
    return `<path d="${d}" fill="none" stroke="${color}" stroke-width="${op.size}"${style}/>`;
  }
  if('from' in op) {
    const {from:a,to:b}=op,shapeStyle=`${style} fill="${op.filled&&op.type!=='line'?color:'none'}" stroke="${op.filled&&op.type!=='line'?'none':color}" stroke-width="${op.size}"`;
    if(op.type==='line')return `<line x1="${a.x}" y1="${a.y}" x2="${b.x}" y2="${b.y}"${shapeStyle}/>`;
    if(op.type==='rectangle')return `<rect x="${Math.min(a.x,b.x)}" y="${Math.min(a.y,b.y)}" width="${Math.abs(b.x-a.x)}" height="${Math.abs(b.y-a.y)}"${shapeStyle}/>`;
    return `<ellipse cx="${(a.x+b.x)/2}" cy="${(a.y+b.y)/2}" rx="${Math.abs(b.x-a.x)/2}" ry="${Math.abs(b.y-a.y)/2}"${shapeStyle}/>`;
  }
  return '';
}
export function documentSvg(doc:PaintDocument) {
  let content='',defs='';
  doc.operations.forEach((operation,i)=>{
    const op=operation.type==='image'?{...operation,sourceSize:operation.sourceSize||{width:doc.width,height:doc.height}}:operation;
    if(op.type==='eraser') {const mask=`erase-${i}`;defs+=`<mask id="${mask}" maskUnits="userSpaceOnUse" x="0" y="0" width="${doc.width}" height="${doc.height}"><rect width="100%" height="100%" fill="white"/>${element(op,true)}</mask>`;content=`<g mask="url(#${mask})">${content}</g>`;}
    else content+=element(op);
  });
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${doc.width}" height="${doc.height}" viewBox="0 0 ${doc.width} ${doc.height}"><title>${xml(doc.name)}</title><defs>${defs}</defs>${doc.background==='transparent'?'':`<rect width="100%" height="100%" fill="${doc.background}"/>`}${content}</svg>`;
}
