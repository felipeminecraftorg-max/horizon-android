//! Lane 2 of the Android mod converter, mob texture variants: a Java mod that gives vanilla mobs
//! more looks (more cows, pigs, zombies...) ships them as textures for the vanilla model, and
//! its Java code picks one per mob. Here the converter's table of those textures
//! (`lane2/mob_textures.txt` beside `options.txt`, a line `mod\tfolder\ttexture` each) gives a
//! mob one of its species' looks, or the vanilla one, chosen by the mob's UUID: the same mob
//! keeps its look, on every client, with nothing saved or sent.
//!
//! A variant applies only where the mob draws its adult base model (`cow#main`), the model the
//! texture was made for.
use std::{
    collections::{BTreeSet, HashMap},
    path::{Path, PathBuf},
    sync::{Mutex, RwLock},
    time::{Instant, SystemTime},
};

/// The table's file under the game folder.
pub const FILE: &str = "lane2/mob_textures.txt";

#[derive(Default)]
struct Table {
    /// Vanilla texture folder (`cow`) → the mods' textures for it (`mobvariants:entity/cow/ashen`).
    by_folder: HashMap<String, Vec<String>>,
}

impl Table {
    fn parse(text: &str, on: &dyn Fn(&str) -> bool) -> Self {
        let mut by_folder: HashMap<String, Vec<String>> = HashMap::new();
        for line in text.lines() {
            let mut parts = line.split('\t');
            let (Some(id), Some(folder), Some(texture)) =
                (parts.next(), parts.next(), parts.next())
            else {
                continue;
            };
            if on(id) {
                by_folder
                    .entry(folder.to_owned())
                    .or_default()
                    .push(texture.to_owned());
            }
        }
        for list in by_folder.values_mut() {
            list.sort();
            list.dedup();
        }
        Table { by_folder }
    }
}

struct Loaded {
    path: PathBuf,
    checked: Instant,
    modified: Option<SystemTime>,
    table: Table,
}

static TABLE: Mutex<Option<Loaded>> = Mutex::new(None);
/// Entity id → a hash of its UUID, as the server sent it.
static UUIDS: RwLock<Option<HashMap<u32, u64>>> = RwLock::new(None);

fn load(path: &Path) -> Table {
    let home = path
        .parent()
        .and_then(Path::parent)
        .unwrap_or(Path::new("."));
    let on: BTreeSet<String> = crate::mod_list::ModList::load(home).enabled;
    let text = std::fs::read_to_string(path).unwrap_or_default();
    Table::parse(&text, &|id| on.contains(id))
}

/// A mob's UUID, for its look (`ClientboundAddEntityPacket`).
pub fn remember(id: u32, uuid: u128) {
    let hash = (uuid as u64) ^ ((uuid >> 64) as u64).rotate_left(29);
    let mut uuids = UUIDS.write().unwrap_or_else(|e| e.into_inner());
    uuids.get_or_insert_with(HashMap::new).insert(id, hash);
}

pub fn forget(id: u32) {
    let mut uuids = UUIDS.write().unwrap_or_else(|e| e.into_inner());
    if let Some(u) = uuids.as_mut() {
        u.remove(&id);
    }
}

/// The folder of a vanilla entity texture (`entity/cow/cow_temperate` → `cow`).
fn folder(texture: &str) -> Option<&str> {
    let rest = texture.strip_prefix("entity/")?;
    let (folder, _) = rest.split_once('/')?;
    Some(folder)
}

/// Which look of `choices` (the vanilla one first) a UUID hash takes.
fn choose(hash: u64, choices: usize) -> usize {
    // splitmix64's finalizer: neighbouring UUIDs land far apart.
    let mut z = hash.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    (z % choices.max(1) as u64) as usize
}

/// The texture mob `id` draws with model `model`: `texture`, or a mod's look for it.
pub fn pick(id: u32, model: &str, texture: String) -> String {
    let Some(folder) = folder(&texture) else {
        return texture;
    };
    if model.strip_suffix("#main") != Some(folder) {
        return texture;
    }
    let Some(hash) = UUIDS
        .read()
        .ok()
        .and_then(|u| u.as_ref().and_then(|u| u.get(&id).copied()))
    else {
        return texture;
    };
    let mut guard = TABLE.lock().unwrap_or_else(|e| e.into_inner());
    let fresh = guard
        .as_ref()
        .is_some_and(|l| l.checked.elapsed().as_secs() < 3);
    if !fresh {
        let path = crate::logging::game_home().join(FILE);
        let modified = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
        let same = guard
            .as_ref()
            .is_some_and(|l| l.path == path && l.modified == modified);
        if same {
            if let Some(l) = guard.as_mut() {
                l.checked = Instant::now();
            }
        } else {
            *guard = Some(Loaded {
                table: load(&path),
                path,
                checked: Instant::now(),
                modified,
            });
        }
    }
    let Some(looks) = guard
        .as_ref()
        .and_then(|l| l.table.by_folder.get(folder))
        .filter(|v| !v.is_empty())
    else {
        return texture;
    };
    match choose(hash, looks.len() + 1) {
        0 => texture,
        n => looks[n - 1].clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_mods_that_are_on_give_looks() {
        let t = Table::parse(
            "mobvariants\tcow\tmobvariants:entity/cow/ashen\n\
             off\tcow\toff:entity/cow/x\n\
             mobvariants\tpig\tmobvariants:entity/pig/mud\nbroken line\n",
            &|id| id == "mobvariants",
        );
        assert_eq!(t.by_folder["cow"], ["mobvariants:entity/cow/ashen"]);
        assert_eq!(t.by_folder["pig"], ["mobvariants:entity/pig/mud"]);
    }

    #[test]
    fn a_mob_keeps_its_look_and_looks_spread() {
        assert_eq!(folder("entity/cow/cow_temperate"), Some("cow"));
        assert_eq!(folder("ns:entity/cow"), None);
        let picks: Vec<usize> = (0..300u64).map(|h| choose(h, 3)).collect();
        assert_eq!(picks, (0..300u64).map(|h| choose(h, 3)).collect::<Vec<_>>());
        for n in 0..3 {
            let share = picks.iter().filter(|p| **p == n).count();
            assert!((60..140).contains(&share), "look {n}: {share} of 300");
        }
    }
}
