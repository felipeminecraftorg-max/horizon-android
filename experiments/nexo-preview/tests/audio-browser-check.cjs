'use strict';
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),http=require('node:http');
const {chromium}=require(process.env.NEXO_PLAYWRIGHT_MODULE||'playwright');
const root=path.resolve(__dirname,'../android/src/main/assets'),out=path.resolve('dist/nexo-audio-qa');fs.mkdirSync(out,{recursive:true});
const server=http.createServer((req,res)=>{const name=decodeURIComponent(req.url.split('?')[0]),file=path.resolve(root,'.'+(name==='/'?'/index.html':name));if(!file.startsWith(root+path.sep)){res.writeHead(403).end();return;}fs.readFile(file,(error,data)=>{if(error){res.writeHead(404).end();return;}res.setHeader('Content-Type',file.endsWith('.js')?'text/javascript':file.endsWith('.css')?'text/css':file.endsWith('.ttf')?'font/ttf':file.endsWith('.ogg')?'audio/ogg':file.endsWith('.json')?'application/json':'text/html');res.end(data);});});
(async()=>{
 await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
 const browser=await chromium.launch({executablePath:'/usr/bin/google-chrome',headless:true,args:['--no-sandbox','--use-angle=swiftshader','--enable-unsafe-swiftshader']});
 const context=await browser.newContext({viewport:{width:852,height:393},hasTouch:true,deviceScaleFactor:1}),page=await context.newPage(),errors=[];
 page.on('pageerror',e=>errors.push(e.message));page.setDefaultTimeout(15000);
 try{
  await page.addInitScript(()=>localStorage.setItem('horizon.nexo.preview.v1',JSON.stringify({version:1,preferences:{introEnabled:false,ambientMotion:false,sceneLighting:false,motionBlur:false,renderScale:.6}})));
  await page.goto('http://127.0.0.1:'+server.address().port);await page.waitForFunction(()=>window.NexoScene?.getState().ready&&NexoScene.getState().faces.some(f=>f.index===1)&&!NexoIntro.isActive());
  // Native touch selects the exposed physical Mods card, exercising its real sound binding.
  const point=await page.evaluate(()=>{const s=NexoScene.getState(),r=document.querySelector('#scene').getBoundingClientRect(),f=s.faces.find(f=>f.index===1);return{x:r.x+r.width*.9,y:r.y+r.height*.5,face:f};});
  function contains(poly,x,y){let yes=false;for(let a=0,b=poly.length-1;a<poly.length;b=a++){const p=poly[a],q=poly[b];if((p[1]>y)!==(q[1]>y)&&x<(q[0]-p[0])*(y-p[1])/(q[1]-p[1])+p[0])yes=!yes;}return yes;}
  const faces=await page.evaluate(()=>NexoScene.getState().faces),front=faces.find(f=>f.index===0);let touch=null;
  for(let x=.95;x>-.95&&!touch;x-=.025)if(contains(point.face.points,x,0)&&!contains(front.points,x,0))touch={x:(x+1)*852/2,y:393/2};
  assert.ok(touch,'Mods card has no exposed touch target');await page.touchscreen.tap(touch.x,touch.y);await page.waitForFunction(()=>NexoAudio.getState().played.select>0);
  await page.evaluate(()=>NexoScene.setVisible(false));
  const decoded=await page.evaluate(async()=>{const ctx=new AudioContext(),rows=[],manifest=await fetch('sounds/sources.json').then(r=>r.json());for(const clip of NexoAudio.getState().clips){const bytes=await fetch(clip.src).then(r=>r.arrayBuffer()),hash=[...new Uint8Array(await crypto.subtle.digest('SHA-256',bytes))].map(x=>x.toString(16).padStart(2,'0')).join(''),b=await ctx.decodeAudioData(bytes);let energy=0,peak=0;for(const x of b.getChannelData(0)){energy+=x*x;peak=Math.max(peak,Math.abs(x));}rows.push({kind:clip.kind,duration:b.duration,channels:b.numberOfChannels,peak,energy:energy/b.length,sha256:hash,expected:manifest.samples.find(s=>s.event===clip.kind).sha256});}await ctx.close();return rows;});
  assert.equal(decoded.length,7);assert.equal(new Set(decoded.map(x=>x.sha256)).size,7);
  for(const x of decoded){assert.equal(x.sha256,x.expected);assert.equal(x.channels,1);assert.ok(x.duration>.03&&x.duration<.8&&x.peak<.32&&x.energy>1e-7,JSON.stringify(x));}
  const kinds=['select','open','back','confirm','error','toggle','click'];
  for(const kind of kinds){await page.waitForTimeout(210);const before=await page.evaluate(k=>NexoAudio.getState().played[k]||0,kind);await page.evaluate(k=>NexoAudio.play(k,.5),kind);await page.waitForFunction(({kind,before})=>(NexoAudio.getState().played[kind]||0)>before,{kind,before});}
  await page.waitForTimeout(210);await page.evaluate(()=>NexoAudio.play('open',.5));await page.waitForTimeout(125);await page.evaluate(()=>NexoAudio.play('back',.5));await page.waitForFunction(()=>NexoAudio.getState().clips.find(x=>x.kind==='back').time>0);
  const overlap=await page.evaluate(()=>NexoAudio.getState().clips);assert.equal(overlap.filter(x=>!x.paused).length,1);assert.equal(overlap.find(x=>x.kind==='open').time,0);
  const before=await page.evaluate(()=>NexoAudio.getState().played.back);await page.evaluate(()=>NexoAudio.play('back',.5));await page.waitForTimeout(40);assert.equal(await page.evaluate(()=>NexoAudio.getState().played.back),before,'repeat guard failed');
  await page.evaluate(()=>NexoAudio.setVolume(0));const muted=await page.evaluate(()=>NexoAudio.getState().played);for(const kind of kinds)await page.evaluate(k=>NexoAudio.play(k,0),kind);assert.deepEqual(await page.evaluate(()=>NexoAudio.getState().played),muted);assert.ok(await page.evaluate(()=>NexoAudio.getState().clips.every(x=>x.paused&&x.volume===0)));
  await page.waitForTimeout(210);await page.evaluate(()=>NexoAudio.play('open',.5));await page.waitForFunction(()=>NexoAudio.getState().clips.find(x=>x.kind==='open').time>0);await page.evaluate(()=>Nexo.setVisible(false));assert.ok(await page.evaluate(()=>NexoAudio.getState().clips.every(x=>x.paused&&x.time===0)));await page.evaluate(()=>Nexo.setVisible(true));
  await page.waitForTimeout(210);await page.evaluate(()=>NexoAudio.play('confirm',.5));await page.waitForFunction(()=>NexoAudio.getState().clips.find(x=>x.kind==='confirm').time>0);
  assert.deepEqual(await page.evaluate(()=>NexoAudio.getState().failures),{});assert.deepEqual(errors,[]);
  fs.writeFileSync(path.join(out,'RESULT.json'),JSON.stringify({passed:true,checks:['actual card touch plays the new selection asset','all seven distinct assets decode and match provenance hashes','all seven event cues play','single voice and repeat guards','zero-volume mute','native pause and resume'],decoded,note:'Objective playback/level checks do not establish headphone comfort or Redmi behavior.'},null,2));
  console.log('Nexo Firefly audio browser checks passed');
 }finally{await browser.close();server.close();}
})().catch(error=>{console.error(error);server.close();process.exitCode=1;});
