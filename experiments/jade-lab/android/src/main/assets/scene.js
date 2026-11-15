/* Original console navigation and shared scene geometry. All visible panel
 * poses feed BOTH the GPU and the DOM hit/text planes, in CSS-pixel units. */
(function(){
  'use strict';
  const PAGES=['Play','Mods','Packs','Shaders','Files','Skins','Settings'];
  const DATA={
    Play:{label:'Jogar',tag:'SEU ESPAÇO PARA JOGAR',title:'Um novo horizonte.',sub:'Minecraft · Java Edition 26.3',options:['Iniciar Minecraft','Gerenciar perfis','Sobre esta prévia'],symbol:'play'},
    Mods:{label:'Mods',tag:'CRIE DO SEU JEITO',title:'Novas possibilidades.',sub:'Extensões e experiências para o seu mundo.',options:['Biblioteca de mods','Perfis de modificação','Sobre os mods'],symbol:'cube'},
    Packs:{label:'Pacotes',tag:'OUTRA PERSPECTIVA',title:'Mude a paisagem.',sub:'Texturas, interfaces e pequenos detalhes.',options:['Pacotes de recursos','Importar preset do Lab','Exportar aparência'],symbol:'layers'},
    Shaders:{label:'Shaders',tag:'LUZ, MATÉRIA E MOVIMENTO',title:'Sinta a atmosfera.',sub:'Explore os materiais desta cena em tempo real.',options:['Editar iluminação','Explorar materiais','Editar ondas'],symbol:'sun'},
    Files:{label:'Arquivos',tag:'TUDO NO SEU LUGAR',title:'Seu espaço pessoal.',sub:'Presets e referências deste laboratório.',options:['Importar preset','Exportar preset','Exportar pack completo'],symbol:'folder'},
    Skins:{label:'Skins',tag:'SUA IDENTIDADE',title:'Entre em cena.',sub:'Personagem geométrico de demonstração.',options:['Ver personagem','Movimento do personagem','Iluminar personagem'],symbol:'person'},
    Settings:{label:'Sistema',tag:'AJUSTE CADA DETALHE',title:'O seu equilíbrio.',sub:'Aparência, movimento e desempenho.',options:['Aparência e paletas','Desempenho','Movimento e acessibilidade'],symbol:'settings'}
  };
  const ICONS={play:'M9 5l12 9-12 9Z',cube:'M14 3 3 9v12l11 6 11-6V9Zm0 12L3 9m11 6 11-6m-11 6v12',layers:'m3 8 11-5 11 5-11 5Zm0 7 11 5 11-5M3 22l11 5 11-5',sun:'M14 8a6 6 0 1 0 0 12 6 6 0 0 0 0-12ZM14 1v3m0 20v3M1 14h3m20 0h3M5 5l2 2m14 14 2 2M5 23l2-2M21 7l2-2',folder:'M3 7h8l3 3h11v14H3Z',person:'M14 3a5 5 0 1 0 0 10 5 5 0 0 0 0-10ZM5 26v-6c0-5 18-5 18 0v6',settings:'M14 9a5 5 0 1 0 0 10 5 5 0 0 0 0-10ZM14 2v4m0 16v4M2 14h4m16 0h4M5.5 5.5l3 3m11 11 3 3M5.5 22.5l3-3m11-11 3-3'};
  const icon=name=>'<svg viewBox="0 0 28 28" aria-hidden="true"><path d="'+ICONS[name]+'"/></svg>';
  const clamp=(v,a,b)=>Math.max(a,Math.min(b,v));
  function model(){
    let category=0,level='categories',dialog=false;
    const rows=PAGES.map(()=>0);
    return {state:()=>({page:PAGES[category],category,row:rows[category],level,dialog}),
      select(i){category=clamp(i,0,6);level='categories';dialog=false;return this.state();},
      step(dir){return this.select(clamp(category+dir,0,6));},
      move(dir){level='options';rows[category]=clamp(rows[category]+dir,0,2);return this.state();},
      enter(row){if(Number.isInteger(row))rows[category]=clamp(row,0,2);if(level==='categories'){level='options';return false;}dialog=true;return true;},
      close(){dialog=false;},
      back(){if(dialog){dialog=false;return true;}if(level==='options'){level='categories';return true;}if(category!==0){this.select(0);return true;}return false;}
    };
  }
  // Camera is at (viewport/2, focal), looking toward -Z. A plane has two
  // orthonormal axes; its homography is used unchanged for text and hit tests.
  function pose(r,z,rx,ry,viewport){
    const F=1200,cy=Math.cos(ry),sy=Math.sin(ry),cx=Math.cos(rx),sx=Math.sin(rx);
    const u=[cy,0,-sy],v=[sy*sx,cx,cy*sx],n=[sy*cx,-sx,cy*cx];
    const c=[r.x+r.width/2,r.y+r.height/2,z],w=r.width,h=r.height;
    const origin=c.map((a,i)=>a-u[i]*w/2-v[i]*h/2);
    const C=[viewport[0]/2,viewport[1]/2];
    const m=[u[0]-C[0]*u[2]/F,u[1]-C[1]*u[2]/F,0,-u[2]/F,
      v[0]-C[0]*v[2]/F,v[1]-C[1]*v[2]/F,0,-v[2]/F,
      0,0,1,0,origin[0]-C[0]*origin[2]/F,origin[1]-C[1]*origin[2]/F,0,1-origin[2]/F];
    const project=(x,y)=>{const p=c.map((a,i)=>a+u[i]*(x-w/2)+v[i]*(y-h/2));return [C[0]+(p[0]-C[0])*F/(F-p[2]),C[1]+(p[1]-C[1])*F/(F-p[2])];};
    return {c,u,v,n,w,h,m,corners:[[0,0],[w,0],[w,h],[0,h]].map(q=>project(...q))};
  }
  let nav=model(),preset={},nodes=[],geometry=[],boxes=[],dirty=true,callbacks={},impulse=0,selectedAt=0;
  let animation=0,initialized=false,focusRow=-1,lastInput='',swipe=null,gamepadLast=0,gamepadHeld=false;
  const $=id=>document.getElementById(id);
  function invalidate(){dirty=true;window.JadeRenderer?.invalidate?.();}
  function render(focus=false){
    const s=nav.state(),d=DATA[s.page];
    $('hero-eyebrow').textContent=d.tag;$('hero-title').textContent=d.title;
    $('hero-subtitle').textContent=d.sub;$('art-category').textContent=d.label.toUpperCase();
    document.querySelectorAll('[data-category]').forEach(b=>{const active=Number(b.dataset.category)===s.category;b.setAttribute('aria-selected',String(active));b.tabIndex=active?0:-1;});
    document.querySelectorAll('[data-option]').forEach((b,i)=>{b.querySelector('.option-label').textContent=d.options[i];b.setAttribute('aria-current',s.level==='options'&&s.row===i?'true':'false');});
    $('scene-caption').textContent=String(s.category+1).padStart(2,'0')+' / 07 · '+(s.level==='options'?d.options[s.row]:d.label);
    $('context-panel').hidden=!s.dialog;
    for(const id of ['console-categories'])$(id).inert=s.dialog;
    document.querySelectorAll('.masthead,.console-options,.console-footer').forEach(el=>el.inert=s.dialog);
    if(focus){const target=s.level==='options'?document.querySelector('[data-option="'+s.row+'"]'):document.querySelector('[data-category="'+s.category+'"]');target?.focus({preventScroll:true});}
    const active=document.querySelector('[data-category="'+s.category+'"]');
    active?.scrollIntoView?.({block:'nearest',inline:'nearest',behavior:'instant'});
    impulse=1;selectedAt=performance.now();animation=preset.reducedMotion?0:1;invalidate();
  }
  function openEditor(tab){callbacks.changeTab?.(tab);$('editor').classList.remove('hidden','collapsed');nav.close();render();}
  function activate(row){
    nav.enter(row); // Pointer activation enters the row directly.
    if(!nav.state().dialog)nav.enter(row);
    const s=nav.state(),editorTabs={Shaders:['Luz','Materiais','Fundo'],Settings:['Paletas','Desempenho','Movimento'],Skins:[null,'Componentes','Luz']};
    if(editorTabs[s.page]?.[row]){openEditor(editorTabs[s.page][row]);return;}
    if(s.page==='Files'||(s.page==='Packs'&&row>0)){
      if((s.page==='Files'&&row===0)||(s.page==='Packs'&&row===1))$('import-file').click();
      else if(s.page==='Files'&&row===1)callbacks.exportPreset?.();
      else callbacks.exportPack?.();
      nav.close();render();return;
    }
    const title=DATA[s.page].options[row];$('context-title').textContent=title;
    const descriptions={Play:['Esta é a prévia visual do Horizon. A instalação e a abertura do Minecraft pertencem ao launcher principal.','Perfil de demonstração · Minecraft Java Edition 26.3. Os dados do launcher ficam no aplicativo principal.','Jade Lab é um laboratório independente de navegação, luz e materiais. Edite a cena e exporte seus ajustes para referência.'],Mods:['A biblioteca real de mods pertence ao Horizon. Esta tela demonstra o acesso e o retorno à categoria.','Os perfis reais permanecem no launcher principal.','Esta prévia não altera nem instala mods.'],Packs:['Experimente o visual em Shaders ou use os presets do editor.'],Skins:['O personagem é formado por volumes 3D na mesma cena dos painéis. Mova a luz e observe o reflexo e a sombra sobre o cartão.']};
    $('context-description').textContent=descriptions[s.page]?.[row]||'Prévia da interface do Horizon.';
    render();$('context-close').focus({preventScroll:true});
  }
  function back(){const e=$('editor');if(e&&!e.classList.contains('hidden')&&!e.classList.contains('collapsed')){e.classList.add('collapsed');return true;}const result=nav.back();if(result)render(true);return result;}
  function input(action){
    const before=nav.state();if(before.dialog){if(action==='back')back();return;}
    if(action==='left'||action==='right')nav.step(action==='left'?-1:1);
    else if(action==='up'||action==='down'){if(before.level==='categories'){if(action==='down'){nav.enter();}else return;}else if(action==='up'&&before.row===0){nav.back();}else nav.move(action==='down'?1:-1);}
    else if(action==='enter'){if(nav.enter()){activate(nav.state().row);return;}}
    else if(action==='back'){back();return;}
    render(true);
  }
  function init(cb){
    callbacks=cb||{};if(initialized)return;initialized=true;
    $('console-categories').innerHTML=PAGES.map((p,i)=>'<button role="tab" data-category="'+i+'" aria-label="'+DATA[p].label+'">'+icon(DATA[p].symbol)+'<span>'+DATA[p].label+'</span></button>').join('');
    nodes=Array.from(document.querySelectorAll('[data-node]')).map(el=>({el,face:el.querySelector('.surface-face'),id:el.dataset.node}));
    $('editor').classList.add('collapsed');
    $('console-categories').addEventListener('click',e=>{const b=e.target.closest('[data-category]');if(b){nav.select(Number(b.dataset.category));render();}});
    document.querySelectorAll('[data-option]').forEach(b=>b.addEventListener('click',()=>activate(Number(b.dataset.option))));
    $('context-close').addEventListener('click',()=>back());
    $('console-back').addEventListener('click',()=>back());
    $('console-categories').addEventListener('scroll',invalidate,{passive:true});
    const area=$('console-scene');
    area.addEventListener('pointerdown',e=>{if(e.target.closest('button,#context-panel'))return;swipe={id:e.pointerId,x:e.clientX,y:e.clientY};},{passive:true});
    area.addEventListener('pointerup',e=>{if(!swipe||e.pointerId!==swipe.id)return;const dx=e.clientX-swipe.x,dy=e.clientY-swipe.y;swipe=null;if(Math.abs(dx)>52&&Math.abs(dx)>Math.abs(dy)*1.5)input(dx<0?'right':'left');},{passive:true});
    area.addEventListener('pointercancel',()=>{swipe=null;},{passive:true});
    document.addEventListener('keydown',e=>{
      if(e.defaultPrevented||e.altKey||e.metaKey||e.ctrlKey)return;
      if(e.key==='Escape'||e.key==='BrowserBack'){if(back())e.preventDefault();return;}
      if(e.target.closest?.('#editor,input,select,textarea,[contenteditable],#context-panel'))return;
      const keys={ArrowLeft:'left',ArrowRight:'right',PageUp:'left',PageDown:'right',ArrowUp:'up',ArrowDown:'down',Enter:'enter',' ':'enter'};
      if(keys[e.key]){e.preventDefault();input(keys[e.key]);}
    });
    if(typeof ResizeObserver!=='undefined')new ResizeObserver(invalidate).observe($('preview'));
    window.addEventListener('resize',()=>{invalidate();if(!window.JadeRenderer?.state?.().webgl2)tick(0);});window.addEventListener('gamepadconnected',()=>window.JadeRenderer?.invalidate?.());
    render();
  }
  function tick(time){
    if(!initialized)return {surfaces:[],boxes:[],impulse:0,animating:false};
    const s=nav.state(),age=(performance.now()-selectedAt)/1000;
    const activeMotion=!preset.reducedMotion&&!window.JadeRenderer?.state?.().paused;
    impulse=activeMotion?Math.max(0,1-age/1.3):0;
    animation=impulse>0?1:0;
    const target=s.level==='options'?s.row:-1;
    focusRow=focusRow< -1?target:focusRow+(target-focusRow)*.22;
    const moving=Math.abs(target-focusRow)>.002;
    if(!activeMotion)focusRow=target;
    if(dirty||moving){
      geometry=nodes.map((o,i)=>{
        const r=o.el.getBoundingClientRect(),art=o.id==='art',row=Number(o.el.dataset.row),selected=!art&&s.level==='options'&&row===s.row;
        const lift=art?24:18*Math.max(0,1-Math.abs(row-focusRow));
        const ry=(art?-1:0.12)*Number(preset.sceneTilt||0)*Math.PI/180;
        const p=pose(r,lift*(.6+Number(preset.sceneDepth||0)),0,ry,[innerWidth,innerHeight]);
        o.face.style.width=r.width+'px';o.face.style.height=r.height+'px';o.face.style.transform='matrix3d('+p.m.join(',')+')';
        return {...p,kind:art?1:2,selected:art?false:selected,glass:art?!!preset.glassHero:!!preset.glassCards};
      });dirty=false;
    }
    const art=geometry.find(p=>p.kind===1);
    if(art){
      const scale=Math.min(art.w/320,art.h/255)*.9;
      const x=art.c[0]+art.w*.18,y=art.c[1]+art.h*.03,z=92;
      const sway=activeMotion?Math.sin(time*.65)*Number(preset.avatarMotion||0)*.04:0;
      const parts=[[-0,-58,0,19,20,18,0],[0,-14,0,24,26,13,1],[-34,-13,0,8,27,10,1],[34,-13,0,8,27,10,1],[-13,29,0,10,20,10,2],[13,29,0,10,20,10,2]];
      boxes=parts.map(a=>({c:[x+a[0]*scale,y+a[1]*scale,z+a[2]*scale],size:a.slice(3,6).map(n=>n*scale),kind:a[6],angle:-.32+sway}));
    }
    // Poll connected controllers without creating a second animation loop.
    const pad=typeof navigator!=='undefined'&&navigator.getGamepads?.()?.find(p=>p?.connected);
    if(pad){const b=pad.buttons,a=pad.axes;let command=b[12]?.pressed||a[1]<-.6?'up':b[13]?.pressed||a[1]>.6?'down':b[14]?.pressed||b[4]?.pressed||a[0]<-.6?'left':b[15]?.pressed||b[5]?.pressed||a[0]>.6?'right':b[0]?.pressed?'enter':b[1]?.pressed?'back':null;const now=performance.now();if(command&&(!gamepadHeld||now-gamepadLast>230)){input(command);gamepadLast=now;}gamepadHeld=!!command;}
    return {surfaces:geometry,boxes,impulse,animating:moving||animation>0||!!pad};
  }
  window.JadeScene={init,tick,back,input,model,pose,icon,pages:PAGES,data:DATA,state:()=>nav.state(),
    setPreset(p){preset=p;if(typeof document!=='undefined'){document.documentElement.style.setProperty('--option-gap',(4+Number(p.sceneSpacing||0)*5)+'px');}invalidate();},invalidate};
})();
