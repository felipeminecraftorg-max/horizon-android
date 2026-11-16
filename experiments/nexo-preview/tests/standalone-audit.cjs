'use strict';
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),http=require('node:http');
const {chromium}=require(process.env.NEXO_PLAYWRIGHT_MODULE||'playwright');
const root=path.resolve(__dirname,'../android/src/main/assets'),out=path.resolve('dist/nexo-native-qa');fs.mkdirSync(out,{recursive:true});
const server=http.createServer((req,res)=>{const p=path.resolve(root,'.'+(req.url==='/'?'/nexo.html':req.url.split('?')[0]));if(!p.startsWith(root+path.sep)){res.writeHead(403).end();return;}const file=p.includes('/icons/')?path.resolve('crates/horizon-client/src/launcher_icons/jade',path.basename(p)):p;fs.readFile(file,(e,data)=>{if(e){res.writeHead(404).end();return;}res.setHeader('Content-Type',p.endsWith('.js')?'text/javascript':p.endsWith('.css')?'text/css':p.endsWith('.ttf')?'font/ttf':p.endsWith('.png')?'image/png':p.endsWith('.ogg')?'audio/ogg':'text/html');res.end(data);});});
const audits=[],errors=[];let page,browser;
async function inspect(name){
 const result=await page.evaluate(()=>{
  const modal=document.querySelector('#nexo-modal'),module=document.querySelector('#module'),scope=modal.open?modal:module.hidden?document.querySelector('.header-actions'):module;
  const failures=[],controls=[];const rect=n=>n.getBoundingClientRect();
  function visible(n){const r=rect(n),s=getComputedStyle(n);if(!r.width||!r.height||s.visibility==='hidden'||s.display==='none')return false;for(let p=n;p&&p!==document.body;p=p.parentElement){const cs=getComputedStyle(p);if(Number(cs.opacity)<.1||p.hidden)return false;if(/auto|scroll|hidden|clip/.test(cs.overflow+cs.overflowY+cs.overflowX)){const a=rect(p);if(r.top<a.top-1||r.bottom>a.bottom+1||r.left<a.left-1||r.right>a.right+1)return false;}}return true;}
  for(const n of scope.querySelectorAll('button,input,select')){
   if(!visible(n)||n.matches('.nx-placed'))continue;const r=rect(n),font=getComputedStyle(n).fontFamily;
   if(!font.includes('Nexo Read'))failures.push('Wrong font: '+n.outerHTML.slice(0,100));
   if(r.left<-1||r.top<-1||r.right>innerWidth+1||r.bottom>innerHeight+1)failures.push('Outside viewport: '+n.textContent);
   for(const icon of n.querySelectorAll('svg,.nx-switch')){const i=rect(icon);if(i.left<r.left+3||i.top<r.top+3||i.right>r.right-3||i.bottom>r.bottom-3)failures.push('Glyph touches edge: '+(n.getAttribute('aria-label')||n.textContent));}
   const walker=document.createTreeWalker(n,NodeFilter.SHOW_TEXT);let textNode;while((textNode=walker.nextNode())){if(!textNode.textContent.trim()||!visible(textNode.parentElement))continue;const range=document.createRange();range.selectNodeContents(textNode);for(const line of range.getClientRects())if(line.top<r.top+2||line.bottom>r.bottom-2)failures.push('Text touches control border: '+textNode.textContent.trim());}
   controls.push({node:n,r,label:n.getAttribute('aria-label')||n.textContent.trim().slice(0,60)});
  }
  for(let a=0;a<controls.length;a++)for(let b=a+1;b<controls.length;b++){const x=controls[a],y=controls[b];if(x.node.contains(y.node)||y.node.contains(x.node))continue;const w=Math.min(x.r.right,y.r.right)-Math.max(x.r.left,y.r.left),h=Math.min(x.r.bottom,y.r.bottom)-Math.max(x.r.top,y.r.top);if(w>1&&h>1)failures.push('Overlapping controls: '+x.label+' / '+y.label);}
  if(!modal.open&&!module.hidden){const b=rect(module),h=rect(module.querySelector('.module-header')),title=rect(document.querySelector('.module-heading')),actions=rect(module.querySelector('.module-actions'));if(h.left<b.left+5||h.right>b.right-5||h.top<b.top+4||h.bottom>b.bottom-4)failures.push('Header touches frame');if(title.right>actions.left-3)failures.push('Title collides with actions');const layout=window.NexoScene.inspectLayout?.();if(layout?.icon&&layout.slot?.width){const i=layout.icon,s=layout.slot;if(i.left<s.left-1||i.top<s.top-1||i.right>s.left+s.width+1||i.bottom>s.top+s.height+1)failures.push('3D emblem outside reserved slot: '+JSON.stringify(layout));}}
  return {failures,controls:controls.length,width:innerWidth,height:innerHeight,view:Nexo.state().native?.view.type,modal:Nexo.state().native?.modal?.type||null};
 });
 audits.push({name,...result});if(result.failures.length){await page.screenshot({path:path.join(out,'audit-failure.png')});console.log('NEXO_AUDIT_FAILURE '+JSON.stringify({name,...result}));}assert.deepEqual(result.failures,[],name);
}
async function receive(view,modal=null){await page.evaluate(({view,modal})=>{nativeState.view=view;nativeState.modal=modal;nativeState.replies=[];HorizonHost.receive(JSON.stringify(nativeState));}, {view,modal});await page.waitForTimeout(70);}
(async()=>{
 await new Promise(r=>server.listen(0,'127.0.0.1',r));browser=await chromium.launch({executablePath:process.env.NEXO_CHROME||'/usr/bin/google-chrome',headless:true,args:['--no-sandbox','--use-angle=swiftshader','--enable-unsafe-swiftshader']});
 const context=await browser.newContext({viewport:{width:852,height:393},hasTouch:true,deviceScaleFactor:1});await context.addInitScript(require('./standalone-fixture.cjs'));page=await context.newPage();page.on('pageerror',e=>{errors.push(e.message);console.log('NEXO_AUDIT_ERROR '+e.stack);});page.setDefaultTimeout(15000);await page.goto('http://127.0.0.1:'+server.address().port);await page.waitForFunction(()=>window.Nexo?.state().native&&window.NexoScene?.getState().ready&&!NexoIntro.isActive());
 await page.evaluate(()=>{NexoApp.store.get().preferences.reducedMotion=true;NexoScene.refresh();document.querySelector('#settings-button').click();});await page.waitForFunction(()=>Nexo.state().native.view.type==='settings'&&!document.querySelector('#module').inert);
 const nativeSettings=await page.evaluate(()=>structuredClone(Nexo.state().native.view));
 await page.evaluate(()=>HorizonHost.request('page',{page:'mods'}));const nativeCatalog=await page.evaluate(()=>structuredClone(Nexo.state().native.view));
 const controls=[{id:301,kind:'toggle',label:'Iluminação e atmosfera de longa duração',value:true},{id:302,kind:'option',label:'Perfil do dispositivo',caption:'Nome bastante comprido do dispositivo'},{id:303,kind:'range',label:'Distância de visão e simulação',value:.65,caption:'24 chunks'},{id:304,kind:'field',key:'name',label:'Nome do jogador',value:'Nome extenso com acentuação'},{id:305,kind:'button',label:'Compartilhar relatório de diagnóstico'}];
 const settings={...nativeSettings,groups:nativeSettings.groups.map(g=>({...g,controls}))};
 const fields=[{id:411,key:'ely_username',label:'Usuário ou e-mail',kind:'field',value:'nome@example.invalid'},{id:412,key:'ely_password',label:'Senha',kind:'password',value:''},{id:413,key:'ely_otp',label:'Código de verificação',kind:'password',value:''},{id:414,key:'name',label:'Nome do jogador',kind:'field',value:'Nexo QA'}];
 const actions={microsoft:401,ely:402,offline:403,start:404,elySubmit:405,saved:406,add:407,close:408,copy:409,link:410,cancel:415,signout:416,selectOffline:417};
 for(const viewport of[{width:852,height:393},{width:393,height:852},{width:360,height:640},{width:568,height:320}]){
  await page.setViewportSize(viewport);await page.waitForTimeout(200);await receive(settings);await inspect('settings-hub');
  await page.locator('#module-options').click();await inspect('session-shortcuts');await page.locator('#portal-close').click();
  for(const group of ['appearance',...settings.groups.map(g=>g.id)]){
   await page.evaluate(id=>{const title=id==='appearance'?'Atmosfera Nexo':nativeState.view.groups.find(g=>g.id===id).title;[...document.querySelectorAll('.nx-settings-tile')].find(b=>b.textContent.includes(title)).click();},group);
   await inspect('settings-'+group);await page.getByRole('button',{name:'Todas as configurações'}).click();
  }
  await receive({type:'files',path:'profiles/profile-2/resourcepacks/Uma pasta com nome comprido',selected:1,canPaste:true,roots:[{id:501,label:'Jogo'},{id:502,label:'Pacotes'},{id:503,label:'Capturas'}],entries:Array.from({length:12},(_,n)=>({name:'Arquivo com título bastante extenso '+n+'.hmod',key:'file'+n,directory:false,size:'125 KiB',kind:'H-mod',selected:n===0,open:510+n*6,select:511+n*6,details:512+n*6,rename:513+n*6,export:514+n*6,remove:515+n*6})),tools:['Pasta anterior','Importar','Nova pasta','Selecionar tudo','Limpar seleção','Copiar','Mover','Colar','Compartilhar seleção','Excluir seleção'].map((label,n)=>({label,id:600+n}))});await inspect('files');
  await page.getByRole('button',{name:'Organizar arquivos'}).click();await inspect('file-tools');await page.locator('#portal-close').click();
  await page.getByRole('button',{name:'Ações de Arquivo com título bastante extenso 0.hmod',exact:true}).click();await inspect('file-actions');await page.locator('#portal-close').click();
  await page.evaluate(()=>{nativeState.download={title:'Pacote de recursos com um nome bastante extenso',progress:.6};});await receive(nativeCatalog);await inspect('download-controls');await page.evaluate(()=>nativeState.download=null);
  const wardrobe={type:'wardrobe',capes:false,model:'slim',hideCape:false,preview:null,note:'Sua aparência local',items:[{id:'one',title:'Personagem de nome comprido para a coleção',active:true,choose:701}],cloudCapes:[],tools:{skins:702,capes:703,import:704,cloud:705,upload:706,model:707,hideCape:708}};await receive(wardrobe);await inspect('wardrobe');
  for(const mode of ['providers','offline','ely','signin','accounts']){await receive(settings,{type:'account',mode,pending:false,code:'ABCD1234',fields,actions,accounts:[{name:'Conta de personagem com nome comprido',provider:'Microsoft',choose:418}]});await inspect('account-'+mode);}
  for(const modal of[
   {type:'choice',title:'Escolha uma configuração para o dispositivo',description:'Opções disponíveis para esta instalação.',choices:[{id:801,title:'Opção padrão de longa duração',description:'Uma descrição bastante longa que precisa respeitar as margens do painel.',active:true},{id:802,title:'Compatibilidade',description:'Alternativa',active:false}]},
   {type:'confirm',title:'Excluir os arquivos selecionados?',description:'Confira os itens antes de continuar.',actions:[{id:803,label:'Cancelar'},{id:804,label:'Excluir arquivos'}]},
   {type:'form',title:'Renomear uma pasta com um título comprido',field:{id:805,kind:'field',label:'Novo nome',value:'Arquivo bastante comprido'},submit:806,close:807},
   {type:'project',title:'Um projeto com um título extenso e informações detalhadas',description:'Descrição do projeto e opções de instalação.',author:'Autor',license:'MIT',versions:[{id:808,title:'26.3 / versão estável',active:true,blocked:false}],screenshots:[],install:809,close:810},
   {type:'details',title:'Informações da sessão',description:'Estado atual do aplicativo.',preview:['Uma linha longa de informações que pode quebrar em várias linhas sem invadir o botão.'],close:811}
  ]){await receive(settings,modal);await inspect('modal-'+modal.type);}
  const placement={center:[.2,.65],size:.16,opacity:.8,width_ratio:1,height_ratio:1,roundness:6,visible_in_game:true};await receive({type:'editor',name:'Meu layout',mode:0,controls:[{index:0,title:'Movimento',custom:false,visible:true,placement}]});await inspect('editor');
  await page.locator('.nx-placed').click();await inspect('editor-inspector');
  await page.getByRole('button',{name:'Modo do layout'}).click();await inspect('editor-mode');await page.locator('#portal-close').click();await page.getByRole('button',{name:'Adicionar atalho'}).click();await inspect('editor-add');await page.locator('#portal-close').click();await page.getByRole('button',{name:'Restaurar',exact:true}).click();await inspect('editor-reset');await page.locator('#portal-close').click();
  await receive(settings);await page.screenshot({path:path.join(out,'audit-settings-'+viewport.width+'x'+viewport.height+'.png')});
 }
 await receive(settings);await page.evaluate(()=>{NexoApp.store.get().preferences.textScale=1.3;document.documentElement.style.setProperty('--scale',1.3);NexoScene.refresh();});await inspect('large-text-settings');
 await page.getByRole('button',{name:/Atmosfera Nexo/}).click();await inspect('large-text-atmosphere');
 await page.evaluate(()=>document.querySelector('#scene').getContext('webgl').getExtension('WEBGL_lose_context')?.loseContext());await page.waitForFunction(()=>document.querySelector('#nexo-app').classList.contains('nx-compatible'));await inspect('webgl-recovery');await page.screenshot({path:path.join(out,'audit-gpu-recovery.png')});
 assert.deepEqual(errors,[]);const result={scope:'visual geometry and component coverage with domain fixtures',screens:audits.length,controls:audits.reduce((n,a)=>n+a.controls,0),audits,errors};fs.writeFileSync(path.join(out,'AUDIT.json'),JSON.stringify(result,null,2));console.log('NEXO_AUDIT_RESULT '+JSON.stringify({screens:result.screens,controls:result.controls,errors}));await browser.close();server.close();
})().catch(async error=>{console.error(error);if(page)try{await page.screenshot({path:path.join(out,'audit-failure.png')});}catch(_){}server.close();process.exit(1);});
