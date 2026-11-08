// Written for Horizon Android; the repository's licence.
//! Horizon's own version of Sound Physics Remastered (the converter's known-mod lane, turned
//! on by converting the mod's jar): a world sound is muffled by the blocks between it and the
//! listener, echoes in the space around it (short in a room, long in a great cave, hardly at
//! all in the open), and every sound is dulled under water. Written for Horizon from what the
//! mod does, not from its code; the mixer side is `horizon_client::audio` (`Physics`).
//!
//! Each new world sound is measured once, on the window's own copy of the world, and a
//! looping one again every second while it plays.
use crate::game::Game;
use horizon_client::audio::Physics;
use horizon_client::blocks::{Fluid, StateFacts};

pub(crate) const SOUND_PHYSICS: &str = "sound_physics_remastered";

/// Rays cast from a sound to find the space around it, and how often each bounces.
const RAYS: usize = 16;
const BOUNCES: usize = 3;
/// How far one ray goes between bounces.
const REACH: f64 = 64.0;
/// Occlusion stops counting here: ten full blocks are as quiet as it gets.
const MAX_OCCLUSION: f32 = 10.0;
/// How much of the highs each unit of occlusion takes (`exp(-occlusion * ABSORPTION)`).
const ABSORPTION: f32 = 1.0;
/// The air's own toll on the highs, per block.
const AIR: f32 = 0.004;
/// How loud the echoes are overall.
const REVERB_GAIN: f32 = 1.0;
/// Sounds measured per frame at most; the rest wait for the next frames.
const PER_FRAME: usize = 6;
/// A looping sound is measured again this often.
const AGAIN: std::time::Duration = std::time::Duration::from_secs(1);

/// What a block does to sound: how much it blocks (a full block 1) and how much it reflects.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Material {
    pub occlusion: f32,
    pub reflectivity: f32,
}

/// A block's material, by the sound it makes (wool deadens, stone rings) and its shape (a
/// slab blocks half as much as a block, a flower almost nothing).
pub(crate) fn material(f: &StateFacts, air: bool) -> Material {
    if air || (f.fluid != Fluid::None && !f.waterlogged) {
        return Material {
            occlusion: 0.0,
            reflectivity: 0.0,
        };
    }
    let family = f
        .sounds
        .break_sound
        .split_once("block.")
        .and_then(|(_, rest)| rest.split('.').next())
        .unwrap_or("");
    let leaves = f.has_class("LeavesBlock");
    let (occlusion, reflectivity) = match family {
        _ if leaves => (0.3, 0.2),
        "wool" | "moss" | "moss_carpet" | "snow" | "powder_snow" | "sponge" | "wet_sponge" => {
            (1.5, 0.1)
        }
        "glass" => (0.75, 0.75),
        "stone" | "deepslate" | "deepslate_bricks" | "deepslate_tiles" | "polished_deepslate"
        | "nether_bricks" | "netherrack" | "basalt" | "calcite" | "tuff" | "tuff_bricks"
        | "polished_tuff" | "dripstone_block" | "bone_block" | "nether_ore" | "ancient_debris"
        | "lodestone" | "netherite_block" | "amethyst_block" | "amethyst_cluster"
        | "mud_bricks" => (1.0, 1.5),
        "metal" | "anvil" | "chain" | "copper" | "copper_bulb" | "copper_grate" | "heavy_core"
        | "vault" | "trial_spawner" => (1.0, 1.25),
        "wood" | "bamboo_wood" | "cherry_wood" | "nether_wood" | "ladder" | "scaffolding" => {
            (1.0, 0.4)
        }
        "gravel" | "rooted_dirt" | "mud" | "packed_mud" | "soul_sand" | "soul_soil" => (1.0, 0.3),
        "sand" | "suspicious_sand" | "suspicious_gravel" => (1.0, 0.2),
        "honey_block" | "slime_block" => (0.5, 0.5),
        "grass" => (1.0, 0.3),
        _ => (1.0, 0.5),
    };
    // A shape that doesn't fill its block blocks as much as it fills.
    let fill = if f.full_collision || f.can_occlude {
        1.0
    } else {
        f.outline
            .iter()
            .map(|b| ((b[3] - b[0]) * (b[4] - b[1]) * (b[5] - b[2])).max(0.0))
            .sum::<f64>()
            .min(1.0) as f32
    };
    Material {
        occlusion: occlusion * fill,
        reflectivity: reflectivity * fill.max(0.25),
    }
}

