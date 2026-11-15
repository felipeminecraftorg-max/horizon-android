/* Real pointer/touch, GPU and responsive checks for the spatial laboratory. */
'use strict';
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),http=require('node:http');
const {chromium}=require(process.env.JADE_PLAYWRIGHT_MODULE||'playwright');
const root=path.resolve(__dirname,'../android/src/main/assets'),out=path.resolve('dist/spatial-qa');fs.mkdirSync(out,{recursive:true});
const server=http.createServer((req,res)=>{let name=decodeURIComponent(req.url.split('?')[0]);if(name==='/')name='/index.html';const file=path.resolve(root,'.'+name);if(!file.startsWith(root+path.sep)){res.writeHead(403).end();return;}fs.readFile(file,(e,data)=>{if(e){res.writeHead(404).end();return;}res.setHeader('Content-Type',file.endsWith('.js')?'text/javascript':file.endsWith('.css')?'text/css':'text/html');res.end(data);});});
(async()=>{
 await new Promise(r=>server.listen(0,'127.0.0.1',r));
 const browser=await chromium.launch({executablePath:'/usr/bin/google-chrome',headless:true,args:['--no-sandbox','--use-angle=swiftshader','--enable-unsafe-swiftshader']});
 const context=await browser.newContext({viewport:{width:1280,height:720},deviceScaleFactor:1,hasTouch:true});
 const page=await context.newPage();page.setDefaultTimeout(20000);page.setDefaultNavigationTimeout(20000);const errors=[],checks=[],viewports=[];page.on('pageerror',e=>errors.push(e.message));
 const cdp=await context.newCDPSession(page);
 async function pick(name){await page.locator('#deck-overview').click();await page.locator('[data-pick="'+name+'"]').click();await page.waitForTimeout(700);assert.equal(await page.evaluate(()=>JadeNavigation.state().page),name);}
 async function open(name){console.log('Opening module:',name);await pick(name);assert.equal(await page.evaluate(()=>JadeNavigation.state().location),'home');await page.locator('#hero-action').click();await page.waitForFunction(()=>JadeNavigation.state().phase==='open');assert.equal(await page.locator('#module-screen').getAttribute('data-module'),name);}
 async function close(){console.log('Returning to deck');await page.getByRole('button',{name:'Voltar ao deck',exact:true}).click();await page.waitForFunction(()=>JadeNavigation.state().phase==='home');}
 async function faceCheck(scope){const faces=await page.locator(scope+' [data-spatial-surface]').evaluateAll(els=>els.filter(el=>el.getClientRects().length&&getComputedStyle(el).visibility!=='hidden').map(el=>{const s=getComputedStyle(el),m=new DOMMatrixReadOnly(s.transform);return {tag:el.tagName,id:el.id||el.textContent.trim().slice(0,35),tilt:Math.abs(m.m13)+Math.abs(m.m23)+Math.abs(m.m31)+Math.abs(m.m32)};}));assert.ok(faces.length>=4);for(const f of faces)assert.ok(f.tilt>.015,'flat interactive face '+f.id);}
 async function exposedPoint(){return page.evaluate(()=>{
   const middle=document.querySelector('.tile[data-stack="middle"]'),front=document.querySelector('.tile[data-stack="front"]');
   const quad=el=>Array.from(el.querySelectorAll(':scope > .deck-corner')).slice(0,4).map(m=>{const r=m.getBoundingClientRect();return[r.x,r.y];});
   // Independent even/odd polygon test, based on measured visual corners.
   const inside=(p,x,y)=>{let on=false;for(let i=0,j=p.length-1;i<p.length;j=i++)if(((p[i][1]>y)!==(p[j][1]>y))&&(x<(p[j][0]-p[i][0])*(y-p[i][1])/(p[j][1]-p[i][1])+p[i][0]))on=!on;return on;};
   const p=quad(middle),f=quad(front),r=middle.getBoundingClientRect();
   for(let y=r.bottom-8;y>r.top+4;y-=4)for(const part of [.3,.5,.7]){const x=r.left+r.width*part;if(inside(p,x,y)&&!inside(f,x,y))return{x,y,page:middle.dataset.page};}
   return null;
 });}

 try{
  await page.goto('http://127.0.0.1:'+server.address().port);
  await page.waitForFunction(()=>JadeDeckFinish.state().ready||JadeDeckFinish.state().error);
  assert.equal(await page.evaluate(()=>JadeDeckFinish.state().ready),true);
  assert.equal(await page.locator('#scene-prev,#scene-next').count(),0);
  assert.equal(await page.locator('.tile[data-stack="front"],.tile[data-stack="middle"],.tile[data-stack="rear"]').count(),3);
  await page.waitForTimeout(850);await faceCheck('#preview');console.log('Home surfaces verified');
  await page.screenshot({path:path.join(out,'home-1280x720.png'),animations:'disabled'});
  const target=await exposedPoint();assert.ok(target,'no directly tappable rear card');
  await page.touchscreen.tap(target.x,target.y);assert.equal(await page.evaluate(()=>JadeNavigation.state().page),target.page);assert.equal(await page.evaluate(()=>JadeNavigation.state().location),'home');
  await page.waitForTimeout(750);
  const before=await page.locator('.tile[data-stack="front"]').boundingBox(),start={x:before.x+before.width*.5,y:before.y+before.height*.45};
  await cdp.send('Input.dispatchTouchEvent',{type:'touchStart',touchPoints:[{x:start.x,y:start.y,id:1}]});
  for(let i=1;i<=4;i++)await cdp.send('Input.dispatchTouchEvent',{type:'touchMove',touchPoints:[{x:start.x-i*11,y:start.y,id:1}]});
  await page.waitForTimeout(80);
  assert.equal(await page.evaluate(()=>JadeNavigation.state().dragging),true);
  const moving=await page.locator('.tile[data-stack="front"]').boundingBox();assert.ok(Math.abs(moving.x-before.x)>20,'card does not follow the finger');
  assert.equal(await page.evaluate(()=>JadeNavigation.state().page),target.page,'selection changed before release');
  await cdp.send('Input.dispatchTouchEvent',{type:'touchMove',touchPoints:[{x:start.x-100,y:start.y,id:1}]});await cdp.send('Input.dispatchTouchEvent',{type:'touchEnd',touchPoints:[]});
  await page.waitForTimeout(750);assert.equal(await page.evaluate(()=>JadeNavigation.state().page),'Packs');
  console.log('Native touch selection and dragging verified');checks.push('native touch drag follows finger, then snaps','rear card taps select without activation','on-screen arrows removed','shared 3D faces');
  await open('Mods');await faceCheck('#module-screen');
  await page.getByRole('button',{name:'Som',exact:true}).click();assert.equal(await page.locator('#mod-list .mod-row').count(),1);
  await page.locator('#mod-search').fill('audio');assert.equal(await page.locator('#mod-list .mod-row').count(),1,'accent-insensitive search failed');
  const selectedBefore=await page.evaluate(()=>JadeNavigation.state().selectedMods.length);
  await page.getByRole('button',{name:'Selecionar Áudio espacial',exact:true}).click();assert.equal(await page.evaluate(()=>JadeNavigation.state().selectedMods.length),selectedBefore-1);
  await page.locator('#mod-search').fill('');await page.getByRole('button',{name:'Todos',exact:true}).click();assert.equal(await page.locator('#mod-list .mod-row').count(),4);
  await close();assert.equal(await page.evaluate(()=>JadeNavigation.state().page),'Mods','return lost the selected card');
  checks.push('Mods has working filters, search and persistent preview selection','button enters a separate module and Back restores its card');
  await page.evaluate(()=>JadeRenderer.pause(true));
  const pages=['Play','Mods','Packs','Shaders','Files','Skins','Settings'],hashes=[];
  for(const name of pages){await open(name);await faceCheck('#module-screen');const env=await page.evaluate(()=>JadeNavigation.environment());assert.equal(env[0],pages.indexOf(name)+1);assert.ok(env[1]>.99);hashes.push(await page.evaluate(()=>JadeRenderer.screenshot()));await page.screenshot({path:path.join(out,name.toLowerCase()+'-1280x720.png'),animations:'disabled'});await close();}
  assert.equal(new Set(hashes).size,7,'module atmospheres are identical with the clock frozen');
  checks.push('seven different live shader atmospheres with frozen scene clock','all seven module interfaces share perspective materials');
  await open('Packs');await page.locator('[data-module-action="palette:azure"]').click();assert.equal(await page.evaluate(()=>JadeLab.selectedPalette()),'azure');await close();await page.evaluate(()=>JadeLab.applyPalette('jade'));
  await open('Shaders');await page.getByRole('button',{name:'Fosco',exact:true}).click();assert.equal(await page.evaluate(()=>JadeLab.getPreset().controls.glassStyle),'matte');await page.getByRole('button',{name:'Iluminação',exact:true}).click();assert.equal(await page.locator('#editor').evaluate(el=>el.classList.contains('hidden')||el.classList.contains('collapsed')),false);await page.keyboard.press('Escape');assert.equal(await page.evaluate(()=>JadeNavigation.state().location),'module','editor Back also closed module');await close();
  await open('Skins');await page.getByRole('button',{name:'Cobre',exact:true}).click();assert.equal(await page.locator('body').getAttribute('data-jacket'),'copper');await close();
  await open('Settings');await page.getByRole('button',{name:'60 FPS',exact:true}).click();assert.equal(await page.evaluate(()=>JadeLab.getPreset().controls.fps),60);await page.getByRole('button',{name:'Movimento completo',exact:true}).click();assert.equal(await page.evaluate(()=>JadeLab.getPreset().controls.reducedMotion),true);await page.getByRole('button',{name:'Movimento reduzido',exact:true}).click();await close();
  await open('Play');await page.getByRole('button',{name:'Perfil 2',exact:true}).click();await close();assert.equal(await page.locator('#profile-name').textContent(),'Perfil 2');
  await open('Files');const presetDownload=page.waitForEvent('download');await page.getByRole('button',{name:'Exportar preset',exact:true}).click();const downloaded=await presetDownload;assert.equal(JSON.parse(fs.readFileSync(await downloaded.path(),'utf8')).format,'HORIZON_JADE_LAB_V1');
  const packDownload=page.waitForEvent('download');await page.getByRole('button',{name:'Exportar pack completo',exact:true}).click();const pack=await packDownload,zip=fs.readFileSync(await pack.path());assert.ok(zip.includes(Buffer.from('navigation.json')));assert.ok(zip.includes(Buffer.from('shaders/deck-coat.frag')));await close();
  checks.push('Packs palette, Shader material/editor, Skins finish and Settings FPS/motion controls work','profile selection survives return','Files exports real preset and shader pack');
  for(const [width,height] of [[393,852],[852,393],[360,640]]){
    await page.setViewportSize({width,height});await page.waitForTimeout(800);await pick('Mods');
    const point=await exposedPoint();assert.ok(point,`${width}x${height}: rear card has no exposed tap target`);
    await page.screenshot({path:path.join(out,`home-${width}x${height}.png`),animations:'disabled'});
    await page.locator('#hero-action').click();await page.waitForFunction(()=>JadeNavigation.state().phase==='open');
    const back=page.getByRole('button',{name:'Voltar ao deck',exact:true}),r=await back.boundingBox();assert.ok(r&&r.x>=0&&r.y>=0&&r.x+r.width<=width&&r.y+r.height<=height,'clipped Back button');
    const usable=await page.evaluate(({x,y})=>document.elementFromPoint(x,y)?.closest('[data-module-action="back"]')!==null,{x:r.x+r.width*.5,y:r.y+r.height*.5});assert.ok(usable,'blocked module Back');
    const scroll=await page.locator('#mod-list').evaluate(el=>({height:el.clientHeight,total:el.scrollHeight}));assert.ok(scroll.height>45,'unusable module list height');
    await page.screenshot({path:path.join(out,`mods-${width}x${height}.png`),animations:'disabled'});viewports.push({width,height,list:scroll});await close();
  }
  assert.deepEqual(errors,[]);
  fs.writeFileSync(path.join(out,'RESULT.json'),JSON.stringify({passed:true,checks,viewports,navigation:await page.evaluate(()=>JadeNavigation.state())},null,2));console.log('Spatial deck and module checks passed');
 }catch(e){await page.screenshot({path:path.join(out,'failure.png'),animations:'disabled'});console.error('JS errors',errors);console.error('navigation',await page.evaluate(()=>window.JadeNavigation?.state()));fs.writeFileSync(path.join(out,'failure-state.json'),JSON.stringify(await page.evaluate(()=>({navigation:window.JadeNavigation?.state(),coat:window.JadeDeckFinish?.state(),moduleRect:document.getElementById('module-screen').getBoundingClientRect().toJSON()})),null,2));throw e;}
 finally{await browser.close();server.close();}
})().catch(e=>{console.error(e);server.close();process.exitCode=1;});
