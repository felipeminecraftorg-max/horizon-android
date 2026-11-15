'use strict';
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),http=require('node:http');
const {chromium}=require(process.env.JADE_PLAYWRIGHT_MODULE||'playwright');
const root=path.resolve(__dirname,'../android/src/main/assets'),baseline=process.env.JADE_BASELINE_DIR;
const out=path.resolve('dist/deck-qa');fs.mkdirSync(out,{recursive:true});
const server=http.createServer((req,res)=>{let name=decodeURIComponent(req.url.split('?')[0]);const base=name.startsWith('/baseline/')?baseline:root;if(name.startsWith('/baseline/'))name=name.slice(9);if(name==='/'||name==='')name='/index.html';const file=path.resolve(base,'.'+(name.startsWith('/')?name:'/'+name));if(!file.startsWith(base+path.sep)){res.writeHead(403).end();return;}fs.readFile(file,(e,data)=>{if(e){res.writeHead(404).end();return;}res.setHeader('Content-Type',file.endsWith('.js')?'text/javascript':file.endsWith('.css')?'text/css':'text/html');res.end(data);});});
(async()=>{
 await new Promise(r=>server.listen(0,'127.0.0.1',r));
 const browser=await chromium.launch({executablePath:'/usr/bin/google-chrome',headless:true,args:['--no-sandbox','--use-angle=swiftshader','--enable-unsafe-swiftshader']});
 const errors=[],page=await browser.newPage(),original=await browser.newPage();page.on('pageerror',e=>errors.push(e.message));
 const url='http://127.0.0.1:'+server.address().port,report=[];
 try{
  await original.goto(url+'/baseline/');await page.goto(url);
  await page.waitForFunction(()=>window.JadeDeckFinish?.state().ready||window.JadeDeckFinish?.state().error);
  const status=await page.evaluate(()=>JadeDeckFinish.state());assert.equal(status.ready,true,status.error);
  for(const p of [original,page])await p.addStyleTag({content:'.avatar-figure{animation:none!important}.tile,.hero{transition:none!important}'});
  const selectors=['.topbar','.hero','.cards','.avatar-stage','.scene-navigation','.tile[data-stack="front"]','.tile[data-stack="middle"]','.tile[data-stack="rear"]'];
  for(const [width,height] of [[1280,720],[393,852],[852,393],[360,640]]){
   await original.setViewportSize({width,height});await page.setViewportSize({width,height});
   await page.waitForTimeout(900);
   const geometry=[];
   for(const selector of selectors){
    const a=await original.locator(selector).boundingBox(),b=await page.locator(selector).boundingBox();assert.ok(a&&b,selector);
    for(const k of ['x','y','width','height'])assert.ok(Math.abs(a[k]-b[k])<1.1,`${width}x${height} ${selector} ${k}: ${a[k]} != ${b[k]}`);
    geometry.push({selector,original:a,improved:b});
   }
   const coat=await page.evaluate(()=>JadeDeckFinish.state());assert.equal(coat.cards.length,3);
   assert.deepEqual(coat.cards.map(c=>c.stack),['rear','middle','front']);
   assert.equal(await page.locator('#console-categories').count(),0,'rejected navigation returned');
   const front=page.locator('.tile[data-stack="front"]'),r=await front.boundingBox();
   const hit=await page.evaluate(({x,y})=>document.elementFromPoint(x,y)?.closest('.tile')?.dataset.stack,{x:r.x+r.width*.5,y:r.y+r.height*.5});assert.equal(hit,'front','GPU finish intercepted input');
   await original.screenshot({path:path.join(out,`original-${width}x${height}.png`),animations:'disabled'});await page.screenshot({path:path.join(out,`improved-${width}x${height}.png`),animations:'disabled'});
   report.push({viewport:[width,height],geometry});
  }
  await page.setViewportSize({width:1280,height:720});await page.waitForTimeout(800);
  const front=await page.locator('.tile[data-stack="front"]').boundingBox();
  await page.mouse.move(front.x+front.width*.6,front.y+front.height*.5);await page.mouse.down();await page.mouse.move(front.x+front.width*.6-90,front.y+front.height*.5,{steps:12});await page.mouse.up();
  await page.waitForTimeout(200);
  assert.equal(await page.locator('.tile[data-stack="front"]').getAttribute('data-page'),'Mods','swipe was swallowed by a button click');
  await page.keyboard.press('ArrowDown');assert.equal(await page.locator('.tile[data-stack="front"]').getAttribute('data-page'),'Packs');
  await page.keyboard.press('ArrowUp');assert.equal(await page.locator('.tile[data-stack="front"]').getAttribute('data-page'),'Mods');
  await page.keyboard.press('Escape');assert.equal(await page.locator('.tile[data-stack="front"]').getAttribute('data-page'),'Play');
  await page.locator('#open-editor').click();await page.getByRole('tab',{name:'◈ Materiais',exact:true}).click();
  await page.locator('#hide-editor').click();await page.waitForTimeout(800);
  // Restore animation CSS for the actual shared pause test.
  await page.evaluate(()=>document.querySelectorAll('style').forEach(s=>s.remove()));
  await page.locator('#toggle-pause').click();await page.waitForTimeout(100);
  const t=await page.evaluate(()=>({time:JadeRenderer.state().elapsed,avatar:getComputedStyle(document.querySelector('.avatar-figure')).transform}));
  const still=await page.evaluate(()=>JadeDeckFinish.screenshot());
  await page.waitForTimeout(350);assert.deepEqual(await page.evaluate(()=>({time:JadeRenderer.state().elapsed,avatar:getComputedStyle(document.querySelector('.avatar-figure')).transform})),t);
  const after=await page.evaluate(()=>JadeDeckFinish.screenshot());
  const delta=await page.evaluate(async([a,b])=>{
    const images=await Promise.all([a,b].map(src=>new Promise(resolve=>{const i=new Image();i.onload=()=>resolve(i);i.src=src;})));
    const c=document.createElement('canvas');c.width=images[0].width;c.height=images[0].height;const ctx=c.getContext('2d');const pixels=images.map(i=>{ctx.clearRect(0,0,c.width,c.height);ctx.drawImage(i,0,0);return ctx.getImageData(0,0,c.width,c.height).data;});
    let sum=0;for(let i=0;i<pixels[0].length;i++)sum+=Math.abs(pixels[0][i]-pixels[1][i]);return sum/pixels[0].length;
  },[still,after]);
  assert.ok(delta<.5,'paused finish pixel drift: '+delta);
  await page.mouse.move(200,100);await page.waitForFunction(()=>JadeDeckFinish.state().pointer[0]===200);assert.notEqual(await page.evaluate(()=>JadeDeckFinish.screenshot()),still,'card lighting did not respond');
  const before=await page.evaluate(()=>JadeLab.getPreset().controls);
  const download=page.waitForEvent('download');await page.locator('#open-editor').click();await page.locator('#download-pack').click();const file=await download,zip=fs.readFileSync(await file.path());assert.ok(zip.includes(Buffer.from('shaders/deck-coat.frag')));assert.ok(zip.includes(Buffer.from('uNormal')));assert.deepEqual(await page.evaluate(()=>JadeLab.getPreset().controls),before);
  assert.deepEqual(errors,[]);
  fs.writeFileSync(path.join(out,'RESULT.json'),JSON.stringify({passed:true,composition:'v1.4 positions, sizes, stacking and perspective preserved',checks:['4 viewport baseline comparisons','three-card perspective deck','card swipe without synthetic click','Up/Down navigation','live GPU card lighting','shared pause including avatar','shader pack export'],viewports:report},null,2));
  console.log('Deck preservation and behavior checks passed');
 }catch(e){await page.screenshot({path:path.join(out,'failure.png'),animations:'disabled'});console.error('errors',errors);console.error('finish',await page.evaluate(()=>window.JadeDeckFinish?.state()));throw e;}
 finally{await browser.close();server.close();}
})().catch(e=>{console.error(e);server.close();process.exitCode=1;});
