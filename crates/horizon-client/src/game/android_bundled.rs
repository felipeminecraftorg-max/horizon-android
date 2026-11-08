//! Fixed MIT-licensed Android mod bundle. No downloaded code or phone compiler.

use super::*;

use super::native_hooks::*;

#[cfg(target_os = "android")]
#[path = "android_bundled/zoom/runtime.rs"]
pub(super) mod official_zoom;

#[cfg(target_os = "android")]
#[path = "android_bundled/zoom/zoom_key.rs"]
mod zoom_key;

#[cfg(target_os = "android")]
#[path = "android_bundled/zoom/zoom_wheel.rs"]
mod zoom_wheel;

#[cfg(target_os = "android")]
#[path = "android_bundled/zoom/zoom_zoom.rs"]
mod zoom_zoom;

#[cfg(target_os = "android")]
#[path = "android_bundled/zoom/zoom_list.rs"]
mod zoom_list;

#[cfg(target_os = "android")]
#[path = "android_bundled/zoom/zoom_settings.rs"]
mod zoom_settings;

#[cfg(target_os = "android")]
#[path = "android_bundled/zoom/zoom_click.rs"]
mod zoom_click;

#[cfg(target_os = "android")]
#[path = "android_bundled/minimap/runtime.rs"]
pub(super) mod official_minimap;

#[cfg(target_os = "android")]
#[path = "android_bundled/minimap/minimap_list.rs"]
mod minimap_list;

#[cfg(target_os = "android")]
#[path = "android_bundled/minimap/minimap_settings.rs"]
mod minimap_settings;

#[cfg(target_os = "android")]
#[path = "android_bundled/minimap/minimap_click.rs"]
mod minimap_click;

#[cfg(target_os = "android")]
#[path = "android_bundled/minimap/minimap_minimap.rs"]
mod minimap_minimap;

#[cfg(target_os = "android")]
#[path = "android_bundled/recipes/runtime.rs"]
pub(super) mod official_recipes;

#[cfg(target_os = "android")]
#[path = "android_bundled/recipes/glue.rs"]
pub(super) mod official_recipes_glue;

#[cfg(target_os = "android")]
#[path = "android_bundled/recipes/recipes_list.rs"]
mod recipes_list;

#[cfg(target_os = "android")]
#[path = "android_bundled/recipes/recipes_settings.rs"]
mod recipes_settings;

#[cfg(target_os = "android")]
#[path = "android_bundled/recipes/recipes_click.rs"]
mod recipes_click;

#[cfg(target_os = "android")]
#[path = "android_bundled/recipes/recipes_frame.rs"]
mod recipes_frame;

#[cfg(target_os = "android")]
#[path = "android_bundled/recipes/recipes_event.rs"]
mod recipes_event;

#[cfg(target_os = "android")]
#[path = "android_bundled/smartf3/runtime.rs"]
pub(super) mod smartf3;

#[cfg(target_os = "android")]
#[path = "android_bundled/smartf3/native.rs"]
pub(super) mod smartf3_native;

#[cfg(target_os = "android")]
#[path = "android_bundled/smartf3/smartf3_command.rs"]
mod smartf3_command;

#[cfg(target_os = "android")]
#[path = "android_bundled/smartf3/smartf3_list.rs"]
mod smartf3_list;

#[cfg(target_os = "android")]
#[path = "android_bundled/smartf3/smartf3_settings.rs"]
mod smartf3_settings;

#[cfg(target_os = "android")]
#[path = "android_bundled/smartf3/smartf3_click.rs"]
mod smartf3_click;

/// Horizon's own versions of Java mods' behaviour, which the converter turns on.
#[cfg(target_os = "android")]
#[path = "android_bundled/compat.rs"]
pub(super) mod compat;

/// Java mods' own code, run by Android's runtime (the mod converter's lane 3).
#[cfg(target_os = "android")]
#[path = "android_bundled/lane3.rs"]
pub(super) mod lane3;

#[cfg(target_os = "android")]
#[path = "android_bundled/sound_physics.rs"]
pub(super) mod sound_physics;

#[cfg(target_os = "android")]
#[path = "android_bundled/bundled_settings.rs"]
pub(super) mod bundled_settings;

