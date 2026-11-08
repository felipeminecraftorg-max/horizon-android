//! Entity Model Features (EMF) and OptiFine's Custom Entity Models in resource packs:
//! `assets/minecraft/emf/cem/<model>.jem` (or `optifine/cem/`). A `.jem` file's `animations` move
//! a model's parts every frame from what the entity does, with variables of its own that keep
//! their value from one frame to the next ([`expr`]).
//!
//! Horizon runs the animations of the player's models (`player`, `player_slim`, with
//! `player_cape` and `elytra`, which share the player's variables) over the vanilla parts: a
//! pack whose boxes are the vanilla model's (as animation packs' are) looks as it does with EMF.
//! A part's `tx`/`ty`/`tz` is its pivot in pixels and `rx`/`ry`/`rz` its rotation in radians, read
//! after the vanilla animation and written back.
pub mod expr;

use crate::mobs::{
    Mob,
    anim::{Poses, State},
};
use serde_json::Value;
use std::{
    collections::HashMap,
    path::Path,
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};

/// The entity values a program reads, by slot (EMF's names).
pub const INPUTS: [&str; 40] = [
    "frame_counter",
    "frame_time",
    "age",
    "is_paused",
    "is_in_gui",
    "limb_swing",
    "limb_speed",
    "move_forward",
    "move_strafing",
    "head_yaw",
    "head_pitch",
    "pos_x",
    "pos_y",
    "pos_z",
    "is_on_ground",
    "is_in_water",
    "is_wet",
    "is_climbing",
    "is_riding",
    "is_sitting",
    "is_sneaking",
    "is_sprinting",
    "is_swimming",
    "is_gliding",
    "is_burning",
    "is_blocking",
    "hurt_time",
    "fluid_depth",
    "is_holding_item_right",
    "is_holding_item_left",
    "is_first_person_hand",
    "swing_progress",
    "health",
    "max_health",
    "death_time",
    "is_alive",
    "is_hurt",
    "is_in_lava",
    "is_invisible",
    "id",
];

fn slot(name: &str) -> usize {
    INPUTS.iter().position(|n| *n == name).unwrap_or(0)
}

/// The vanilla player model's part for an EMF part name.
fn vanilla_part(name: &str) -> &str {
    match name {
        "headwear" => "hat",
        "cloak" => "cape",
        other => other,
    }
}

/// A model's parts by slot, as a program names them.
#[derive(Default)]
struct Parts(Vec<String>);
impl expr::Names for Parts {
    fn part(&mut self, name: &str) -> Option<usize> {
        if name.contains('.') || name == "render" {
            return None;
        }
        Some(match self.0.iter().position(|p| p == name) {
            Some(i) => i,
            None => {
                self.0.push(name.to_owned());
                self.0.len() - 1
            }
        })
    }
    fn input(&self, name: &str) -> Option<usize> {
        INPUTS.iter().position(|n| *n == name)
    }
}

/// The animations of one entity's models, compiled together.
pub struct Animation {
    program: expr::Program,
    /// The vanilla part each slot moves.
    parts: Vec<String>,
    /// The slots some line writes.
    written: Vec<bool>,
}

impl Animation {
    /// The `animations` of the `.jem` files `files` (in order: the entity's own model first),
    /// with what could not be read.
    pub fn compile(files: &[(&str, &Value)]) -> (Self, Vec<String>) {
        let mut program = expr::Program::default();
        let mut parts = Parts::default();
        let mut errors = Vec::new();
        for (name, jem) in files {
            for model in jem["models"].as_array().into_iter().flatten() {
                for block in model["animations"].as_array().into_iter().flatten() {
                    let Some(lines) = block.as_object() else {
                        continue;
                    };
                    let lines = lines
                        .iter()
                        .filter_map(|(k, v)| Some((k.as_str(), v.as_str()?)));
                    errors.extend(
                        program
                            .add(lines, &mut parts)
                            .into_iter()
                            .map(|e| format!("{name}: {e}")),
                    );
                }
            }
        }
        let mut written = vec![false; parts.0.len()];
        for (target, _) in &program.lines {
            if let expr::Target::Part(p, _) = target {
                written[*p] = true;
            }
        }
        let parts = parts.0.iter().map(|p| vanilla_part(p).to_owned()).collect();
        (
            Self {
                program,
                parts,
                written,
            },
            errors,
        )
    }
}

/// The packs' player animations: the wide and the slim skin's.
#[derive(Default)]
pub struct Models {
    pub player: Option<Animation>,
    pub player_slim: Option<Animation>,
}

