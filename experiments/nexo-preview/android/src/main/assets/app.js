/* Nexo preview controls. Game/account/filesystem operations remain host capabilities. */
(function () {
  'use strict';
  if(window.HorizonHost)return;
  const $ = id => document.getElementById(id), root = $('nexo-app');
  let storage; try { storage = localStorage; } catch (_) { storage = null; }
  const store = NexoState.createStore(storage), dialogs = [...document.querySelectorAll('dialog:not(#intro-dialog)')];
  let page = null, listElement = null, queryElement = null, filterElement = null, sortElement = null, countElement = null, filterButton=null,clearButton=null;
  let scrollTimer = 0, noticeTimer = 0, launchBusy = false, rangeEdit = null, restoreScroll = false, pendingScroll = 0;
  function element(tag, attrs = {}, ...children) {
    const node = document.createElement(tag);
    for (const [key, value] of Object.entries(attrs)) {
      if (key.startsWith('on')) node.addEventListener(key.slice(2), value);
      else if (key === 'className') node.className = value;
      else node.setAttribute(key, String(value));
    }
    for (const child of children) node.append(child instanceof Node ? child : document.createTextNode(String(child)));
    return node;
  }
  function profile() { const state = store.get(); return state.profiles.find(p => p.id === state.profile) || state.profiles[0]; }
  function notice(message, undo = false) {
    clearTimeout(noticeTimer); $('notice').hidden = false; $('notice').querySelector('span').textContent = message;
    $('notice-undo').hidden = !undo;
    noticeTimer = setTimeout(() => { $('notice').hidden = true; }, undo ? 10000 : 7500);
  }
  function sound(kind = 'select') {
    const prefs=store.get().preferences;
    if (document.hidden) return;
    if (prefs.haptics && window.AndroidBridge?.haptic) window.AndroidBridge.haptic();
    NexoAudio.play(kind,prefs.sound);
  }
  let lastLabelKey = null,lastTextScale=null;
  function projectedTransform(points) {
    const [a,b,c,d]=points,dx1=b[0]-c[0],dx2=d[0]-c[0],dx3=a[0]-b[0]+c[0]-d[0],dy1=b[1]-c[1],dy2=d[1]-c[1],dy3=a[1]-b[1]+c[1]-d[1],det=dx1*dy2-dx2*dy1;
    if(Math.abs(det)<.000001)return;
    const g=(dx3*dy2-dx2*dy3)/det,h=(dx1*dy3-dx3*dy1)/det;
    const matrix=[(b[0]-a[0]+g*b[0])/1024,(b[1]-a[1]+g*b[1])/1024,0,g/1024,(d[0]-a[0]+h*d[0])/668,(d[1]-a[1]+h*d[1])/668,0,h/668,0,0,1,0,a[0],a[1],0,1];
    return 'matrix3d('+matrix.join(',')+')';
  }
  function projectLabel(points,index,opacity,blur=0){
    $('front-label').style.transform=projectedTransform(points);$('front-label').style.opacity=opacity;
    const labelKey=index+'|'+profile().name+'|'+profile().version;if(lastLabelKey===labelKey)return;lastLabelKey=labelKey;
    $('front-label').querySelector('.front-prefix').textContent=String(index+1).padStart(2,'0')+' / HORIZON';
    $('front-label').querySelector('.front-title').textContent=NexoState.PAGES[index];
    $('front-label').querySelector('.front-caption').textContent=index===0?profile().name+' · '+profile().version:['','Sua experiência','Camadas e texturas','Luz e atmosfera','Sua identidade'][index];
  }
  function refreshPreferences() {
    const prefs = store.get().preferences;
    NexoAudio.setVolume(prefs.sound);
    document.documentElement.style.setProperty('--scale', prefs.textScale);
    document.body.classList.toggle('reduced-motion', prefs.reducedMotion);
    document.querySelectorAll('[data-pref]').forEach(input => {
      const value = prefs[input.dataset.pref];
      if (input.type === 'checkbox') input.checked = value;
      else input.value = value;
    });
    $('profile-current').textContent = profile().name;
    $('profile-button').setAttribute('aria-label', 'Trocar perfil: ' + profile().name);
    if(window.NexoScene&&lastTextScale!==prefs.textScale){lastTextScale=prefs.textScale;NexoScene.updateLabels(true);}
    window.NexoScene?.refresh();
  }
  function openDialog(dialog) { if (!dialog.open) { dialog.showModal(); } }
  function closeDialog(dialog) { dialog.close(); }
  dialogs.forEach(dialog => {
    dialog.querySelector('[data-close]').addEventListener('click', () => {closeDialog(dialog);});
    dialog.addEventListener('click', event => { if (event.target === dialog) { const r = dialog.getBoundingClientRect(); if (event.clientX < r.left || event.clientX > r.right || event.clientY < r.top || event.clientY > r.bottom) closeDialog(dialog); } });
  });
  function rememberScroll() { if (page && listElement && !restoreScroll) store.setView(page, { scroll: listElement.scrollTop }); }
  function clearFilters() {
    store.setView(page, { query: '', filter: 'all', scroll: 0 });
    queryElement.value = ''; filterElement.value = 'all'; renderRows(); queryElement.focus({ preventScroll: true });
  }
  function showDetails(row) {
    rememberScroll(); $('details-title').textContent = row.name;
    const body = $('details-content'); body.replaceChildren(
      element('p', { className: 'details-description' }, row.description),
      element('p', { className: 'details-meta' }, row.category + ' · Compatibilidade desconhecida'),
      element('p', { className: 'note' }, 'Item de demonstração local. Esta seleção não instala nem altera mods no jogo.')
    );
    body.append(element('button', { type: 'button', 'aria-pressed': row.favorite, onclick: () => { store.toggle(row.id, 'favorite'); renderRows(); closeDialog($('details-dialog')); sound('confirm'); notice('Favorito atualizado.', true); } }, row.favorite ? 'Remover dos favoritos' : 'Adicionar aos favoritos'));
    if (row.page === 'Pacotes') {
      body.append(element('div', { className: 'actions' },
        element('button', { type: 'button', onclick: () => { store.move(row.id, -1); sortElement.value = 'order'; renderRows(); sound('confirm'); notice('Prioridade atualizada.', true); } }, 'Subir prioridade'),
        element('button', { type: 'button', onclick: () => { store.move(row.id, 1); sortElement.value = 'order'; renderRows(); sound('confirm'); notice('Prioridade atualizada.', true); } }, 'Descer prioridade')));
    }
    openDialog($('details-dialog'));
  }
  function renderRows() {
    if (!listElement || !page) return;
    const oldScroll = listElement.scrollTop, rows = store.rows(page), total = store.get().catalog.filter(r => r.page === page).length;
    listElement.replaceChildren(); countElement.textContent = rows.length + ' de ' + total + ' · prévia';clearButton.hidden=!queryElement.value;filterButton.setAttribute('aria-pressed',String(store.view(page).filter!=='all'||store.view(page).sort!=='name'));
    rows.forEach(row => {
      const detail = element('button', { type: 'button', className: 'row-detail', 'aria-label': 'Detalhes de ' + row.name, onclick: () => showDetails(row) },
        element('span', { className: 'row-name' }, row.name), element('span', { className: 'row-caption' }, (row.favorite ? 'Favorito · ' : '') + row.category));
      const toggle = element('button', { type: 'button', className: 'row-toggle', 'aria-pressed': row.enabled, 'aria-label': (row.enabled ? 'Desativar ' : 'Ativar ') + row.name,
        onclick: () => { store.toggle(row.id); renderRows(); sound('toggle'); notice('Seleção da prévia atualizada.', true); } }, element('span',{className:'switch-track','aria-hidden':'true'}),element('span',{className:'switch-caption'},row.enabled?'Ativado':'Desativado'));
      listElement.append(element('article', { className: 'catalog-row', 'data-item': row.id }, detail, toggle));
    });
    if (!rows.length) {
      const empty = element('div', { className: 'empty' }, element('p', {}, total ? 'Nenhum exemplo corresponde à busca e aos filtros.' : 'Sua lista de prévia está vazia. Importe um manifesto Nexo para começar.'));
      if (total) empty.append(element('button', { type: 'button', onclick: clearFilters }, 'Limpar busca e filtros'));
      listElement.append(empty);
    }
    listElement.scrollTop = oldScroll;
  }
  function icon(path){const svg=document.createElementNS('http://www.w3.org/2000/svg','svg'),shape=document.createElementNS(svg.namespaceURI,'path');for(const [key,value] of Object.entries({viewBox:'0 0 24 24',width:'20',height:'20',fill:'none','aria-hidden':'true'}))svg.setAttribute(key,value);for(const [key,value] of Object.entries({d:path,stroke:'currentColor','stroke-width':'1.5','stroke-linecap':'round','stroke-linejoin':'round'}))shape.setAttribute(key,value);svg.append(shape);return svg;}
  function buildCatalog() {
    const view=store.view(page),content=$('module-content');
    queryElement=element('input',{type:'search',placeholder:'Buscar em '+page,'aria-label':'Buscar em '+page,autocomplete:'off',maxlength:100});queryElement.value=view.query;
    queryElement.addEventListener('input',()=>{store.setView(page,{query:queryElement.value,scroll:0});renderRows();listElement.scrollTop=0;});
    filterElement=element('select',{'aria-label':'Filtrar estado'});for(const [value,label] of [['all','Todos'],['on','Ativados'],['off','Desativados'],['favorite','Favoritos']])filterElement.append(element('option',{value},label));filterElement.value=view.filter;
    filterElement.addEventListener('change',()=>{store.setView(page,{filter:filterElement.value,scroll:0});renderRows();listElement.scrollTop=0;});
    sortElement=element('select',{'aria-label':'Ordenar biblioteca'});for(const [value,label] of [['name','Nome'],['recent','Alterados'],['order','Prioridade']])sortElement.append(element('option',{value},label));sortElement.value=view.sort;
    sortElement.addEventListener('change',()=>{store.setView(page,{sort:sortElement.value});renderRows();});
    clearButton=element('button',{type:'button',className:'catalog-clear','aria-label':'Limpar busca',onclick:()=>{queryElement.value='';queryElement.dispatchEvent(new Event('input'));queryElement.focus({preventScroll:true});}},icon('M7 7l10 10M17 7 7 17'));
    const search=element('div',{className:'catalog-search'},icon('M16 16l5 5M18 10a8 8 0 1 1-16 0 8 8 0 0 1 16 0'),queryElement,clearButton);
    filterButton=element('button',{type:'button',className:'catalog-filter-toggle','aria-label':'Filtrar e ordenar','aria-haspopup':'dialog',onclick:()=>openDialog($('catalog-filters-dialog'))},icon('M4 6h16M4 12h16M4 18h16M9 3v6M15 9v6M9 15v6'));
    $('catalog-filters-body').replaceChildren(element('label',{},'Mostrar',filterElement),element('label',{},'Ordenar',sortElement),element('button',{type:'button',onclick:()=>{store.setView(page,{filter:'all',sort:'name',scroll:0});filterElement.value='all';sortElement.value='name';renderRows();listElement.scrollTop=0;}},'Restaurar filtros'));
    countElement=$('module-eyebrow');listElement=element('div',{className:'catalog-list','aria-label':'Resultados da biblioteca'});
    listElement.addEventListener('scroll',()=>{if(restoreScroll)return;clearTimeout(scrollTimer);scrollTimer=setTimeout(rememberScroll,150);},{passive:true});
    content.append(element('div',{className:'catalog-tools'},search,filterButton),listElement);pendingScroll=view.scroll;restoreScroll=true;renderRows();
  }
  function buildSkins() {
    const area = element('div', { className: 'skin-space', role: 'img', 'aria-label': 'Personagem de demonstração. Arraste horizontalmente para girar.' });
    let gesture = null;
    area.addEventListener('pointerdown', event => { if (!event.isPrimary || event.button !== 0) return; gesture = { id: event.pointerId, x: event.clientX, start: store.get().skinTurn }; });
    area.addEventListener('pointermove', event => { if (!gesture || gesture.id !== event.pointerId || Math.abs(event.clientX - gesture.x) < 6) return; area.setPointerCapture(event.pointerId); store.skinTurn(gesture.start + (event.clientX - gesture.x) * .015); NexoScene.refresh(); });
    for (const name of ['pointerup', 'pointercancel', 'lostpointercapture']) area.addEventListener(name, () => { gesture = null; });
    $('module-content').append(area, element('div', { className: 'skin-actions' }, element('button', { type: 'button', onclick: () => { store.skinTurn(store.get().skinTurn - .45); NexoScene.refresh(); } }, 'Girar à esquerda'), element('button', { type: 'button', onclick: () => { store.skinTurn(store.get().skinTurn + .45); NexoScene.refresh(); } }, 'Girar à direita')),
      element('p', { className: 'note' }, 'Personagem de demonstração. Importação de skins e conexão com a conta ainda pendentes.'));
  }
  function moduleChanged(name, isOpen) {
    root.classList.toggle('has-module', isOpen);if(isOpen)document.querySelector('.app-header').inert=true;
    if (!isOpen) { rememberScroll(); $('module').inert = true; return; }
    if (page === name) return;
    rememberScroll(); page = name; listElement = null; restoreScroll = false;
    $('module-options').hidden = false;for(const id of ['import-catalog','export-catalog','reset-filters'])$(id).hidden=name==='Skins';
    $('module-title').textContent = name; $('module-eyebrow').textContent = name === 'Skins' ? 'PERSONAGEM DE PRÉVIA' : 'BIBLIOTECA DE PRÉVIA';
    $('module-content').replaceChildren(); $('module').hidden = false; $('module').inert = true;
    if (name === 'Skins') buildSkins(); else buildCatalog();
  }
  async function exportCatalog() {
    const text = store.exportText();sound('confirm');
    try {
      if (window.AndroidBridge?.saveBase64) {
        const bytes = new TextEncoder().encode(text); let binary = ''; for (const byte of bytes) binary += String.fromCharCode(byte);
        AndroidBridge.saveBase64('nexo-catalogo.json', 'application/json', btoa(binary)); notice('Exportação enviada para Downloads/Nexo.');
      } else {
        const url = URL.createObjectURL(new Blob([text], { type: 'application/json' })), link = element('a', { href: url, download: 'nexo-catalogo.json' });
        document.body.append(link); link.click(); link.remove(); setTimeout(() => URL.revokeObjectURL(url), 1000); notice('Catálogo exportado.');
      }
    } catch (_) { notice('Não foi possível exportar. Sua lista foi preservada.'); }
  }
  async function play() {
    if (launchBusy) return;
    const host = window.NexoNativeHost;
    if (!host || typeof host.launchGame !== 'function') { sound('error');notice('Este APK testa a interface. O lançamento pelo Horizon ainda está pendente.'); return; }
    launchBusy = true;sound('confirm'); NexoScene.setLaunching(true);
    try { await host.launchGame({ profile: { ...profile() } }, message => { root.querySelector('.nexo-state').textContent = String(message).slice(0, 150); }); }
    catch (error) { notice('Não foi possível iniciar: ' + String(error.message || error).slice(0, 180)); }
    finally { launchBusy = false; NexoScene.setLaunching(false); }
  }
  function back(fromAndroid = false) {
    if(window.NexoIntro?.isActive()){NexoIntro.skip();return true;}
    const active = document.activeElement;
    if (!fromAndroid && active && ['INPUT', 'TEXTAREA', 'SELECT'].includes(active.tagName)) { active.blur(); return true; }
    const dialog = dialogs.slice().reverse().find(d => d.open);
    if (dialog) { dialog.close(); return true; }
    if (NexoScene.getState().open || NexoScene.getState().launching) { NexoScene.close(); return true; }
    return false;
  }
  function showProfiles() {
    const list = $('profile-list'); list.replaceChildren();
    store.get().profiles.forEach(p => list.append(element('button', { type: 'button', className: 'profile-choice', 'aria-pressed': p.id === store.get().profile,
      onclick: () => { store.chooseProfile(p.id);sound('confirm'); refreshPreferences(); NexoScene.updateLabels(); $('profiles-dialog').close(); } }, p.name, element('small', {}, p.version + ' · Perfil de prévia'))));
    openDialog($('profiles-dialog'));
  }
  $('module-options').addEventListener('click', () => openDialog($('catalog-options-dialog')));
  $('import-catalog').addEventListener('click', () => { $('catalog-options-dialog').close(); $('catalog-file').click(); });
  $('export-catalog').addEventListener('click', () => { $('catalog-options-dialog').close(); exportCatalog(); });
  $('reset-filters').addEventListener('click', () => { $('catalog-options-dialog').close(); clearFilters(); });
  $('profile-button').addEventListener('click', showProfiles);
  $('profile-form').addEventListener('submit', event => {
    event.preventDefault(); try { store.addProfile($('profile-name').value);sound('confirm'); $('profile-name').value = ''; $('profile-error').textContent = ''; refreshPreferences(); NexoScene.updateLabels(); showProfiles(); }
    catch (error) { $('profile-error').textContent = error.message; }
  });
  function showSettings(){refreshPreferences();openDialog($('settings-dialog'));}
  $('settings-button').addEventListener('click',showSettings);
  $('module-settings').addEventListener('click',()=>{$('catalog-options-dialog').close();showSettings();});
  $('module-profile').addEventListener('click',()=>{$('catalog-options-dialog').close();showProfiles();});
  $('replay-intro').addEventListener('click',()=>{$('settings-dialog').close();NexoIntro.start({replay:true});});
  document.querySelectorAll('[data-pref]').forEach(input => {
    input.addEventListener(input.type === 'range' ? 'input' : 'change', () => {
      const name = input.dataset.pref, value = input.type === 'checkbox' ? input.checked : Number(input.value);
      if (input.type === 'range' && rangeEdit !== name) { store.beginEdit(); rangeEdit = name; }
      store.preference(name, value, input.type !== 'range'); refreshPreferences();
    });
    if (input.type === 'range') { input.addEventListener('change', () => { rangeEdit = null;if(input.dataset.pref==='sound')sound('select'); }); input.addEventListener('blur', () => { rangeEdit = null; }); }
  });
  $('reset-appearance').addEventListener('click', () => { store.resetAppearance(); refreshPreferences(); notice('Aparência restaurada.', true); });
  function undo() { if (store.undo()) { refreshPreferences(); if (page && page !== 'Skins') renderRows(); notice('Alteração desfeita.'); } }
  $('undo-appearance').addEventListener('click', () => { if (store.undoAppearance()) { refreshPreferences(); notice('Aparência anterior restaurada.'); } }); $('notice-undo').addEventListener('click', undo);
  $('notice-close').addEventListener('click', () => { $('notice').hidden = true; });
  $('catalog-file').addEventListener('change', async event => {
    const file = event.target.files?.[0]; if (!file) return;
    if (file.size > 1024 * 1024) { notice('Use um manifesto Nexo de até 1 MB.'); event.target.value = ''; return; }
    try { const count = store.importText(await file.text()); if (page && page !== 'Skins') { $('module-content').replaceChildren(); buildCatalog(); } sound('confirm'); notice(count + ' itens importados na prévia.', true); }
    catch (error) { sound('error'); notice(error.message + ' Sua lista anterior foi preservada.'); }
    event.target.value = '';
  });
  document.addEventListener('keydown', event => {
    if (event.key === 'Escape') { if (back()) event.preventDefault(); return; }
  });
  function setVisible(visible) {
    if (!visible) rememberScroll();
    NexoAudio.setVisible(visible);window.NexoIntro?.setVisible(visible);
    window.NexoScene?.setVisible(visible);
  }
  document.addEventListener('visibilitychange', () => setVisible(!document.hidden));
  window.addEventListener('pagehide', rememberScroll);
  window.NexoApp = {
    store, profile, play, sound, onHomeFace:projectLabel,hideMovingLabel(){$('front-label').style.opacity=0;}, onTabFace(points,index){document.querySelector('[data-card="'+index+'"]').style.transform=projectedTransform(points);}, skinRect(){ if(page!=='Skins')return null;const r=document.querySelector('.skin-space').getBoundingClientRect(),c=$('scene').getBoundingClientRect();return {x:r.left-c.left,y:r.top-c.top,width:r.width,height:r.height}; }, ready() { refreshPreferences(); if (store.storageFailed()) notice('O armazenamento está indisponível; esta sessão continuará em memória.'); },
    selectionChanged(index, isOpen) { const changed = store.get().selected !== index; store.select(index); moduleChanged(NexoState.PAGES[index], isOpen); if (changed) sound(); },
    onFrame({ opening, rect }) {
      root.style.setProperty('--module-open',opening.toFixed(4));document.querySelector('.app-header').inert=opening>.01;document.querySelector('.app-footer').setAttribute('aria-hidden',String(opening>.01));
      if (!page) return;
      if(opening<.001&&!root.classList.contains('has-module')){$('module').hidden=true;page=null;return;}
      Object.assign($('module').style, { left: rect.left + 'px', top: rect.top + 'px', width: rect.width + 'px', height: rect.height + 'px', opacity: Math.max(0, Math.min(1, (opening - .76) / .24)) });
      $('module').inert = opening < .97 || !root.classList.contains('has-module');
      if (opening >= .9999 && root.classList.contains('has-module') && restoreScroll && listElement) { listElement.scrollTop = pendingScroll; restoreScroll = false; store.setView(page, {scroll:listElement.scrollTop}); }
    }
  };
  window.Nexo = { back, setVisible, state: () => ({ ...store.snapshot(), launchBusy, page }) };
  window.JadeLab = { back };
})();