#[cfg(target_os = "android")]
#[path = "android_bundled/camera_overhaul.rs"]
pub(super) mod camera_overhaul;

/// A player breaks a block (`native_destroy_block`): a lane 3 mod may keep it; otherwise a chop
/// or a vein takes the rest.
pub(super) fn native_destroy_block(
    game: &mut Game<'_>,
    pos: [i32; 3],
    state: BlockStateId,
    canceled: &mut bool,
) {
    let _ = (&game, &pos, &state, &canceled);
    #[cfg(target_os = "android")]
    {
        lane3::destroy_block(&mut *game, pos, state, &mut *canceled);
        if !*canceled {
            compat::destroy_block(&mut *game, pos, state);
        }
    }
}

/// A player broke a block: lane 3 mods hear it.
pub(super) fn native_block_broken(game: &mut Game<'_>, pos: [i32; 3], state: BlockStateId) {
    let _ = (&game, &pos, &state);
    #[cfg(target_os = "android")]
    {
        lane3::block_broken(&mut *game, pos, state);
    }
}

/// A player right clicks a block (`native_right_click_block`): a lane 3 mod answers first,
/// then a grown crop is harvested.
pub(super) fn native_right_click_block(
    game: &mut Game<'_>,
    pos: [i32; 3],
    face: [i32; 3],
    canceled: &mut bool,
) {
    let _ = (&game, &pos, &face, &canceled);
    #[cfg(target_os = "android")]
    {
        lane3::right_click_block(&mut *game, pos, face, &mut *canceled);
        if !*canceled && compat::right_click_block(&mut *game, pos) {
            *canceled = true;
        }
    }
}

/// The server's tick starts or ends: lane 3 mods start with the first and hear each.
/// CameraOverhaul's view, every frame: the yaw and pitch with its effects, and the roll
/// (degrees, the right side down when positive).
pub(super) fn native_camera(
    game: &Game<'_>,
    camera: [f64; 3],
    yaw: f32,
    pitch: f32,
) -> (f32, f32, f32) {
    let _ = (&game, &camera);
    #[cfg(target_os = "android")]
    {
        camera_overhaul::run(game, camera, yaw, pitch)
    }
    #[cfg(not(target_os = "android"))]
    {
        (yaw, pitch, 0.0)
    }
}

/// Something shook the screen, for CameraOverhaul: an explosion `distance` blocks away (none:
/// thunder), or the player's swing (`swing`).
pub(super) fn native_jolt(game: &Game<'_>, distance: Option<f64>, swing: bool) {
    let _ = (&game, &distance, &swing);
    #[cfg(target_os = "android")]
    {
        use camera_overhaul::Jolt;
        let jolt = match (distance, swing) {
            (_, true) => Jolt::Swing,
            (Some(d), _) => Jolt::Explosion(d),
            (None, _) => Jolt::Thunder,
        };
        camera_overhaul::jolt(game, jolt);
    }
}

/// First-person Model's settings (its page in Options > Mods).
#[derive(Clone, Copy, Debug)]
pub(super) struct FirstPerson {
    /// The model's own arms hold the items, instead of the first-person view's hands.
    pub player_arms: bool,
    /// How far back the body is set from the eyes.
    pub offset: f64,
    pub armor: bool,
}

pub(super) fn first_person_settings(game: &Game<'_>) -> FirstPerson {
    let _ = &game;
    #[cfg(target_os = "android")]
    {
        use bundled_settings::FIRST_PERSON as S;
        let id = compat::FIRST_PERSON;
        FirstPerson {
            player_arms: S[0].value(game, id) == "player",
            offset: f64::from(S[1].number(game, id)),
            armor: S[2].on(game, id),
        }
    }
    #[cfg(not(target_os = "android"))]
    {
        FirstPerson {
            player_arms: false,
            offset: 0.3,
            armor: true,
        }
    }
}

/// FirstPerson's body in first person (`perspective.rs`): on when its port is.
pub(super) fn first_person_body_on(game: &Game<'_>) -> bool {
    let _ = &game;
    #[cfg(target_os = "android")]
    {
        game.native_on(compat::FIRST_PERSON)
    }
    #[cfg(not(target_os = "android"))]
    {
        false
    }
}