static MODELS: RwLock<Option<Arc<Models>>> = RwLock::new(None);

/// Reads the resource root's `.jem` animations of the player's models (none: EMF is off).
pub fn load(root: &Path) {
    let read = |name: &str| -> Option<Value> {
        ["emf/cem", "optifine/cem"].iter().find_map(|dir| {
            let path = root.join(format!("assets/minecraft/{dir}/{name}.jem"));
            let bytes = std::fs::read(path).ok()?;
            serde_json::from_slice(&bytes).ok()
        })
    };
    let cape = read("player_cape");
    let elytra = read("elytra");
    let build = |own: &str| -> Option<Animation> {
        let own_jem = read(own)?;
        let mut files = vec![(own, &own_jem)];
        if let Some(c) = &cape {
            files.push(("player_cape", c));
        }
        if let Some(e) = &elytra {
            files.push(("elytra", e));
        }
        let (animation, errors) = Animation::compile(&files);
        for e in errors.iter().take(20) {
            log::warn!("Horizon: EMF {e}");
        }
        if !animation.program.unknown.is_empty() {
            log::info!(
                "Horizon: EMF {own}: names Horizon gives no value (they read 0): {}",
                animation.program.unknown.join(", ")
            );
        }
        log::info!(
            "Horizon: EMF {own}: {} animation lines",
            animation.program.lines.len()
        );
        (!animation.program.lines.is_empty()).then_some(animation)
    };
    let models = Models {
        player: build("player"),
        player_slim: build("player_slim"),
    };
    let any = models.player.is_some() || models.player_slim.is_some();
    *MODELS.write().unwrap_or_else(|e| e.into_inner()) = any.then(|| Arc::new(models));
    *LIVE.lock().unwrap_or_else(|e| e.into_inner()) = None;
}

/// Whether a resource pack animates the player's model.
pub fn active() -> bool {
    MODELS.read().map(|m| m.is_some()).unwrap_or(false)
}

static FRAME: AtomicU64 = AtomicU64::new(0);

/// The frame clock: when the last frame began and how long it took, and whether the game is
/// paused.
struct Clock {
    at: Option<Instant>,
    seconds: f64,
    paused: bool,
}
static CLOCK: Mutex<Clock> = Mutex::new(Clock {
    at: None,
    seconds: 1.0 / 60.0,
    paused: false,
});

/// A frame begins: the animations run again.
pub fn next_frame(paused: bool) {
    let frame = FRAME.fetch_add(1, Ordering::Relaxed) + 1;
    let now = Instant::now();
    if let Ok(mut c) = CLOCK.lock() {
        if let Some(at) = c.at {
            c.seconds = now.duration_since(at).as_secs_f64().clamp(1e-4, 1.0);
        }
        c.at = Some(now);
        c.paused = paused;
    }
    // Entities not drawn for a while (gone, out of sight) let their variables go.
    if frame % 600 == 0
        && let Ok(mut live) = LIVE.lock()
        && let Some(map) = live.as_mut()
    {
        map.retain(|_, l| frame.saturating_sub(l.frame) < 1200);
    }
}

/// An entity's animation state: its variables and this frame's parts.
struct Live {
    vars: Vec<f64>,
    frame: u64,
    slim: bool,
    out: Vec<[f32; 9]>,
}
static LIVE: Mutex<Option<HashMap<u32, Live>>> = Mutex::new(None);

