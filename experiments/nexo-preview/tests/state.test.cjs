'use strict';
const test = require('node:test'), assert = require('node:assert/strict');
const { createStore } = require('../android/src/main/assets/state.js');
function memory() { const data = new Map(); return { getItem: key => data.get(key) || null, setItem: (key, value) => data.set(key, value) }; }
test('local search tolerates accents/case and keeps independent section filters', () => {
  const s = createStore(memory()); s.setView('Mods', { query: 'AUDIO', filter: 'off' });
  assert.deepEqual(s.rows('Mods').map(r => r.name), ['Áudio espacial']);
  assert.equal(s.rows('Pacotes').length, 2); s.setView('Mods', { filter: 'on' }); assert.equal(s.rows('Mods').length, 0);
});
test('preview toggles and favorites persist separately and can be undone', () => {
  const storage = memory(), s = createStore(storage, () => 500);
  s.toggle('audio'); s.toggle('audio', 'favorite'); let again = createStore(storage);
  assert.equal(again.get().catalog.find(r => r.id === 'audio').enabled, true);
  again.setView('Mods', { filter: 'favorite' }); assert.deepEqual(again.rows('Mods').map(r => r.id), ['audio']);
  s.undo(); again = createStore(storage); assert.equal(again.get().catalog.find(r => r.id === 'audio').enabled, true); assert.equal(again.get().catalog.find(r => r.id === 'audio').favorite, false);
});
test('pack priority changes affect only peers and obey boundaries', () => {
  const s = createStore(memory()); s.move('mineral', -1);
  assert.deepEqual(s.rows('Pacotes').map(r => r.id), ['mineral', 'satin']);
  s.move('mineral', -1); assert.deepEqual(s.rows('Pacotes').map(r => r.id), ['mineral', 'satin']);
  assert.equal(s.get().catalog.filter(r => r.page === 'Mods').length, 4);
  s.undo(); s.setView('Pacotes', { sort: 'order' }); assert.deepEqual(s.rows('Pacotes').map(r => r.id), ['satin', 'mineral']);
});
test('invalid imports are atomic and preserve the current search and selection', () => {
  const s = createStore(memory()); s.setView('Mods', { query: 'audio' }); s.select(3); const before = s.snapshot();
  for (const text of ['{', JSON.stringify({format:'other'}), JSON.stringify({format:'HORIZON_NEXO_CATALOG_V1',catalog:[{id:'a',name:'A',page:'Mods'},{id:'a',name:'B',page:'Mods'}]})]) assert.throws(() => s.importText(text));
  assert.deepEqual(s.snapshot(), before);
});
test('export/import roundtrip preserves user text as data and supports an empty catalog', () => {
  const s = createStore(memory()); const payload = {format:'HORIZON_NEXO_CATALOG_V1',catalog:[{id:'safe',page:'Mods',name:'<b>Áudio & câmera</b>',description:'Uma descrição\ncom duas linhas',enabled:true}]};
  assert.equal(s.importText(JSON.stringify(payload)), 1); const second = createStore(memory()); second.importText(s.exportText());
  assert.equal(second.rows('Mods')[0].name, '<b>Áudio & câmera</b>'); assert.equal(second.rows('Mods')[0].description, 'Uma descrição\ncom duas linhas');
  second.importText(JSON.stringify({format:'HORIZON_NEXO_CATALOG_V1',catalog:[]})); assert.equal(second.rows('Mods').length, 0);
});
test('a range editing session undoes to its starting value', () => {
  const s = createStore(memory()); s.beginEdit(); s.preference('warmth', .7, false); s.preference('warmth', .9, false); s.undo(); assert.equal(s.get().preferences.warmth, .5);
});
test('profile selection survives restart and names remain bounded', () => {
  const storage = memory(), s = createStore(storage, () => 1); const id = s.addProfile('  Teste  '); s.addProfile('Outro'); s.chooseProfile(id);
  const again = createStore(storage); assert.equal(again.get().profiles.find(p => p.id === again.get().profile).name, 'Teste');
  assert.equal(new Set(again.get().profiles.map(p => p.id)).size, 3); assert.throws(() => s.addProfile('   '));
});
test('storage failure keeps the session usable without reporting a persisted result', () => {
  const s = createStore({getItem(){throw Error('blocked')},setItem(){throw Error('full')}}); s.toggle('audio'); assert.equal(s.get().catalog.find(r => r.id === 'audio').enabled, true); assert.equal(s.storageFailed(), true);
});
test('malformed persisted preferences are bounded and a valid profile is selected', () => {
  const storage = memory(); storage.setItem('horizon.nexo.preview.v1', JSON.stringify({version:1,selected:100,profile:'missing',profiles:[{id:'x',name:'Nome',version:'26.3'}],preferences:{textScale:200,renderScale:-1,fps:999}}));
  const s = createStore(storage); assert.equal(s.get().selected, 4); assert.equal(s.get().profile, 'x'); assert.equal(s.get().preferences.textScale,1.4); assert.equal(s.get().preferences.renderScale,.6); assert.equal(s.get().preferences.fps,60);
});
test('catalog priorities and recent changes survive a restart and export/import', () => {
  const storage = memory(), s = createStore(storage, () => 9876); s.toggle('audio'); s.move('mineral', -1);
  const again = createStore(storage); again.setView('Mods', {sort:'recent'}); again.setView('Pacotes', {sort:'order'});
  assert.equal(again.rows('Mods')[0].id,'audio'); assert.deepEqual(again.rows('Pacotes').map(r=>r.id),['mineral','satin']);
  const imported = createStore(memory()); imported.importText(again.exportText()); imported.setView('Pacotes',{sort:'order'}); assert.deepEqual(imported.rows('Pacotes').map(r=>r.id),['mineral','satin']);
});
test('appearance undo restores only preferences and keeps later catalog changes', () => {
  const s=createStore(memory());s.preference('warmth',.9);s.toggle('audio');assert.equal(s.undoAppearance(),true);assert.equal(s.get().preferences.warmth,.5);assert.equal(s.get().catalog.find(r=>r.id==='audio').enabled,true);
});
test('old preview state migrates without losing data and ambient choice survives restart', () => {
  const storage=memory();storage.setItem('horizon.nexo.preview.v1',JSON.stringify({version:1,selected:2,preferences:{warmth:.7},catalog:[{id:'mine',page:'Mods',name:'Meu mod',enabled:true}]}));
  const s=createStore(storage);assert.equal(s.get().preferences.ambientMotion,true);assert.equal(s.get().selected,2);assert.equal(s.rows('Mods')[0].name,'Meu mod');
  s.preference('ambientMotion',false);s.preference('sceneLighting',false);const again=createStore(storage);assert.equal(again.get().preferences.sceneLighting,false);assert.equal(again.get().preferences.ambientMotion,false);assert.equal(again.get().preferences.warmth,.7);
  again.resetAppearance();again.undoAppearance();assert.equal(again.get().preferences.ambientMotion,false);assert.equal(again.rows('Mods')[0].enabled,true);
});
