// Real Rust/Wasm interactions, always in isolated headless profiles.
import {chromium} from '@playwright/test';
import assert from 'node:assert/strict';
import {readFile, mkdir} from 'node:fs/promises';
import {deflateSync} from 'node:zlib';
const browser=await chromium.launch();
const context=await browser.newContext({viewport:{width:1440,height:940},acceptDownloads:true,permissions:['clipboard-read','clipboard-write']});
const page=await context.newPage();page.on('filechooser',()=>{});
const errors=[];page.on('pageerror',e=>errors.push(String(e)));
const base=process.env.PAINT_WEB_URL||'http://127.0.0.1:49325/';
const wait=()=>page.waitForTimeout(400);
const draft=()=>page.evaluate(async()=>{const db=await new Promise((ok,no)=>{const r=indexedDB.open('openaec-paint',1);r.onsuccess=()=>ok(r.result);r.onerror=()=>no(r.error);});try{return await new Promise((ok,no)=>{const r=db.transaction('drafts').objectStore('drafts').get('current');r.onsuccess=()=>ok(r.result);r.onerror=()=>no(r.error);});}finally{db.close();}});
async function expectDraft(predicate,label){for(let i=0;i<120;i++){const d=await draft();if(predicate(d))return d;await page.waitForTimeout(100);}throw new Error(label+': '+JSON.stringify(await draft()));}
async function choose(keys,name,mimeType,buffer){await page.locator('canvas').focus();await page.keyboard.press(keys);const input=page.locator('#rfd-input');await input.waitFor({state:'attached'});const accept=await input.getAttribute('accept');await input.setInputFiles({name,mimeType,buffer});return accept;}
// Independent PNG fixture writer, avoiding reliance on the application's codec.
function png(w,h){let table=Array.from({length:256},(_,n)=>{let c=n;for(let k=0;k<8;k++)c=(c&1)?0xedb88320^(c>>>1):c>>>1;return c>>>0;});const chunk=(tag,data)=>{let type=Buffer.from(tag),size=Buffer.alloc(4),crc=0xffffffff;size.writeUInt32BE(data.length);for(const b of Buffer.concat([type,data]))crc=table[(crc^b)&255]^(crc>>>8);const end=Buffer.alloc(4);end.writeUInt32BE((crc^0xffffffff)>>>0);return Buffer.concat([size,type,data,end]);};let header=Buffer.alloc(13);header.writeUInt32BE(w);header.writeUInt32BE(h,4);header[8]=8;header[9]=6;let row=Buffer.alloc(w*4+1);for(let i=1;i<row.length;i+=4){row[i]=20;row[i+1]=90;row[i+2]=230;row[i+3]=128;}return Buffer.concat([Buffer.from([137,80,78,71,13,10,26,10]),chunk('IHDR',header),chunk('IDAT',deflateSync(Buffer.concat(Array.from({length:h},()=>row)))),chunk('IEND',Buffer.alloc(0))]);}
try{
 await mkdir('artifacts/browser',{recursive:true});
 await page.goto(base,{timeout:120000});await page.locator('canvas').first().waitFor({timeout:120000});await page.waitForTimeout(2000);
 const source={format:'openaec-paint',version:1,name:'Workflow',width:1200,height:800,background:'transparent',retainVectors:true,operations:[
  {type:'text',at:{x:80,y:60},text:'First\nSecond',size:24,color:'#36363e',opacity:0.7,bold:true,fontFamily:'sans-serif',transform:[1,0.1,0,1,8,4]},
  {type:'rectangle',from:{x:30,y:160},to:{x:230,y:220},size:4,color:'#d97706',filled:true,opacity:1},
  {type:'ellipse',from:{x:300,y:180},to:{x:450,y:280},size:3,color:'#2563eb',opacity:0.5}
 ]};
 const filter=await choose('Control+Shift+o','Workflow.paint.json','application/json',Buffer.from(JSON.stringify({format:'openaec-paint-project',version:1,document:source})));
 assert(filter.includes('.json')&&!filter.includes('.oapaint'));
 const baseline=await expectDraft(d=>d?.name==='Workflow'&&d.operations.length===3,'open editable project');
 const downloadPromise=page.waitForEvent('download');await page.keyboard.press('Control+s');await page.locator('#paint-download-link').click();const download=await downloadPromise;
 assert.equal(download.suggestedFilename(),'Workflow.paint.json');const saved=JSON.parse(await readFile(await download.path(),'utf8'));
 assert.equal(saved.format,'openaec-paint-project');assert.equal(saved.document.operations[0].text,'First\nSecond');
 assert.equal(saved.document.operations[0].opacity,0.7);assert.deepEqual(saved.document.operations[0].transform,[1,0.1,0,1,8,4]);
 await choose('Control+Shift+o','Workflow.paint.json','application/json',Buffer.from(JSON.stringify(saved)));
 const restored=await expectDraft(d=>d?.name==='Workflow','reopen project');assert.deepEqual(restored,baseline);
 console.log('project download and reopen passed');
 await page.keyboard.press('b');await page.mouse.click(600,640);await expectDraft(d=>d?.operations.length===4,'dirty edit');
 const dirtyBeforeCancel=await draft();await page.keyboard.press('Control+s');await page.locator('#paint-download-cancel').click();await wait();
 await choose('Control+Shift+o','Workflow.paint.json','application/json',Buffer.from(JSON.stringify(saved)));await wait();
 assert.deepEqual(await draft(),dirtyBeforeCancel,'cancel download must keep dirty flag and require replacement confirmation');await page.keyboard.press('Escape');await page.keyboard.press('Control+z');await expectDraft(d=>d?.operations.length===3,'undo dirty test');
 console.log('download cancellation preserves unsaved work');
 const clipboardPng=png(64,48);await page.evaluate(async data=>{const blob=await(await fetch('data:image/png;base64,'+data)).blob();await navigator.clipboard.write([new ClipboardItem({'image/png':blob})]);},clipboardPng.toString('base64'));
 await page.locator('canvas').focus();await page.keyboard.press('Control+v');
 await expectDraft(d=>d?.operations?.length===4&&d.operations[3].type==='image','paste image');
 await page.keyboard.press('Control+z');await expectDraft(d=>d?.operations?.length===3,'undo paste');
 await page.keyboard.press('Control+c');await wait();
 const copied=await page.evaluate(async()=>{const items=await navigator.clipboard.read();const blob=await items[0].getType('image/png');const b=await createImageBitmap(blob);const c=document.createElement('canvas');c.width=b.width;c.height=b.height;const ctx=c.getContext('2d');ctx.drawImage(b,0,0);const alpha=ctx.getImageData(1100,700,1,1).data[3];return {width:b.width,height:b.height,alpha};});
 assert.deepEqual(copied,{width:1200,height:800,alpha:0});console.log('real browser clipboard PNG paste/copy and alpha passed');
 await page.evaluate(()=>navigator.clipboard.writeText('plain text, no image'));
 const beforeEmpty=await draft();await page.keyboard.press('Control+v');await wait();assert.deepEqual(await draft(),beforeEmpty,'unsupported clipboard cannot change document');
 await page.keyboard.press('t');await page.mouse.click(600,640);await wait();await page.keyboard.type('Old');await page.keyboard.press('Control+a');await page.keyboard.type('Replacement text');for(const modifier of ['Control','Meta']) {await page.keyboard.press(modifier+'+a');await page.keyboard.press(modifier+'+c');await wait();assert.equal(await page.evaluate(()=>navigator.clipboard.readText()),'Replacement text');await page.keyboard.type('Wrong');await page.keyboard.press(modifier+'+a');await page.keyboard.press(modifier+'+v');await wait();}await page.keyboard.press('Control+Enter');
 await expectDraft(d=>d?.operations.some(o=>o.type==='text'&&o.text==='Replacement text'),'normal text copy/paste with Ctrl and Cmd');console.log('real text clipboard with Ctrl and Cmd passed');
 const beforeLatePaste=await draft();
 await page.evaluate(()=>{window.paintOriginalReadText=navigator.clipboard.readText.bind(navigator.clipboard);navigator.clipboard.readText=()=>new Promise(resolve=>setTimeout(()=>resolve('Late text'),300));});
 await page.keyboard.press('t');await page.mouse.click(600,700);await wait();await page.keyboard.type('Uncommitted');await page.keyboard.press('Control+a');await page.keyboard.press('Control+v');await page.keyboard.press('Escape');await page.waitForTimeout(700);
 assert.deepEqual(await draft(),beforeLatePaste,'late clipboard read cannot paste after text editor closes');
 await page.evaluate(()=>{navigator.clipboard.readText=window.paintOriginalReadText;delete window.paintOriginalReadText;});console.log('late text paste cancellation passed');
 await page.keyboard.press('F6');await wait();await page.screenshot({path:'artifacts/browser/objects-dialog.png'});await page.keyboard.press('Escape');
 const beforePixel=await draft();await page.keyboard.press('p');await wait();
 for(let i=0;i<2;i++){await page.keyboard.press('Tab');await page.waitForTimeout(150);}await page.keyboard.press('Enter');await wait();
 await page.mouse.move(175,385);await page.mouse.down();await page.mouse.move(235,420,{steps:3});await page.mouse.up();await wait();assert.deepEqual(await draft(),beforePixel,'selecting pixels must not edit');
 await page.mouse.move(190,395);await page.mouse.down();await page.mouse.move(250,430,{steps:3});await page.mouse.up();
 await expectDraft(d=>d?.operations.length===1&&d.operations[0].type==='image','pixel move composition');
 await page.keyboard.press('Control+z');await expectDraft(d=>d?.operations.length===beforePixel.operations.length,'undo pixel move');assert.deepEqual(await draft(),beforePixel);await page.keyboard.press('Escape');console.log('pixel mode keyboard access, move and vector undo passed');
 const beforeLarge=await draft();const imageFilter=await choose('Control+o','Large.png','image/png',png(6000,4000));
 assert(imageFilter.includes('.png')&&!imageFilter.includes('.json'));
 await page.waitForTimeout(3500);assert.deepEqual(await draft(),beforeLarge,'resized import cannot replace before confirmation');await page.screenshot({path:'artifacts/browser/import-confirmation.png'});
 await page.keyboard.press('Escape');await wait();assert.deepEqual(await draft(),beforeLarge,'cancel large import preserves draft');console.log('large import cancellation passed');
 await page.reload({timeout:120000});await page.locator('canvas').first().waitFor();await page.waitForTimeout(1800);assert.deepEqual(await draft(),beforeLarge,'new project/draft survives reload');
 await page.setViewportSize({width:560,height:800});await wait();await page.keyboard.press('F6');await wait();await page.screenshot({path:'artifacts/browser/objects-dialog-narrow.png'});assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth),560);await page.keyboard.press('Escape');
 assert.deepEqual(errors,[]);
 const registrations=await page.evaluate(()=>navigator.serviceWorker?.getRegistrations().then(r=>r.length)??0);assert.equal(registrations,0,'no offline PWA is claimed');
 await context.setOffline(true);let warmOffline=false;try{await page.reload({timeout:5000});await page.locator('canvas').waitFor({timeout:3000});warmOffline=true;}catch{}
 const cold=await browser.newContext({offline:true});const coldPage=await cold.newPage();let coldOffline=false;try{await coldPage.goto(base,{timeout:5000});await coldPage.locator('canvas').waitFor({timeout:3000});coldOffline=true;}catch{}await cold.close();
 console.log('browser offline observations:',JSON.stringify({serviceWorkers:registrations,warmReload: warmOffline,coldStart:coldOffline}));
 console.log('browser version:',browser.version());console.log('new Rust workflows passed');
}catch(e){await page.screenshot({path:'artifacts/browser/workflow-failure.png'}).catch(()=>{});throw e;}finally{await browser.close();}
