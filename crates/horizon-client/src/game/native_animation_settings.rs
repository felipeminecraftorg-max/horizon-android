//! Local developer configuration for the two native animation previews.
//! No network access, no executable scripting, no changes to player data.
//! This is deliberately separate from the original JAR configs until parity.
use std::{fs, path::Path};

const MAX_SETTINGS_BYTES: u64 = 16 * 1024;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReAnimated {
    pub enabled: bool,
    pub speed: f32,
    pub reduced_motion: bool,
    pub buttons: bool,
}
impl Default for ReAnimated {
    fn default() -> Self {
        Self { enabled:false, speed:1.0, reduced_motion:false, buttons:true }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cinematic {
    pub enabled: bool,
    pub speed: f32,
    pub roll_strength: f32,
    pub reduced_motion: bool,
}
impl Default for Cinematic {
    fn default()->Self {
        Self { enabled:false, speed:1.0, roll_strength:1.0, reduced_motion:false }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PreviewSettings {
    pub reanimated: ReAnimated,
    pub cinematic: Cinematic,
}
fn boolean(node:&serde_json::Value, key:&str, default:bool)->bool {
    node.get(key).and_then(serde_json::Value::as_bool).unwrap_or(default)
}
fn scalar(node:&serde_json::Value, key:&str, default:f32, low:f32, high:f32)->f32 {
    node.get(key).and_then(serde_json::Value::as_f64)
        .filter(|x|x.is_finite())
        .map(|x|(x as f32).clamp(low,high))
        .unwrap_or(default)
}
impl PreviewSettings {
    /// Malformed settings fail closed (disabled previews), not crash the game.
    pub fn parse(text:&str)->Self {
        let Ok(root)=serde_json::from_str::<serde_json::Value>(text) else {
            return Self::default();
        };
        let mut out=Self::default();
        if let Some(re)=root.get("reanimated") {
            out.reanimated.enabled=boolean(re,"enabled",false);
            out.reanimated.speed=scalar(re,"speed",1.0,0.25,3.0);
            out.reanimated.reduced_motion=boolean(re,"reduced_motion",false);
            out.reanimated.buttons=boolean(re,"buttons",true);
        }
        if let Some(ci)=root.get("cinematic") {
            out.cinematic.enabled=boolean(ci,"enabled",false);
            out.cinematic.speed=scalar(ci,"speed",1.0,0.35,2.5);
            out.cinematic.roll_strength=scalar(ci,"roll_strength",1.0,0.0,1.0);
            out.cinematic.reduced_motion=boolean(ci,"reduced_motion",false);
        }
        out
    }

    pub fn load(root:&Path)->Self {
        let path=root.join("horizon-native-animations.json");
        match fs::metadata(&path) {
            Ok(meta) if meta.len() <= MAX_SETTINGS_BYTES =>
                fs::read_to_string(path).map(|c|Self::parse(&c)).unwrap_or_default(),
            _ => Self::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn parses_both_mods_and_clamps_settings() {
        let settings=PreviewSettings::parse(r#"{
            "reanimated":{"enabled":true,"speed":500,"buttons":false},
            "cinematic":{"enabled":true,"roll_strength":-8,"speed":0.0}
        }"#);
        assert!(settings.reanimated.enabled && settings.cinematic.enabled);
        assert_eq!(settings.reanimated.speed,3.0);
        assert_eq!(settings.cinematic.roll_strength,0.0);
        assert_eq!(settings.cinematic.speed,0.35);
        assert!(!settings.reanimated.buttons);
    }
    #[test] fn malformed_settings_fail_closed() {
        assert_eq!(PreviewSettings::parse("{bad"),PreviewSettings::default());
        assert_eq!(PreviewSettings::parse("{}"),PreviewSettings::default());
    }
    #[test] fn unknown_keys_do_not_enable_experimental_features() {
        assert_eq!(PreviewSettings::parse(r#"{"other":{"enabled":true}}"#),
            PreviewSettings::default());
    }
}