/// The blocks a straight line from `from` to `to` passes through, in order (Amanatides and
/// Woo), each with the face it entered by (0, 1, 2: the x, y or z face), at most `limit`.
pub(crate) fn line(
    from: [f64; 3],
    to: [f64; 3],
    limit: usize,
    mut visit: impl FnMut([i32; 3], usize, f64) -> bool,
) {
    let d = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
    let length = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if length < 1e-9 {
        return;
    }
    let mut cell = from.map(|v| v.floor() as i32);
    let end = to.map(|v| v.floor() as i32);
    let step = d.map(|v| if v > 0.0 { 1 } else { -1 });
    let mut next = [0.0f64; 3];
    let mut delta = [0.0f64; 3];
    for a in 0..3 {
        if d[a].abs() < 1e-12 {
            next[a] = f64::INFINITY;
            delta[a] = f64::INFINITY;
        } else {
            let edge = if d[a] > 0.0 {
                f64::from(cell[a]) + 1.0
            } else {
                f64::from(cell[a])
            };
            next[a] = (edge - from[a]) / d[a];
            delta[a] = 1.0 / d[a].abs();
        }
    }
    for _ in 0..limit {
        if cell == end {
            return;
        }
        let a = if next[0] <= next[1] && next[0] <= next[2] {
            0
        } else if next[1] <= next[2] {
            1
        } else {
            2
        };
        let t = next[a];
        if t > 1.0 {
            return;
        }
        cell[a] += step[a];
        next[a] += delta[a];
        if !visit(cell, a, t * length) {
            return;
        }
    }
}

/// How much the blocks between two points occlude, capped.
pub(crate) fn occlusion(
    from: [f64; 3],
    to: [f64; 3],
    blocks: &dyn Fn([i32; 3]) -> Material,
) -> f32 {
    let mut total = 0.0;
    let last = to.map(|v| v.floor() as i32);
    line(from, to, 600, |cell, _, _| {
        if cell != last {
            total += blocks(cell).occlusion;
        }
        total < MAX_OCCLUSION
    });
    total.min(MAX_OCCLUSION)
}

/// Directions spread evenly over the sphere (a golden-angle spiral).
fn directions() -> impl Iterator<Item = [f64; 3]> {
    let golden = std::f64::consts::PI * (3.0 - 5f64.sqrt());
    (0..RAYS).map(move |i| {
        let y = 1.0 - (i as f64 + 0.5) / RAYS as f64 * 2.0;
        let r = (1.0 - y * y).sqrt();
        let a = golden * i as f64;
        [a.cos() * r, y, a.sin() * r]
    })
}

/// What the world does to a sound at `at` heard at `listener`.
pub(crate) fn measure(
    at: [f64; 3],
    listener: [f64; 3],
    blocks: &dyn Fn([i32; 3]) -> Material,
) -> Physics {
    let source = at.map(|v| v.floor() as i32);
    // The sound gets round small things: the least occluded of a few nearby starting points.
    let mut occluded = occlusion(at, listener, blocks);
    if occluded > 0.0 {
        for o in [
            [0.4, 0.0, 0.0],
            [-0.4, 0.0, 0.0],
            [0.0, 0.0, 0.4],
            [0.0, 0.0, -0.4],
            [0.0, 0.4, 0.0],
        ] {
            let from = [at[0] + o[0], at[1] + o[1], at[2] + o[2]];
            if blocks(from.map(|v| v.floor() as i32)).occlusion > 0.0 {
                continue;
            }
            occluded = occluded.min(occlusion(from, listener, blocks));
        }
    }
    let d = ((at[0] - listener[0]).powi(2)
        + (at[1] - listener[1]).powi(2)
        + (at[2] - listener[2]).powi(2))
    .sqrt();
    let through = (-occluded * ABSORPTION).exp();
    let highs = through * (-(d as f32) * AIR).exp();
    // The echoes: rays from the sound, bounced off what they hit; each bounce point the
    // listener can hear from feeds the echo its path's length belongs to.
    let mut sends = [0.0f32; 4];
    for mut dir in directions() {
        let mut from = at;
        let mut path = d.min(REACH);
        let mut energy = 1.0f32;
        for _ in 0..BOUNCES {
            let to = [
                from[0] + dir[0] * REACH,
                from[1] + dir[1] * REACH,
                from[2] + dir[2] * REACH,
            ];
            let mut hit = None;
            line(from, to, 400, |cell, face, t| {
                if cell == source {
                    return true;
                }
                let m = blocks(cell);
                if m.occlusion >= 0.5 {
                    hit = Some((face, t, m.reflectivity));
                    return false;
                }
                true
            });
            let Some((face, t, reflectivity)) = hit else {
                break;
            };
            path += t;
            energy *= reflectivity.min(1.0) * 0.9 + 0.05;
            // Step back off the face, and turn the ray around on it.
            let point = [
                from[0] + dir[0] * (t - 0.01),
                from[1] + dir[1] * (t - 0.01),
                from[2] + dir[2] * (t - 0.01),
            ];
            if occlusion(point, listener, blocks) < 0.5 {
                let heard = energy * reflectivity.min(1.5);
                // Which echo: by path length, blended between neighbours.
                let x = match path {
                    p if p < 12.0 => (p / 12.0) as f32,
                    p if p < 40.0 => 1.0 + ((p - 12.0) / 28.0) as f32,
                    p if p < 100.0 => 2.0 + ((p - 40.0) / 60.0) as f32,
                    _ => 3.0,
                };
                for (k, send) in sends.iter_mut().enumerate() {
                    let w = (1.0 - (x - k as f32).abs()).max(0.0);
                    *send += heard * w;
                }
            }
            dir[face] = -dir[face];
            from = point;
            if energy < 0.02 {
                break;
            }
        }
    }
    let rays = RAYS as f32;
    Physics {
        gain: through.powf(0.1),
        highs,
        sends: sends.map(|s| (s / rays * 2.0 * REVERB_GAIN).min(1.0)),
    }
}

