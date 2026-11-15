/* Tactile spatial navigation for the original Jade deck. Selection, activation
 * and return are distinct actions. Modules are offline laboratory interfaces. */
(function(){
  'use strict';
  const PAGES=['Play','Mods','Packs','Shaders','Files','Skins','Settings'];
  const META={
    Play:{label:'Jogar',eyebrow:'SEU PRÓXIMO MUNDO',title:'PRONTO PARA JOGAR',summary:'Escolha um perfil e prepare a sua próxima sessão.',accent:'#9cd4b6',counter:'#b9c5e2'},
    Mods:{label:'Mods',eyebrow:'MONTE A SUA EXPERIÊNCIA',title:'SUA BIBLIOTECA',summary:'Explore categorias e organize a seleção de mods.',accent:'#8bd8b3',counter:'#d8af88'},
    Packs:{label:'Pacotes',eyebrow:'TEXTURAS E IDENTIDADE',title:'OUTRO OLHAR',summary:'Camadas de cor, materiais e uma nova atmosfera.',accent:'#a6bddf',counter:'#d6b895'},
    Shaders:{label:'Shaders',eyebrow:'UM ESTÚDIO DE LUZ',title:'SINTA A MATÉRIA',summary:'Reflexos, acabamento e iluminação em tempo real.',accent:'#a2d3df',counter:'#dfa7b5'},
    Files:{label:'Arquivos',eyebrow:'SEU ACERVO PESSOAL',title:'TUDO CONECTADO',summary:'Importe e exporte os presets deste laboratório.',accent:'#b5cbd5',counter:'#d7c497'},
    Skins:{label:'Skins',eyebrow:'PRESENÇA E PERSONALIDADE',title:'ENTRE EM CENA',summary:'A aparência do personagem de demonstração.',accent:'#d3c7ae',counter:'#98cbd3'},
    Settings:{label:'Sistema',eyebrow:'CADA DETALHE É SEU',title:'SEU EQUILÍBRIO',summary:'Aparência, desempenho e movimento.',accent:'#c4d0d5',counter:'#b7c5a0'}
  };
  function createModel(){
    let index=0,location='home';
    return {state:()=>({page:PAGES[index],index,location}),
      select(name){const next=PAGES.indexOf(name);if(next<0)return false;index=next;return true;},
      step(delta){index=(index+delta%7+7)%7;return PAGES[index];},
      open(){location='module';return PAGES[index];},
      close(){if(location==='module'){location='home';return true;}return false;},
      back(){if(this.close())return true;if(index!==0){index=0;return true;}return false;}
    };
  }
  function releaseStep(distance,velocity){return Math.abs(distance)>=56||(Math.abs(distance)>=20&&Math.abs(velocity)>=.5)?(distance<0?1:-1):0;}
  function deckOrder(index,direction=1){return [0,1,2].map(i=>PAGES[(index+i*direction+14)%7]);}
  const MODS=[
    {id:'presence',name:'Primeira pessoa',category:'Imersão',icon:'◈',description:'Presença e corpo na exploração.',size:'PERSONAGEM'},
    {id:'resonance',name:'Áudio espacial',category:'Som',icon:'≋',description:'Uma camada de profundidade sonora.',size:'AMBIENTE'},
    {id:'camera',name:'Câmera dinâmica',category:'Imersão',icon:'◉',description:'Movimento e sensação de espaço.',size:'MOVIMENTO'},
    {id:'light',name:'Luz atmosférica',category:'Visual',icon:'✧',description:'Contraste, névoa e materiais.',size:'ILUMINAÇÃO'}
  ];
  let model=createModel(),callbacks={},initialized=false,gesture=null,suppressUntil=0,phase='home',phaseAt=0,transitionTimer=0,filter='Todos',query='';
  let selectedMods=new Set(['presence','resonance']),profile='Perfil 1',jacket='theme',status='';
  const $=id=>document.getElementById(id),clamp=(x,a,b)=>Math.max(a,Math.min(b,x));
  const fold=s=>String(s).normalize('NFD').replace(/[\u0300-\u036f]/g,'').toLowerCase();
  const escape=s=>String(s).replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
  const surface=(kind='panel')=>'data-spatial-surface="'+kind+'"';
  const button=(text,action,extra='')=>'<button class="spatial-button" '+surface('button')+' data-module-action="'+action+'" '+extra+'>'+text+'</button>';
  const preset=()=>window.JadeLab?.getPreset?.().controls||{};
  function save(){try{localStorage.setItem('jade.spatial.lab.samples.v1',JSON.stringify({mods:[...selectedMods],profile,jacket}));}catch(e){}}
  function restore(){try{const s=JSON.parse(localStorage.getItem('jade.spatial.lab.samples.v1')||'null');if(s){selectedMods=new Set((s.mods||[]).filter(id=>MODS.some(m=>m.id===id)));if(['Perfil 1','Perfil 2'].includes(s.profile))profile=s.profile;if(['theme','silver','copper'].includes(s.jacket))jacket=s.jacket;}}catch(e){}}
  function refresh(){window.JadeDeckFinish?.register?.();window.JadeDeckFinish?.changed();window.JadeRenderer?.refresh?.();}
  function reduced(){return !!preset().reducedMotion||window.matchMedia?.('(prefers-reduced-motion: reduce)').matches;}
  function updateIndicator(){const s=model.state();$('scene-position').textContent=String(s.index+1).padStart(2,'0')+' / 07';$('deck-current-label').textContent=META[s.page].label;}
  function selected(name){model.select(name);updateIndicator();}
  function select(name){if(!model.select(name))return;callbacks.select?.(name);updateIndicator();closePicker();refresh();}
  function step(delta){select(model.step(delta));}
  function preview(direction){
    const order=deckOrder(model.state().index,direction);
    document.querySelectorAll('.cards .tile').forEach(tile=>{const at=order.indexOf(tile.dataset.page);tile.dataset.stack=at<0?'hidden':['front','middle','rear'][at];tile.setAttribute('aria-hidden',String(at<0));tile.tabIndex=at<0?-1:0;});
    refresh();
  }
  function setDrag(dx,dy){const style=$('console-scene').style;style.setProperty('--gesture-x',clamp(dx,-150,150)+'px');style.setProperty('--gesture-y',clamp(dy,-150,150)+'px');style.setProperty('--gesture-turn',clamp(dx*.075,-12,12)+'deg');}
  function resetDrag(commit=0){
    const scene=$('console-scene');scene.classList.remove('is-dragging');
    if(commit)step(commit);else callbacks.select?.(model.state().page);
    setDrag(0,0);refresh();
  }
  function pointInQuad(p,x,y){
    if(p.length!==4)return false;let sign=0;
    for(let i=0;i<4;i++){const a=p[i],b=p[(i+1)%4],cross=(b[0]-a[0])*(y-a[1])-(b[1]-a[1])*(x-a[0]);if(Math.abs(cross)<.00001)continue;const next=Math.sign(cross);if(sign&&next!==sign)return false;sign=next;}
    return sign!==0;
  }
  function hitTest(x,y){
    // CSS negative-Z planes can paint behind their parent's native hit plane.
    // Route input against the same projected corners that the GPU draws.
    for(const stack of ['front','middle','rear']){
      const el=document.querySelector('.tile[data-stack="'+stack+'"]');if(!el)continue;
      const p=Array.from(el.querySelectorAll(':scope > .deck-corner')).slice(0,4).map(m=>{const r=m.getBoundingClientRect();return[r.x,r.y];});
      if(pointInQuad(p,x,y))return el.dataset.page;
    }
    return null;
  }
  function pointerDown(e){
    if(model.state().location!=='home'||gesture||!e.isPrimary||e.button!==0||e.target.closest('#deck-picker,.deck-access,.hero,.floating-tools'))return;
    const target=hitTest(e.clientX,e.clientY);if(!target)return;
    e.preventDefault();
    gesture={id:e.pointerId,x:e.clientX,y:e.clientY,lastX:e.clientX,lastY:e.clientY,lastAt:performance.now(),velocity:0,axis:null,moved:false,direction:1,target};
    $('console-scene').setPointerCapture(e.pointerId);
  }
  function pointerMove(e){
    if(!gesture||gesture.id!==e.pointerId)return;
    const g=gesture,dx=e.clientX-g.x,dy=e.clientY-g.y;
    if(!g.axis&&Math.max(Math.abs(dx),Math.abs(dy))>8){g.axis=Math.abs(dx)>Math.abs(dy)?'x':'y';g.moved=true;$('console-scene').classList.add('is-dragging');$('console-scene').setPointerCapture(e.pointerId);}
    if(!g.axis)return;
    e.preventDefault();const now=performance.now(),distance=g.axis==='x'?dx:dy;
    const lastDelta=g.axis==='x'?e.clientX-g.lastX:e.clientY-g.lastY;
    g.velocity=lastDelta/Math.max(1,now-g.lastAt);g.lastAt=now;g.lastX=e.clientX;g.lastY=e.clientY;
    const direction=distance<0?1:-1;if(direction!==g.direction){g.direction=direction;preview(direction);}
    setDrag(g.axis==='x'?dx:dx*.12,g.axis==='y'?dy:dy*.12);window.JadeDeckFinish?.changed();
  }
  function pointerUp(e,cancelled=false){
    if(!gesture||gesture.id!==e.pointerId)return;const g=gesture;gesture=null;
    if(!g.moved&&!cancelled){select(g.target);suppressUntil=performance.now()+300;}
    if(g.moved){suppressUntil=performance.now()+400;const distance=g.axis==='x'?e.clientX-g.x:e.clientY-g.y;const velocity=performance.now()-g.lastAt<100?g.velocity:0;resetDrag(cancelled?0:releaseStep(distance,velocity));}
    if($('console-scene').hasPointerCapture?.(e.pointerId))$('console-scene').releasePointerCapture(e.pointerId);
  }
  function openPicker(){
    if(model.state().location!=='home')return;
    $('deck-picker').hidden=false;$('deck-picker').inert=false;document.body.classList.add('is-picking');document.querySelectorAll('.cards,.hero,.avatar-stage,.deck-access,.topbar').forEach(e=>e.inert=true);
    $('deck-picker').querySelector('[data-pick="'+model.state().page+'"]').focus({preventScroll:true});refresh();
  }
  function closePicker(){const el=$('deck-picker');if(el){el.hidden=true;el.inert=true;document.body.classList.remove('is-picking');document.querySelectorAll('.cards,.hero,.avatar-stage,.deck-access,.topbar').forEach(e=>e.inert=false);}}
  function moduleHeader(name){const m=META[name];return '<header class="module-heading"><div><small>'+m.eyebrow+'</small><h1 id="module-title">'+m.label+'</h1></div>'+button('Voltar ao deck','back','aria-label="Voltar ao deck"')+'</header>';}
  function lead(name,extra=''){const m=META[name];return '<section class="module-lead spatial-panel" '+surface()+'><span class="module-code">'+String(PAGES.indexOf(name)+1).padStart(2,'0')+' / HORIZON</span><h2>'+m.title+'</h2><p>'+m.summary+'</p>'+extra+'</section>';}
  function modRows(){
    return MODS.filter(m=>(filter==='Todos'||m.category===filter)&&fold(m.name).includes(fold(query))).map(m=>'<article class="mod-row spatial-panel" '+surface('card')+'><span class="module-object" aria-hidden="true">'+m.icon+'</span><div><small>'+m.category+' · '+m.size+'</small><h3>'+m.name+'</h3><p>'+m.description+'</p></div>'+button(selectedMods.has(m.id)?'Na seleção':'Selecionar','mod:'+m.id,'aria-pressed="'+selectedMods.has(m.id)+'" aria-label="Selecionar '+m.name+'"')+'</article>').join('')||'<p class="module-empty">Nenhum exemplo nesta busca.</p>';
  }
  function mods(){return '<div class="module-layout mods-layout">'+lead('Mods','<span class="module-sigil" aria-hidden="true">✣</span><small class="sample-label">EXEMPLOS DE INTERFACE</small>')+'<section class="module-main"><div class="module-filter"><input id="mod-search" '+surface('button')+' aria-label="Buscar exemplos de mods" placeholder="Buscar na biblioteca" value="'+escape(query)+'"><div class="module-tags">'+['Todos','Imersão','Visual','Som'].map(f=>button(f,'filter:'+f,'aria-pressed="'+(filter===f)+'"')).join('')+'</div></div><div class="module-scroll" id="mod-list">'+modRows()+'</div></section><aside class="module-aside spatial-panel" '+surface()+'><small>SUA COMBINAÇÃO</small><strong id="mod-count">'+selectedMods.size+'</strong><p>exemplos na seleção</p><div class="selection-orbit" aria-hidden="true"><i></i><i></i><i></i></div>'+button('Limpar seleção','clear-mods')+'<small class="sample-note">A seleção organiza a prévia do laboratório.</small></aside></div>';}
  function packs(){return '<div class="module-layout packs-layout">'+lead('Packs','<span class="module-sigil" aria-hidden="true">⬡</span>')+'<section class="pack-gallery module-scroll">'+[['jade','Jade','Orgânico e acetinado'],['azure','Azure','Frio e cristalino'],['obsidian','Obsidian','Profundo e perolado']].map(([id,title,sub])=>'<button class="pack-card spatial-panel" '+surface('card')+' data-module-action="palette:'+id+'"><span class="pack-swatch swatch-'+id+'" aria-hidden="true"></span><small>APARÊNCIA DO LAB</small><h3>'+title+'</h3><p>'+sub+'</p></button>').join('')+'</section><aside class="module-aside spatial-panel" '+surface()+'><small>SUAS CAMADAS</small><h3>Leve a aparência com você.</h3><p>Presets do laboratório em JSON e ZIP.</p>'+button('Importar preset','import')+button('Exportar pack','pack')+'</aside></div>';}
  function shaders(){return '<div class="module-layout shaders-layout">'+lead('Shaders','<span class="module-sigil" aria-hidden="true">✧</span>')+'<section class="shader-stage spatial-panel" '+surface()+'><div class="material-sculpture" aria-hidden="true"><i></i><b></b><span></span></div><div class="module-tags">'+button('Frosted','material:frosted')+button('Fosco','material:matte')+button('Sólido','material:solid')+'</div><small id="material-name">'+escape(preset().glassStyle)+'</small></section><aside class="module-aside spatial-panel" '+surface()+'><small>ESTÚDIO DA CENA</small><h3>Ajuste a luz.</h3><p>O acabamento acompanha os controles reais do laboratório.</p>'+button('Iluminação','editor:Luz')+button('Materiais','editor:Materiais')+button('Ondas e fundo','editor:Fundo')+'</aside></div>';}
  function files(){const p=preset();return '<div class="module-layout files-layout">'+lead('Files','<span class="module-sigil" aria-hidden="true">▱</span>')+'<section class="file-cabinet spatial-panel" '+surface()+'><small>PRESET ATUAL</small><div class="file-document" aria-hidden="true"><span>JADE</span><i></i><i></i><i></i></div><h3>horizon-jade-preset.json</h3><dl><div><dt>Paleta</dt><dd>'+escape(p.paletteId)+'</dd></div><div><dt>Material</dt><dd>'+escape(p.glassStyle)+'</dd></div><div><dt>Limite</dt><dd>'+p.fps+' FPS</dd></div></dl></section><aside class="module-aside spatial-panel" '+surface()+'><small>ARQUIVOS DO LABORATÓRIO</small>'+button('Importar JSON / ZIP','import')+button('Exportar preset','preset')+button('Exportar pack completo','pack')+'<p>Os arquivos exportados são salvos em Downloads/JadeLab.</p></aside></div>';}
  function skins(){return '<div class="module-layout skins-layout">'+lead('Skins','<small class="sample-label">PERSONAGEM DE DEMONSTRAÇÃO</small>')+'<section class="identity-stage spatial-panel" '+surface()+'><div class="module-avatar">'+document.querySelector('.avatar-stage').innerHTML+'</div><div class="module-tags">'+button('Tema','jacket:theme')+button('Prata','jacket:silver')+button('Cobre','jacket:copper')+'</div></section><aside class="module-aside spatial-panel" '+surface()+'><small>SEU PERSONAGEM</small><h3>Presença na cena.</h3><p>Escolha um acabamento para o personagem desta prévia.</p>'+button('Luz do personagem','editor:Luz')+button('Movimento','editor:Componentes')+'</aside></div>';}
  function settings(){const p=preset();return '<div class="module-layout settings-layout">'+lead('Settings')+'<section class="settings-stations module-scroll"><article class="setting-station spatial-panel" '+surface('card')+'><span class="module-object">◈</span><div><h3>Aparência</h3><p>Paleta, metais e vidro.</p></div>'+button('Personalizar','editor:Paletas')+'</article><article class="setting-station spatial-panel" '+surface('card')+'><span class="module-object">▤</span><div><h3>Desempenho</h3><p>Limite de quadros e escala da GPU.</p></div><div class="module-tags">'+[30,45,60].map(f=>button(f+' FPS','fps:'+f,'aria-pressed="'+(p.fps===f)+'"')).join('')+'</div></article><article class="setting-station spatial-panel" '+surface('card')+'><span class="module-object">≋</span><div><h3>Movimento</h3><p>Controle o ritmo do ambiente.</p></div>'+button(p.reducedMotion?'Movimento reduzido':'Movimento completo','motion','aria-pressed="'+p.reducedMotion+'"')+'</article></section><aside class="module-aside spatial-panel" '+surface()+'><small>AJUSTES FINOS</small><p>Explore cada parâmetro e exporte sua aparência.</p>'+button('Abrir editor','editor:Desempenho')+button('Salvar preset','preset')+'</aside></div>';}
  function play(){return '<div class="module-layout play-layout">'+lead('Play','<span class="module-sigil" aria-hidden="true">▶</span>')+'<section class="session-stage spatial-panel" '+surface()+'><small>JAVA EDITION</small><h2>MINECRAFT</h2><div class="world-horizon" aria-hidden="true"><i></i><b></b><span></span></div><p>26.3 · '+profile+'</p><div class="module-tags">'+['Perfil 1','Perfil 2'].map(p=>button(p,'profile:'+p,'aria-pressed="'+(profile===p)+'"')).join('')+'</div></section><aside class="module-aside spatial-panel" '+surface()+'><small>PRÉVIA DA SESSÃO</small><h3>Seu ponto de partida.</h3><p>Esta tela demonstra a experiência de abertura. O jogo é iniciado pelo Horizon principal.</p>'+button('Prévia de abertura','session')+'</aside></div>';}
  const content={Play:play,Mods:mods,Packs:packs,Shaders:shaders,Files:files,Skins:skins,Settings:settings};
  function renderModule(){const name=model.state().page,m=META[name],root=$('module-screen');root.dataset.module=name;root.style.setProperty('--module-accent',m.accent);root.style.setProperty('--module-counter',m.counter);$('module-frame').innerHTML=moduleHeader(name)+content[name]()+'<footer class="module-footer"><span>'+m.label.toUpperCase()+' / HORIZON JADE</span><span id="module-status" role="status">'+escape(status)+'</span></footer>';document.body.dataset.jacket=jacket;refresh();}
  function open(name=model.state().page){
    if(!PAGES.includes(name))return;
    clearTimeout(transitionTimer);select(name);model.open();phase='entering';phaseAt=performance.now();status='';renderModule();
    const root=$('module-screen');root.hidden=false;root.inert=false;$('console-scene').inert=true;
    document.body.dataset.module=name;document.body.classList.add('has-module');root.dataset.phase='entering';refresh();
    requestAnimationFrame(()=>{if(model.state().location==='module')root.dataset.phase='open';});
    transitionTimer=setTimeout(()=>{if(model.state().location==='module'){phase='open';root.querySelector('[data-module-action="back"]').focus({preventScroll:true});}},reduced()?0:480);
  }
  function close(){
    if(!model.close())return false;clearTimeout(transitionTimer);phase='leaving';phaseAt=performance.now();$('module-screen').dataset.phase='leaving';$('module-screen').inert=true;document.body.classList.remove('has-module');
    transitionTimer=setTimeout(()=>{if(model.state().location==='home'){$('module-screen').hidden=true;phase='home';$('console-scene').inert=false;document.querySelector('.tile[data-stack="front"]').focus({preventScroll:true});refresh();}},reduced()?0:420);refresh();return true;
  }
  function back(){if(!$('deck-picker').hidden){closePicker();return true;}return close();}
  function showStatus(text){status=text;const s=$('module-status');if(s)s.textContent=text;}
  function editor(tab){window.JadeLab.changeTab(tab);$('editor').classList.remove('hidden','collapsed');$('editor').inert=false;}
  async function action(name){
    const [type,...parts]=name.split(':'),value=parts.join(':');
    if(type==='back'){close();return;}
    if(type==='filter'){filter=value;$('mod-list').innerHTML=modRows();document.querySelectorAll('[data-module-action^="filter:"]').forEach(b=>b.setAttribute('aria-pressed',String(b.dataset.moduleAction==='filter:'+filter)));refresh();return;}
    if(type==='mod'){selectedMods.has(value)?selectedMods.delete(value):selectedMods.add(value);save();$('mod-list').innerHTML=modRows();$('mod-count').textContent=selectedMods.size;showStatus('Seleção da prévia atualizada.');refresh();return;}
    if(type==='clear-mods'){selectedMods.clear();save();$('mod-list').innerHTML=modRows();$('mod-count').textContent=0;refresh();return;}
    if(type==='editor'){editor(value);return;}
    if(type==='palette'){window.JadeLab.applyPalette(value);showStatus('Aparência '+value+' aplicada.');refresh();return;}
    if(type==='material'){window.JadeLab.setField('glassStyle',value);$('material-name').textContent=value;showStatus('Acabamento atualizado.');refresh();return;}
    if(type==='jacket'){jacket=value;document.body.dataset.jacket=value;save();showStatus('Acabamento do personagem atualizado.');return;}
    if(type==='fps'){window.JadeLab.setField('fps',Number(value));renderModule();return;}
    if(type==='motion'){window.JadeLab.setField('reducedMotion',!preset().reducedMotion);renderModule();return;}
    if(type==='profile'){profile=value;$('profile-name').textContent=profile;save();renderModule();return;}
    if(type==='session'){showStatus('Prévia preparada · '+profile+'. A sessão real é aberta no launcher principal.');$('module-screen').classList.remove('session-pulse');void $('module-screen').offsetWidth;$('module-screen').classList.add('session-pulse');return;}
    if(type==='import'){$('import-file').click();return;}
    if(type==='pack'||type==='preset'){try{showStatus('Exportando…');await(type==='pack'?window.JadeLab.exportPack():window.JadeLab.exportPreset());showStatus('Arquivo exportado.');}catch(e){showStatus('Falha na exportação: '+e.message);}return;}
  }
  function environment(){
    const age=(performance.now()-phaseAt)/1000,duration=reduced()?.001:.45;
    const amount=phase==='entering'?clamp(age/duration,0,1):phase==='leaving'?1-clamp(age/duration,0,1):phase==='open'?1:0;
    return [PAGES.indexOf(model.state().page)+1,amount,0,0];
  }
  function init(cb){
    if(initialized)return;initialized=true;callbacks=cb||{};restore();$('profile-name').textContent=profile;document.body.dataset.jacket=jacket;
    const scene=$('console-scene');scene.addEventListener('pointerdown',pointerDown);scene.addEventListener('pointermove',pointerMove,{passive:false});scene.addEventListener('pointerup',e=>pointerUp(e));scene.addEventListener('pointercancel',e=>pointerUp(e,true));scene.addEventListener('lostpointercapture',e=>{if(gesture)pointerUp(e,true);});
    scene.addEventListener('click',e=>{if(e.target.closest('#deck-picker,.deck-access,.hero'))return;if(performance.now()<suppressUntil&&(e.target===scene||e.target.closest('.cards'))){e.preventDefault();e.stopImmediatePropagation();return;}const tile=e.target.closest('.tile[data-page]'),target=hitTest(e.clientX,e.clientY)||tile?.dataset.page;if(target)select(target);},{capture:true});
    $('hero-action').addEventListener('click',()=>open());$('profile-action').addEventListener('click',()=>open('Play'));
    $('quick-settings').addEventListener('click',()=>open('Settings'));$('account-action').addEventListener('click',()=>open('Skins'));
    $('deck-overview').addEventListener('click',openPicker);$('close-deck-picker').addEventListener('click',closePicker);
    $('deck-picker').addEventListener('click',e=>{const b=e.target.closest('[data-pick]');if(b)select(b.dataset.pick);});
    $('module-screen').addEventListener('click',e=>{const b=e.target.closest('[data-module-action]');if(b)action(b.dataset.moduleAction);});
    $('module-screen').addEventListener('input',e=>{if(e.target.id==='mod-search'){query=e.target.value;$('mod-list').innerHTML=modRows();refresh();}});
    document.addEventListener('keydown',e=>{
      if(e.defaultPrevented||e.altKey||e.ctrlKey||e.metaKey||e.target.closest?.('#editor,input,textarea,select,[contenteditable]'))return;
      if(model.state().location==='module'){if(e.key==='Tab'){const buttons=Array.from($('module-screen').querySelectorAll('button,input')).filter(b=>!b.disabled&&b.getClientRects().length);if(e.shiftKey&&document.activeElement===buttons[0]){buttons.at(-1).focus();e.preventDefault();}else if(!e.shiftKey&&document.activeElement===buttons.at(-1)){buttons[0].focus();e.preventDefault();}}return;}
      if(['ArrowRight','ArrowDown','PageDown'].includes(e.key)){step(1);e.preventDefault();}else if(['ArrowLeft','ArrowUp','PageUp'].includes(e.key)){step(-1);e.preventDefault();}
    });
    updateIndicator();
  }
  window.JadeNavigation={init,select,selected,open,back,environment,createModel,releaseStep,deckOrder,pointInQuad,hitTest,meta:META,refreshModule(){if(model.state().location==='module')renderModule();},state:()=>({...model.state(),phase,dragging:!!gesture?.moved,filter,query,selectedMods:[...selectedMods]})};
})();
