"use strict";
const test=require("node:test");
const assert=require("node:assert/strict");
const fs=require("node:fs");
const path=require("node:path");
const vm=require("node:vm");
const root=path.resolve(__dirname,"../android/src/main/assets");
const renderer=fs.readFileSync(path.join(root,"renderer.js"),"utf8");
const editor=fs.readFileSync(path.join(root,"app.js"),"utf8");

function sandbox() {
  const set = {};
  const win = {};
  const document = {
    readyState:"loading",
    documentElement:{style:{setProperty(){}}},
    addEventListener(){},
    querySelectorAll(){return [];},
    getElementById(){return null;}
  };
  const env = {
    window:win,document,Uint8Array,Uint32Array,DataView,TextEncoder,TextDecoder,
    Blob,JSON,Date,Number,Object,String,Math,parseInt,btoa,atob,
    localStorage:{setItem(k,v){set[k]=v;},getItem(k){return set[k]||null;}}
  };
  vm.createContext(env);
  vm.runInContext(renderer,env,{filename:"renderer.js"});
  vm.runInContext(editor,env,{filename:"app.js"});
  return win;
}

test("real GLSL shader and full-scene renderer are exported",()=>{
  const x=sandbox();
  assert.match(x.JadeRenderer.fragmentSource,/^#version 300 es/m);
  assert.match(x.JadeRenderer.fragmentSource,/ggxD/);
  assert.match(x.JadeRenderer.fragmentSource,/uRects\[12\]/);
  assert.match(x.JadeRenderer.fragmentSource,/uPointer/);
  assert.match(x.JadeRenderer.vertexSource,/aPosition/);
});

test("default preset is dark-only and covers all material families",()=>{
  const x=sandbox(),p=x.JadeLab.getPreset();
  assert.equal(p.format,"HORIZON_JADE_LAB_V1");
  assert.equal(p.theme,"oldschool-dark");
  for(const k of ["fog","waves","roughness","metallic","reflectivity","azimuth",
    "bevel","ao","heroGlow","accent","quality","fps"]){
    assert.ok(k in p.controls,"missing "+k);
  }
});

test("every settings tab can be exported as a separate JSON object",()=>{
  const x=sandbox();
  for(const tab of ["Atmosfera","Luz","Materiais","Cores","Componentes",
    "Movimento","Desempenho","Exportar"]){
    const one=x.JadeLab.getSection(tab);
    assert.equal(one.format,"HORIZON_JADE_TAB_V1");
    assert.equal(one.theme,"oldschool-dark");
    assert.equal(one.tab,tab);
    if(tab!=="Exportar")assert.ok(Object.keys(one.controls).length>0,tab);
  }
});

test("sliders clamp out-of-range values without removing other controls",()=>{
  const x=sandbox();
  x.JadeLab.setField("roughness",20);
  x.JadeLab.setField("fog",-4);
  x.JadeLab.setField("azimuth",-999);
  x.JadeLab.setField("accent","#12fa56");
  x.JadeLab.setField("fake-key",NaN);
  const p=x.JadeLab.getPreset().controls;
  assert.equal(p.roughness,.96);
  assert.equal(p.fog,0);
  assert.equal(p.azimuth,-180);
  assert.equal(p.accent,"#12fa56");
  assert.equal("fake-key" in p,false);
});

test("exported dependency-free stored ZIP can reimport preset",async()=>{
  const x=sandbox(),p=x.JadeLab.getPreset();
  const bytes=new TextEncoder().encode(JSON.stringify(p));
  const zip=x.JadeLab.zipStore([
    {name:"preset.json",bytes},
    {name:"tabs/atmosfera.json",bytes:new TextEncoder().encode("{}")},
    {name:"shaders/jade_scene.frag",bytes:new TextEncoder().encode(x.JadeRenderer.fragmentSource)}
  ]);
  const arr=new Uint8Array(await zip.arrayBuffer());
  assert.equal(new DataView(arr.buffer).getUint32(0,true),0x04034b50);
  assert.equal(new DataView(arr.buffer).getUint32(arr.length-22,true),0x06054b50);
  const loaded=x.JadeLab.findStoredZipPreset(arr);
  assert.equal(loaded.format,p.format);
  assert.equal(loaded.controls.accent,p.controls.accent);
  assert.equal(loaded.theme,"oldschool-dark");
});

test("Android manifest is a separate offline app with no broad permissions",()=>{
  const xml=fs.readFileSync(path.resolve(__dirname,"../android/src/main/AndroidManifest.xml"),"utf8");
  assert.match(xml,/dev\.horizon\.jadelab/);
  assert.doesNotMatch(xml,/uses-permission/);
  assert.match(xml,/hardwareAccelerated="true"/);
  assert.match(xml,/usesCleartextTraffic="false"/);
});
