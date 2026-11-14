"use strict";
const test=require("node:test");
const assert=require("node:assert/strict");
const fs=require("node:fs");
const path=require("node:path");
const vm=require("node:vm");
const root=path.resolve(__dirname,"../android/src/main/assets");
const renderer=fs.readFileSync(path.join(root,"renderer.js"),"utf8");
const palettes=fs.readFileSync(path.join(root,"palettes.js"),"utf8");
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
  vm.runInContext(palettes,env,{filename:"palettes.js"});
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
  for(const tab of ["Atmosfera","Luz","Materiais","Paletas","Cores","Componentes",
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

test("seven curated dark palettes have independent material, atmosphere and readable accents",()=>{
  const x=sandbox();
  const catalog=x.JadePalettes;
  assert.equal(catalog.ids.length,7);
  assert.deepEqual(Array.from(catalog.ids).sort(),
    ["amethyst","azure","cobalt","graphite","jade","obsidian","smoke"].sort());
  const signatures=new Set();
  for(const id of catalog.ids){
    const p=catalog.profiles[id];
    for(const section of ["colors","material","lighting","atmosphere"])
      assert.ok(Object.keys(p[section]).length>=3,id+":"+section);
    const c=p.colors;
    for(const key of ["bg","surface","accent","cool","warm"])
      assert.match(c[key],/^#[0-9a-f]{6}$/i);
    assert.ok(catalog.contrast(c.accent,c.bg)>=3.0,
      id+" accent/background contrast");
    assert.ok(catalog.contrast(c.accent,c.surface)>=2.8,
      id+" accent/surface contrast");
    assert.ok(catalog.contrast(c.surface,c.bg)>1.1,
      id+" surfaces must be distinct");
    signatures.add([c.bg,c.surface,c.accent,p.material.roughness,
      p.lighting.intensity,p.atmosphere.fog].join("|"));
  }
  assert.equal(signatures.size,7);
  assert.ok(catalog.contrast("#ffffff","#000000")>20);
});

test("palette switches recolor GPU presets while preserving fps and quality",()=>{
  const x=sandbox();
  x.JadeLab.setField("fps",90);
  x.JadeLab.setField("quality",.65);
  x.JadeLab.setField("reducedMotion",true);
  for(const id of x.JadePalettes.ids){
    assert.equal(x.JadeLab.applyPalette(id),true);
    const p=x.JadeLab.getPreset();
    const spec=x.JadePalettes.profiles[id];
    assert.equal(p.controls.paletteId,id);
    assert.equal(p.palette,id);
    assert.equal(p.theme,"oldschool-dark");
    assert.equal(p.controls.accent,spec.colors.accent);
    assert.equal(p.controls.roughness,spec.material.roughness);
    assert.equal(p.controls.ambient,spec.lighting.ambient);
    assert.equal(p.controls.fog,spec.atmosphere.fog);
    assert.equal(p.controls.fps,90);
    assert.equal(p.controls.quality,.65);
    assert.equal(p.controls.reducedMotion,true);
  }
  assert.equal(x.JadeLab.applyPalette("__invalid__"),false);
  assert.equal(x.JadeLab.selectedPalette(),"smoke");
});

test("custom color edits do not overwrite chosen colors or legacy preset format",async()=>{
  const x=sandbox();
  x.JadeLab.applyPalette("amethyst");
  x.JadeLab.setField("accent","#efbb66");
  x.JadeLab.setField("selectionStyle","rim");
  assert.equal(x.JadeLab.selectedPalette(),"custom");
  assert.equal(x.JadeLab.getPreset().controls.accent,"#efbb66");
  assert.equal(x.JadeLab.getSection("Componentes").controls.selectionStyle,"rim");
  assert.equal(x.JadeLab.getSection("Paletas").controls.paletteId,"custom");
  x.JadeLab.setField("selectionStyle","bad");
  assert.equal(x.JadeLab.getPreset().controls.selectionStyle,"rim");
  const partial={format:"HORIZON_JADE_TAB_V1",theme:"oldschool-dark",
    tab:"Paletas",controls:{paletteId:"obsidian"}};
  const file=new Blob([JSON.stringify(partial)],{type:"application/json"});
  file.name="obsidian.json";
  await x.JadeLab.importFile(file);
  assert.equal(x.JadeLab.getPreset().controls.paletteId,"obsidian");
  assert.equal(x.JadeLab.getPreset().controls.accent,
    x.JadePalettes.profiles.obsidian.colors.accent);
  const legacy={format:"HORIZON_JADE_LAB_V1",theme:"oldschool-dark",
    controls:{accent:"#ffcf64",bg:"#140e14"}};
  const old=new Blob([JSON.stringify(legacy)],{type:"application/json"});
  old.name="old-jade.json";
  await x.JadeLab.importFile(old);
  assert.equal(x.JadeLab.selectedPalette(),"custom");
  assert.equal(x.JadeLab.getPreset().controls.accent,"#ffcf64");
});

test("selection is dynamic GPU lighting without fixed left-hand indicator",()=>{
  const x=sandbox();
  const css=fs.readFileSync(path.join(root,"styles.css"),"utf8");
  const dynamicCss=fs.readFileSync(path.join(root,"palette.css"),"utf8");
  const html=fs.readFileSync(path.join(root,"index.html"),"utf8");
  assert.doesNotMatch(css,/\.hero:before\s*\{/);
  assert.match(html,/palette\.css/);
  assert.match(html,/palettes\.js/);
  assert.ok(html.indexOf("palettes.js")<html.indexOf("app.js"));
  assert.match(x.JadeRenderer.fragmentSource,/uniform int uFocusIndex/);
  assert.match(x.JadeRenderer.fragmentSource,/uniform int uSelectionStyle/);
  assert.match(x.JadeRenderer.fragmentSource,/bool focused=i==uFocusIndex/);
  assert.match(dynamicCss,/\[data-selection="rim"\]/);
});

test("Android manifest is a separate offline app with no broad permissions",()=>{
  const xml=fs.readFileSync(path.resolve(__dirname,"../android/src/main/AndroidManifest.xml"),"utf8");
  assert.match(xml,/dev\.horizon\.jadelab/);
  assert.doesNotMatch(xml,/uses-permission/);
  assert.match(xml,/hardwareAccelerated="true"/);
  assert.match(xml,/usesCleartextTraffic="false"/);
});
