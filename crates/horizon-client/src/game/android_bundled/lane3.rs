// Written for Horizon Android; the repository's licence.
//! Lane 3 of the mod converter in the game: the Java mods the converter made ready
//! (`android/lane3.rs`) start when the server thread's world opens, hear the events this file
//! fires, and reach the world through the `Native` methods below, which act on the `Game` that
//! fired the event.
use crate::game::Game;
use horizon_world::registry::BlockStateId;
use jni::{
    JNIEnv,
    objects::{JClass, JString},
    sys::{jboolean, jint, jlong, jstring},
};
use std::{
    cell::Cell,
    collections::{HashMap, HashSet},
    panic::AssertUnwindSafe,
};

/// What a lane 3 mod's block is owed in a level: its `onPlace` or its scheduled `tick`, run
/// after the change (the game's level is busy while it happens), in that level.
enum Owed {
    Placed {
        state: BlockStateId,
        old: BlockStateId,
        at: [i32; 3],
    },
    Ticked {
        state: BlockStateId,
        at: [i32; 3],
    },
}

/// At most this many owed calls wait at once (a block placing blocks without end stops there).
const OWED_MAX: usize = 4096;

thread_local! {
    /// The mods' blocks with their own `onPlace`, and with their own `tick`.
    static HOOKS: std::cell::RefCell<(HashSet<String>, HashSet<String>)> =
        std::cell::RefCell::new((HashSet::new(), HashSet::new()));
    /// What the mods' blocks are owed, with the level it is owed in.
    static OWED: std::cell::RefCell<Vec<(&'static str, Owed)>> =
        const { std::cell::RefCell::new(Vec::new()) };
    /// Each player's level at the end of the last tick, by entity id.
    static LEVELS: std::cell::RefCell<HashMap<i32, String>> =
        std::cell::RefCell::new(HashMap::new());
    /// The players inside mods' blocks this tick: (entity id, block, position), each once.
    static INSIDE: std::cell::RefCell<Vec<(i32, String, [i32; 3])>> =
        const { std::cell::RefCell::new(Vec::new()) };
    /// The game an event is running for, while the Java code runs.
    static CURRENT: Cell<*mut ()> = const { Cell::new(std::ptr::null_mut()) };
    /// Whether this server thread started its lane 3 mods, and how many it has.
    static STARTED: Cell<Option<usize>> = const { Cell::new(None) };
}

fn with_current<R>(game: &mut Game<'_>, f: impl FnOnce() -> R) -> R {
    let previous = CURRENT.with(|c| c.replace(game as *mut Game<'_> as *mut ()));
    let out = f();
    CURRENT.with(|c| c.set(previous));
    out
}

/// The game of the event running now, for a `Native` method.
fn current<R>(f: impl FnOnce(&mut Game<'static>) -> R) -> Option<R> {
    let p = CURRENT.with(Cell::get);
    if p.is_null() {
        return None;
    }
    // Safety: set by `with_current` for the length of a call into Java on this thread, while
    // the caller doesn't touch the game; a Java callback runs inside that call.
    let game = unsafe { &mut *(p as *mut Game<'static>) };
    std::panic::catch_unwind(AssertUnwindSafe(|| f(game))).ok()
}

fn active() -> bool {
    STARTED.with(Cell::get).is_some_and(|n| n > 0)
}

/// A block state as the Java half reads it: `minecraft:wheat[age=7]`.
fn state_text(game: &Game<'_>, state: BlockStateId) -> String {
    let Some(s) = game.pack.assets.registry.state(state) else {
        return "minecraft:air".into();
    };
    if s.properties.is_empty() {
        return s.name.clone();
    }
    let props: Vec<String> = s
        .properties
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect();
    format!("{}[{}]", s.name, props.join(","))
}

fn parse_state(game: &Game<'_>, text: &str) -> Option<BlockStateId> {
    let (name, props) = match text.split_once('[') {
        Some((name, rest)) => (name, rest.strip_suffix(']').unwrap_or(rest)),
        None => (text, ""),
    };
    let mut properties = horizon_world::chunk::BlockProperties::new();
    for part in props.split(',').filter(|p| !p.is_empty()) {
        let (k, v) = part.split_once('=')?;
        properties.insert(k.to_owned(), v.to_owned());
    }
    let registry = &game.pack.assets.registry;
    let mut state = horizon_world::chunk::BlockState {
        name: name.to_owned(),
        properties,
    };
    if let Ok(id) = registry.resolve(&state) {
        return Some(id);
    }
    // A property or value Horizon's block doesn't have (a mod's block carried without it):
    // the others still count.
    let default = registry.state(registry.default_state(name)?)?;
    state.properties = std::mem::replace(
        &mut state.properties,
        horizon_world::chunk::BlockProperties::new(),
    )
    .into_iter()
    .filter(|(k, v)| {
        default.properties.contains_key(k.as_str())
            && registry.property_values(name, k).contains(&v.as_str())
    })
    .collect();
    registry.resolve(&state).ok()
}

/// Starts the lane 3 mods that are on, once per server thread.
fn start(game: &mut Game<'_>) {
    if STARTED.with(Cell::get).is_some() {
        return;
    }
    STARTED.with(|s| s.set(Some(0)));
    if !crate::lane3::ready() {
        return;
    }
    let home = horizon_client::logging::game_home();
    let on = horizon_client::mod_list::ModList::load(&home);
    let (dex, entries) = crate::lane3::installed(&home, &|id: &str| on.enabled.contains(id));
    if dex.is_empty() {
        return;
    }
    let dir = home.join(crate::lane3::DIR);
    let report = with_current(game, || crate::lane3::start(&dex, &entries, &dir));
    let text = match &report {
        Ok(lines) => lines
            .iter()
            .map(|(id, s)| format!("{id}: {s:?}\n"))
            .collect::<String>(),
        Err(e) => format!("not started: {e}\n"),
    };
    log::info!("lane 3: {} mods\n{text}", dex.len());
    let _ = std::fs::write(dir.join("last-start.txt"), text);
    STARTED.with(|s| s.set(Some(dex.len())));
    let hooks = with_current(game, crate::lane3::hooks);
    HOOKS.with(|h| {
        let mut h = h.borrow_mut();
        *h = (HashSet::new(), HashSet::new());
        for line in hooks.lines() {
            match line.split_once('\t') {
                Some(("place", id)) => {
                    h.0.insert(id.to_owned());
                }
                Some(("tick", id)) => {
                    h.1.insert(id.to_owned());
                }
                _ => {}
            }
        }
    });
}

/// A block took the state `new` (it was `old`): a mod's block with its own `onPlace` is owed it.
pub(super) fn placed(block: &str, new: BlockStateId, old: BlockStateId, at: [i32; 3]) {
    if !active() || !HOOKS.with(|h| h.borrow().0.contains(block)) {
        return;
    }
    owe(Owed::Placed {
        state: new,
        old,
        at,
    });
}

/// A scheduled tick of `block` is due: a mod's block with its own `tick` takes it.
pub(super) fn ticked(block: &str, state: BlockStateId, at: [i32; 3], handled: &mut bool) {
    if !active() || !HOOKS.with(|h| h.borrow().1.contains(block)) {
        return;
    }
    if owe(Owed::Ticked { state, at }) {
        *handled = true;
    }
}

/// Keeps `owed` for the level acting now (the server thread's): whether it was kept.
fn owe(owed: Owed) -> bool {
    let Some(level) = horizon_client::block_contact::level() else {
        return false;
    };
    OWED.with(|o| {
        let mut o = o.borrow_mut();
        if o.len() >= OWED_MAX {
            return false;
        }
        o.push((level, owed));
        true
    })
}

/// Runs what the mods' blocks are owed in the level in place (what that runs may owe more,
/// run in turn, a few rounds at most).
fn pay(game: &mut Game<'_>) {
    let here = game.dim.id();
    // What runs acts in this level, so what it places is owed here in turn.
    horizon_client::block_contact::in_level(Some(here), || pay_in(game, here));
}

fn pay_in(game: &mut Game<'_>, here: &'static str) {
    for _ in 0..8 {
        let due: Vec<Owed> = OWED.with(|o| {
            let mut o = o.borrow_mut();
            let (due, rest): (Vec<_>, Vec<_>) = o.drain(..).partition(|(l, _)| *l == here);
            *o = rest;
            due.into_iter().map(|(_, owed)| owed).collect()
        });
        if due.is_empty() {
            return;
        }
        for owed in due {
            match owed {
                Owed::Placed { state, old, at } => {
                    let (state, old) = (state_text(game, state), state_text(game, old));
                    with_current(game, || crate::lane3::placed(&state, at, &old));
                }
                Owed::Ticked { state, at } => {
                    let state = state_text(game, state);
                    with_current(game, || crate::lane3::ticked(&state, at));
                }
            }
        }
    }
}

/// A level finished its tick on the server: what the mods' blocks are owed there runs.
pub(super) fn level_tick_post(game: &mut Game<'_>) {
    if active() {
        pay(game);
    }
}

/// A player is inside a mod's block: kept for the tick's end, once a block.
pub(super) fn inside(entity: i32, kind: &str, block: &str, position: [i32; 3]) {
    if !active() || kind != "minecraft:player" {
        return;
    }
    INSIDE.with(|i| {
        let mut i = i.borrow_mut();
        if i.len() < 64
            && !i
                .iter()
                .any(|(e, b, p)| *e == entity && b == block && *p == position)
        {
            i.push((entity, block.to_owned(), position));
        }
    });
}

/// The server tick starts or ends: the first one starts the mods. At its end the mods' blocks
/// the players were inside run their `entityInside`, and the players who changed level hear
/// it, each player acting for itself in its own level.
pub(super) fn server_tick(game: &mut Game<'_>, end: bool) {
    start(game);
    if !active() {
        return;
    }
    // The mods' server tick acts as the first player (single player's own), in its level: a
    // mod's queued work names no level of its own.
    let first = game
        .lan
        .as_ref()
        .and_then(|h| h.hub.guests.first().map(|g| g.entity_id));
    match first {
        Some(entity) => as_player(game, entity, |g| {
            with_current(g, || crate::lane3::server_tick(end));
        }),
        None => with_current(game, || crate::lane3::server_tick(end)),
    }
    if !end {
        return;
    }
    pay(game);
    let touched = INSIDE.with(|i| std::mem::take(&mut *i.borrow_mut()));
    for (entity, block, position) in touched {
        as_player(game, entity, |g| {
            with_current(g, || crate::lane3::entity_inside(&block, position));
        });
    }
    for (entity, from, to) in level_changes(game) {
        as_player(game, entity, |g| {
            with_current(g, || crate::lane3::changed_dimension(&from, &to));
        });
    }
}

/// The players whose level changed since the last tick's end: (entity id, from, to).
fn level_changes(game: &Game<'_>) -> Vec<(i32, String, String)> {
    let Some(host) = game.lan.as_ref() else {
        return Vec::new();
    };
    LEVELS.with(|l| {
        let mut l = l.borrow_mut();
        let mut out = Vec::new();
        let mut now = HashMap::new();
        for g in &host.hub.guests {
            if let Some(before) = l.get(&g.entity_id)
                && *before != g.dimension
            {
                out.push((g.entity_id, before.clone(), g.dimension.clone()));
            }
            now.insert(g.entity_id, g.dimension.clone());
        }
        *l = now;
        out
    })
}

/// Runs `f` as the player `entity` (a server's guest), in its level: then what the mods'
/// blocks were owed there runs too.
fn as_player(game: &mut Game<'_>, entity: i32, f: impl FnOnce(&mut Game<'_>)) {
    let Some((conn, level)) = game.lan.as_ref().and_then(|h| {
        h.hub
            .guests
            .iter()
            .find(|g| g.entity_id == entity)
            .map(|g| (g.conn, g.dimension.clone()))
    }) else {
        return;
    };
    let back = game.enter_command_level(crate::game::Dim::from_id(&level));
    if let Some(mut host) = game.lan.take() {
        game.as_guest(&mut host, conn, |g| {
            f(g);
            pay(g);
        });
        game.lan = Some(host);
    }
    if let Some(back) = back {
        game.enter_command_level(back);
    }
}

/// The server stops: what the mods threw goes beside them.
pub(super) fn server_stopping() {
    if active() {
        let errors = crate::lane3::errors();
        crate::lane3::stop();
        if !errors.is_empty() {
            let dir = horizon_client::logging::game_home().join(crate::lane3::DIR);
            let _ = std::fs::write(dir.join("last-errors.txt"), errors);
        }
    }
    STARTED.with(|s| s.set(None));
    OWED.with(|o| o.borrow_mut().clear());
    LEVELS.with(|l| l.borrow_mut().clear());
}

/// `Direction.get3DDataValue` of a face vector.
fn face_index(face: [i32; 3]) -> i32 {
    match face {
        [0, -1, 0] => 0,
        [0, 1, 0] => 1,
        [0, 0, -1] => 2,
        [0, 0, 1] => 3,
        [-1, 0, 0] => 4,
        _ => 5,
    }
}

/// A player right-clicks a block (`UseBlockCallback`): a mod's answer other than PASS takes the click.
pub(super) fn right_click_block(
    game: &mut Game<'_>,
    pos: [i32; 3],
    face: [i32; 3],
    canceled: &mut bool,
) {
    if !active() || *canceled {
        return;
    }
    let hand = i32::from(game.hand == horizon_client::first_person::Hand::Off);
    match with_current(game, || {
        crate::lane3::use_block(pos, face_index(face), hand)
    }) {
        1 => {
            game.swing_arm();
            *canceled = true;
        }
        2 | 3 => *canceled = true,
        _ => {}
    }
}

/// A player is about to break a block (`PlayerBlockBreakEvents.BEFORE`).
pub(super) fn destroy_block(
    game: &mut Game<'_>,
    pos: [i32; 3],
    state: BlockStateId,
    canceled: &mut bool,
) {
    if !active() || *canceled {
        return;
    }
    let text = state_text(game, state);
    if !with_current(game, || crate::lane3::before_break(pos, &text)) {
        *canceled = true;
    }
}

/// A player broke a block (`PlayerBlockBreakEvents.AFTER`).
pub(super) fn block_broken(game: &mut Game<'_>, pos: [i32; 3], state: BlockStateId) {
    if !active() {
        return;
    }
    let text = state_text(game, state);
    with_current(game, || crate::lane3::after_break(pos, &text));
}

fn java_string(env: &mut JNIEnv, text: &str) -> jstring {
    env.new_string(text)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

fn rust_string(env: &mut JNIEnv, text: &JString) -> String {
    env.get_string(text).map(String::from).unwrap_or_default()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_horizon_android_lane3_Native_block<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    x: jint,
    y: jint,
    z: jint,
) -> jstring {
    let text = current(|g| state_text(g, g.world.block(x, y, z))).unwrap_or_default();
    java_string(&mut env, &text)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_horizon_android_lane3_Native_setBlock<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    x: jint,
    y: jint,
    z: jint,
    state: JString<'l>,
    flags: jint,
) -> jboolean {
    let text = rust_string(&mut env, &state);
    let done = current(|g| {
        parse_state(g, &text).is_some_and(|s| g.level_set_with_flags([x, y, z], s, flags).is_ok())
    });
    u8::from(done == Some(true))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_horizon_android_lane3_Native_destroy<'l>(
    _env: JNIEnv<'l>,
    _class: JClass<'l>,
    x: jint,
    y: jint,
    z: jint,
    drop: jboolean,
) -> jboolean {
    let done = current(|g| {
        let at = horizon_core::position::BlockPos::new(x, y, z);
        let broke = g.with_level(|b, level, ticks| b.destroy_block(level, ticks, at, drop != 0));
        let _ = g.process_level_events();
        broke
    });
    u8::from(done == Some(true))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_horizon_android_lane3_Native_sneaking<'l>(
    _env: JNIEnv<'l>,
    _class: JClass<'l>,
) -> jboolean {
    // The acting player's own (a server's guest's body), or this window's key.
    let on = current(|g| {
        g.player.as_ref().is_some_and(|p| p.shift_key_down)
            || g.keys.contains(&winit::keyboard::KeyCode::ShiftLeft)
    });
    u8::from(on == Some(true))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_horizon_android_lane3_Native_creative<'l>(
    _env: JNIEnv<'l>,
    _class: JClass<'l>,
) -> jboolean {
    u8::from(current(|g| g.creative) == Some(true))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_horizon_android_lane3_Native_held<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    hand: jint,
) -> jstring {
    let text = current(|g| {
        let stack = if hand == 1 {
            g.offhand.clone()
        } else {
            g.inventory.get(g.selected).cloned().flatten()
        };
        stack
            .map(|s| format!("{} {} {}", s.item, s.count, s.damage))
            .unwrap_or_default()
    })
    .unwrap_or_default();
    java_string(&mut env, &text)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_horizon_android_lane3_Native_swing<'l>(
    _env: JNIEnv<'l>,
    _class: JClass<'l>,
    _hand: jint,
) {
    let _ = current(|g| g.swing_arm());
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_horizon_android_lane3_Native_time<'l>(
    _env: JNIEnv<'l>,
    _class: JClass<'l>,
) -> jlong {
    current(|g| g.world.game_time).unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_horizon_android_lane3_Native_log<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    line: JString<'l>,
) {
    let text = rust_string(&mut env, &line);
    log::info!("lane 3 {text}");
}

/// A mod runs a command as the acting player (`Commands.performPrefixedCommand` with the
/// player's source at the mods' permission, 4, its output suppressed): whether it ran.
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_horizon_android_lane3_Native_command<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    command: JString<'l>,
) -> jboolean {
    let text = rust_string(&mut env, &command);
    let line = text.trim().trim_start_matches('/').to_owned();
    if line.is_empty() {
        return 0;
    }
    let ran = current(|g| {
        let level = g.ui.registries.permission;
        g.ui.registries.permission = 4;
        let ok = g.run_command_quietly(&line);
        g.ui.registries.permission = level;
        ok
    });
    u8::from(ran == Some(true))
}

/// The acting player, as `x y z yaw pitch dimension`, or nothing outside an event.
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_horizon_android_lane3_Native_player<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
) -> jstring {
    let text = current(|g| {
        let (x, y, z) = g.player.as_ref().map_or((0.0, 0.0, 0.0), |p| {
            (p.position.x, p.position.y, p.position.z)
        });
        format!("{x} {y} {z} {} {} {}", g.yaw, g.pitch, g.dim.id())
    })
    .unwrap_or_default();
    java_string(&mut env, &text)
}

/// Whether the acting player has an effect (`minecraft:blindness`, a mod's `ns:path`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_horizon_android_lane3_Native_hasEffect<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    effect: JString<'l>,
) -> jboolean {
    let id = rust_string(&mut env, &effect);
    let short = id.strip_prefix("minecraft:").unwrap_or(&id).to_owned();
    u8::from(current(|g| g.potions.effects.get(&short).is_some()) == Some(true))
}

/// The block `id`'s default state, as `state_text` writes it, or "" for a block Horizon lacks.
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_horizon_android_lane3_Native_defaultState<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    id: JString<'l>,
) -> jstring {
    let id = rust_string(&mut env, &id);
    let text = current(|g| {
        g.pack
            .assets
            .registry
            .default_state(&id)
            .map(|s| state_text(g, s))
    })
    .flatten()
    .unwrap_or_default();
    java_string(&mut env, &text)
}

/// The block `id`'s properties with their values, in the registry's order:
/// `facing=north,south,west,east;lit=true,false`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_horizon_android_lane3_Native_properties<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    id: JString<'l>,
) -> jstring {
    let id = rust_string(&mut env, &id);
    let text = current(|g| {
        let registry = &g.pack.assets.registry;
        let default = registry.state(registry.default_state(&id)?)?;
        let parts: Vec<String> = default
            .properties
            .keys()
            .map(|k| format!("{k}={}", registry.property_values(&id, k).join(",")))
            .collect();
        Some(parts.join(";"))
    })
    .flatten()
    .unwrap_or_default();
    java_string(&mut env, &text)
}

/// A mod's block asks for a tick at x, y, z in `delay` ticks (`Level.scheduleTick`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_horizon_android_lane3_Native_scheduleTick<'l>(
    _env: JNIEnv<'l>,
    _class: JClass<'l>,
    x: jint,
    y: jint,
    z: jint,
    delay: jint,
) {
    let _ = current(|g| {
        let at = horizon_core::position::BlockPos::new(x, y, z);
        g.with_level(|b, level, ticks| b.schedule_tick(level, ticks, at, delay.max(0)));
    });
}

/// The biome at x, y, z in the level in place (`minecraft:plains`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_horizon_android_lane3_Native_biome<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    x: jint,
    y: jint,
    z: jint,
) -> jstring {
    let text = current(|g| {
        let at = horizon_core::geometry::Vec3::new(
            f64::from(x) + 0.5,
            f64::from(y) + 0.5,
            f64::from(z) + 0.5,
        );
        g.biome_name_at(at).map(|name| {
            if name.contains(':') {
                name
            } else {
                format!("minecraft:{name}")
            }
        })
    })
    .flatten()
    .unwrap_or_default();
    java_string(&mut env, &text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn faces_are_vanilla_s_direction_order() {
        let faces = [
            [0, -1, 0],
            [0, 1, 0],
            [0, 0, -1],
            [0, 0, 1],
            [-1, 0, 0],
            [1, 0, 0],
        ];
        let order: Vec<i32> = faces.iter().map(|f| face_index(*f)).collect();
        assert_eq!(order, [0, 1, 2, 3, 4, 5]);
    }
}
