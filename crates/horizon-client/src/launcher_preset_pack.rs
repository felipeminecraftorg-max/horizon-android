//! Native WGSL uniform bridge for the user's exported GLASS_LAB_PACK_V7.
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
            [1.0, 1.0, 0.75, 0.90],
            [0.57357645, -0.81915206, 1.148, 1.26],
            [0.4785, 21.84, 0.48, 0.068],
            [0.90, 1.386, 0.19, 0.24],
            [0.14, 0.58, 0.79, 0.12],
            [0.028, 0.0, 0.30, 0.91],
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn v7_materials_are_independent_and_preserve_signed_depth() {
        let liquid=uniform_profile(ExperiencePreset::LiquidGlass);
        let old=uniform_profile(ExperiencePreset::OldSchool);
        assert_eq!(liquid[0][0], 0.0);
        assert_eq!(old[0][0], 1.0);
        assert_eq!(old[0][1], 1.0); // exported PS3-pearl finish
        assert!((liquid[2][0]-1.122).abs()<0.0001);
        assert!((old[2][0]-0.4785).abs()<0.0001);
        assert_eq!(old[5][3], 0.91); // solid scrollbar thumb
        assert_ne!(liquid[3], old[3]);
    }
}