/// Every frame: sound physics follows its switch, and the sounds waiting are measured.
pub(super) fn run(game: &mut Game<'_>, listener: [f64; 3], underwater: bool) {
    let on = game.native_on(SOUND_PHYSICS);
    game.audio.set_sound_physics(on, underwater);
    if !on {
        return;
    }
    let mut wanted = game.audio.physics_wanted(AGAIN);
    if wanted.is_empty() {
        return;
    }
    // Nearest first.
    let away = |p: &[f64; 3]| {
        (p[0] - listener[0]).powi(2) + (p[1] - listener[1]).powi(2) + (p[2] - listener[2]).powi(2)
    };
    wanted.sort_by(|a, b| away(&a.1).total_cmp(&away(&b.1)));
    wanted.truncate(PER_FRAME);
    let world = &game.world;
    let facts = game.res.facts();
    let blocks = |c: [i32; 3]| {
        let state = world.block(c[0], c[1], c[2]);
        material(facts.get(state), facts.is_air(state))
    };
    let measured = wanted
        .into_iter()
        .map(|(id, at)| (id, measure(at, listener, &blocks)))
        .collect();
    game.audio.set_physics(measured);
}

#[cfg(test)]
mod tests {
    use super::*;

    const AIR_: Material = Material {
        occlusion: 0.0,
        reflectivity: 0.0,
    };
    const STONE: Material = Material {
        occlusion: 1.0,
        reflectivity: 1.5,
    };

    #[test]
    fn a_line_visits_each_block_it_crosses_once() {
        let mut cells = Vec::new();
        line([0.5, 0.5, 0.5], [3.5, 0.5, 0.5], 100, |c, face, _| {
            cells.push((c, face));
            true
        });
        assert_eq!(cells, [([1, 0, 0], 0), ([2, 0, 0], 0), ([3, 0, 0], 0)]);
    }

    #[test]
    fn a_wall_muffles_and_a_cave_echoes() {
        let open = |_: [i32; 3]| AIR_;
        let walled = |c: [i32; 3]| if c[0] == 3 { STONE } else { AIR_ };
        let outside = measure([0.5, 64.5, 0.5], [6.5, 64.5, 0.5], &open);
        let behind = measure([0.5, 64.5, 0.5], [6.5, 64.5, 0.5], &walled);
        assert_eq!(outside.sends, [0.0; 4]);
        assert!(outside.highs > 0.9);
        assert!(behind.highs < 0.5 && behind.gain < outside.gain);
        // A closed room of stone, 9 blocks wide: echoes.
        let room = |c: [i32; 3]| {
            if c.iter().any(|v| *v <= -5 || *v >= 5) {
                STONE
            } else {
                AIR_
            }
        };
        let inside = measure([0.5, 0.5, 0.5], [2.5, 0.5, 0.5], &room);
        assert!(inside.sends.iter().sum::<f32>() > 0.1, "{inside:?}");
        assert!(inside.highs > 0.9);
    }
}
