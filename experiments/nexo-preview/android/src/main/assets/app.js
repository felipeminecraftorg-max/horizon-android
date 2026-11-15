/* Nexo preview controls. Game/account/filesystem operations remain host capabilities. */
(function () {
  'use strict';
  const $ = id => document.getElementById(id), root = $('nexo-app');
  let storage; try { storage = localStorage; } catch (_) { storage = null; }
  const store = NexoState.createStore(storage), dialogs = [...document.querySelectorAll('dialog')];
  let page = null, listElement = null, queryElement = null, filterElement = null, sortElement = null, countElement = null;
  let scrollTimer = 0, noticeTimer = 0, audio = null, launchBusy = false, rangeEdit = null, restoreScroll = false, pendingScroll = 0;
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
    const volume = store.get().preferences.sound;
    if (document.hidden) return;
    if (store.get().preferences.haptics && window.AndroidBridge?.haptic) window.AndroidBridge.haptic();
    if (!volume) return;
    try {
      audio ||= new (window.AudioContext || window.webkitAudioContext)();
      if (audio.state === 'suspended') audio.resume();
      const oscillator = audio.createOscillator(), gain = audio.createGain(), now = audio.currentTime;
      oscillator.type = 'sine'; oscillator.frequency.setValueAtTime(kind === 'open' ? 520 : 660, now);
      oscillator.frequency.exponentialRampToValueAtTime(kind === 'open' ? 780 : 570, now + .075);
      gain.gain.setValueAtTime(volume * .045, now); gain.gain.exponentialRampToValueAtTime(.0001, now + .10);
      oscillator.connect(gain); gain.connect(audio.destination); oscillator.start(now); oscillator.stop(now + .11);
    } catch (_) { /* audio is optional */ }
  }
  let lastLabelKey = null;
  function projectLabel(points, index, opacity) {
    const [a,b,c,d]=points,dx1=b[0]-c[0],dx2=d[0]-c[0],dx3=a[0]-b[0]+c[0]-d[0],dy1=b[1]-c[1],dy2=d[1]-c[1],dy3=a[1]-b[1]+c[1]-d[1],det=dx1*dy2-dx2*dy1;
    if(Math.abs(det)<.000001)return;
    const g=(dx3*dy2-dx2*dy3)/det,h=(dx1*dy3-dx3*dy1)/det;
    const matrix=[(b[0]-a[0]+g*b[0])/1024,(b[1]-a[1]+g*b[1])/1024,0,g/1024,(d[0]-a[0]+h*d[0])/668,(d[1]-a[1]+h*d[1])/668,0,h/668,0,0,1,0,a[0],a[1],0,1];
    $('front-label').style.transform='matrix3d('+matrix.join(',')+')';$('front-label').style.opacity=opacity;
    const labelKey=index+'|'+profile().name+'|'+profile().version;if(lastLabelKey===labelKey)return;lastLabelKey=labelKey;
    $('front-label').querySelector('.front-prefix').textContent=String(index+1).padStart(2,'0')+' / HORIZON';
    $('front-label').querySelector('.front-title').textContent=NexoState.PAGES[index];
    $('front-label').querySelector('.front-caption').textContent=index===0?profile().name+' · '+profile().version:['','Sua experiência','Camadas e texturas','Luz e atmosfera','Sua identidade'][index];
  }
  function refreshPreferences() {
    const prefs = store.get().preferences;
    document.documentElement.style.setProperty('--scale', prefs.textScale);
    document.body.classList.toggle('reduced-motion', prefs.reducedMotion);
    document.querySelectorAll('[data-pref]').forEach(input => {
      const value = prefs[input.dataset.pref];
      if (input.type === 'checkbox') input.checked = value;
      else input.value = value;
    });
    $('profile-button').textContent = profile().name;
    $('profile-button').setAttribute('aria-label', 'Trocar perfil: ' + profile().name);
    window.NexoScene?.refresh();
  }
  function openDialog(dialog) { if (!dialog.open) dialog.showModal(); }
  function closeDialog(dialog) { dialog.close(); }
  dialogs.forEach(dialog => {
    dialog.querySelector('[data-close]').addEventListener('click', () => closeDialog(dialog));
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
    body.append(element('button', { type: 'button', 'aria-pressed': row.favorite, onclick: () => { store.toggle(row.id, 'favorite'); renderRows(); closeDialog($('details-dialog')); notice('Favorito atualizado.', true); } }, row.favorite ? 'Remover dos favoritos' : 'Adicionar aos favoritos'));
    if (row.page === 'Pacotes') {
      body.append(element('div', { className: 'actions' },
        element('button', { type: 'button', onclick: () => { store.move(row.id, -1); sortElement.value = 'order'; renderRows(); notice('Prioridade atualizada.', true); } }, 'Subir prioridade'),
        element('button', { type: 'button', onclick: () => { store.move(row.id, 1); sortElement.value = 'order'; renderRows(); notice('Prioridade atualizada.', true); } }, 'Descer prioridade')));
    }
    openDialog($('details-dialog'));
  }
  function renderRows() {
    if (!listElement || !page) return;
    const oldScroll = listElement.scrollTop, rows = store.rows(page), total = store.get().catalog.filter(r => r.page === page).length;
    listElement.replaceChildren(); countElement.textContent = rows.length + ' de ' + total + ' exemplos locais';
    rows.forEach(row => {
      const detail = element('button', { type: 'button', className: 'row-detail', 'aria-label': 'Detalhes de ' + row.name, onclick: () => showDetails(row) },
        element('span', { className: 'row-name' }, row.name), element('span', { className: 'row-caption' }, (row.favorite ? 'Favorito · ' : '') + row.category));
      const toggle = element('button', { type: 'button', className: 'row-toggle', 'aria-pressed': row.enabled, 'aria-label': (row.enabled ? 'Desativar ' : 'Ativar ') + row.name,
        onclick: () => { store.toggle(row.id); renderRows(); sound(); notice('Seleção da prévia atualizada.', true); } }, row.enabled ? 'Ativado' : 'Desativado');
      listElement.append(element('article', { className: 'catalog-row', 'data-item': row.id }, detail, toggle));
    });
    if (!rows.length) {
      const empty = element('div', { className: 'empty' }, element('p', {}, total ? 'Nenhum exemplo corresponde à busca e aos filtros.' : 'Sua lista de prévia está vazia. Importe um manifesto Nexo para começar.'));
      if (total) empty.append(element('button', { type: 'button', onclick: clearFilters }, 'Limpar busca e filtros'));
      listElement.append(empty);
    }
    listElement.scrollTop = oldScroll;
  }
  function buildCatalog() {
    const view = store.view(page), content = $('module-content');
    queryElement = element('input', { type: 'search', placeholder: 'Buscar na biblioteca', 'aria-label': 'Buscar em ' + page, autocomplete: 'off', maxlength: 100 }); queryElement.value = view.query;
    queryElement.addEventListener('input', () => { store.setView(page, { query: queryElement.value, scroll: 0 }); renderRows(); listElement.scrollTop = 0; });
    filterElement = element('select', { 'aria-label': 'Filtrar estado' });
    for (const [value, label] of [['all', 'Todos'], ['on', 'Ativados'], ['off', 'Desativados'], ['favorite', 'Favoritos']]) filterElement.append(element('option', { value }, label));
    filterElement.value = view.filter;
    filterElement.addEventListener('change', () => { store.setView(page, { filter: filterElement.value, scroll: 0 }); renderRows(); listElement.scrollTop = 0; });
    sortElement = element('select', { 'aria-label': 'Ordenar biblioteca' });
    for (const [value, label] of [['name', 'Nome'], ['recent', 'Alterados'], ['order', 'Prioridade']]) sortElement.append(element('option', { value }, label));
    sortElement.value = view.sort;
    sortElement.addEventListener('change', () => { store.setView(page, { sort: sortElement.value }); renderRows(); });
    const search = element('div', { className: 'catalog-search' }, queryElement, element('button', { type: 'button', 'aria-label': 'Limpar busca', onclick: () => { queryElement.value = ''; queryElement.dispatchEvent(new Event('input')); queryElement.focus({ preventScroll: true }); } }, 'Limpar'));
    countElement = element('p', { className: 'result-count', 'aria-live': 'polite' });
    listElement = element('div', { className: 'catalog-list', 'aria-label': 'Resultados da biblioteca' });
    listElement.addEventListener('scroll', () => { if(restoreScroll)return;clearTimeout(scrollTimer); scrollTimer = setTimeout(rememberScroll, 150); }, { passive: true });
    content.append(element('div', { className: 'catalog-filter' }, search, filterElement, sortElement), countElement, listElement,
      element('p', { className: 'catalog-hint' }, 'Exemplos para testar a interface; sem instalação no jogo.'));
    pendingScroll = view.scroll; restoreScroll = true; renderRows(); listElement.scrollTop = pendingScroll;
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
    root.classList.toggle('has-module', isOpen);
    if (!isOpen) { rememberScroll(); page = null; $('module').inert = true; $('module').hidden = true; return; }
    if (page === name) return;
    rememberScroll(); page = name; listElement = null; restoreScroll = false;
    $('module-options').hidden = name === 'Skins';
    $('module-title').textContent = name; $('module-eyebrow').textContent = name === 'Skins' ? 'PERSONAGEM DE PRÉVIA' : 'BIBLIOTECA DE PRÉVIA';
    $('module-content').replaceChildren(); $('module').hidden = false; $('module').inert = true;
    if (name === 'Skins') buildSkins(); else buildCatalog();
  }
  async function exportCatalog() {
    const text = store.exportText();
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
    if (!host || typeof host.launchGame !== 'function') { notice('Este APK testa a interface. O lançamento pelo Horizon ainda está pendente.'); return; }
    launchBusy = true; NexoScene.setLaunching(true);
    try { await host.launchGame({ profile: { ...profile() } }, message => { root.querySelector('.nexo-state').textContent = String(message).slice(0, 150); }); }
    catch (error) { notice('Não foi possível iniciar: ' + String(error.message || error).slice(0, 180)); }
    finally { launchBusy = false; NexoScene.setLaunching(false); }
  }
  function back(fromAndroid = false) {
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
      onclick: () => { store.chooseProfile(p.id); refreshPreferences(); NexoScene.updateLabels(); $('profiles-dialog').close(); } }, p.name, element('small', {}, p.version + ' · Perfil de prévia'))));
    openDialog($('profiles-dialog'));
  }
  $('module-options').addEventListener('click', () => openDialog($('catalog-options-dialog')));
  $('import-catalog').addEventListener('click', () => { $('catalog-options-dialog').close(); $('catalog-file').click(); });
  $('export-catalog').addEventListener('click', () => { $('catalog-options-dialog').close(); exportCatalog(); });
  $('reset-filters').addEventListener('click', () => { $('catalog-options-dialog').close(); clearFilters(); });
  $('profile-button').addEventListener('click', showProfiles);
  $('profile-form').addEventListener('submit', event => {
    event.preventDefault(); try { store.addProfile($('profile-name').value); $('profile-name').value = ''; $('profile-error').textContent = ''; refreshPreferences(); NexoScene.updateLabels(); showProfiles(); }
    catch (error) { $('profile-error').textContent = error.message; }
  });
  $('settings-button').addEventListener('click', () => { refreshPreferences(); openDialog($('settings-dialog')); });
  document.querySelectorAll('[data-pref]').forEach(input => {
    input.addEventListener(input.type === 'range' ? 'input' : 'change', () => {
      const name = input.dataset.pref, value = input.type === 'checkbox' ? input.checked : Number(input.value);
      if (input.type === 'range' && rangeEdit !== name) { store.beginEdit(); rangeEdit = name; }
      store.preference(name, value, input.type !== 'range'); refreshPreferences();
    });
    if (input.type === 'range') { input.addEventListener('change', () => { rangeEdit = null; }); input.addEventListener('blur', () => { rangeEdit = null; }); }
  });
  $('reset-appearance').addEventListener('click', () => { store.resetAppearance(); refreshPreferences(); notice('Aparência restaurada.', true); });
  function undo() { if (store.undo()) { refreshPreferences(); if (page && page !== 'Skins') renderRows(); notice('Alteração desfeita.'); } }
  $('undo-appearance').addEventListener('click', () => { if (store.undoAppearance()) { refreshPreferences(); notice('Aparência anterior restaurada.'); } }); $('notice-undo').addEventListener('click', undo);
  $('notice-close').addEventListener('click', () => { $('notice').hidden = true; });
  $('catalog-file').addEventListener('change', async event => {
    const file = event.target.files?.[0]; if (!file) return;
    if (file.size > 1024 * 1024) { notice('Use um manifesto Nexo de até 1 MB.'); event.target.value = ''; return; }
    try { const count = store.importText(await file.text()); if (page && page !== 'Skins') { $('module-content').replaceChildren(); buildCatalog(); } notice(count + ' itens importados na prévia.', true); }
    catch (error) { notice(error.message + ' Sua lista anterior foi preservada.'); }
    event.target.value = '';
  });
  document.addEventListener('keydown', event => {
    if (event.key === 'Escape') { if (back()) event.preventDefault(); return; }
    if (document.querySelector('dialog[open]') || ['INPUT', 'TEXTAREA', 'SELECT'].includes(event.target.tagName) || !window.NexoScene || NexoScene.getState().open) return;
    if (['ArrowRight', 'ArrowLeft', 'Home', 'End'].includes(event.key)) {
      const index = store.get().selected;
      NexoScene.select(event.key === 'Home' ? 0 : event.key === 'End' ? 4 : (index + (event.key === 'ArrowRight' ? 1 : 4)) % 5); event.preventDefault();
    }
  });
  function setVisible(visible) {
    if (!visible) { rememberScroll(); if (audio?.state === 'running') audio.suspend(); }
    window.NexoScene?.setVisible(visible);
  }
  document.addEventListener('visibilitychange', () => setVisible(!document.hidden));
  window.addEventListener('pagehide', rememberScroll);
  window.NexoApp = {
    store, profile, play, sound, onHomeFace:projectLabel, skinRect(){ if(page!=='Skins')return null;const r=document.querySelector('.skin-space').getBoundingClientRect(),c=$('scene').getBoundingClientRect();return {x:r.left-c.left,y:r.top-c.top,width:r.width,height:r.height}; }, ready() { refreshPreferences(); if (store.storageFailed()) notice('O armazenamento está indisponível; esta sessão continuará em memória.'); },
    selectionChanged(index, isOpen) { const changed = store.get().selected !== index; store.select(index); moduleChanged(NexoState.PAGES[index], isOpen); if (changed) sound(); },
    onFrame({ opening, rect }) {
      if (!page) return;
      Object.assign($('module').style, { left: rect.left + 'px', top: rect.top + 'px', width: rect.width + 'px', height: rect.height + 'px', opacity: Math.max(0, Math.min(1, (opening - .76) / .24)) });
      $('module').inert = opening < .97;
      if (opening >= .97 && restoreScroll && listElement) { listElement.scrollTop = pendingScroll; restoreScroll = false; store.setView(page, {scroll:listElement.scrollTop}); }
    }
  };
  window.Nexo = { back, setVisible, state: () => ({ ...store.snapshot(), launchBusy, page }) };
  window.JadeLab = { back };
})();
