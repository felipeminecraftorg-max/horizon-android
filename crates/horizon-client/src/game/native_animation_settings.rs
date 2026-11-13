//! Local developer configuration for the two native animation previews.
//! No network access, no executable scripting, no changes to player data.
//! This is deliberately separate from the original JAR configs until parity.
use std::{collections::BTreeMap, fs, path::Path};
use super::reanimated::{Ease, ScreenOverride};

const MAX_SETTINGS_BYTES: u64 = 16 * 1024;

#[derive(Clone, Debug, PartialEq)]
pub struct ReAnimated {
    pub enabled: bool,
    pub speed: f32,
    pub reduced_motion: bool,
    pub buttons: bool,
    pub hotbar: bool,
    pub hotbar_speed: f32,
    pub toasts: bool,
    pub per_screen: BTreeMap<String,ScreenOverride>,
}
impl Default for ReAnimated {
    fn default() -> Self {
        Self { enabled:false, speed:1.0, reduced_motion:false,
            buttons:true,hotbar:true,hotbar_speed:1.0,toasts:true,
            per_screen:BTreeMap::new() }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cinematic {
    pub enabled: bool,
    pub speed: f32,
    pub roll_strength: f32,
    pub reduced_motion: bool,
    pub scene_move: bool,
}
impl Default for Cinematic {
    fn default()->Self {
        Self { enabled:false, speed:1.0, roll_strength:1.0,
            reduced_motion:false, scene_move:true }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
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
            out.reanimated.hotbar=boolean(re,"hotbar",true);
            out.reanimated.hotbar_speed=scalar(re,"hotbar_speed",1.0,0.25,4.0);
            out.reanimated.toasts=boolean(re,"toasts",true);
            if let Some(profiles)=re.get("per_screen").and_then(serde_json::Value::as_object) {
                for (screen,profile) in profiles.iter().take(128) {
                    if screen.is_empty() || screen.len()>100
                        || !screen.chars().all(|ch| ch.is_ascii_alphanumeric()
                            || matches!(ch,'_'|'-'|'.'|':')) {continue;}
                    let enabled=profile.get("enabled").and_then(serde_json::Value::as_bool);
                    let scale=profile.get("scale").and_then(serde_json::Value::as_bool);
                    let duration_ms=profile.get("duration_ms")
                        .and_then(serde_json::Value::as_f64)
                        .filter(|n|n.is_finite())
                        .map(|n|(n as f32).clamp(50.0,2000.0));
                    let easing=profile.get("easing").and_then(serde_json::Value::as_str)
                        .and_then(|x| match x {
                            "smooth" => Some(Ease::Smooth),
                            "out_cubic" => Some(Ease::OutCubic),
                            "out_quart" => Some(Ease::OutQuart),
                            "out_back" => Some(Ease::OutBack),
                            _=>None,
                        });
                    out.reanimated.per_screen.insert(screen.clone(),ScreenOverride{
                        enabled,scale,duration_ms,easing,
                    });
                }
            }
        }
        if let Some(ci)=root.get("cinematic") {
            out.cinematic.enabled=boolean(ci,"enabled",false);
            out.cinematic.speed=scalar(ci,"speed",1.0,0.35,2.5);
            out.cinematic.roll_strength=scalar(ci,"roll_strength",1.0,0.0,1.0);
            out.cinematic.reduced_motion=boolean(ci,"reduced_motion",false);
            out.cinematic.scene_move=boolean(ci,"scene_move",true);
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
            "reanimated":{"enabled":true,"speed":500,"buttons":false,"hotbar_speed":100,"toasts":false},
            "cinematic":{"enabled":true,"roll_strength":-8,"speed":0.0,"scene_move":false}
        }"#);
        assert!(settings.reanimated.enabled && settings.cinematic.enabled);
        assert_eq!(settings.reanimated.speed,3.0);
        assert_eq!(settings.cinematic.roll_strength,0.0);
        assert_eq!(settings.cinematic.speed,0.35);
        assert!(!settings.cinematic.scene_move);
        assert!(!settings.reanimated.buttons);
        assert!(!settings.reanimated.toasts);
        assert_eq!(settings.reanimated.hotbar_speed,4.0);
    }
    #[test] fn parses_per_screen_easing_and_rejects_invalid_profiles() {
        let settings=PreviewSettings::parse(r#"{
            "reanimated":{"enabled":true,"per_screen":{
                "inventory":{"duration_ms":120,"easing":"out_back","scale":false},
                "options":{"enabled":false},
                "../unsafe":{"enabled":true}
            }}
        }"#);
        assert_eq!(settings.reanimated.per_screen.len(),2);
        assert_eq!(settings.reanimated.per_screen["inventory"].duration_ms,Some(120.0));
        assert_eq!(settings.reanimated.per_screen["inventory"].easing,Some(Ease::OutBack));
        assert_eq!(settings.reanimated.per_screen["options"].enabled,Some(false));
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
