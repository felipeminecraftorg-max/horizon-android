// Written for Horizon Android; the repository's licence.
//! Horizon's own versions of what popular Java mods' code does: the converter's known-mod lane
//! (`android/mod_convert.rs`). Like the other bundled mods they are off until turned on in the
//! mod list; converting a pack that has one of the Java mods turns its version on.
//!
//! - Right Click Harvest: right-clicking a grown crop picks it and replants it.
//! - Tree Harvester: breaking a natural tree's log with an axe fells the tree (not sneaking).
//! - Veinminer: sneaking while mining an ore with the right tool mines its vein.
//! - Sound Physics (Sound Physics Remastered): `sound_physics.rs`, on the window's side.
//! - First-person Model: the body in first person (`game/perspective.rs`), on the window's side.
//! - CameraOverhaul: `camera_overhaul.rs`, on the window's side.
//!
//! They run where the world runs (the server thread, a LAN host, a dedicated server), for every
//! player, through the game's own block breaking: drops, experience, tool wear, hunger and
//! statistics are vanilla's for each block.
use crate::game::Game;
use horizon_world::registry::BlockStateId;
use std::{cell::Cell, collections::HashSet};

pub(crate) const RIGHT_CLICK_HARVEST: &str = "rightclickharvest";
pub(crate) const TREE_HARVESTER: &str = "treeharvester";
pub(crate) const VEINMINER: &str = "veinminer";
/// FirstPerson (First-person Model): the body drawn in first person (`perspective.rs`).
pub(crate) const FIRST_PERSON: &str = "firstperson";

/// The bundled versions: id, title, description (Options > Mods and the launcher).
pub(crate) const MODS: [(&str, &str, &str); 6] = [
    (
        RIGHT_CLICK_HARVEST,
        "Right Click Harvest",
        "Right-click a grown crop to harvest and replant it",
    ),
    (
        TREE_HARVESTER,
        "Tree Harvester",
        "Break a tree's log with an axe to fell the whole tree; sneak to break one",
    ),
    (
        VEINMINER,
        "Veinminer",
        "Sneak while mining an ore with the right tool to mine its whole vein",
    ),
    (
        super::sound_physics::SOUND_PHYSICS,
        "Sound Physics",
        "Sounds muffled behind walls, echoing in caves and dulled under water",
    ),
    (
        FIRST_PERSON,
        "First-person Model",
        "See your own body and legs when you look down in first person",
    ),
    (
        super::camera_overhaul::CAMERA_OVERHAUL,
        "CameraOverhaul",
        "The camera rolls and tilts as you move and turn, sways while idle, shakes from blasts",
    ),
];

/// The most logs one chop fells.
const MAX_LOGS: usize = 256;
/// The most blocks of a vein one break mines.
const MAX_VEIN: usize = 64;
/// Natural leaves a felled tree has at least: a log wall or a house is no tree.
const MIN_LEAVES: usize = 3;

thread_local! {
    /// Set while a chain breaks its blocks: those breaks don't start chains of their own.
    static CHAINING: Cell<bool> = const { Cell::new(false) };
}

/// Clears `CHAINING` however the chain ends.
struct Chain;
impl Chain {
    fn start() -> Option<Self> {
        if CHAINING.with(Cell::get) {
            None
        } else {
            CHAINING.with(|c| c.set(true));
            Some(Chain)
        }
    }
}
impl Drop for Chain {
    fn drop(&mut self) {
        CHAINING.with(|c| c.set(false));
    }
}

pub(super) fn list(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    for (id, title, description) in MODS {
        mods.push(crate::game::native_hooks::NativeMod {
            id: id.into(),
            title: format!("{title} (Horizon)"),
            description: description.into(),
            keys: Vec::new(),
        });
    }
}

fn sneaking(game: &Game<'_>) -> bool {
    game.keys.contains(&winit::keyboard::KeyCode::ShiftLeft)
}

fn name_of(game: &Game<'_>, state: BlockStateId) -> String {
    game.res.facts().get(state).name.to_string()
}

fn property(game: &Game<'_>, state: BlockStateId, key: &str) -> Option<String> {
    game.pack
        .assets
        .registry
        .state(state)?
        .properties
        .get(key)
        .cloned()
}

