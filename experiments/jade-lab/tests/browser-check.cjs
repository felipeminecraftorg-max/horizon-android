/* Browser integration checks. Software WebGL verifies rendering correctness,
 * never handset FPS. Run with JADE_PLAYWRIGHT_MODULE pointing to Playwright. */
'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');const path=require('node:path');const http=require('node:http');
const {chromium}=require(process.env.JADE_PLAYWRIGHT_MODULE||'playwright');
const root=path.resolve(__dirname,'../android/src/main/assets');
const out=path.resolve(process.env.JADE_QA_DIR||'dist/visual-qa');fs.mkdirSync(out,{recursive:true});
const server=http.createServer((req,res)=>{const file=path.join(root,decodeURIComponent(req.url.split('?')[0]==='/'?'/index.html':req.url.split('?')[0]));if(!file.startsWith(root+path.sep)){res.writeHead(403).end();return;}fs.readFile(file,(e,data)=>{if(e){res.writeHead(404).end();return;}res.setHeader('Content-Type',file.endsWith('.js')?'text/javascript':file.endsWith('.css')?'text/css':'text/html');res.end(data);});});
(async()=>{
  await new Promise(r=>server.listen(0,'127.0.0.1',r));
  const browser=await chromium.launch({executablePath:process.env.JADE_CHROME||'/usr/bin/google-chrome',headless:true,args:['--no-sandbox','--use-angle=swiftshader','--enable-unsafe-swiftshader']});
  const page=await browser.newPage({viewport:{width:1280,height:720},deviceScaleFactor:1});
  const errors=[];page.on('pageerror',e=>errors.push(e.message));
  const report=[];
  try{
    await page.goto('http://127.0.0.1:'+server.address().port);
    await page.waitForFunction(()=>window.JadeRenderer?.state().webgl2||document.body.classList.contains('gpu-unavailable'));
    let gpu=await page.evaluate(()=>JadeRenderer.state());assert.equal(gpu.webgl2,true,gpu.error);
    await page.waitForFunction(()=>JadeRenderer.state().surfaces===4);
    for(const [width,height] of [[1280,720],[393,852],[852,393],[360,640]]){
      await page.setViewportSize({width,height});await page.waitForTimeout(300);
      const data=await page.evaluate(()=>{
        const geom=JadeRenderer.geometry();
        return Array.from(document.querySelectorAll('[data-node]')).map((el,i)=>{
          const b=el.querySelector('.surface-face'),r=b.getBoundingClientRect(),p=geom.surfaces[i];
          const xs=p.corners.map(q=>q[0]),ys=p.corners.map(q=>q[1]);
          const top=document.elementFromPoint(r.x+r.width*.5,r.y+r.height*.5);
          return {id:el.dataset.node,rect:{x:r.x,y:r.y,w:r.width,h:r.height},projected:[Math.min(...xs),Math.min(...ys),Math.max(...xs),Math.max(...ys)],hit:el.dataset.node==='art'||top?.closest('[data-option]')===b};
        });
      });
      for(const d of data){
        const r=d.rect,p=d.projected;
        assert.ok(Math.abs(r.x-p[0])<1&&Math.abs(r.y-p[1])<1&&Math.abs(r.x+r.w-p[2])<1&&Math.abs(r.y+r.h-p[3])<1,'camera/text mismatch '+d.id);
        assert.ok(r.x>=0&&r.y>=0&&r.x+r.w<=width&&r.y+r.h<=height-25,`${width}x${height}: clipped ${d.id} ${JSON.stringify(r)}`);
        assert.ok(d.hit,`${width}x${height}: blocked hitbox ${d.id}`);
      }
      const copy=await page.locator('.scene-copy').boundingBox(),options=await page.locator('.option-slot').first().boundingBox();
      assert.ok(copy.y+copy.height<options.y,`${width}x${height}: intro overlaps options`);
      await page.screenshot({path:path.join(out,`jade-${width}x${height}.png`)});
      report.push({viewport:[width,height],geometry:data});
    }
    await page.setViewportSize({width:1280,height:720});
    await page.getByRole('tab',{name:'Mods',exact:true}).click();
    assert.equal(await page.evaluate(()=>JadeScene.state().page),'Mods');
    await page.keyboard.press('ArrowDown');assert.equal(await page.evaluate(()=>JadeScene.state().level),'options');
    await page.keyboard.press('ArrowDown');assert.equal(await page.evaluate(()=>JadeScene.state().row),1);
    await page.keyboard.press('Enter');await page.getByRole('dialog').waitFor({state:'visible'});
    await page.keyboard.press('Escape');assert.equal(await page.evaluate(()=>JadeScene.state().level),'options');
    await page.keyboard.press('Escape');assert.equal(await page.evaluate(()=>JadeScene.state().level),'categories');
    await page.keyboard.press('Escape');assert.equal(await page.evaluate(()=>JadeScene.state().page),'Play');
    await page.getByRole('tab',{name:'Shaders',exact:true}).click();
    await page.getByRole('button',{name:/Editar iluminação/}).click();
    await page.getByRole('tab',{name:'☀ Luz',exact:true}).waitFor({state:'visible'});
    await page.getByRole('button',{name:'Ocultar painel',exact:true}).click();
    await page.getByRole('tab',{name:'Sistema',exact:true}).click();
    await page.getByRole('button',{name:/Aparência e paletas/}).click();
    await page.locator('[data-palette="azure"]').click();
    assert.equal(await page.evaluate(()=>JadeLab.getPreset().controls.warm),'#eba78f');
    await page.getByRole('button',{name:'Ocultar painel',exact:true}).click();
    await page.getByRole('tab',{name:'Jogar',exact:true}).click();
    await page.evaluate(()=>{JadeLab.applyPalette('jade');JadeRenderer.pause(true);});
    await page.waitForTimeout(300);
    const frozen=await page.evaluate(()=>JadeRenderer.state().elapsed);
    const png1=await page.evaluate(()=>JadeRenderer.screenshot());
    await page.waitForTimeout(250);
    assert.equal(await page.evaluate(()=>JadeRenderer.state().elapsed),frozen,'pause advanced scene clock');
    const png2=await page.evaluate(()=>JadeRenderer.screenshot());assert.equal(png1,png2,'paused scene changed');
    await page.mouse.move(300,250);await page.waitForTimeout(150);
    const moved=await page.evaluate(()=>JadeRenderer.screenshot());assert.notEqual(png2,moved,'light did not respond while paused');
    await page.evaluate(()=>{JadeRenderer.pause(false);JadeLab.setField('reducedMotion',false);});
    await page.waitForTimeout(1200);
    const animated=await page.evaluate(()=>JadeRenderer.screenshot());assert.notEqual(animated,moved,'live waves did not animate');
    const before=await page.evaluate(()=>JadeLab.getPreset().controls);
    const presetDownload=page.waitForEvent('download');await page.getByRole('button',{name:'Abrir editor',exact:true}).click();await page.getByRole('button',{name:'↓ PRESET JSON',exact:true}).click();const preset=await presetDownload;const saved=JSON.parse(fs.readFileSync(await preset.path(),'utf8'));assert.deepEqual(saved.controls,before);
    const packDownload=page.waitForEvent('download');await page.getByRole('button',{name:'↓ PACK ZIP',exact:true}).click();const pack=await packDownload;const zip=fs.readFileSync(await pack.path());assert.ok(zip.includes(Buffer.from('scene/geometry.json')),'geometry missing in pack');assert.ok(zip.includes(Buffer.from('uBoxCenters')),'wrong shader exported');
    await page.getByRole('button',{name:'Ocultar painel',exact:true}).click();
    await page.evaluate(()=>JadeRenderer.pause(true));await page.waitForTimeout(250);
    await page.screenshot({path:path.join(out,'jade-final.png')});
    assert.deepEqual(errors,[],'browser errors');
    gpu=await page.evaluate(()=>JadeRenderer.state());
    fs.writeFileSync(path.join(out,'RESULT.json'),JSON.stringify({passed:true,renderer:gpu,checks:['4 responsive viewports','shared GPU/DOM projection','unblocked touch targets','category + row + dialog Back hierarchy','editor routes','complementary palette','pause','pointer lighting','live waves','preset and ZIP exports'],viewports:report},null,2));
    console.log('Browser checks passed:',out);
  }catch(e){await page.screenshot({path:path.join(out,'failure.png')});console.error('GPU',await page.evaluate(()=>window.JadeRenderer?.state()));console.error('Browser errors',errors);throw e;}
  finally{await browser.close();server.close();}
})().catch(e=>{console.error(e);server.close();process.exitCode=1;});
