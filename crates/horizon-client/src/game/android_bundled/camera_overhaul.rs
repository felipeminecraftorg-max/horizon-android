// Written for Horizon Android; the repository's licence.
//! Horizon's own version of CameraOverhaul (the converter's known-mod lane, turned on by
//! converting the mod's jar): the camera rolls when strafing and when turning, pitches with
//! forward and vertical speed, sways gently while the player idles, and shakes from
//! explosions, thunder and swings. Written for Horizon from what the mod does, not from its
//! code. Only the view moves: aim, the crosshair's target and the hands don't.
use crate::game::Game;
use std::{sync::Mutex, time::Instant};

pub(crate) const CAMERA_OVERHAUL: &str = "cameraoverhaul";

/// What shakes the screen, and how much trauma (0 to 1) each adds.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Jolt {
    /// An explosion this far away.
    Explosion(f64),
    Thunder,
    Swing,
}

/// The player's settings (`bundled_settings::CAMERA_OVERHAUL`), each a factor (1 the mod's
/// default).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Tuning {
    pub third_person: bool,
    pub strafing_roll: f32,
    pub forward_pitch: f32,
    pub vertical_pitch: f32,
    pub turning_roll: f32,
    pub sway: f32,
    pub shakes: f32,
}
impl Default for Tuning {
    fn default() -> Self {
        Self {
            third_person: true,
            strafing_roll: 1.0,
            forward_pitch: 1.0,
            vertical_pitch: 1.0,
            turning_roll: 1.0,
            sway: 1.0,
            shakes: 1.0,
        }
    }
}

/// The camera's movement from one frame to the next.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Motion {
    /// Speeds in blocks a second: to the camera's right, ahead (level), up.
    pub strafe: f32,
    pub forward: f32,
    pub up: f32,
    /// How fast the view turns, in degrees a second (right positive).
    pub turn: f32,
    pub seconds: f32,
}

/// The effects' state from frame to frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Effects {
    roll: f32,
    pitch: f32,
    turn_roll: f32,
    trauma: f32,
    idle: f32,
    time: f32,
}

/// Smooth noise in [-1, 1]: a sum of sines at unrelated frequencies.
fn noise(t: f32, seed: f32) -> f32 {
    ((t * 1.0 + seed).sin() * 0.5
        + (t * 2.31 + seed * 1.7).sin() * 0.3
        + (t * 4.17 + seed * 2.9).sin() * 0.2)
        .clamp(-1.0, 1.0)
}

impl Effects {
    /// A jolt's trauma: an explosion's fades with distance (none past 48 blocks).
    pub(crate) fn jolt(&mut self, jolt: Jolt) {
        let add = match jolt {
            Jolt::Explosion(d) => 0.6 * (1.0 - (d / 48.0).min(1.0)) as f32,
            Jolt::Thunder => 0.25,
            Jolt::Swing => 0.03,
        };
        self.trauma = (self.trauma + add).min(1.0);
    }

    /// One frame: the offsets to add to the view, in degrees (yaw, pitch, roll; a positive roll
    /// takes the right side of the view down).
    pub(crate) fn step(&mut self, m: Motion, t: &Tuning) -> (f32, f32, f32) {
        let dt = m.seconds.clamp(0.0, 0.25);
        self.time += dt;
        let approach = |x: &mut f32, target: f32, rate: f32| {
            *x += (target - *x) * (1.0 - (-dt * rate).exp());
        };
        // Strafing leans into the side moved to (a positive roll takes the right side down), a
        // forward run dips the view, a fall lifts it.
        let roll_target = (m.strafe * 0.9 * t.strafing_roll).clamp(-10.0, 10.0);
        let pitch_target = (m.forward * 0.35 * t.forward_pitch - m.up * 0.45 * t.vertical_pitch)
            .clamp(-10.0, 10.0);
        approach(&mut self.roll, roll_target, 8.0);
        approach(&mut self.pitch, pitch_target, 6.0);
        // Turning adds to a roll that dies away.
        // Turning rolls the view into the turn, as fast as it turns, and lets go when it stops.
        let turn_target = (m.turn * 0.025 * t.turning_roll).clamp(-8.0, 8.0);
        approach(&mut self.turn_roll, turn_target, 6.0);
        // Idle a few seconds and the view starts to sway; any move fades it out quickly.
        let still = m.strafe.abs() + m.forward.abs() + m.up.abs() < 0.05 && m.turn.abs() < 1.0;
        if still {
            self.idle += dt;
        } else {
            self.idle = (self.idle - dt * 4.0).max(0.0).min(3.0);
        }
        let sway_in = ((self.idle - 3.0) / 2.0).clamp(0.0, 1.0) * t.sway;
        let s = self.time * 0.16 * std::f32::consts::TAU;
        let sway = [noise(s, 0.0), noise(s, 3.7), noise(s, 7.1)].map(|n| n * 0.6 * sway_in);
        // Trauma shakes with its square, and wears off in about a second.
        let shake = self.trauma * self.trauma * 2.5 * t.shakes;
        let q = self.time * 6.0 * std::f32::consts::TAU;
        let shook = [noise(q, 1.3), noise(q, 5.9), noise(q, 9.4)].map(|n| n * shake);
        self.trauma = (self.trauma - dt).max(0.0);
        (
            sway[0] + shook[0],
            self.pitch + sway[1] + shook[1],
            self.roll + self.turn_roll + sway[2] + shook[2],
        )
    }
}