/// The entity values for `m` drawn with `s`.
fn inputs(m: &Mob, s: &State) -> Vec<f64> {
    let (seconds, paused) = CLOCK
        .lock()
        .map(|c| (c.seconds, c.paused))
        .unwrap_or((1.0 / 60.0, false));
    let mut v = vec![0.0; INPUTS.len()];
    let mut set = |name: &str, x: f64| v[slot(name)] = x;
    let flag = |b: bool| if b { 1.0 } else { 0.0 };
    let e = &m.extra;
    let a = f64::from(s.partial);
    let at = [
        m.previous.x + (m.position.x - m.previous.x) * a,
        m.previous.y + (m.position.y - m.previous.y) * a,
        m.previous.z + (m.position.z - m.previous.z) * a,
    ];
    // How fast it moves along its facing and across it, about 1 at a walk.
    let yaw = f64::from(m.body_yaw).to_radians();
    let (vx, vz) = (m.position.x - m.previous.x, m.position.z - m.previous.z);
    let forward = (-yaw.sin() * vx + yaw.cos() * vz) / 0.2;
    let strafe = (yaw.cos() * vx + yaw.sin() * vz) / 0.2;
    let pose = e.avatar_pose;
    let in_water = m.fluids.in_water;
    set("frame_counter", FRAME.load(Ordering::Relaxed) as f64);
    set("frame_time", seconds);
    set("age", f64::from(s.age));
    set("is_paused", flag(paused));
    set("limb_swing", f64::from(s.pos));
    set("limb_speed", f64::from(s.speed));
    set("move_forward", forward.clamp(-1.0, 1.0));
    set("move_strafing", strafe.clamp(-1.0, 1.0));
    set("head_yaw", f64::from(s.y_rot));
    set("head_pitch", f64::from(s.x_rot));
    set("pos_x", at[0]);
    set("pos_y", at[1]);
    set("pos_z", at[2]);
    set("is_on_ground", flag(m.on_ground));
    set("is_in_water", flag(in_water));
    set("is_wet", flag(in_water || m.fluids.eye_in_water));
    set("is_climbing", flag(m.climbing));
    set("is_riding", flag(e.passenger));
    set(
        "is_sneaking",
        flag(e.crouching || pose == crate::pose::Pose::Crouching),
    );
    set("is_sprinting", flag(e.sprinting));
    set("is_swimming", flag(pose == crate::pose::Pose::Swimming));
    set(
        "is_gliding",
        flag(e.fall_flying || pose == crate::pose::Pose::FallFlying),
    );
    set("is_burning", flag(m.fire_ticks > 0));
    set("is_blocking", flag(e.blocking));
    set("hurt_time", f64::from(m.hurt_time.max(0)));
    set("fluid_depth", m.fluids.water.max(m.fluids.lava));
    // The main hand is the right one unless the player is left-handed.
    let (right, left) = if e.main_hand_left {
        (&e.off_hand, &e.main_hand)
    } else {
        (&e.main_hand, &e.off_hand)
    };
    set("is_holding_item_right", flag(right.is_some()));
    set("is_holding_item_left", flag(left.is_some()));
    set("swing_progress", f64::from(s.swing));
    set("health", f64::from(m.health));
    set("max_health", f64::from(m.kind.max_health));
    set("death_time", f64::from(m.death_time));
    set("is_alive", flag(m.health > 0.0));
    set("is_hurt", flag(m.hurt_time > 0));
    set("is_in_lava", flag(m.fluids.in_lava));
    set("is_invisible", flag(m.invisible));
    set("id", f64::from(m.id));
    v
}

/// Moves the parts of a player's model (`p`, set up by the vanilla animation) as the packs'
/// animations do. They run once a frame for each player, from the first of its models drawn
/// (the body, its layers and the cape share them).
pub fn animate(m: &Mob, s: &State, p: &mut Poses) {
    if m.kind.id != "minecraft:player" {
        return;
    }
    let Some(models) = MODELS.read().ok().and_then(|g| g.clone()) else {
        return;
    };
    let Some(anim) = (if m.slim {
        models.player_slim.as_ref().or(models.player.as_ref())
    } else {
        models.player.as_ref().or(models.player_slim.as_ref())
    }) else {
        return;
    };
    let frame = FRAME.load(Ordering::Relaxed);
    let Ok(mut guard) = LIVE.lock() else {
        return;
    };
    let live = guard
        .get_or_insert_with(HashMap::new)
        .entry(m.id)
        .or_insert_with(|| Live {
            vars: Vec::new(),
            frame: 0,
            slim: m.slim,
            out: Vec::new(),
        });
    if live.slim != m.slim {
        // Another skin, another model: its variables start over.
        *live = Live {
            vars: Vec::new(),
            frame: 0,
            slim: m.slim,
            out: Vec::new(),
        };
    }
    // The vanilla poses only a whole player model gives.
    let whole = [
        "head",
        "body",
        "right_arm",
        "left_arm",
        "right_leg",
        "left_leg",
    ]
    .iter()
    .all(|n| p.has(n));
    if live.frame != frame && whole {
        let mut parts: Vec<[f32; 9]> = anim
            .parts
            .iter()
            .map(|name| {
                if p.has(name) {
                    *p.get(name)
                } else {
                    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0]
                }
            })
            .collect();
        anim.program.run(&mut live.vars, &inputs(m, s), &mut parts);
        live.out = parts;
        live.frame = frame;
    }
    if live.out.len() != anim.parts.len() {
        return;
    }
    for (i, name) in anim.parts.iter().enumerate() {
        // The cape's swing is the cape layer's own.
        if anim.written[i] && name != "cape" && p.has(name) {
            *p.get(name) = live.out[i];
        }
    }
}
