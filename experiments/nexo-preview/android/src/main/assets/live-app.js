/* Nexo Old School, driven by Horizon's real Rust launcher. No demo catalog/account writes. */
(function(){
 'use strict';if(!window.HorizonHost)return;document.title='Horizon · Nexo Old School';
 const $=id=>document.getElementById(id),root=$('nexo-app'),host=HorizonHost;
 let storage;try{storage={getItem:()=>localStorage.getItem('horizon.nexo.live.v1'),setItem:(_,value)=>localStorage.setItem('horizon.nexo.live.v1',value)};}catch(_){storage=null;}
 const store=NexoState.createStore(storage);store.get().catalog=[];store.get().profiles=[];
 let colorKey=null,colorValue=null;
 let state=null,utility=null,autoView=false,opened=false,requested=null,renderKey='',labelKey='',noticeTimer=0,launchBusy=false;
 const controls=new Map(),timers=new Map(),fieldNodes=new Map(),pressing=new Set();
 const pageMap={Mods:'mods',Pacotes:'packs',Shaders:'shaders',Skins:'skins'},titles={Play:'Jogar',Mods:'Mods','Resource Pack':'Pacotes',Shaders:'Shaders',Skins:'Skins',Installed:'Arquivos',Settings:'Ajustes'};
 const dialogs=[...document.querySelectorAll('dialog:not(#intro-dialog)')];
 function el(tag,attrs={},...children){const n=document.createElement(tag);for(const[k,v]of Object.entries(attrs)){if(k==='className')n.className=v;else if(k.startsWith('on'))n.addEventListener(k.slice(2),v);else n.setAttribute(k,String(v));}for(const c of children)if(c!==null&&c!==undefined)n.append(c instanceof Node?c:document.createTextNode(String(c)));return n;}
 function icon(kind){const paths={play:'M5 19 14 10M9 4h9v3h3v5h-3V9h-6L9 12 6 9Z',mods:'M4 6h16v15H4ZM3 3h18v5H3ZM7 4v3m5-3v3m5-3v3M8 12h8v3h-3v4h-2v-4H8Z',packs:'M3 6 12 2 21 6v5l-9 4-9-4ZM3 14l9 4 9-4M3 18l9 4 9-4M12 7v7',shaders:'M12 2 7 8l1 10 4 4 4-4 1-10ZM4 12 2 15l3 7 3-3M20 11l2 4-3 7-3-3',search:'M4 3h11v3h3v10h-3v3H4v-3H1V6h3ZM16 16l6 6',plus:'M10 2h4v8h8v4h-8v8h-4v-8H2v-4h8Z',check:'M3 11h4v4h3V9h4V5h6v4h-3v4h-3v4h-4v4H6v-4H3Z',trash:'M4 6h16M9 2h6v4M6 9v12h12V9M10 10v8m4-8v8',back:'M11 3H8v3H5v3H2v6h3v3h3v3h3v-6h11V9H11Z',files:'M3 7h18v13H3ZM3 7V4h7v3M3 11h18M10 10h4v4h-4Z',account:'M8 2h8v8H8ZM5 13h14v9H5Z',settings:'M9 2h6v4h3v3h4v6h-4v3h-3v4H9v-4H6v-3H2V9h4V6h3ZM9 9v6h6V9Z',close:'M5 3 12 10 19 3 21 5 14 12 21 19 19 21 12 14 5 21 3 19 10 12 3 5Z'};const svg=document.createElementNS('http://www.w3.org/2000/svg','svg'),p=document.createElementNS(svg.namespaceURI,'path');svg.setAttribute('viewBox','0 0 24 24');svg.setAttribute('width','24');svg.setAttribute('height','24');svg.setAttribute('aria-hidden','true');p.setAttribute('d',paths[kind]||paths.files);p.setAttribute('fill','none');p.setAttribute('stroke','currentColor');p.setAttribute('stroke-width','1.2');p.setAttribute('stroke-linejoin','bevel');svg.append(p);return svg;}
 function profile(){return{name:state?.name||'Horizon',version:'26.3'};}
 function themeColors(){
  const key=state?.accent||'#87b8c2';if(key===colorKey)return colorValue;colorKey=key;
  if(key.toLowerCase()==='#87b8c2')return colorValue={cool:[.62,.76,.80],warm:[.64,.49,.29]};
  const rgb=[1,3,5].map(i=>parseInt(key.slice(i,i+2),16)/255),hi=Math.max(...rgb),lo=Math.min(...rgb),d=hi-lo;let hue=0;
  if(d){hue=hi===rgb[0]?((rgb[1]-rgb[2])/d+6)%6:hi===rgb[1]?(rgb[2]-rgb[0])/d+2:(rgb[0]-rgb[1])/d+4;hue/=6;}
  const hsl=(h,s,l)=>{const f=n=>{const k=(n+h*12)%12;return l-s*Math.min(l,1-l)*Math.max(-1,Math.min(k-3,9-k,1));};return[f(0),f(8),f(4)];};
  return colorValue={cool:hsl(hue,.32,.68),warm:hsl((hue+.5)%1,.34,.49)};
 }
 function notice(text){clearTimeout(noticeTimer);$('notice').hidden=false;$('notice').querySelector('span').textContent=text;$('notice-undo').hidden=true;noticeTimer=setTimeout(()=>$('notice').hidden=true,7000);}
 function sound(kind='select'){if(!document.hidden)NexoAudio.play(kind,state?.sounds===false?0:store.get().preferences.sound);}
 function send(op,body){return host.request(op,body).catch(error=>{sound('error');notice(error.message);throw error;});}
 function run(op,body){send(op,body).catch(()=>{});}
 function preferences(){const p=store.get().preferences;document.documentElement.style.setProperty('--scale',p.textScale);document.body.classList.toggle('reduced-motion',p.reducedMotion||!!state?.reducedMotion);NexoAudio.setVolume(state?.sounds===false?0:p.sound);for(const n of document.querySelectorAll('[data-pref]')){if(n.type==='checkbox')n.checked=p[n.dataset.pref];else n.value=p[n.dataset.pref];}window.NexoScene?.refresh();}
 function transform(points){const[a,b,c,d]=points,dx1=b[0]-c[0],dx2=d[0]-c[0],dx3=a[0]-b[0]+c[0]-d[0],dy1=b[1]-c[1],dy2=d[1]-c[1],dy3=a[1]-b[1]+c[1]-d[1],det=dx1*dy2-dx2*dy1;if(Math.abs(det)<1e-6)return;const g=(dx3*dy2-dx2*dy3)/det,h=(dx1*dy3-dx3*dy1)/det;return'matrix3d('+[(b[0]-a[0]+g*b[0])/1024,(b[1]-a[1]+g*b[1])/1024,0,g/1024,(d[0]-a[0]+h*d[0])/668,(d[1]-a[1]+h*d[1])/668,0,h/668,0,0,1,0,a[0],a[1],0,1].join(',')+')';}
 function requestPage(destination){requested=destination;renderKey='';$('module-content').replaceChildren(el('p',{className:'live-loading'},'Abrindo…'));run('page',{page:destination});}
 function show(destination){utility=destination;opened=true;root.classList.add('has-module');$('module').hidden=false;requestPage(destination);window.NexoScene?.openView();}
 function closeView(){utility=null;opened=false;requested=null;renderKey='';root.classList.remove('has-module');run('page',{page:'play'});}
 function beforeClose(){if(state?.modal||(state?.page==='Settings'&&state.settingsCategory!=='Settings'&&state.settingsCategory!=='Overview')||(state?.page==='Installed'&&state.fileCanBack)){sound('back');run('back');return false;}return true;}
 function back(fromAndroid=false){if(window.NexoIntro?.isActive()){NexoIntro.skip();return true;}const dialog=dialogs.slice().reverse().find(d=>d.open);if(dialog){dialog.close();return true;}const active=document.activeElement;if(active&&['INPUT','TEXTAREA','SELECT'].includes(active.tagName)){active.blur();return true;}if(NexoScene.getState().open){NexoScene.close();return true;}run('back');return true;}
 function translated(s){return({Search:'Buscar',Import:'Importar','Open folder':'Abrir pasta','Load more':'Carregar mais',Install:'Instalar',Remove:'Remover',Delete:'Excluir',Enabled:'Ativado',Disabled:'Desativado',Close:'Fechar',Cancel:'Cancelar',Back:'Voltar',Apply:'Aplicar',Save:'Salvar',Settings:'Ajustes',Account:'Conta','Resource Pack':'Pacotes',Overview:'Visão geral'})[s]||s;}
 function press(c){if(pressing.has(c.id))return;pressing.add(c.id);sound(c.kind==='toggle'?'toggle':'confirm');send('press',{control:c.id}).catch(()=>{}).finally(()=>pressing.delete(c.id));}
 function field(c){const prior=fieldNodes.get(c.id);if(prior){const input=prior.querySelector('input');if(document.activeElement!==input&&c.kind!=='password'&&c.field!=='ely_otp')input.value=c.value||'';return prior;}const label=translated(c.label||c.hint||'Texto'),input=el('input',{type:c.kind==='password'?'password':/search/.test(c.field)?'search':'text','aria-label':label,placeholder:c.hint||label,autocomplete:c.kind==='password'?'current-password':'off',maxlength:c.field?.startsWith('ely_')?1024:4096,'data-control':c.id});input.value=c.value||'';
  const flush=finish=>{clearTimeout(timers.get(c.id));if(controls.has(c.id))run('edit',{control:c.id,text:input.value,finish});};
  input.addEventListener('input',()=>{clearTimeout(timers.get(c.id));timers.set(c.id,setTimeout(()=>flush(false),180));});input.addEventListener('blur',()=>flush(true));input.addEventListener('keydown',event=>{if(event.key==='Enter'){event.preventDefault();flush(false);const search=[...controls.values()].find(v=>/^Search$|^Convert$|^Import \.hmod$/.test(v.label));if(search)press(search);else input.blur();}});
  const node=el('label',{className:'live-field'},el('span',{},label),input);fieldNodes.set(c.id,node);return node;
 }
 function control(c){
  if(c.kind==='field'||c.kind==='password')return field(c);
  if(c.kind==='range'){const input=el('input',{type:'range',min:0,max:1,step:.01,value:c.fraction??.5,'aria-label':translated(c.label),'data-control':c.id});input.addEventListener('input',()=>{clearTimeout(timers.get(c.id));timers.set(c.id,setTimeout(()=>run('range',{control:c.id,value:Number(input.value)}),70));});input.addEventListener('change',()=>{clearTimeout(timers.get(c.id));run('range',{control:c.id,value:Number(input.value)});});return el('label',{className:'live-range'},el('span',{},translated(c.label)),input);}
  const b=el('button',{type:'button',className:'live-control live-'+c.kind,'data-control':c.id,onclick:()=>press(c)});
  if(c.image)b.append(el('img',{src:c.image,alt:'',loading:'lazy'}));else if(c.icon)b.append(icon(c.icon));
  if(c.color){b.classList.add('live-swatch');b.style.setProperty('--swatch',c.color);b.setAttribute('aria-label','Cor '+c.color);}
  b.append(el('span',{className:'live-control-copy'},el('strong',{},translated(c.label)),...c.lines.filter(v=>v!==c.label).slice(0,3).map(v=>el('small',{},translated(v)))));
  if(c.kind==='toggle'){b.setAttribute('role','switch');b.setAttribute('aria-checked',String(!!c.checked));b.append(el('span',{className:'switch-track','aria-hidden':'true'}));}
  else if(c.selected)b.setAttribute('aria-pressed','true');return b;
 }
 function render(){
  if(!state||!opened)return;
  const expected={mods:'Mods',packs:'Resource Pack',shaders:'Shaders',skins:'Skins',files:'Installed',settings:'Settings'}[requested];if(expected&&state.page!==expected&&!state.modal)return;
  const active=document.activeElement,focused=active?.dataset.control;
  const key=JSON.stringify([state.page,state.modal,state.title,state.settingsCategory,state.controls,state.texts]);
  if(key===renderKey){const image=$('live-skin');if(image&&state.skin)image.src=state.skin;return;}if(focused&&state.controls.some(c=>String(c.id)===focused))return;
  renderKey=key;const content=$('module-content'),oldScroll=content.querySelector('.live-scroll')?.scrollTop||0;content.replaceChildren();controls.clear();
  const title=state.modal?translated(state.title):utility==='account'?'Conta':utility==='profile'?'Perfil':titles[state.page]||translated(state.title);
  $('module-title').textContent=title;$('module-eyebrow').textContent=state.modal?'HORIZON · '+(titles[state.page]||'CONTA'):'HORIZON · '+(state.busy?'EM ANDAMENTO':'SUA BIBLIOTECA');$('module-options').hidden=false;
  const currentIds=new Set(state.controls.map(c=>c.id));for(const id of fieldNodes.keys())if(!currentIds.has(id))fieldNodes.delete(id);
  const all=state.controls.filter(c=>c.id>0),toolbar=el('div',{className:'live-toolbar'}),scroll=el('div',{className:'live-scroll'}),grid=el('div',{className:'live-grid'}),groups=new Map();
  for(const c of all){controls.set(c.id,c);if(c.group){if(!groups.has(c.group))groups.set(c.group,[]);groups.get(c.group).push(c);}}
  const handled=new Set();
  for(const c of all){if(handled.has(c.id))continue;
   if(c.group){const items=groups.get(c.group),article=el('article',{className:'live-item','data-item':c.group});for(const item of items){handled.add(item.id);article.append(control(item));}grid.append(article);}
   else if(!state.modal&&(c.field==='search'||c.field==='installed_search'||(c.rect[1]<180&&state.page!=='Settings'&&state.page!=='Play')))toolbar.append(control(c));
   else grid.append(control(c));
  }
  const textList=state.texts.filter(t=>t.text.trim()&&!all.some(c=>c.label===t.text||c.lines.includes(t.text)));
  if(state.page==='Skins'&&!state.modal&&state.skin){const image=el('img',{id:'live-skin',src:state.skin,alt:'Prévia da sua skin. Arraste para girar.',draggable:'false'}),space=el('div',{className:'live-skin-space'},image);let drag=null;space.addEventListener('pointerdown',e=>{drag={id:e.pointerId,x:e.clientX,y:e.clientY};space.setPointerCapture(e.pointerId);});space.addEventListener('pointermove',e=>{if(!drag||drag.id!==e.pointerId)return;const dx=e.clientX-drag.x,dy=e.clientY-drag.y;drag={id:e.pointerId,x:e.clientX,y:e.clientY};if(dx||dy)run('rotate',{dx:Math.max(-720,Math.min(720,dx)),dy:Math.max(-720,Math.min(720,dy))});});for(const name of['pointerup','pointercancel','lostpointercapture'])space.addEventListener(name,()=>drag=null);scroll.append(space);}
  if(toolbar.childElementCount)content.append(toolbar);
  const info=el('div',{className:'live-info'});for(const t of textList){if(t.text===state.title||t.text===state.name||/HORIZON|Jade|Minecraft.*26\.3/.test(t.text))continue;info.append(el(t.size>=17?'h3':'p',{className:t.size<12?'note':''},translated(t.text)));}
  if(info.childElementCount&&(state.modal||!groups.size))scroll.append(info);scroll.append(grid);content.append(scroll);scroll.scrollTop=oldScroll;
  if(!all.length&&!info.childElementCount)grid.append(el('p',{className:'empty'},state.busy?'Carregando conteúdo…':'Sua biblioteca está vazia. Use Importar ou Buscar para adicionar conteúdo.'));
 }
 function receive(value){const oldName=state?.name;state=value;if(state.reducedMotion&&window.NexoIntro?.isActive())NexoIntro.skip();
  $('profile-current').textContent=profile().name;$('profile-button').setAttribute('aria-label','Conta: '+profile().name);
  root.querySelector('.nexo-state').textContent=state.status||profile().name+' · 26.3';preferences();
  if(oldName!==state.name)window.NexoScene?.updateLabels(true);
  if(state.modal&&!opened){autoView=requested!=='play';utility='native';opened=true;root.classList.add('has-module');$('module').hidden=false;window.NexoScene?.openView();}
  else if(!state.modal&&autoView&&opened){autoView=false;window.NexoScene?.close(true);}
  else if((state.busy||!state.ready)&&!opened&&requested==='play'){utility='play';opened=true;root.classList.add('has-module');$('module').hidden=false;window.NexoScene?.openView();}
  window.NexoScene?.setLaunching(!!state.launching);if(launchBusy&&!state.launching&&!state.busy)launchBusy=false;render();
 }
 async function play(){if(launchBusy)return;launchBusy=true;requested='play';sound('confirm');try{await send('play');}catch(_){launchBusy=false;}setTimeout(()=>{launchBusy=false;},1000);}
 for(const dialog of dialogs){dialog.querySelector('[data-close]')?.addEventListener('click',()=>dialog.close());dialog.addEventListener('click',event=>{if(event.target===dialog){const r=dialog.getBoundingClientRect();if(event.clientX<r.left||event.clientX>r.right||event.clientY<r.top||event.clientY>r.bottom)dialog.close();}});}
 $('profile-button').replaceChildren(icon('account'),el('span',{id:'profile-current'},'Horizon'));$('settings-button').replaceChildren(icon('settings'));$('profile-button').onclick=()=>show('account');$('settings-button').onclick=()=>show('settings');
 const files=el('button',{type:'button',id:'files-button','aria-label':'Arquivos',onclick:()=>show('files')},icon('files'));document.querySelector('.header-actions').insertBefore(files,$('settings-button'));
 document.querySelector('.nexo-back').replaceChildren(icon('back'));
 $('module-options').onclick=()=>{const body=$('catalog-options-dialog').querySelector('.dialog-body');body.replaceChildren(...[['files','Arquivos'],['profile','Editar perfil'],['account','Contas'],['settings','Ajustes do Horizon']].map(([dest,label])=>el('button',{type:'button',onclick:()=>{$('catalog-options-dialog').close();show(dest);}},label)),el('button',{type:'button',onclick:()=>{$('catalog-options-dialog').close();preferences();$('settings-dialog').showModal();}},'Efeitos do Nexo'));$('catalog-options-dialog').showModal();};
 $('settings-dialog').querySelector('.note').textContent='Os efeitos do Nexo são independentes das opções gráficas do jogo.';
 for(const input of document.querySelectorAll('[data-pref]'))input.addEventListener(input.type==='range'?'input':'change',()=>{store.preference(input.dataset.pref,input.type==='checkbox'?input.checked:Number(input.value));preferences();});
 $('reset-appearance').onclick=()=>{store.resetAppearance();preferences();};$('undo-appearance').onclick=()=>{store.undoAppearance();preferences();};$('replay-intro').onclick=()=>{$('settings-dialog').close();NexoIntro.start({replay:true});};$('notice-close').onclick=()=>$('notice').hidden=true;
 document.addEventListener('focusout',()=>setTimeout(()=>{renderKey='';render();},250));document.addEventListener('keydown',event=>{if(event.key==='Escape'){event.preventDefault();back();}});
 function setVisible(visible){NexoAudio.setVisible(visible);window.NexoIntro?.setVisible(visible);window.NexoScene?.setVisible(visible);}
 document.addEventListener('visibilitychange',()=>setVisible(!document.hidden));
 window.NexoApp={store,profile,sound,play,beforeClose,themeColors,reducedMotion:()=>!!state?.reducedMotion,skinRect:()=>null,ready(){preferences();run('ready');},hideMovingLabel(){$('front-label').style.opacity=0;},onTabFace(points,index){document.querySelector('[data-card="'+index+'"]').style.transform=transform(points);},onHomeFace(points,index,opacity){$('front-label').style.transform=transform(points);$('front-label').style.opacity=opacity;const k=index+'|'+profile().name;if(k===labelKey)return;labelKey=k;$('front-label').querySelector('.front-prefix').textContent=String(index+1).padStart(2,'0')+' / HORIZON';$('front-label').querySelector('.front-title').textContent=NexoState.PAGES[index];$('front-label').querySelector('.front-caption').textContent=index===0?profile().name+' · 26.3':['','Sua experiência','Camadas e texturas','Luz e atmosfera','Sua identidade'][index];},
 selectionChanged(index,isOpen){const previous=opened,changed=store.get().selected!==index;store.select(index);opened=isOpen;root.classList.toggle('has-module',isOpen);if(isOpen){$('module').hidden=false;if(!previous&&!utility)requestPage(pageMap[NexoState.PAGES[index]]||'play');render();}else if(previous)closeView();if(changed)sound();},
 onFrame({opening,rect}){root.style.setProperty('--module-open',opening.toFixed(4));document.querySelector('.app-header').inert=opening>.01;document.querySelector('.app-footer').setAttribute('aria-hidden',String(opening>.01));if(opening<.001&&!opened){$('module').hidden=true;return;}Object.assign($('module').style,{left:rect.left+'px',top:rect.top+'px',width:rect.width+'px',height:rect.height+'px',opacity:Math.max(0,Math.min(1,(opening-.76)/.24))});$('module').inert=opening<.97||!opened;}
 };
 window.Nexo={back,setVisible,state:()=>({live:true,page:state?.page,native:state,launchBusy})};window.JadeLab={back};root.dataset.host='native';host.subscribe(receive);setTimeout(()=>{if(!window.NexoScene||document.querySelector('.nexo-error:not([hidden])'))run('fallback');},5000);
})();