/// The window's effects and the last frame's camera.
struct State {
    effects: Effects,
    last: Option<(Instant, [f64; 3], f32)>,
}
static STATE: Mutex<State> = Mutex::new(State {
    effects: Effects {
        roll: 0.0,
        pitch: 0.0,
        turn_roll: 0.0,
        trauma: 0.0,
        idle: 0.0,
        time: 0.0,
    },
    last: None,
});

/// Something shook the screen (an explosion, thunder, a swing).
pub(crate) fn jolt(game: &Game<'_>, jolt: Jolt) {
    if !game.native_on(CAMERA_OVERHAUL) {
        return;
    }
    if let Ok(mut s) = STATE.lock() {
        s.effects.jolt(jolt);
    }
}

/// The player's settings for CameraOverhaul, from its page.
fn tuning(game: &Game<'_>) -> Tuning {
    use super::bundled_settings::CAMERA_OVERHAUL as S;
    let id = CAMERA_OVERHAUL;
    Tuning {
        third_person: S[0].on(game, id),
        strafing_roll: S[1].number(game, id),
        forward_pitch: S[2].number(game, id),
        vertical_pitch: S[3].number(game, id),
        turning_roll: S[4].number(game, id),
        sway: S[5].number(game, id),
        shakes: S[6].number(game, id),
    }
}

/// Every frame: the view's yaw and pitch with the effects, and its roll (degrees).
pub(super) fn run(game: &Game<'_>, camera: [f64; 3], yaw: f32, pitch: f32) -> (f32, f32, f32) {
    let Ok(mut s) = STATE.lock() else {
        return (yaw, pitch, 0.0);
    };
    let now = Instant::now();
    let last = s.last.replace((now, camera, yaw));
    if !game.native_on(CAMERA_OVERHAUL) {
        s.effects = Effects::default();
        return (yaw, pitch, 0.0);
    }
    let t = tuning(game);
    if !t.third_person && !game.camera_type.first_person() {
        s.effects = Effects::default();
        return (yaw, pitch, 0.0);
    }
    let Some((then, before, yaw_before)) = last else {
        return (yaw, pitch, 0.0);
    };
    let seconds = now.duration_since(then).as_secs_f32().max(1e-4);
    let v = [0, 1, 2].map(|a| ((camera[a] - before[a]) / f64::from(seconds)) as f32);
    // A teleport or a respawn is no motion.
    let motion = if v.iter().any(|x| x.abs() > 60.0) {
        Motion {
            seconds,
            ..Motion::default()
        }
    } else {
        let r = yaw.to_radians();
        let (forward, right) = ([-r.sin(), r.cos()], [-r.cos(), -r.sin()]);
        Motion {
            strafe: v[0] * right[0] + v[2] * right[1],
            forward: v[0] * forward[0] + v[2] * forward[1],
            up: v[1],
            turn: ((yaw - yaw_before + 540.0).rem_euclid(360.0) - 180.0) / seconds,
            seconds,
        }
    };
    let (dy, dp, roll) = s.effects.step(motion, &t);
    (yaw + dy, (pitch + dp).clamp(-90.0, 90.0), roll)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frames(e: &mut Effects, m: Motion, n: usize) -> (f32, f32, f32) {
        let mut out = (0.0, 0.0, 0.0);
        for _ in 0..n {
            out = e.step(m, &Tuning::default());
        }
        out
    }

    #[test]
    fn strafing_rolls_and_standing_still_settles() {
        let mut e = Effects::default();
        let right = Motion {
            strafe: 4.3,
            seconds: 1.0 / 60.0,
            ..Motion::default()
        };
        let (_, _, roll) = frames(&mut e, right, 60);
        assert!(roll > 2.0, "{roll}");
        let left = Motion {
            strafe: -4.3,
            ..right
        };
        let (_, _, roll) = frames(&mut e, left, 60);
        assert!(roll < -2.0, "{roll}");
        let still = Motion {
            seconds: 1.0 / 60.0,
            ..Motion::default()
        };
        let (_, _, roll) = frames(&mut e, still, 60);
        assert!(roll.abs() < 0.5, "{roll}");
    }

    #[test]
    fn an_explosion_shakes_and_the_shake_wears_off() {
        let mut e = Effects::default();
        let still = Motion {
            seconds: 1.0 / 60.0,
            ..Motion::default()
        };
        e.jolt(Jolt::Explosion(4.0));
        let mut most = 0.0f32;
        for _ in 0..10 {
            let (y, p, r) = e.step(still, &Tuning::default());
            most = most.max(y.abs()).max(p.abs()).max(r.abs());
        }
        assert!(most > 0.1, "{most}");
        frames(&mut e, still, 120);
        assert_eq!(e.trauma, 0.0);
        // Too far away to feel.
        let mut far = Effects::default();
        far.jolt(Jolt::Explosion(100.0));
        assert_eq!(far.trauma, 0.0);
    }
}
