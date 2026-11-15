/* Original Nexo preview state. Catalog entries are local UI examples, never game installations. */
(function (scope) {
  'use strict';
  const PAGES = ['Jogar', 'Mods', 'Pacotes', 'Shaders', 'Skins'];
  const DEFAULTS = { textScale: 1, reducedMotion: false, ambientMotion: true, sceneLighting: true, renderScale: 1, fps: 60, sound: .25, haptics: false, warmth: .5 };
  const EXAMPLES = [
    { id: 'presence', page: 'Mods', name: 'Primeira pessoa', category: 'Imersão', description: 'Exemplo de presença e movimento do personagem.', enabled: true },
    { id: 'audio', page: 'Mods', name: 'Áudio espacial', category: 'Som', description: 'Exemplo de uma camada de profundidade sonora.', enabled: false },
    { id: 'camera', page: 'Mods', name: 'Câmera dinâmica', category: 'Imersão', description: 'Exemplo de movimento da câmera.', enabled: true },
    { id: 'light', page: 'Mods', name: 'Luz atmosférica', category: 'Visual', description: 'Exemplo de luz, contraste e atmosfera.', enabled: false },
    { id: 'satin', page: 'Pacotes', name: 'Superfícies acetinadas', category: 'Texturas', description: 'Exemplo de pacote com acabamento suave.', enabled: true },
    { id: 'mineral', page: 'Pacotes', name: 'Camadas minerais', category: 'Texturas', description: 'Exemplo de pacote com detalhes minerais.', enabled: false },
    { id: 'studio', page: 'Shaders', name: 'Luz de estúdio', category: 'Iluminação', description: 'Exemplo de apresentação com luz direcional.', enabled: true },
    { id: 'dusk', page: 'Shaders', name: 'Horizonte azul', category: 'Iluminação', description: 'Exemplo de contraste entre luz fria e quente.', enabled: false }
  ];
  const clone = value => JSON.parse(JSON.stringify(value));
  const fold = value => String(value).normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLowerCase();
  const clean = (value, max = 120) => String(value || '').trim().slice(0, max);
  function fresh() {
    return { version: 1, selected: 0, profile: 'profile-1', profiles: [{ id: 'profile-1', name: 'Meu perfil', version: '26.3' }],
      preferences: clone(DEFAULTS), views: {}, catalog: EXAMPLES.map((entry, order) => ({ ...entry, order, favorite: false, changed: 0 })), skinTurn: 0 };
  }
  function validateCatalog(list) {
    if (!Array.isArray(list) || list.length > 500) throw Error('A lista deve conter no máximo 500 itens.');
    const ids = new Set();
    return list.map((row, order) => {
      if (!row || typeof row !== 'object' || !['Mods', 'Pacotes', 'Shaders'].includes(row.page)) throw Error('Uma seção do arquivo não é reconhecida.');
      const id = clean(row.id, 80), name = clean(row.name, 100);
      if (!id || !name || ids.has(id) || !/^[a-zA-Z0-9._-]+$/.test(id)) throw Error('Cada item precisa de nome e identificador únicos.');
      ids.add(id);
      return { id, name, page: row.page, category: clean(row.category, 40) || 'Geral', description: clean(row.description, 500),
        enabled: row.enabled === true, favorite: row.favorite === true, order: Number.isSafeInteger(row.order) && row.order >= 0 && row.order <= 500 ? row.order : order, changed: Number.isSafeInteger(row.changed) && row.changed > 0 ? row.changed : 0 };
    });
  }
  function normalize(raw) {
    const result = fresh();
    if (!raw || raw.version !== 1) return result;
    result.selected = Number.isInteger(raw.selected) ? Math.max(0, Math.min(4, raw.selected)) : 0;
    if (Array.isArray(raw.profiles)) {
      const seen = new Set();
      const profiles = raw.profiles.filter(p => p && clean(p.name, 50) && /^[a-zA-Z0-9._-]{1,80}$/.test(p.id || '') && !seen.has(p.id) && seen.add(p.id))
        .slice(0, 20).map(p => ({ id: p.id, name: clean(p.name, 50), version: clean(p.version, 20) || '26.3' }));
      if (profiles.length) result.profiles = profiles;
    }
    if (result.profiles.some(p => p.id === raw.profile)) result.profile = raw.profile;
    else result.profile = result.profiles[0].id;
    if (Array.isArray(raw.catalog)) { try { result.catalog = validateCatalog(raw.catalog); } catch (_) { /* keep valid starter data */ } }
    const p = raw.preferences || {};
    for (const [key, min, max] of [['textScale', 1, 1.4], ['renderScale', .6, 1], ['sound', 0, 1], ['warmth', 0, 1]]) {
      if (Number.isFinite(p[key])) result.preferences[key] = Math.max(min, Math.min(max, p[key]));
    }
    result.preferences.reducedMotion = p.reducedMotion === true;
    result.preferences.ambientMotion = p.ambientMotion !== false;
    result.preferences.sceneLighting = p.sceneLighting !== false;
    result.preferences.haptics = p.haptics === true;
    if ([30, 45, 60].includes(p.fps)) result.preferences.fps = p.fps;
    for (const page of PAGES) {
      const v = raw.views?.[page];
      if (v) result.views[page] = { query: clean(v.query, 100), filter: ['all', 'on', 'off', 'favorite'].includes(v.filter) ? v.filter : 'all',
        sort: ['name', 'recent', 'order'].includes(v.sort) ? v.sort : 'name', scroll: Math.max(0, Math.min(100000, Number(v.scroll) || 0)) };
    }
    result.skinTurn = Number.isFinite(raw.skinTurn) ? raw.skinTurn % (Math.PI * 2) : 0;
    return result;
  }
  function createStore(storage, clock = () => Date.now()) {
    const key = 'horizon.nexo.preview.v1';
    let state = fresh(), undo = null, appearanceUndo = null, storageError = !storage;
    try { state = normalize(JSON.parse(storage?.getItem(key) || 'null')); } catch (_) { storageError = true; }
    function save() { if (!storage) { storageError = true; return; } try { storage.setItem(key, JSON.stringify(state)); storageError = false; } catch (_) { storageError = true; } }
    function view(page) { return state.views[page] ||= { query: '', filter: 'all', sort: 'name', scroll: 0 }; }
    function rows(page) {
      const v = view(page), query = fold(v.query);
      const list = state.catalog.filter(row => row.page === page && fold(row.name + ' ' + row.description).includes(query)
        && (v.filter === 'all' || v.filter === 'on' && row.enabled || v.filter === 'off' && !row.enabled || v.filter === 'favorite' && row.favorite));
      return list.sort(v.sort === 'order' ? (a, b) => a.order - b.order : v.sort === 'recent' ? (a, b) => b.changed - a.changed || a.name.localeCompare(b.name, 'pt-BR') : (a, b) => a.name.localeCompare(b.name, 'pt-BR'));
    }
    return {
      snapshot: () => clone(state), get: () => state, storageFailed: () => storageError,
      select(index) { state.selected = Math.max(0, Math.min(4, Math.round(index))); save(); },
      view, rows,
      setView(page, patch) { Object.assign(view(page), patch); save(); },
      toggle(id, property = 'enabled') {
        const row = state.catalog.find(r => r.id === id); if (!row || !['enabled', 'favorite'].includes(property)) return false;
        undo = clone(state); row[property] = !row[property]; row.changed = clock(); save(); return row[property];
      },
      move(id, delta) {
        const row = state.catalog.find(r => r.id === id); if (!row) return;
        const peers = state.catalog.filter(r => r.page === row.page).sort((a, b) => a.order - b.order), index = peers.indexOf(row), next = index + delta;
        if (next < 0 || next >= peers.length) return;
        undo = clone(state); [peers[index], peers[next]] = [peers[next], peers[index]];
        peers.forEach((r, i) => { r.order = i; }); view(row.page).sort = 'order'; save();
      },
      undo() { if (!undo) return false; state = undo; undo = null; save(); return true; },
      beginEdit() { undo = clone(state); appearanceUndo = clone(state.preferences); },
      preference(name, value, checkpoint = true) {
        if (!(name in DEFAULTS)) return;
        if (checkpoint) { undo = clone(state); appearanceUndo = clone(state.preferences); } const candidate = clone(state); candidate.preferences[name] = value;
        state.preferences = normalize(candidate).preferences; save();
      },
      undoAppearance() { if (!appearanceUndo) return false; state.preferences = appearanceUndo; appearanceUndo = null; undo = null; save(); return true; },
      resetAppearance() { undo = clone(state); appearanceUndo = clone(state.preferences); Object.assign(state.preferences, { warmth: DEFAULTS.warmth, renderScale: DEFAULTS.renderScale, ambientMotion: DEFAULTS.ambientMotion, sceneLighting: DEFAULTS.sceneLighting }); save(); },
      chooseProfile(id) { if (state.profiles.some(p => p.id === id)) { state.profile = id; save(); } },
      addProfile(name) {
        name = clean(name, 50); if (!name) throw Error('Escreva um nome para o perfil.');
        if (state.profiles.length >= 20) throw Error('O limite desta prévia é de 20 perfis.');
        let id = 'profile-' + clock(), suffix = 0; while (state.profiles.some(p => p.id === id)) id = 'profile-' + clock() + '-' + (++suffix);
        state.profiles.push({ id, name, version: '26.3' }); state.profile = id; save(); return id;
      },
      skinTurn(value) { state.skinTurn = value % (Math.PI * 2); save(); },
      importText(text) {
        if (typeof text !== 'string' || text.length > 1024 * 1024) throw Error('Use um manifesto Nexo de até 1 MB.');
        let parsed; try { parsed = JSON.parse(text); } catch (_) { throw Error('O arquivo não contém JSON válido.'); }
        if (parsed.format !== 'HORIZON_NEXO_CATALOG_V1') throw Error('Este update aceita manifestos de catálogo Nexo (.json).');
        const catalog = validateCatalog(parsed.catalog); undo = clone(state); state.catalog = catalog; state.views = {}; save(); return catalog.length;
      },
      exportText: () => JSON.stringify({ format: 'HORIZON_NEXO_CATALOG_V1', catalog: state.catalog }, null, 2)
    };
  }
  const api = { PAGES, DEFAULTS, createStore, fresh, normalize, fold };
  if (typeof module === 'object' && module.exports) module.exports = api;
  else scope.NexoState = api;
})(typeof window === 'object' ? window : globalThis);
