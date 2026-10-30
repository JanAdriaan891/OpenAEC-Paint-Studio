// Development-only browser test. The application itself is Rust/Wasm.
import { chromium } from '@playwright/test';
import assert from 'node:assert/strict';
import { mkdir, readFile } from 'node:fs/promises';
const browser = await chromium.launch({channel:process.env.PAINT_BROWSER_CHANNEL || undefined});
const context = await browser.newContext({viewport:{width:1440,height:940},acceptDownloads:true});
const page = await context.newPage();
const errors = [];
page.on('pageerror', e => errors.push(String(e)));
page.on('filechooser', () => {});
page.on('console', m => {if(m.type()==='error') errors.push(m.text());});
const base = process.env.PAINT_WEB_URL || 'http://127.0.0.1:49325/';
const wait = () => page.waitForTimeout(350);
const draft = (target=page) => target.evaluate(async () => {
 const db = await new Promise((resolve,reject) => {const r=indexedDB.open('openaec-paint',1);r.onsuccess=()=>resolve(r.result);r.onerror=()=>reject(r.error);});
 try {return await new Promise((resolve,reject)=>{const r=db.transaction('drafts').objectStore('drafts').get('current');r.onsuccess=()=>resolve(r.result);r.onerror=()=>reject(r.error);});} finally {db.close();}
});
async function expectDraft(predicate,label) {for(let i=0;i<80;i++){const d=await draft();if(predicate(d))return d;await page.waitForTimeout(100);}throw new Error(label+' failed: '+JSON.stringify(await draft()));}
try {
 await mkdir('artifacts/browser',{recursive:true});
 await page.goto(base,{timeout:120000});
 await page.locator('canvas').first().waitFor({state:'visible',timeout:120000});
 await page.waitForTimeout(2500);console.log('loaded');
 await page.mouse.move(280,370);await page.mouse.down();await page.mouse.move(420,430,{steps:4});await page.mouse.up();
 await expectDraft(d=>d?.operations?.length===1,'draw and save draft');console.log('draw saved');
 await page.keyboard.press('Control+z');await expectDraft(d=>d?.operations?.length===0,'undo');
 await page.keyboard.press('Control+y');await expectDraft(d=>d?.operations?.length===1,'redo');
 await page.keyboard.press('Escape');await page.mouse.click(600,500);await wait();assert.equal((await draft()).operations.length,1,'Escape must select');
 await page.keyboard.press('Control+a');assert.equal(await page.evaluate(()=>window.getSelection()?.toString()),'','Ctrl+A must not select page text');
 await page.mouse.move(700,500);await page.keyboard.down('Control');await page.mouse.wheel(0,-120);await page.keyboard.up('Control');await wait();
 await page.mouse.move(700,500);await page.mouse.down({button:'middle'});await page.mouse.move(730,540);await page.mouse.up({button:'middle'});await wait();
 assert.equal((await draft()).operations.length,1,'pan/zoom must not edit');
 console.log('shortcuts passed');await page.keyboard.press('t');await page.mouse.click(500,450);await wait();
 // The Rust text editor receives focus automatically after the canvas click.
 await page.keyboard.type('Rust text');await page.keyboard.press('Control+Enter');
 await expectDraft(d=>d?.operations?.some(o=>o.type==='text'&&o.text==='Rust text'),'editable text');
 await page.screenshot({path:'artifacts/browser/editor-wide.png'});
 await page.keyboard.press('Control+o');const keyboardInput=page.locator('#rfd-input');await keyboardInput.waitFor({state:'attached'});
 const keyboardFilter=await keyboardInput.getAttribute('accept');assert(keyboardFilter.includes('.png')&&!keyboardFilter.includes('.ifcx')&&!keyboardFilter.includes('.oapaint'));
 await keyboardInput.setInputFiles([]);await wait();
 const before=await draft();await page.reload({timeout:120000});await page.locator('canvas').first().waitFor();await page.waitForTimeout(1800);
 assert.deepEqual(await draft(),before,'draft must survive reload');
 await page.setViewportSize({width:560,height:800});await page.waitForTimeout(800);await page.screenshot({path:'artifacts/browser/editor-narrow.png'});
 assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth),560,'no horizontal scrollbar');
 // Test open filtering in a separate empty profile so no user document is replaced.
 const openContext=await browser.newContext({viewport:{width:1440,height:940}});const openPage=await openContext.newPage();openPage.on('filechooser',()=>{});openPage.on('pageerror',e=>errors.push(String(e)));
 await openPage.goto(base,{timeout:120000});await openPage.locator('canvas').first().waitFor();await openPage.waitForTimeout(1800);
 await openPage.locator('canvas').focus();await openPage.mouse.move(55,92);await openPage.waitForTimeout(200);
 await openPage.mouse.click(55,92,{delay:100});const input=openPage.locator('#rfd-input');await input.waitFor({state:'attached'});
 const accept=await input.getAttribute('accept');assert(accept.includes('.png')&&accept.includes('.heic')&&!accept.includes('.ifcx')&&!accept.includes('.oapaint'));
 await input.setInputFiles({name:'vector.svg',mimeType:'image/svg+xml',buffer:Buffer.from('<svg xmlns="http://www.w3.org/2000/svg" width="80" height="60"><rect width="80" height="60" fill="#0066ee"/></svg>')});
 for(let i=0;i<80;i++){const d=await draft(openPage);if(d?.name==='vector')break;await openPage.waitForTimeout(100);}
 const imported=await draft(openPage);assert.equal(imported?.name,'vector');assert.equal(imported.width,80);assert.equal(imported.height,60);
 await openPage.screenshot({path:'artifacts/browser/opened-image.png'});
 for(const [extension,file] of [['tiff','blue.tiff'],['psd','blue.psd'],['heic','blue.heic'],['avif','blue.avif'],['jxl','blue.jxl'],['jp2','blue.jp2'],['dng','sensor.dng']]) {
   await openPage.mouse.move(55,92);await openPage.mouse.click(55,92,{delay:100});const selected=openPage.locator('#rfd-input');await selected.waitFor({state:'attached'});
   await selected.setInputFiles({name:`format-${extension}.${extension}`,mimeType:'application/octet-stream',buffer:await readFile(`crates/paint-core/tests/fixtures/${file}`)});
   for(let i=0;i<100;i++){const d=await draft(openPage);if(d?.name===`format-${extension}`)break;await openPage.waitForTimeout(100);}
   const d=await draft(openPage);assert.equal(d?.name,`format-${extension}`);assert.equal(d.width,32);assert.equal(d.height,24);
   console.log(`browser import ${extension} passed`);
 }

 assert.deepEqual(errors,[],'browser must not raise errors');
 await openContext.close();console.log('Rust browser smoke: drawing, undo/redo, Escape, Ctrl+A, zoom, pan, text, draft reload, ribbon and image filter passed.');
} catch(error) {let i=0;for(const c of browser.contexts())for(const p of c.pages())await p.screenshot({path:`artifacts/browser/failure-${i++}.png`,timeout:15000}).catch(()=>{});throw error;} finally {await browser.close();}
