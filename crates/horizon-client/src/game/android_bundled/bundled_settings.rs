// Written for Horizon Android; the repository's licence.
//! Settings pages for Horizon's own versions of known Java mods (Sound Physics, First-person
//! Model): each setting is a button on the mod's page in Options > Mods that steps through its
//! values, kept with the mod's other settings (`native_setting`). The game reads them every
//! frame, so they are cached here and the cache follows each click.
use crate::game::Game;
use std::{collections::HashMap, sync::Mutex};

/// One setting: its key, its button's label and its values (as stored, as shown).
pub(crate) struct Choice {
    pub key: &'static str,
    pub label: &'static str,
    pub values: &'static [(&'static str, &'static str)],
    pub default: usize,
}

static CACHE: Mutex<Option<HashMap<(String, String), Option<String>>>> = Mutex::new(None);

fn stored(game: &Game<'_>, mod_id: &str, key: &str) -> Option<String> {
    let Ok(mut cache) = CACHE.lock() else {
        return game.native_setting(mod_id, key);
    };
    cache
        .get_or_insert_with(HashMap::new)
        .entry((mod_id.to_owned(), key.to_owned()))
        .or_insert_with(|| game.native_setting(mod_id, key))
        .clone()
}

impl Choice {
    pub(crate) fn index(&self, game: &Game<'_>, mod_id: &str) -> usize {
        stored(game, mod_id, self.key)
            .and_then(|v| self.values.iter().position(|(s, _)| *s == v))
            .unwrap_or(self.default)
    }
    pub(crate) fn value(&self, game: &Game<'_>, mod_id: &str) -> &'static str {
        self.values[self.index(game, mod_id).min(self.values.len() - 1)].0
    }
    pub(crate) fn on(&self, game: &Game<'_>, mod_id: &str) -> bool {
        self.value(game, mod_id) == "on"
    }
    pub(crate) fn number(&self, game: &Game<'_>, mod_id: &str) -> f32 {
        self.value(game, mod_id).parse().unwrap_or(0.0)
    }
}

/// The page's buttons: `Label: value` for each setting.
pub(crate) fn labels(game: &Game<'_>, mod_id: &str, choices: &[Choice], buttons: &mut Vec<String>) {
    for c in choices {
        let shown = c.values[c.index(game, mod_id).min(c.values.len() - 1)].1;
        buttons.push(format!("{}: {shown}", c.label));
    }
}

/// A click on button `button`: that setting's next value.
pub(crate) fn click(game: &mut Game<'_>, mod_id: &str, choices: &[Choice], button: usize) {
    let Some(c) = choices.get(button) else {
        return;
    };
    let next = (c.index(game, mod_id) + 1) % c.values.len();
    let value = c.values[next].0;
    game.native_set_setting(mod_id, c.key, value);
    if let Ok(mut cache) = CACHE.lock() {
        cache.get_or_insert_with(HashMap::new).insert(
            (mod_id.to_owned(), c.key.to_owned()),
            Some(value.to_owned()),
        );
    }
}

const ON_OFF: &[(&str, &str)] = &[("on", "ON"), ("off", "OFF")];
const PERCENT: &[(&str, &str)] = &[
    ("0", "OFF"),
    ("0.5", "50%"),
    ("1", "100%"),
    ("1.5", "150%"),
    ("2", "200%"),
];

/// First-person Model's settings.
pub(crate) const FIRST_PERSON: [Choice; 3] = [
    Choice {
        key: "arms",
        label: "Arms",
        values: &[("player", "Player's arms"), ("vanilla", "Vanilla hands")],
        default: 0,
    },
    Choice {
        key: "offset",
        label: "Body offset",
        values: &[
            ("0.2", "0.20"),
            ("0.25", "0.25"),
            ("0.3", "0.30"),
            ("0.35", "0.35"),
            ("0.4", "0.40"),
            ("0.5", "0.50"),
        ],
        default: 2,
    },
    Choice {
        key: "armor",
        label: "Armor",
        values: &[("on", "Shown"), ("off", "Hidden")],
        default: 0,
    },
];

/// Sound Physics' settings.
pub(crate) const SOUND_PHYSICS: [Choice; 9] = [
    Choice {
        key: "occlusion",
        label: "Muffling",
        values: PERCENT,
        default: 2,
    },
    Choice {
        key: "reverb",
        label: "Echo",
        values: PERCENT,
        default: 2,
    },
    Choice {
        key: "brightness",
        label: "Echo brightness",
        values: &[("3000", "Dark"), ("6000", "Normal"), ("10000", "Bright")],
        default: 1,
    },
    Choice {
        key: "direction",
        label: "Sound direction",
        values: ON_OFF,
        default: 0,
    },
    Choice {
        key: "underwater",
        label: "Underwater filter",
        values: ON_OFF,
        default: 0,
    },
    Choice {
        key: "air",
        label: "Air absorption",
        values: ON_OFF,
        default: 0,
    },
    Choice {
        key: "rays",
        label: "Rays",
        values: &[("8", "8"), ("16", "16"), ("24", "24"), ("32", "32")],
        default: 1,
    },
    Choice {
        key: "bounces",
        label: "Bounces",
        values: &[("2", "2"), ("3", "3"), ("4", "4")],
        default: 1,
    },
    Choice {
        key: "reach",
        label: "Ray distance",
        values: &[("32", "32"), ("64", "64"), ("128", "128"), ("256", "256")],
        default: 1,
    },
];

/// CameraOverhaul's settings.
pub(crate) const CAMERA_OVERHAUL: [Choice; 7] = [
    Choice {
        key: "third_person",
        label: "In third person",
        values: ON_OFF,
        default: 0,
    },
    Choice {
        key: "strafing_roll",
        label: "Strafing roll",
        values: PERCENT,
        default: 2,
    },
    Choice {
        key: "forward_pitch",
        label: "Forward velocity pitch",
        values: PERCENT,
        default: 2,
    },
    Choice {
        key: "vertical_pitch",
        label: "Vertical velocity pitch",
        values: PERCENT,
        default: 2,
    },
    Choice {
        key: "turning_roll",
        label: "Mouselook roll",
        values: PERCENT,
        default: 2,
    },
    Choice {
        key: "sway",
        label: "Idle sway",
        values: PERCENT,
        default: 2,
    },
    Choice {
        key: "shakes",
        label: "Screen shakes",
        values: PERCENT,
        default: 2,
    },
];

/// The settings page of mod `mod_id`, when it is one of these.
fn page(mod_id: &str) -> Option<&'static [Choice]> {
    match mod_id {
        super::compat::FIRST_PERSON => Some(&FIRST_PERSON),
        super::sound_physics::SOUND_PHYSICS => Some(&SOUND_PHYSICS),
        super::camera_overhaul::CAMERA_OVERHAUL => Some(&CAMERA_OVERHAUL),
        _ => None,
    }
}

pub(super) fn settings(game: &mut Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    if let Some(choices) = page(mod_id) {
        labels(game, mod_id, choices, buttons);
    }
}

pub(super) fn settings_click(game: &mut Game<'_>, mod_id: &str, button: usize) {
    if let Some(choices) = page(mod_id) {
        click(game, mod_id, choices, button);
    }
}
