// Development-only browser test. The application itself is Rust/Wasm.
import { chromium } from '@playwright/test';
import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';
const browser = await chromium.launch({channel:process.env.PAINT_BROWSER_CHANNEL || undefined});
const context = await browser.newContext({viewport:{width:1440,height:940},acceptDownloads:true});
const page = await context.newPage();
const errors = [];
page.on('pageerror', e => errors.push(String(e)));
page.on('console', m => {if(m.type()==='error') errors.push(m.text());});
const base = process.env.PAINT_WEB_URL || 'http://127.0.0.1:49325/';
const wait = () => page.waitForTimeout(350);
const draft = () => page.evaluate(async () => {
 const db = await new Promise((resolve,reject) => {const r=indexedDB.open('openaec-paint',1);r.onsuccess=()=>resolve(r.result);r.onerror=()=>reject(r.error);});
 try {return await new Promise((resolve,reject)=>{const r=db.transaction('drafts').objectStore('drafts').get('current');r.onsuccess=()=>resolve(r.result);r.onerror=()=>reject(r.error);});} finally {db.close();}
});
async function expectDraft(predicate,label) {for(let i=0;i<80;i++){const d=await draft();if(predicate(d))return d;await page.waitForTimeout(100);}throw new Error(label+' failed: '+JSON.stringify(await draft()));}
try {
 await mkdir('artifacts/browser',{recursive:true});
 await page.goto(base,{timeout:120000});
 await page.locator('canvas').first().waitFor({state:'visible',timeout:120000});
 await page.waitForTimeout(2500);console.log('loaded');
 await page.mouse.move(280,370);await page.mouse.down();await page.mouse.move(420,430,{steps:12});await page.mouse.up();
 await expectDraft(d=>d?.operations?.length===1,'draw and save draft');console.log('draw saved');
 await page.keyboard.press('Control+z');await expectDraft(d=>d?.operations?.length===0,'undo');
 await page.keyboard.press('Control+y');await expectDraft(d=>d?.operations?.length===1,'redo');
 await page.keyboard.press('Escape');await page.mouse.click(600,500);await wait();assert.equal((await draft()).operations.length,1,'Escape must select');
 await page.keyboard.press('Control+a');assert.equal(await page.evaluate(()=>window.getSelection()?.toString()),'','Ctrl+A must not select page text');
 await page.mouse.move(700,500);await page.keyboard.down('Control');await page.mouse.wheel(0,-120);await page.keyboard.up('Control');await wait();
 await page.mouse.move(700,500);await page.mouse.down({button:'middle'});await page.mouse.move(730,540);await page.mouse.up({button:'middle'});await wait();
 assert.equal((await draft()).operations.length,1,'pan/zoom must not edit');
 console.log('shortcuts passed');await page.keyboard.press('t');await page.mouse.click(500,450);await wait();
 // Clicking editor also verifies that text stays editable rather than becoming page text.
 await page.mouse.click(600,440);await page.keyboard.type('Rust text');await page.keyboard.press('Control+Enter');
 await expectDraft(d=>d?.operations?.some(o=>o.type==='text'&&o.text==='Rust text'),'editable text');
 await page.screenshot({path:'artifacts/browser/editor-wide.png'});
 const before=await draft();await page.reload({timeout:120000});await page.locator('canvas').first().waitFor();await page.waitForTimeout(1800);
 assert.deepEqual(await draft(),before,'draft must survive reload');
 await page.setViewportSize({width:560,height:800});await page.waitForTimeout(800);await page.screenshot({path:'artifacts/browser/editor-narrow.png'});
 assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth),560,'no horizontal scrollbar');
 // Test open filtering in a separate empty profile so no user document is replaced.
 const openContext=await browser.newContext({viewport:{width:1440,height:940}});const openPage=await openContext.newPage();
 await openPage.goto(base,{timeout:120000});await openPage.locator('canvas').first().waitFor();await openPage.waitForTimeout(1800);
 await openPage.keyboard.press('Control+o');await openPage.locator('#rfd-input').waitFor();
 const accept=await openPage.locator('#rfd-input').getAttribute('accept');assert(accept.includes('.png')&&accept.includes('.heic')&&!accept.includes('.ifcx')&&!accept.includes('.oapaint'));
 await openPage.locator('#rfd-input').setInputFiles({name:'vector.svg',mimeType:'image/svg+xml',buffer:Buffer.from('<svg xmlns="http://www.w3.org/2000/svg" width="80" height="60"><rect width="80" height="60" fill="#0066ee"/></svg>')});
 await openPage.getByRole('button',{name:'Ok',exact:true}).click();await openPage.waitForTimeout(1800);
 await openPage.screenshot({path:'artifacts/browser/opened-image.png'});
 assert.deepEqual(errors,[],'browser must not raise errors');
 await openContext.close();console.log('Rust browser smoke: drawing, undo/redo, Escape, Ctrl+A, zoom, pan, text, draft reload, ribbon and image filter passed.');
} catch(error) {await page.screenshot({path:'artifacts/browser/failure.png',timeout:15000}).catch(()=>{});throw error;} finally {await browser.close();}
