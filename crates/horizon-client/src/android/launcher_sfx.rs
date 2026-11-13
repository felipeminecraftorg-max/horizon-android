//! Glass Lab V7's 20 exact sound assignments: ten control families per style.
//! A button press triggers one selected Ogg sample; no random variants/mixing.
use super::Action;
use horizon_client::ui_theme::{ThemePreset, launcher_visual::AppearanceAction as A};

pub(super) fn component(action: Action) -> &'static str {
    match action {
        Action::Page(_) | Action::SettingsCat(_) | Action::SelectThemePreset(_)
        | Action::SetInstalledSubTab(_) | Action::Appearance(A::Preset(_))
        | Action::Appearance(A::Wallpaper(_)) => "tabs",
        Action::Play | Action::InstallModalProject | Action::Install(_)
        | Action::FinishNameEdit | Action::SignIn | Action::SubmitEly => "pill",
        Action::Profile | Action::ChooseProfile(_) | Action::Renderer
        | Action::ChooseRenderer(_) | Action::ChooseUiScale(_) => "chip",
        Action::Appearance(A::Effects) | Action::Appearance(A::Sounds)
        | Action::Appearance(A::Motion) | Action::ToggleVsync
        | Action::ToggleSmoothLighting | Action::ToggleLauncherAnimations
        | Action::ToggleGameAnimations | Action::ToggleTouchVibrate => "switch",
        Action::Appearance(A::Transparency) | Action::Appearance(A::Blur)
        | Action::Appearance(A::Refraction)
        | Action::RenderScaleDelta(_) | Action::MouseSensDelta(_)
        | Action::TouchSensDelta(_) | Action::VirtualMouseSpeedDelta(_)
        | Action::ButtonScaleDelta(_) | Action::GyroSensDelta(_)
        | Action::FovDelta(_) => "slider",
        Action::Next | Action::Previous | Action::StepGradientMidPos(_) => "scrollbar",
        Action::OpenProject(_) | Action::OpenBundledProject(_)
        | Action::OpenFolder(_) => "card",
        Action::Account | Action::AddAccount | Action::SavedAccounts => "bubble",
        Action::ToggleSection(_) | Action::SetInstalledFilter(_) => "square",
        _ => "icon",
    }
}

/// The assignments match audio/per-component-assignments.json in toma.zip.
/// Volume is a quiet, non-overdriven UI gain multiplied by the style's
/// exported profile: Liquid 100%, Old School 49%.
pub(super) fn selection(preset: ThemePreset, action: Action) -> Option<(&'static str, f32)> {
    let kind = component(action);
    let (sound, gain) = match preset {
        ThemePreset::LiquidGlass => {
            (match kind {
                "bubble" | "square" => "sfx/modern-1.ogg",
                "pill" | "switch" => "sfx/modern-7.ogg",
                "icon" => "sfx/modern-4.ogg",
                "slider" | "scrollbar" => "sfx/coffee-1.ogg",
                "tabs" => "sfx/ultimate-ui-select-2.ogg",
                "chip" => "sfx/wood-block-3.ogg",
                "card" => "sfx/modern-10.ogg",
                _ => return None,
            }, 0.20)
        }
        ThemePreset::OldSchool => {
            (match kind {
                "bubble" | "square" => "sfx/retro-1.ogg",
                "pill" => "sfx/retro-9.ogg",
                "icon" => "sfx/retro-3.ogg",
                "slider" => "sfx/retro-5.ogg",
                "scrollbar" => "sfx/modern-12.ogg",
                "switch" | "card" => "sfx/minimalist-13.ogg",
                "tabs" => "sfx/coffee-2.ogg",
                "chip" => "sfx/retro-6.ogg",
                _ => return None,
            }, 0.20 * 0.49)
        }
        _ => return None,
    };
    Some((sound, gain))
}

/// Events are semantic (rather than tied to rejected buttons/cards).
/// No bundled sample is copied, overwritten or removed. The existing SFX
/// preference and native Vorbis decoder remain the only playback path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum XmbCue {
    ItemFocus,
    CategoryFocus,
    ContextOpen,
    Back,
}

pub(super) fn navigation(cue: XmbCue) -> (&'static str, f32) {
    // Names are all in the already-shipped Old School SFX bundle.
    let sample = match cue {
        XmbCue::ItemFocus => "sfx/retro-3.ogg",
        XmbCue::CategoryFocus => "sfx/coffee-2.ogg",
        XmbCue::ContextOpen => "sfx/minimalist-13.ogg",
        XmbCue::Back => "sfx/retro-6.ogg",
    };
    (sample, 0.064) // spatial focus: quieter than a confirmed button (0.098)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn xmb_focus_choreography_uses_existing_quiet_files() {
        for cue in [XmbCue::ItemFocus, XmbCue::CategoryFocus,
                    XmbCue::ContextOpen, XmbCue::Back] {
            let (path, gain) = navigation(cue);
            assert!(path.starts_with("sfx/") && path.ends_with(".ogg"));
            assert!(gain <= selection(ThemePreset::OldSchool, Action::Play).unwrap().1);
        }
    }

    #[test]
    fn distinct_styles_and_control_types_have_exact_user_sounds() {
        assert_eq!(selection(ThemePreset::LiquidGlass, Action::Page(
            crate::catalog::Page::Play)).unwrap().0, "sfx/ultimate-ui-select-2.ogg");
        assert_eq!(selection(ThemePreset::OldSchool, Action::Page(
            crate::catalog::Page::Play)).unwrap().0, "sfx/coffee-2.ogg");
        assert_eq!(selection(ThemePreset::OldSchool, Action::Next).unwrap().0, "sfx/modern-12.ogg");
        assert_eq!(selection(ThemePreset::LiquidGlass, Action::Play).unwrap().0, "sfx/modern-7.ogg");
        assert!(selection(ThemePreset::DarkAero, Action::Play).is_none());
    }
}
