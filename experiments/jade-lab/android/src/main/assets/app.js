/* Jade Lab editor: offline, dark-only, reusable exports. All sections can be
 * minimized AND exported independently. The Android Java bridge writes files
 * through MediaStore; desktop browsers use standard Blob downloads.
 */
(function () {
  "use strict";
  const FORMAT="HORIZON_JADE_LAB_V1";
  const DEFAULTS=Object.freeze({
    fog:.48,waves:.33,noise:.09,vignette:.35,diffusion:.58,
    azimuth:-38,elevation:.72,intensity:1.24,ambient:.66,
    roughness:.29,metallic:.31,reflectivity:.72,bevel:.57,
    depth:.65,ao:.43,edgeLight:.74,heroGlow:.79,bloom:.38,
    bg:"#091611",surface:"#203c2c",accent:"#74da94",cool:"#63aeb8",warm:"#cb995a",
    contrast:1.10,saturation:1.07,
    speed:.48,pulse:.22,parallax:.62,reducedMotion:false,
    quality:.87,fps:60
  });
  const SPECS={
    fog:["Névoa atmosférica",0,1,.01,"Intensidade difusa no fundo"],
    waves:["Ondas de energia",0,1,.01,"Luz ambiental em ondas largas"],
    noise:["Textura fina",0,.6,.01,"Granulação óptica discreta"],
    vignette:["Vinheta",0,1,.01,"Sombra periférica"],
    diffusion:["Difusão atmosférica",0,1,.01,"Equilíbrio de luz no cenário"],
    azimuth:["Azimute da luz",-180,180,1,"° · Direção horizontal da fonte"],
    elevation:["Elevação da luz",.05,1.6,.01,"Altura aparente do emissor"],
    intensity:["Intensidade luminosa",.05,2.4,.01,"Energia da fonte principal"],
    ambient:["Luz ambiente",.10,1.2,.01,"Iluminação recebida nas sombras"],
    roughness:["Rugosidade",.075,.96,.005,"Menor = reflexo mais definido"],
    metallic:["Metallic",0,1,.01,"Proporção de resposta metálica"],
    reflectivity:["Reflexividade",0,2,.01,"Força especular na GPU"],
    bevel:["Relevo das bordas",0,1,.01,"Curvatura normal dos painéis"],
    depth:["Profundidade",0,1.5,.01,"Luz interna e relevo visual"],
    ao:["Oclusão ambiente",0,1,.01,"Escurecimento entre superfícies"],
    edgeLight:["Luz de borda",0,1.8,.01,"Rim e fresnel de contorno"],
    heroGlow:["Halo do painel principal",0,1.5,.01,"Energia luminosa localizada"],
    bloom:["Bloom local",0,1.5,.01,"Realce luminoso contido nos materiais"],
    bg:["Fundo escuro","color",null,null,"Cor estrutural do cenário"],
    surface:["Base dos painéis","color",null,null,"Pigmento do material"],
    accent:["Jade principal","color",null,null,"Bordas, luz e seleção"],
    cool:["Luz secundária fria","color",null,null,"Nuances de packs e arquivos"],
    warm:["Luz secundária quente","color",null,null,"Nuances técnicas dos shaders"],
    contrast:["Contraste final",.6,1.5,.01,"Contraste fotométrico"],
    saturation:["Resposta da luz",.65,1.5,.01,"Nível geral dos tons claros"],
    speed:["Velocidade ambiental",0,2,.01,"Ritmo das ondas"],
    pulse:["Respiração de luz",0,1,.01,"Amplitude de oscilação"],
    parallax:["Paralaxe da iluminação",0,2,.01,"Quanto o toque move a luz"],
    reducedMotion:["Reduzir movimento","toggle",null,null,"Desativa animação temporal"],
    quality:["Resolução GPU",.45,1,.05,"Escala da renderização, textos permanecem nítidos"],
    fps:["FPS máximo","choice",[30,45,60,90,120],null,"Limite para economizar energia"]
  };
  const TABS=[
    {id:"Atmosfera",icon:"☁",intro:"Atmosfera real: gradiente escuro, luz difusa e ondas suaves.",groups:[
      ["Campo de fundo",["fog","waves","diffusion"]],
      ["Acabamento ambiental",["noise","vignette"]]]},
    {id:"Luz",icon:"☀",intro:"Iluminação direcional com resposta especular em tempo real.",groups:[
      ["Luz principal",["azimuth","elevation","intensity","ambient"]],
      ["Luz nos componentes",["edgeLight","heroGlow","bloom"]]]},
    {id:"Materiais",icon:"◈",intro:"Resposta física GGX: normal, micro-rugosidade e oclusão.",groups:[
      ["Acabamento",["roughness","metallic","reflectivity","bevel"]],
      ["Profundidade dos painéis",["depth","ao"]]]},
    {id:"Cores",icon:"◉",intro:"Somente modo escuro. Todos os matizes são editáveis.",groups:[
      ["Paleta Jade",["bg","surface","accent"]],
      ["Luzes auxiliares",["cool","warm"]],
      ["Tratamento final",["contrast","saturation"]]]},
    {id:"Componentes",icon:"▦",intro:"Ajuste dos materiais sobre os componentes reais da prévia.",groups:[
      ["Hero + botão principal",["heroGlow","depth","bevel"]],
      ["Cards + navegação",["ao","edgeLight","reflectivity","bloom"]]]},
    {id:"Movimento",icon:"≋",intro:"Ondas lentas e resposta ao foco sem partículas invasivas.",groups:[
      ["Comportamento",["speed","pulse","parallax"]],
      ["Acessibilidade",["reducedMotion"]]]},
    {id:"Desempenho",icon:"▤",intro:"Controle do custo na GPU sem diminuir a nitidez dos textos.",groups:[
      ["Renderização",["quality","fps"]],
      ["Economia de energia",["reducedMotion","waves","bloom"]]]},
    {id:"Exportar",icon:"↓",intro:"Todos os controles e abas são exportáveis. Sem internet.",groups:[]}
  ];
  const el=(id)=>document.getElementById(id);
  const clone=(obj)=>JSON.parse(JSON.stringify(obj));
  const validColor=(v)=>typeof v==="string"&&/^#[0-9a-fA-F]{6}$/.test(v);
  const validValue=(key,value)=>{
    const spec=SPECS[key]; if(!spec)return undefined;
    if(spec[1]==="color")return validColor(value)?value.toLowerCase():undefined;
    if(spec[1]==="toggle")return typeof value==="boolean"?value:undefined;
    if(spec[1]==="choice")return spec[2].includes(Number(value))?Number(value):undefined;
    const number=Number(value);
    if(!Number.isFinite(number))return undefined;
    return Math.min(spec[2],Math.max(spec[1],number));
  };
  const PARAM_KEYS=Object.keys(SPECS);
  let state=clone(DEFAULTS),tabId="Atmosfera",page="Play",paused=false,toastId=0;
  const groupOpen={},storeKey="horizon.jade.lab.dark.v1";
  function sanitizeName(name) {
    const clean=String(name||"jade-lab").replace(/[^a-zA-Z0-9._-]/g,"-").replace(/^-+|-+$/g,"").slice(0,64);
    return clean||"jade-lab";
  }
  function toast(message) {
    const node=el("toast");if(!node)return;
    node.textContent=String(message);node.classList.add("show");
    clearTimeout(toastId);toastId=setTimeout(()=>node.classList.remove("show"),2800);
  }
  function safeSave() {
    try{localStorage.setItem(storeKey,JSON.stringify(state));}catch(e){}
  }
  function restore() {
    try {
      const saved=JSON.parse(localStorage.getItem(storeKey)||"null");
      if(saved&&typeof saved==="object") Object.keys(DEFAULTS).forEach(k=>{
        const v=validValue(k,saved[k]);if(v!==undefined)state[k]=v;
      });
    }catch(e){}
  }
  function sync() {
    window.JadeRenderer.setPreset(state);
    document.documentElement.style.setProperty("--accent",state.accent);
    const preview=el("preview");
    if(preview){preview.dataset.mode="dark";}
    const badge=el("live-hint");
    if(badge)badge.textContent="LIVE GPU · DARK · "+state.fps+" FPS";
    safeSave();
  }
  function formatValue(key,value) {
    if(key==="azimuth")return Math.round(value)+"°";
    if(key==="fps")return value+" FPS";
    if(key==="quality")return Math.round(value*100)+"%";
    if(typeof value==="number")return value.toFixed((SPECS[key][3]||1)>=1?0:2).replace(/0+$/,"").replace(/\.$/,"");
    return value===true?"ON":"OFF";
  }
  function escapeHtml(s) {
    return String(s).replace(/&/g,"&amp;").replace(/</g,"&lt;")
      .replace(/>/g,"&gt;").replace(/"/g,"&quot;");
  }
  function field(key) {
    const spec=SPECS[key],val=state[key],label=escapeHtml(spec[0]),hint=escapeHtml(spec[4]);
    let control="";
    if(spec[1]==="color"){
      control='<input aria-label="'+label+'" type="color" data-param="'+key+'" value="'+val+'">';
    }else if(spec[1]==="toggle"){
      control='<select data-param="'+key+'" aria-label="'+label+'"><option value="false"'+(!val?' selected':'')+'>Desligado</option><option value="true"'+(val?' selected':'')+'>Ligado</option></select>';
    }else if(spec[1]==="choice"){
      control='<select data-param="'+key+'" aria-label="'+label+'">'+
        spec[2].map(v=>'<option value="'+v+'"'+(v===val?' selected':'')+'>'+v+' FPS</option>').join("")+'</select>';
    }else{
      control='<input aria-label="'+label+'" type="range" data-param="'+key+
        '" min="'+spec[1]+'" max="'+spec[2]+'" step="'+spec[3]+'" value="'+val+'">';
    }
    return '<div class="field"><div class="field-head"><label>'+label+'</label>'+
      '<output data-readout="'+key+'">'+escapeHtml(formatValue(key,val))+'</output></div>'+
      control+'<small class="hint">'+hint+'</small></div>';
  }
  function renderTabs() {
    const node=el("editor-tabs");node.innerHTML=TABS.map(t=>
      '<button type="button" role="tab" aria-selected="'+(tabId===t.id)+'" class="'+(tabId===t.id?'active':'')+
      '" data-tab="'+t.id+'">'+t.icon+' '+t.id+'</button>').join("");
  }
  function renderPanel() {
    renderTabs();
    const node=el("editor-body");
    const section=TABS.find(t=>t.id===tabId);
    if(tabId==="Exportar"){
      node.innerHTML='<p class="section-intro">'+section.intro+'</p>'+
        '<div class="action-grid">'+
        '<button data-export="preset">↓ <b>Preset completo</b><small class="hint">JSON · todos os controles</small></button>'+
        '<button data-export="pack">↓ <b>Pack completo</b><small class="hint">ZIP · shaders, paleta, parâmetros e prévia</small></button>'+
        '<button data-export="image">↓ <b>Screenshot GPU</b><small class="hint">PNG · fundo e painéis</small></button>'+
        '<button data-export="import">↑ <b>Importar preset</b><small class="hint">JSON ou ZIP deste laboratório</small></button>'+
        '</div><button class="big-action" data-export="section">↓ BAIXAR A ABA ATUAL (EXPORTAR)</button>'+
        '<button class="big-action" data-export="reset">↺ Restaurar valores de fábrica</button>'+
        '<p class="note">O pack inclui o shader GLSL realmente executado pelo aplicativo, os parâmetros e arquivos por aba. Não é um mod carregável diretamente no Horizon: é o pacote de referência para a integração nativa.</p>'+
        '<p class="note">Importação: arquivos JSON e ZIP gerados pelo Jade Lab. O modo escuro permanece fixo.</p>';
      return;
    }
    node.innerHTML='<p class="section-intro">'+section.intro+'</p>'+
      section.groups.map((g,i)=>{
        const key=tabId+"/"+i;const open=groupOpen[key]!==false;
        return '<details class="parameter-group" data-group="'+key+'"'+(open?' open':'')+'>'+
          '<summary>'+escapeHtml(g[0])+'</summary><div class="fields">'+g[1].map(field).join("")+'</div></details>';
      }).join("")+
      '<button class="big-action" data-export="section">↓ Exportar somente '+escapeHtml(tabId)+' (.json)</button>';
  }
  function changeTab(next) {
    if(!TABS.some(t=>t.id===next))return;
    tabId=next;renderPanel();el("editor-body").scrollTop=0;
  }
  function setField(key,value) {
    const next=validValue(key,value);
    if(next===undefined)return;
    state[key]=next;
    const readouts=document.querySelectorAll('[data-readout="'+key+'"]');
    readouts.forEach(o=>{o.textContent=String(formatValue(key,next));});
    sync();
  }
  function presetObject() {
    return {format:FORMAT,preset:"Horizon Jade",theme:"oldschool-dark",
      engine:"WebGL2",createdAt:new Date().toISOString(),controls:clone(state)};
  }
  function sectionObject(id) {
    const t=TABS.find(x=>x.id===id)||TABS[0];
    const keys=[...new Set(t.groups.flatMap(g=>g[1]))];
    const values={};keys.forEach(k=>values[k]=state[k]);
    return {format:"HORIZON_JADE_TAB_V1",theme:"oldschool-dark",tab:t.id,
      createdAt:new Date().toISOString(),controls:values};
  }
  function textBytes(text){return new TextEncoder().encode(text);}
  const CRC_TABLE=(()=>{
    const t=new Uint32Array(256);
    for(let n=0;n<256;n++){let c=n;for(let k=0;k<8;k++)c=c&1?(0xedb88320^(c>>>1)):(c>>>1);t[n]=c>>>0;}
    return t;
  })();
  function crc32(bytes){let c=0xffffffff;for(const b of bytes)c=CRC_TABLE[(c^b)&255]^(c>>>8);return (c^0xffffffff)>>>0;}
  function write16(b,i,n){b[i]=n&255;b[i+1]=n>>>8&255;}
  function write32(b,i,n){write16(b,i,n);write16(b,i+2,n>>>16);}
  function zipStore(entries) {
    let offset=0;const local=[],central=[];
    for(const item of entries){
      const name=textBytes(item.name),data=item.bytes,crc=crc32(data);
      const localHeader=new Uint8Array(30+name.length);
      write32(localHeader,0,0x04034b50);write16(localHeader,4,20);
      write16(localHeader,6,0x0800);write16(localHeader,8,0);
      write32(localHeader,14,crc);write32(localHeader,18,data.length);
      write32(localHeader,22,data.length);write16(localHeader,26,name.length);
      localHeader.set(name,30);local.push(localHeader,data);
      const cd=new Uint8Array(46+name.length);
      write32(cd,0,0x02014b50);write16(cd,4,20);write16(cd,6,20);
      write16(cd,8,0x0800);write32(cd,16,crc);
      write32(cd,20,data.length);write32(cd,24,data.length);
      write16(cd,28,name.length);write32(cd,42,offset);cd.set(name,46);
      central.push(cd);offset+=localHeader.length+data.length;
    }
    const cdSize=central.reduce((n,x)=>n+x.length,0);
    const end=new Uint8Array(22);write32(end,0,0x06054b50);
    write16(end,8,entries.length);write16(end,10,entries.length);
    write32(end,12,cdSize);write32(end,16,offset);
    return new Blob([...local,...central,end],{type:"application/zip"});
  }
  function findStoredZipPreset(bytes) {
    if(bytes.length>16*1024*1024)throw new Error("O ZIP ultrapassa o limite de 16 MB");
    const view=new DataView(bytes.buffer,bytes.byteOffset,bytes.byteLength);
    let at=0;
    for(let i=0;i<160 && at+30<=bytes.length;i++){
      if(view.getUint32(at,true)!==0x04034b50)break;
      const method=view.getUint16(at+8,true),flags=view.getUint16(at+6,true);
      const size=view.getUint32(at+18,true);
      const n=view.getUint16(at+26,true),e=view.getUint16(at+28,true);
      const nameAt=at+30,begin=nameAt+n+e,end=begin+size;
      if(end>bytes.length || flags&8)throw new Error("ZIP inválido");
      const name=new TextDecoder().decode(bytes.slice(nameAt,nameAt+n));
      if(name==="preset.json"){
        if(method!==0 || size>512*1024)throw new Error("Formato de ZIP não compatível");
        return JSON.parse(new TextDecoder().decode(bytes.slice(begin,end)));
      }
      at=end;
    }
    throw new Error("preset.json não encontrado no pack");
  }
  function decodeDataUrl(url) {
    const b64=url.split(",")[1];
    const bin=atob(b64),out=new Uint8Array(bin.length);
    for(let i=0;i<bin.length;i++)out[i]=bin.charCodeAt(i);
    return out;
  }
  function base64FromBytes(bytes) {
    let s="";for(let i=0;i<bytes.length;i+=16384)
      s+=String.fromCharCode.apply(null,bytes.subarray(i,i+16384));
    return btoa(s);
  }
  async function saveBlob(filename,blob) {
    if(blob.size>18*1024*1024)throw new Error("Arquivo excede 18 MB");
    const bytes=new Uint8Array(await blob.arrayBuffer());
    if(window.AndroidBridge&&typeof AndroidBridge.saveBase64==="function"){
      AndroidBridge.saveBase64(sanitizeName(filename),blob.type||"application/octet-stream",
        base64FromBytes(bytes));
    }else{
      const url=URL.createObjectURL(blob),link=document.createElement("a");
      link.href=url;link.download=sanitizeName(filename);document.body.appendChild(link);
      link.click();link.remove();setTimeout(()=>URL.revokeObjectURL(url),2500);
    }
    toast("Arquivo enviado para Downloads/JadeLab: "+filename);
  }
  function fileJson(o){return new Blob([JSON.stringify(o,null,2)+"\n"],{type:"application/json"});}
  async function downloadPreset() {await saveBlob("horizon-jade-preset.json",fileJson(presetObject()));}
  async function downloadSection() {await saveBlob("jade-"+sanitizeName(tabId.toLowerCase())+".json",fileJson(sectionObject(tabId)));}
  async function downloadScreenshot() {
    const data=window.JadeRenderer.screenshot();
    if(!data)throw new Error("A GPU não está disponível para captura");
    await saveBlob("jade-shader-preview.png",new Blob([decodeDataUrl(data)],{type:"image/png"}));
  }
  async function downloadPack() {
    const json=(name,value)=>({name,bytes:textBytes(JSON.stringify(value,null,2)+"\n")});
    const entries=[json("preset.json",presetObject())];
    for(const t of TABS.filter(x=>x.id!=="Exportar")){
      entries.push(json("tabs/"+sanitizeName(t.id.toLowerCase())+".json",sectionObject(t.id)));
    }
    entries.push({name:"shaders/jade_scene.frag",bytes:textBytes(window.JadeRenderer.fragmentSource)});
    entries.push({name:"shaders/fullscreen.vert",bytes:textBytes(window.JadeRenderer.vertexSource)});
    entries.push(json("uniforms.json",{
      engine:"WebGL2 / GLSL ES 300",theme:"oldschool-dark",
      uniforms:["uAtmos","uLight","uMaterial","uSurface","uMotion","uDetail","uPalette","uRects"],
      controls:clone(state),note:"GPU rects use CSS viewport coordinates; see source app."
    }));
    entries.push(json("manifest.json",{
      name:"Horizon Jade Lab Pack",version:1,createdAt:new Date().toISOString(),
      source:"experiments/jade-lab",format:FORMAT,
      licence:"Original shaders and utility; see repository LICENSE",
      target:"Horizon Jade dark-only experimental laboratory"
    }));
    entries.push({name:"README.txt",bytes:textBytes(
      "HORIZON JADE LAB — DARK ONLY\n"+
      "Preset: preset.json. Each category: tabs/*.json.\n"+
      "GPU shader sources: shaders/*.frag, *.vert (WebGL2 / GLSL ES 300).\n"+
      "uniforms.json contains every live parameter used for this export.\n"+
      "This is an engineering reference pack, not a compiled Rust/WGSL mod.\n"+
      "The image preview is the GPU backdrop/material shader, not DOM text.\n"+
      "Import this ZIP back into Jade Lab using Import preset/pack.\n"
    )});
    try{
      const png=window.JadeRenderer.screenshot();
      if(png)entries.push({name:"preview.png",bytes:decodeDataUrl(png)});
    }catch(e){}
    const zip=zipStore(entries);
    await saveBlob("horizon-jade-complete-pack.zip",zip);
  }
  async function importFile(file) {
    if(!file)return;
    if(file.size>16*1024*1024)throw new Error("Arquivo muito grande");
    let incoming;
    if(file.name.toLowerCase().endsWith(".zip")){
      incoming=findStoredZipPreset(new Uint8Array(await file.arrayBuffer()));
    }else incoming=JSON.parse(await file.text());
    if(!incoming || !["HORIZON_JADE_TAB_V1",FORMAT].includes(incoming.format)
       || incoming.theme!=="oldschool-dark" || typeof incoming.controls!=="object")
      throw new Error("Preset incompatível com o Jade Lab Dark");
    const updates={};
    for(const k of PARAM_KEYS){
      if(!(k in incoming.controls))continue;
      const v=validValue(k,incoming.controls[k]);
      if(v===undefined)throw new Error("Parâmetro inválido: "+k);
      updates[k]=v;
    }
    if(!Object.keys(updates).length)throw new Error("Nenhuma configuração reconhecida");
    Object.assign(state,updates);sync();renderPanel();
    toast("Preset importado: "+Object.keys(updates).length+" parâmetros");
  }
  async function doExport(command) {
    try{
      if(command==="preset")await downloadPreset();
      else if(command==="section")await downloadSection();
      else if(command==="pack")await downloadPack();
      else if(command==="image")await downloadScreenshot();
      else if(command==="import")el("import-file").click();
      else if(command==="reset"){
        if(!confirm("Restaurar a paleta e todos os parâmetros de fábrica?"))return;
        Object.assign(state,clone(DEFAULTS));sync();renderPanel();toast("Configurações restauradas");
      }
    }catch(e){toast("Erro: "+(e.message||e));}
  }
  function setupEditor() {
    const editor=el("editor");const body=el("editor-body");const head=el("drag-handle");
    // Start collapsed to maximize usable viewport. A simple floating button
    // can always restore the sheet, including on portrait phones.
    editor.classList.add("collapsed");
    el("open-editor").addEventListener("click",()=>{editor.classList.remove("hidden","collapsed");});
    el("quick-settings").addEventListener("click",()=>{editor.classList.remove("hidden","collapsed");});
    el("minimize-editor").addEventListener("click",()=>editor.classList.toggle("collapsed"));
    el("hide-editor").addEventListener("click",()=>editor.classList.add("hidden"));
    el("maximize-editor").addEventListener("click",()=>{
      editor.classList.remove("hidden","collapsed");
      const isFull=editor.classList.toggle("maximized");
      editor.style.setProperty("--sheet-height",isFull?"calc(100dvh - 24px)":"min(540px,76dvh)");
    });
    let drag=null;
    head.addEventListener("pointerdown",e=>{
      if(e.target.closest("button"))return;
      drag={y:e.clientY,height:editor.getBoundingClientRect().height};
      editor.classList.remove("collapsed");
      head.setPointerCapture(e.pointerId);
    });
    head.addEventListener("pointermove",e=>{
      if(!drag)return;
      const h=Math.max(185,Math.min(innerHeight-15,drag.height+drag.y-e.clientY));
      editor.style.setProperty("--sheet-height",h+"px");
      editor.classList.remove("maximized");
    });
    const stop=()=>{drag=null;};
    head.addEventListener("pointerup",stop);head.addEventListener("pointercancel",stop);
    el("editor-tabs").addEventListener("click",e=>{
      const button=e.target.closest("[data-tab]");
      if(button)changeTab(button.dataset.tab);
    });
    body.addEventListener("input",e=>{
      const key=e.target.dataset.param;
      if(key && e.target.type==="range")setField(key,e.target.value);
      if(key && e.target.type==="color")setField(key,e.target.value);
    });
    body.addEventListener("change",e=>{
      const key=e.target.dataset.param;
      if(key) setField(key,e.target.type==="color"?e.target.value:
        (SPECS[key][1]==="toggle"?e.target.value==="true":e.target.value));
    });
    body.addEventListener("toggle",e=>{
      if(e.target.matches&&e.target.matches("details[data-group]"))
        groupOpen[e.target.dataset.group]=e.target.open;
    },true);
    body.addEventListener("click",e=>{
      const button=e.target.closest("[data-export]");
      if(button)doExport(button.dataset.export);
    });
    el("download-section").addEventListener("click",()=>doExport("section"));
    el("save-preset").addEventListener("click",()=>doExport("preset"));
    el("download-pack").addEventListener("click",()=>doExport("pack"));
    el("import-file").addEventListener("change",async e=>{
      try{await importFile(e.target.files&&e.target.files[0]);}
      catch(err){toast("Importação: "+(err.message||err));}
      e.target.value="";
    });
    el("toggle-compare").addEventListener("click",e=>{
      const enabled=document.body.classList.toggle("comparison");
      e.currentTarget.setAttribute("aria-pressed",String(enabled));
      toast(enabled?"Comparação: superfície simples":"Shader e materiais reativados");
    });
    el("toggle-pause").addEventListener("click",e=>{
      paused=!paused;window.JadeRenderer.pause(paused);
      e.currentTarget.textContent=paused?"▶":"Ⅱ";
      toast(paused?"Animações pausadas":"Animações em execução");
    });
    el("reset-view").addEventListener("click",()=>{
      window.JadeRenderer.recenter();toast("Luz centralizada");
    });
    document.addEventListener("keydown",e=>{
      if(e.key==="Escape" && !editor.classList.contains("collapsed")){
        editor.classList.add("collapsed");e.preventDefault();
      }
    });
  }
  const PAGE_DATA={
    Play:["READY TO PLAY","MINECRAFT","JAVA EDITION 26.3","● Install needed","Install Minecraft"],
    Mods:["EXTENSIONS","MODS","NATIVE + HNMOD","● Presets and projects","Open Mods"],
    Packs:["ASSETS","RESOURCE PACKS","TEXTURES · GUI · ICONS","● Live preview","Open Packs"],
    Shaders:["VISUALS","SHADERS","GPU EFFECTS · WEBGL2","● Material lab","Open Shaders"],
    Files:["STORAGE","MY FILES","FILE MANAGER","● Local packages","Open Files"],
    Settings:["SYSTEM","SETTINGS","PERFORMANCE · GRAPHICS","● Lab controls","Open Settings"],
    Skins:["IDENTITY","SKINS","PLAYER & APPEARANCE","● Profile preview","Open Skins"]
  };
  function pageSelect(next) {
    if(!(next in PAGE_DATA))return;
    page=next;
    const x=PAGE_DATA[page];
    el("hero-eyebrow").textContent=x[0];el("hero-title").textContent=x[1];
    el("hero-subtitle").textContent=x[2];el("hero-status").textContent=x[3];
    el("hero-button-label").textContent=x[4];
    document.querySelectorAll(".channel").forEach(btn=>{
      const current=btn.dataset.page===page;
      btn.classList.toggle("active",current);
      if(current)btn.setAttribute("aria-current","page");else btn.removeAttribute("aria-current");
    });
  }
  function init() {
    restore();sync();renderPanel();setupEditor();
    window.JadeRenderer.init();
    document.querySelectorAll("[data-page]").forEach(button=>
      button.addEventListener("click",()=>pageSelect(button.dataset.page)));
    el("hero-action").addEventListener("click",()=>toast("Prévia de "+page+"; este laboratório não instala o jogo"));
    if(window.JadeRenderer.state().error)toast(window.JadeRenderer.state().error);
    // Native Android may invoke this after entering immersive mode.
    window.addEventListener("pageshow",()=>window.JadeRenderer.refresh());
  }
  window.JadeLab={getPreset:presetObject,getSection:sectionObject,defaults:()=>clone(DEFAULTS),
    exportPreset:downloadPreset,exportPack:downloadPack,exportSection:downloadSection,
    importFile,setField,changeTab,zipStore,findStoredZipPreset};
  if(document.readyState==="loading")document.addEventListener("DOMContentLoaded",init);
  else init();
})();