/// Every frame, once the listener is placed: sound physics measures the new sounds.
pub(super) fn native_sound_physics(game: &mut Game<'_>, listener: [f64; 3], underwater: bool) {
    let _ = (&game, &listener, &underwater);
    #[cfg(target_os = "android")]
    {
        sound_physics::run(&mut *game, listener, underwater);
    }
}

pub(super) fn native_server_tick(game: &mut Game<'_>, end: bool) {
    let _ = (&game, &end);
    #[cfg(target_os = "android")]
    {
        lane3::server_tick(&mut *game, end);
    }
}

/// The server stops.
pub(super) fn native_server_stopping() {
    #[cfg(target_os = "android")]
    {
        lane3::server_stopping();
    }
}

pub(super) fn native_key(game: &mut Game<'_>, key: &str, pressed: bool) {
    let _ = (&game, &key, &pressed);
    #[cfg(target_os = "android")]
    {
        zoom_key::run(&mut *game, key, pressed);
    }
}

pub(super) fn native_wheel(game: &mut Game<'_>, steps: f32, claimed: &mut bool) {
    let _ = (&game, &steps, &claimed);
    #[cfg(target_os = "android")]
    {
        zoom_wheel::run(&*game, steps, &mut *claimed);
    }
}

pub(super) fn native_zoom(game: &Game<'_>, zoom: &mut Option<NativeZoom>) {
    let _ = (&game, &zoom);
    #[cfg(target_os = "android")]
    {
        zoom_zoom::run(game, &mut *zoom);
    }
}

pub(super) fn native_mods(game: &mut Game<'_>, mods: &mut Vec<NativeMod>) {
    let _ = (&game, &mods);
    #[cfg(target_os = "android")]
    {
        zoom_list::run(&mut *mods);
        minimap_list::run(&mut *mods);
        recipes_list::run(&mut *mods);
        smartf3_list::run(&mut *mods);
        compat::list(&mut *mods);
    }
}

pub(super) fn native_settings(game: &mut Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    let _ = (&game, &mod_id, &buttons);
    #[cfg(target_os = "android")]
    {
        zoom_settings::run(&mut *game, mod_id, &mut *buttons);
        minimap_settings::run(&mut *game, mod_id, &mut *buttons);
        recipes_settings::run(&mut *game, mod_id, &mut *buttons);
        smartf3_settings::run(&mut *game, mod_id, &mut *buttons);
        bundled_settings::settings(&mut *game, mod_id, &mut *buttons);
    }
}

pub(super) fn native_settings_click(game: &mut Game<'_>, mod_id: &str, button: usize) {
    let _ = (&game, &mod_id, &button);
    #[cfg(target_os = "android")]
    {
        zoom_click::run(&mut *game, mod_id, button);
        minimap_click::run(&mut *game, mod_id, button);
        recipes_click::run(&mut *game, mod_id, button);
        smartf3_click::run(&mut *game, mod_id, button);
        bundled_settings::settings_click(&mut *game, mod_id, button);
    }
}

pub(super) fn native_minimap(game: &Game<'_>, minimap: &mut Option<NativeMinimap>) {
    let _ = (&game, &minimap);
    #[cfg(target_os = "android")]
    {
        minimap_minimap::run(game, &mut *minimap);
    }
}

pub(super) fn native_frame(game: &mut Game<'_>, elapsed: Duration) {
    let _ = (&game, &elapsed);
    #[cfg(target_os = "android")]
    {
        recipes_frame::run(&mut *game);
    }
}

pub(super) fn native_panel_event(game: &mut Game<'_>, mod_id: &str, event: &NativePanelEvent) {
    let _ = (&game, &mod_id, &event);
    #[cfg(target_os = "android")]
    {
        recipes_event::run(&mut *game, mod_id, event);
    }
}

pub(super) fn native_command(game: &mut Game<'_>, command: &str, ran: &mut bool) {
    let _ = (&game, &command, &ran);
    #[cfg(target_os = "android")]
    {
        smartf3_command::run(&mut *game, command, &mut *ran);
    }
}

#[cfg(target_os = "android")]
pub(super) fn reset() {
    official_zoom::reset();
    official_minimap::reset();
    official_recipes::set_viewer(None);
    smartf3_native::reset();
}
