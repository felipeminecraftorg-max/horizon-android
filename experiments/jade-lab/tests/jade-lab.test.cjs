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
  for(const tab of ["Fundo","Atmosfera","Luz","Vidro","Materiais",
    "Paletas","Cores","Componentes","Movimento","Desempenho","Exportar"]){
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

test("five atmospheric backgrounds have independent GPU uniforms and true GLSL source",()=>{
  const x=sandbox(),glsl=x.JadeRenderer.fragmentSource;
  for(const name of ["uBackdropShape","uBackdropLayers",
    "uBackdropLight","uBackdropDetail"])
      assert.match(glsl,new RegExp("uniform vec4 "+name+";"));
  assert.match(glsl,/vec3 renderBackdrop/);
  assert.match(glsl,/float ribbonHeight/);
  assert.match(glsl,/for\(int layer=0;layer<4;layer\+\+\)/);
  assert.match(renderer,/vec4\("uBackdropShape"/);
  assert.match(renderer,/backgroundCenterX/);
  assert.match(renderer,/backgroundParallax/);
  assert.doesNotMatch(glsl,/texture2D\(/);
});

test("background presets have distinct compositions and respect FPS",()=>{
  const x=sandbox();
  x.JadeLab.setField("fps",90);
  x.JadeLab.setField("quality",.65);
  x.JadeLab.setField("reducedMotion",true);
  const styles=new Set();
  for(const id of x.JadePalettes.ids){
    assert.equal(x.JadeLab.applyPalette(id),true);
    const ctrl=x.JadeLab.getPreset().controls;
    const backdrop=x.JadePalettes.profiles[id].backdrop;
    assert.ok(Object.keys(backdrop).length>=14,"profile "+id);
    assert.equal(ctrl.backgroundStyle,backdrop.backgroundStyle);
    assert.equal(ctrl.ribbonLayers,backdrop.ribbonLayers);
    assert.equal(ctrl.gradientDepth,backdrop.gradientDepth);
    assert.equal(ctrl.backgroundCenterX,backdrop.backgroundCenterX);
    assert.equal(ctrl.backgroundContrast,backdrop.backgroundContrast);
    assert.equal(ctrl.fps,90);
    assert.equal(ctrl.quality,.65);
    assert.equal(ctrl.reducedMotion,true);
    styles.add(ctrl.backgroundStyle);
  }
  assert.equal(styles.size,5,"every background style is used by a curated palette");
});

test("Fundo tab stores all atmospheric controls with strict ranges and JSON import",async()=>{
  const x=sandbox(),before=x.JadeLab.getSection("Fundo");
  assert.equal(before.format,"HORIZON_JADE_TAB_V1");
  const keys=Object.keys(before.controls);
  for(const key of ["backgroundStyle","backgroundIntensity","backgroundScale",
    "backgroundFlow","ribbonLayers","ribbonCurve","ribbonWidth",
    "ribbonSoftness","ribbonGleam","backgroundGlow","gradientDepth",
    "backgroundContrast","backgroundGrain","backgroundCenterX",
    "backgroundCenterY","backgroundParallax"])
    assert.ok(keys.includes(key),"Missing "+key);
  x.JadeLab.setField("backgroundStyle","satin");
  x.JadeLab.setField("backgroundIntensity",20);
  x.JadeLab.setField("backgroundCenterX",-5);
  x.JadeLab.setField("ribbonLayers",4);
  x.JadeLab.setField("backgroundStyle","not-a-style");
  let ctrl=x.JadeLab.getPreset().controls;
  assert.equal(ctrl.backgroundStyle,"satin");
  assert.equal(ctrl.backgroundIntensity,1.8);
  assert.equal(ctrl.backgroundCenterX,0);
  assert.equal(ctrl.ribbonLayers,4);
  const one={format:"HORIZON_JADE_TAB_V1",theme:"oldschool-dark",
    tab:"Fundo",controls:{backgroundStyle:"aurora",ribbonCurve:1.31,
       backgroundGlow:.47}};
  const file=new Blob([JSON.stringify(one)],{type:"application/json"});
  file.name="fundo.json";
  await x.JadeLab.importFile(file);
  ctrl=x.JadeLab.getPreset().controls;
  assert.equal(ctrl.backgroundStyle,"aurora");
  assert.equal(ctrl.ribbonCurve,1.31);
  assert.equal(ctrl.backgroundGlow,.47);
  assert.equal(ctrl.fps,45);
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

test("three optical dark glass variants use actual procedural backdrop sampling",()=>{
  const x=sandbox(),source=x.JadeRenderer.fragmentSource;
  for(const u of ["uGlassModel","uGlassOptics","uGlassDetail","uGlassTargets"])
    assert.match(source,new RegExp("uniform vec4 "+u+";"));
  assert.match(source,/vec3 frostedBackdrop/);
  assert.match(source,/texture\(uSceneTexture,clamp\(texUv\+dx/);
  assert.match(source,/uGlassDetail\.w>1\.5/);
  assert.match(source,/backdropTone=mix\(backdropTone,base,\.48\)/);
  for(const k of ["glassStyle","glassQuality","glassOpacity","glassBlur",
    "glassTransmission","glassTint","glassScatter","glassFresnel",
    "glassRim","glassGrain","glassHighlight","glassShadow",
    "glassHero","glassCards","glassChrome","glassCTA"])
    assert.ok(k in x.JadeLab.getSection("Vidro").controls,k);
  assert.equal(x.JadeLab.getPreset().controls.glassChrome,false);
  x.JadeLab.setField("glassStyle","matte");
  x.JadeLab.setField("glassQuality","high");
  x.JadeLab.setField("glassOpacity",100);
  x.JadeLab.setField("glassBlur",-3);
  assert.equal(x.JadeLab.getPreset().controls.glassStyle,"matte");
  assert.equal(x.JadeLab.getPreset().controls.glassQuality,"high");
  assert.equal(x.JadeLab.getPreset().controls.glassOpacity,1);
  assert.equal(x.JadeLab.getPreset().controls.glassBlur,0);
});

test("seven palettes choose their own optical finishes without forcing FPS changes",()=>{
  const x=sandbox(),kinds=new Set();
  x.JadeLab.setField("fps",90);
  x.JadeLab.setField("quality",.65);
  for(const id of x.JadePalettes.ids){
    x.JadeLab.applyPalette(id);
    const p=x.JadeLab.getPreset().controls;
    const optical=x.JadePalettes.profiles[id].glass;
    assert.equal(p.glassStyle,optical.glassStyle);
    assert.equal(p.glassTransmission,optical.glassTransmission);
    assert.equal(p.glassHighlight,optical.glassHighlight);
    assert.equal(p.fps,90);
    assert.equal(p.quality,.65);
    kinds.add(p.glassStyle);
  }
  assert.equal(kinds.size,3,"at least Frosted, Tinted and Matte finishes");
});

test("console scene gives the backdrop room and stacks just three 3D cards",()=>{
  const html=fs.readFileSync(path.join(root,"index.html"),"utf8");
  const css=fs.readFileSync(path.join(root,"scene.css"),"utf8");
  assert.match(html,/id="console-scene"/);
  assert.match(html,/class="avatar-stage"/);
  assert.match(html,/class="avatar-figure"/);
  assert.match(html,/id="scene-prev"/);
  assert.match(html,/id="scene-next"/);
  assert.match(html,/scene\.css/);
  assert.match(css,/\.avatar-head-front/);
  assert.match(css,/\.avatar-floor/);
  assert.match(css,/\.scene-navigation/);
  assert.match(css,/\.dashboard \.cards\{/);
  assert.match(css,/perspective:940px/);
  for(const mode of ["front","middle","rear","hidden"])
    assert.ok(css.includes('[data-stack="'+mode+'"]'),"missing "+mode);
  assert.match(css,/transform-style:preserve-3d/);
  assert.match(css,/\.dashboard \.cards\s*\{[\s\S]*?position:absolute/);
  assert.match(css,/\.dashboard \.hero[\s\S]*?width:clamp\(220px,33%,480px\)/);
  assert.match(css,/@media\(max-width:720px\) and \(orientation:portrait\)/);
  assert.match(css,/@media\(max-height:480px\) and \(orientation:landscape\)/);
  assert.doesNotMatch(css,/\.hero:before\s*\{/);
});

test("scene interaction moves focus by swipe, arrows and page without adding static overlays",()=>{
  const app=fs.readFileSync(path.join(root,"app.js"),"utf8");
  const renderer=fs.readFileSync(path.join(root,"renderer.js"),"utf8");
  const x=sandbox();
  assert.match(app,/const STACK_PAGES=\["Play","Mods","Packs","Shaders","Files","Skins","Settings"\]/);
  assert.match(app,/difference===2\?"rear":"hidden"/);
  assert.match(app,/Math\.abs\(dx\)>52/);
  assert.match(app,/e\.key==="ArrowRight"/);
  assert.match(app,/tile\.setAttribute\("aria-hidden"/);
  assert.match(renderer,/if\(element\.classList\?\.contains\("tile"\)\)return;/);
  assert.match(renderer,/rectCache\.set\(\[r\.x,r\.y,r\.width,r\.height\],index\*4\)/);
  assert.match(renderer,/uniform1i\(uniform\.uRectCount,NAMES\.length\)/);
  const controls=x.JadeLab.getSection("Componentes").controls;
  for(const key of ["sceneDepth","sceneSpacing","sceneTilt","avatarMotion"])
    assert.ok(Object.hasOwn(controls,key),key);
  x.JadeLab.setField("sceneTilt",100);
  x.JadeLab.setField("sceneDepth",-1);
  assert.equal(x.JadeLab.getPreset().controls.sceneTilt,25);
  assert.equal(x.JadeLab.getPreset().controls.sceneDepth,0);
  assert.equal(x.JadeLab.getPreset().theme,"oldschool-dark");
});

test("cached two-pass GPU scene avoids recursive atmospheric recomputation",()=>{
  const x=sandbox(),src=x.JadeRenderer.fragmentSource,js=renderer;
  assert.match(src,/uniform sampler2D uSceneTexture;/);
  assert.match(src,/uniform int uScenePass;/);
  assert.match(src,/uniform vec4 uSceneNeutral\[3\];/);
  const blur=src.slice(src.indexOf("vec3 frostedBackdrop"),src.indexOf("void main()"));
  assert.match(blur,/texture\(uSceneTexture/);
  assert.doesNotMatch(blur,/renderBackdrop\(/);
  assert.match(js,/gl\.createFramebuffer\(\)/);
  assert.match(js,/gl\.framebufferTexture2D\(/);
  assert.match(js,/gl\.uniform1i\(uniform\.uScenePass,1\)/);
  assert.match(js,/gl\.uniform1i\(uniform\.uScenePass,0\)/);
  assert.match(js,/Math\.min\(scale,pixelCap\)/);
  assert.doesNotMatch(js,/Math\.min\(window\.devicePixelRatio/);
  assert.match(js,/adaptiveScale=Math\.max/);
  assert.match(js,/lastCssLight>45/);
  assert.match(src,/uScenePass==2\?renderBackdrop/);
});

test("one seven-page console navigation replaces duplicate menu row",()=>{
  const html=fs.readFileSync(path.join(root,"index.html"),"utf8");
  const css=fs.readFileSync(path.join(root,"console-refine.css"),"utf8");
  assert.doesNotMatch(html,/<nav class="channels/);
  assert.match(html,/data-surface="playtile" data-page="Play"/);
  assert.match(html,/01 \/ 07/);
  assert.match(html,/console-refine\.css/);
  assert.match(css,/\.preview \.channels,\.preview \.status-footer\{display:none!important\}/);
  assert.match(css,/-webkit-backdrop-filter:none!important/);
  assert.match(css,/--light-pos-x/);
  assert.match(css,/--sceneMetal/);
});

test("themes separate neutral materials from accents and export them independently",()=>{
  const x=sandbox();
  for(const id of x.JadePalettes.ids){
    const p=x.JadePalettes.profiles[id];
    for(const k of ["sceneNeutral","sceneMetal","sceneHighlight",
      "environmentLight","environmentWarmth"])
      assert.ok(k in p.scene,id+" "+k);
    assert.notEqual(p.scene.sceneMetal,p.colors.accent);
    assert.ok(x.JadePalettes.contrast(p.colors.accent,p.colors.bg)>3.0);
    x.JadeLab.applyPalette(id);
    assert.equal(x.JadeLab.getPreset().controls.sceneMetal,p.scene.sceneMetal);
  }
  x.JadeLab.setField("sceneNeutral","#a1a2a3");
  assert.equal(x.JadeLab.selectedPalette(),"custom");
  assert.equal(x.JadeLab.getSection("Cores").controls.sceneNeutral,"#a1a2a3");
});

test("Android manifest is a separate offline app with no broad permissions",()=>{
  const xml=fs.readFileSync(path.resolve(__dirname,"../android/src/main/AndroidManifest.xml"),"utf8");
  assert.match(xml,/dev\.horizon\.jadelab/);
  assert.doesNotMatch(xml,/uses-permission/);
  assert.match(xml,/hardwareAccelerated="true"/);
  assert.match(xml,/usesCleartextTraffic="false"/);
});
