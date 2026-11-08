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
use std::{cell::Cell, panic::AssertUnwindSafe};

thread_local! {
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
    let state = horizon_world::chunk::BlockState {
        name: name.to_owned(),
        properties,
    };
    game.pack.assets.registry.resolve(&state).ok()
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
}

/// The server tick starts or ends: the first one starts the mods.
pub(super) fn server_tick(game: &mut Game<'_>, end: bool) {
    start(game);
    if active() {
        with_current(game, || crate::lane3::server_tick(end));
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
    let on = current(|g| g.keys.contains(&winit::keyboard::KeyCode::ShiftLeft));
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
