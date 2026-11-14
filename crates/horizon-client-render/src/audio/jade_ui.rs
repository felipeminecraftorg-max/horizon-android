//! Original procedural UI cues for Horizon Jade. No Xbox/PS3 sound assets,
//! samples, licensed console boot themes, or bundled music are used.
//! Synthesized exactly once per family (cached in the audio mixer caller).
use std::f32::consts::TAU;

/// Returns a short mono floating-point PCM buffer, sample rate, channel count.
pub(super) fn synthesize(kind: &str) -> Option<(Vec<f32>, u32, u16)> {
    let (duration, start_hz, end_hz, sparkle) = match kind {
        "tabs" => (0.066_f32, 490_f32, 610_f32, 0.25_f32),
        "panel" => (0.116, 320., 525., 0.34),
        "back" => (0.082, 450., 288., 0.14),
        "confirm" | "pill" => (0.102, 435., 674., 0.29),
        "focus" => (0.042, 586., 622., 0.20),
        "card" | "square" => (0.063, 405., 478., 0.18),
        "switch" => (0.060, 565., 425., 0.19),
        "slider" | "scrollbar" => (0.037, 645., 675., 0.10),
        "chip" | "bubble" | "icon" => (0.055, 530., 590., 0.16),
        _ => return None,
    };
    const RATE: u32 = 24_000;
    let total = (duration * RATE as f32).ceil() as usize;
    let mut samples = Vec::with_capacity(total);
    for i in 0..total {
        let t = i as f32 / RATE as f32;
        let p = (t / duration).clamp(0., 1.);
        // Integrated frequency produces a continuous upward/downward glide
        // without the zipper clicks of multiplying an instantaneous frequency.
        let phase = TAU * (start_hz * t + 0.5 * (end_hz - start_hz) * t * p);
        let attack = (p / 0.055).min(1.);
        let release = (1. - p).max(0.).powf(2.3);
        let env = attack * release;
        // Tiny second/third harmonics create a restrained digital timbre;
        // all frequencies stay far below Nyquist at 24 kHz.
        let wave = phase.sin() * 0.73
            + (phase * 2. + 0.31).sin() * sparkle
            + (phase * 3. + 1.1).sin() * 0.08;
        samples.push((wave * env * 0.32).clamp(-0.44, 0.44));
    }
    Some((samples, RATE, 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jade_cues_are_short_mono_original_waveforms_without_clipping() {
        for kind in ["tabs", "panel", "back", "confirm", "focus",
                     "card", "switch", "slider", "chip"] {
            let (samples, rate, channels) = synthesize(kind).unwrap();
            assert_eq!(rate, 24_000);
            assert_eq!(channels, 1);
            assert!((500..=2900).contains(&samples.len()), "{kind}");
            assert!(samples.iter().all(|v| v.is_finite() && v.abs() <= 0.44));
            assert!(samples[0].abs() < 0.001, "{kind} must start quietly");
            assert!(samples.last().unwrap().abs() < 0.001, "{kind} must fade out");
        }
        assert_ne!(synthesize("tabs").unwrap().0,
                   synthesize("back").unwrap().0);
        assert!(synthesize("unknown").is_none());
    }
}