/// A crop Right Click Harvest picks: the age it is grown at, and the item that replants it
/// (one of which is kept from the drops).
fn crop(game: &Game<'_>, pos: [i32; 3], state: BlockStateId, name: &str) -> Option<(u32, String)> {
    let vanilla = match name {
        "minecraft:wheat" => Some((7, "minecraft:wheat_seeds")),
        "minecraft:carrots" => Some((7, "minecraft:carrot")),
        "minecraft:potatoes" => Some((7, "minecraft:potato")),
        "minecraft:beetroots" => Some((3, "minecraft:beetroot_seeds")),
        "minecraft:nether_wart" => Some((3, "minecraft:nether_wart")),
        "minecraft:cocoa" => Some((2, "minecraft:cocoa_beans")),
        _ => None,
    };
    if let Some((age, seed)) = vanilla {
        return Some((age, seed.to_owned()));
    }
    // A mod's crop (#minecraft:crops, outside vanilla's stems and tall crops): grown at its
    // highest age, replanted by the item its block picks.
    if name.starts_with("minecraft:") || !game.items().block_in_tag("minecraft:crops", name) {
        return None;
    }
    property(game, state, "age")?;
    let grown = (1..=15u32).rev().find(|age| {
        game.with_props(state, &[("age", age.to_string().as_str())])
            .is_some()
    })?;
    let seed = game.picked_stack(pos, false)?.item.to_string();
    Some((grown, seed))
}

/// Right Click Harvest: a grown crop under the cursor is picked and replanted. Returns whether
/// it was.
pub(super) fn right_click_block(game: &mut Game<'_>, pos: [i32; 3]) -> bool {
    if !game.native_on(RIGHT_CLICK_HARVEST) || game.spectator() || sneaking(game) {
        return false;
    }
    let state = game.world.block(pos[0], pos[1], pos[2]);
    let name = name_of(game, state);
    let Some((grown, seed)) = crop(game, pos, state, &name) else {
        return false;
    };
    let age = property(game, state, "age").and_then(|a| a.parse::<u32>().ok());
    if age != Some(grown) {
        return false;
    }
    let Some(young) = game.with_props(state, &[("age", "0")]) else {
        return false;
    };
    let held = game.held().cloned();
    let mut drops = game.block_loot_at(pos, state, held.as_ref());
    // The replanted crop keeps one of its seeds.
    if let Some(d) = drops
        .iter_mut()
        .find(|d| &*d.item == seed.as_str() && d.count > 0)
    {
        d.count -= 1;
    }
    if game.level_set(pos, young).is_err() {
        return false;
    }
    // The block's break particles and sound (Level.destroyBlock's event).
    game.level_event_at(2001, pos, state.0 as i32);
    let actor = game.sculk_actor();
    game.post_block_game_event("minecraft:block_change", pos, actor, Some(young));
    for d in drops {
        if d.count > 0 && &*d.item != "minecraft:air" {
            game.pop_resource(pos, d);
        }
    }
    if let Some(p) = game.player.as_mut() {
        p.cause_food_exhaustion(0.005);
    }
    game.stats.add("mined", &name, 1);
    game.swing_arm();
    true
}

/// The blocks around `at`, the 26 that touch it.
fn around(at: [i32; 3]) -> impl Iterator<Item = [i32; 3]> {
    (-1..=1).flat_map(move |dx| {
        (-1..=1).flat_map(move |dy| {
            (-1..=1)
                .map(move |dz| [at[0] + dx, at[1] + dy, at[2] + dz])
                .filter(move |p| *p != at)
        })
    })
}

