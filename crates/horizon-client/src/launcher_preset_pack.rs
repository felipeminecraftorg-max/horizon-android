//! Native WGSL material uniforms. Liquid Glass retains its calibrated look; 
//! Horizon Jade owns a separate restrained, real-time GGX satin-metal finish.
//! Source: styles/{liquid,oldschool}/material-global.json and
//! rendering.uniformValues in the supplied preset TXT files.
//! The six vec4s are streamed with the ordered launcher backdrop lenses.
//! This is NOT a copy of WebGL2 GLSL: the underlying compositor is wgpu/WGSL.
use super::ExperiencePreset;

/// [identity, light, optical, lighting, retro, finishing].
/// identity = [mode, finish, metalness, chrome]; mode -1 is legacy material.
/// optical = [strength, edge_roll_px, signed_depth, chromatic_px].
pub fn uniform_profile(experience: ExperiencePreset) -> [[f32; 4]; 6] {
    match experience {
        ExperiencePreset::LiquidGlass => [
            [0.0, 2.0, 0.56, 0.86],
            [0.57357645, -0.81915206, 1.19, 1.26],
            [1.122, 33.36, 0.35, 1.326],
            [0.70, 1.449, 0.35, 0.56],
            [0.30, 0.53, 0.77, 0.16],
            [1.008, 0.43, 0.47, 0.37],
        ],
        ExperiencePreset::OldSchool => [
            // Horizon Jade: low-glare satin alloy, not PS3 pearl chrome.
            [1.0, 1.0, 0.22, 0.15],
            [0.57357645, -0.81915206, 1.12, 0.72],
            [0.065, 9.0, 0.12, 0.012],
            [0.52, 0.85, 0.03, 0.08],
            [0.06, 0.10, 0.25, 0.23],
            [0.12, 0.32, 0.16, 0.70],
        ],
        ExperiencePreset::Standard => [
            [-1.0, 0.0, 0.0, 0.0],
            [0.0, -1.0, 1.0, 1.0],
            [1.0, 25.0, 0.25, 0.05],
            [0.7, 1.0, 0.15, 0.2],
            [0.0; 4],
            [1.0, 0.0, 0.2, 0.75],
        ],
    }
}

/// Frame-rate-independent damped spring for launcher navigation controls.
/// Source: exported V7 component.animation.physics (mass=1).
/// This changes navigation motion, not the actual Minecraft rendering/ticks.
pub fn spring_position(
    experience: ExperiencePreset,
    position: f32,
    velocity: f32,
    target: f32,
    delta_seconds: f32,
) -> (f32, f32) {
    if !position.is_finite() || !velocity.is_finite() || !target.is_finite() {
        return (target, 0.);
    }
    let dt = delta_seconds.clamp(0., 0.08);
    if dt == 0. { return (position, velocity); }
    let (stiffness, damping) = match experience {
        ExperiencePreset::LiquidGlass => (176.0_f32, 19.5_f32),
        ExperiencePreset::OldSchool => (199.8_f32, 23.525_f32),
        ExperiencePreset::Standard => (225.0_f32, 26.0_f32),
    };
    let omega = stiffness.sqrt();
    let zeta = (damping / (2. * omega)).min(0.99);
    let wd = omega * (1. - zeta * zeta).sqrt();
    let error = position - target;
    let cos_t = (wd * dt).cos();
    let sin_t = (wd * dt).sin();
    let attenuation = (-zeta * omega * dt).exp();
    let combination = (velocity + zeta * omega * error) / wd;
    let next = target + attenuation * (error * cos_t + combination * sin_t);
    let next_velocity = attenuation * (
        velocity * cos_t - (omega * omega * error + zeta * omega * velocity) / wd * sin_t
    );
    if (next - target).abs() < 0.005 && next_velocity.abs() < 0.02 {
        (target, 0.)
    } else {
        (next, next_velocity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn navigation_spring_is_stable_at_30_60_and_120_hz() {
        let mut values=Vec::new();
        for hz in [30,60,120] {
            let mut p=12.0_f32;
            let mut v=0.0_f32;
            for _ in 0..hz {
                (p,v)=spring_position(ExperiencePreset::LiquidGlass,p,v,86.,1./hz as f32);
            }
            values.push(p);
        }
        assert!((values[0]-values[1]).abs()<0.01);
        assert!((values[1]-values[2]).abs()<0.01);
        assert!((values[2]-86.).abs()<0.05);
    }
    #[test]
    fn v7_materials_are_independent_and_preserve_signed_depth() {
        let liquid=uniform_profile(ExperiencePreset::LiquidGlass);
        let old=uniform_profile(ExperiencePreset::OldSchool);
        assert_eq!(liquid[0][0], 0.0);
        assert_eq!(old[0][0], 1.0);
        assert_eq!(old[0][1], 1.0); // Jade satin-metal finish
        assert!((liquid[2][0]-1.122).abs()<0.0001);
        assert!((old[2][0]-0.065).abs()<0.0001);
        assert_eq!(old[5][3], 0.70);
        assert_ne!(liquid[3], old[3]);
    }
}