/// The blocks connected to `start` (not itself) that `same` accepts, nearest first, at most `max`.
fn connected(
    game: &Game<'_>,
    start: [i32; 3],
    max: usize,
    same: &dyn Fn(&Game<'_>, [i32; 3]) -> bool,
) -> Vec<[i32; 3]> {
    let mut seen: HashSet<[i32; 3]> = HashSet::from([start]);
    let mut queue = std::collections::VecDeque::from([start]);
    let mut found = Vec::new();
    while let Some(at) = queue.pop_front() {
        for p in around(at) {
            if found.len() >= max {
                return found;
            }
            if seen.insert(p) && same(game, p) {
                found.push(p);
                queue.push_back(p);
            }
        }
    }
    found
}

/// The same kind of ore: a vein mixes stone and deepslate ores of one metal.
fn ore_kind(name: &str) -> String {
    name.replacen(":deepslate_", ":", 1)
}

fn is_ore(game: &Game<'_>, name: &str) -> bool {
    name.ends_with("_ore")
        || name == "minecraft:ancient_debris"
        || game.items().block_in_tag("c:ores", name)
}

/// The positions a chop or a vein takes along with `pos`, before `pos` itself breaks.
fn chain(game: &Game<'_>, pos: [i32; 3], state: BlockStateId) -> Vec<[i32; 3]> {
    if game.creative || game.spectator() {
        return Vec::new();
    }
    let name = name_of(game, state);
    let held = game.held().map(|s| s.item.to_string());
    let items = game.items();
    let axe = held
        .as_deref()
        .is_some_and(|h| items.item_in_tag("minecraft:axes", h));
    if game.native_on(TREE_HARVESTER)
        && axe
        && !sneaking(game)
        && items.block_in_tag("minecraft:logs", &name)
    {
        // Logs of the same wood at the chop's height or above: a tree, not its neighbour's roots.
        let logs = connected(game, pos, MAX_LOGS, &|g: &Game<'_>, p: [i32; 3]| {
            p[1] >= pos[1] && name_of(g, g.world.block(p[0], p[1], p[2])) == name
        });
        let natural_leaves = std::iter::once(pos)
            .chain(logs.iter().copied())
            .flat_map(|at| {
                [
                    [1, 0, 0],
                    [-1, 0, 0],
                    [0, 1, 0],
                    [0, -1, 0],
                    [0, 0, 1],
                    [0, 0, -1],
                ]
                .map(|d| [at[0] + d[0], at[1] + d[1], at[2] + d[2]])
            })
            .collect::<HashSet<_>>()
            .into_iter()
            .filter(|p| {
                let s = game.world.block(p[0], p[1], p[2]);
                game.items()
                    .block_in_tag("minecraft:leaves", &name_of(game, s))
                    && property(game, s, "persistent").as_deref() != Some("true")
            })
            .count();
        if natural_leaves >= MIN_LEAVES {
            return logs;
        }
        return Vec::new();
    }
    if game.native_on(VEINMINER)
        && sneaking(game)
        && is_ore(game, &name)
        && items.correct_for_drops(game.held(), &name)
    {
        let kind = ore_kind(&name);
        return connected(game, pos, MAX_VEIN - 1, &|g: &Game<'_>, p: [i32; 3]| {
            ore_kind(&name_of(g, g.world.block(p[0], p[1], p[2]))) == kind
        });
    }
    Vec::new()
}

/// A player breaks the block `state` at `pos`: a chop or a vein breaks the rest of the tree or
/// the vein first, each block as the player would, while the tool lasts.
pub(super) fn destroy_block(game: &mut Game<'_>, pos: [i32; 3], state: BlockStateId) {
    let Some(_chain) = Chain::start() else {
        return;
    };
    let blocks = chain(game, pos, state);
    if blocks.is_empty() {
        return;
    }
    let tool = game.held().map(|s| s.item.clone());
    for p in blocks {
        // Stop before the tool breaks: the block the player broke still needs it.
        let Some(stack) = game.held() else {
            break;
        };
        if Some(&stack.item) != tool.as_ref() {
            break;
        }
        let max = game.items().get(&stack.item).map_or(0, |i| i.max_damage);
        if max > 0 && stack.damage + 2 >= max {
            break;
        }
        let _ = game.break_block_as_player(p);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn veins_mix_stone_and_deepslate_ores_of_one_metal() {
        assert_eq!(
            ore_kind("minecraft:deepslate_iron_ore"),
            "minecraft:iron_ore"
        );
        assert_eq!(ore_kind("minecraft:iron_ore"), "minecraft:iron_ore");
        assert_ne!(
            ore_kind("minecraft:deepslate_gold_ore"),
            ore_kind("minecraft:iron_ore")
        );
    }

    #[test]
    fn a_block_has_26_neighbours() {
        let n: HashSet<_> = around([0, 64, 0]).collect();
        assert_eq!(n.len(), 26);
        assert!(!n.contains(&[0, 64, 0]) && n.contains(&[1, 65, -1]));
    }
}
