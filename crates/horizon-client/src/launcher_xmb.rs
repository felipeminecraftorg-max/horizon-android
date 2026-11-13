//! Horizon XMB Mobile — an original console-inspired spatial interface.
//!
//! This is not the previous "Old School" dashboard with a new background.
//! It has an independent cross of category and item axes, an independent
//! GPU atmosphere, and a fresh super-sampled icon language. The sole reused
//! presentation asset is the user's existing licensed launcher sound pack.
//! Other presets never enter this file.
use super::{color, Face, Icon, Ui};
use super::launcher_visual::{HomeAction, HomeView, Layout, ShellAction};

pub const HOME_ACTIONS: [HomeAction; 8] = [
    HomeAction::Play, HomeAction::Profile, HomeAction::Mods,
    HomeAction::Packs, HomeAction::Shaders, HomeAction::Files,
    HomeAction::Skins, HomeAction::Settings,
];

const HOME_LABELS: [&str; 8] = [
    "Minecraft 26.3", "Selecionar perfil", "Mods",
    "Pacotes de recursos", "Shaders", "Meus arquivos",
    "Skins", "Configurações",
];

const CATEGORY_LABELS: [&str; 7] = [
    "Jogo", "Mods", "Pacotes", "Shaders", "Biblioteca", "Sistema", "Skins",
];

const CATEGORY_ICONS: [Icon; 7] = [
    Icon::XmbGame, Icon::XmbMods, Icon::XmbPacks, Icon::XmbShaders,
    Icon::XmbFiles, Icon::XmbSystem, Icon::XmbIdentity,
];

#[derive(Clone, Debug, Default)]
pub struct Navigation {
    selected_home: usize,
}
impl Navigation {
    pub fn selected_home(&self) -> usize { self.selected_home }
    pub fn step_home(&mut self, delta: i32) -> HomeAction {
        self.selected_home = self.selected_home.saturating_add_signed(delta as isize)
            .min(HOME_ACTIONS.len() - 1);
        HOME_ACTIONS[self.selected_home]
    }
    pub fn focus_home(&mut self, action: HomeAction) {
        if let Some(index) = HOME_ACTIONS.iter().position(|item| *item == action) {
            self.selected_home = index;
        }
    }
    pub fn selected_action(&self) -> HomeAction { HOME_ACTIONS[self.selected_home] }
}

#[derive(Clone, Copy, Debug)]
pub struct Cross {
    pub portrait: bool,
    pub x: f32,
    pub category_y: f32,
    pub item_y: f32,
    pub category_step: f32,
    pub item_step: f32,
    pub bottom: f32,
}
impl Cross {
    pub fn from_screen(w: f32, h: f32) -> Self {
        let portrait = h > w || w < 620.;
        let category_y = if portrait {
            (h * 0.205).clamp(101., 165.)
        } else {
            (h * 0.232).clamp(82., 138.)
        };
        Self {
            portrait,
            // The item icons and the selected category share this exact axis.
            // Portrait shifts the cross left for thumb reach and long labels.
            x: if portrait { w * 0.32 } else { w * 0.395 },
            category_y,
            item_y: (if portrait { h * 0.37 } else { h * 0.47 })
                .max(category_y + if portrait { 95. } else { 76. })
                .min(h - 100.),
            category_step: if portrait { 79. } else { 112. },
            item_step: if portrait { 57. } else { 54. },
            bottom: h - if portrait { 60. } else { 57. },
        }
    }
    pub fn category_hit(self, w: f32, selected: usize, index: usize) -> Option<[f32; 4]> {
        let cx = self.x + (index as f32 - selected as f32) * self.category_step;
        let rect = [cx - 27., self.category_y - 27., 54., 62.];
        (rect[0] >= 8. && rect[0] + rect[2] <= w - 8.).then_some(rect)
    }
    pub fn item_hit(self, bounds: [f32; 4], selected: usize, index: usize) -> Option<[f32; 4]> {
        let y = self.item_y + (index as f32 - selected as f32) * self.item_step - 6.;
        let top = (self.category_y + 46.).max(bounds[1]);
        let bottom = self.bottom.min(bounds[1] + bounds[3]);
        let left = (self.x - 28.).max(bounds[0] + 6.);
        let right = if self.portrait {
            bounds[0] + bounds[2] - 8.
        } else {
            (bounds[0] + bounds[2] - 8.).min(self.x + bounds[2] * 0.36)
        };
        let rect = [left, y, (right - left).max(0.), 48.];
        (rect[1] >= top && rect[1] + rect[3] <= bottom && rect[2] >= 48.)
            .then_some(rect)
    }
}

/// Original XMB-native full-screen GPU effect. The renderer handles the
/// optical ribbons directly in WGSL; no old Graphite wallpaper is reused.
pub fn background(ui: &mut Ui<'_>, phase_seconds: f32, detail: f32) {
    ui.xmb_wallpaper(phase_seconds, detail);
}

fn icon_for_home(action: HomeAction) -> Icon {
    match action {
        HomeAction::Play => Icon::XmbGame,
        HomeAction::Profile | HomeAction::Skins => Icon::XmbIdentity,
        HomeAction::Mods => Icon::XmbMods,
        HomeAction::Packs => Icon::XmbPacks,
        HomeAction::Shaders => Icon::XmbShaders,
        HomeAction::Files => Icon::XmbFiles,
        HomeAction::Settings | HomeAction::Appearance => Icon::XmbSystem,
    }
}

fn glyph(ui: &mut Ui<'_>, icon: Icon, cx: f32, cy: f32, size: f32, active: bool) {
    let x = cx - size * 0.5;
    let y = cy - size * 0.5;
    if active {
        // A restrained optical aureole, not a card, plate or pressable pill.
        ui.rounded(cx - size * 0.68, cy - size * 0.68,
            size * 1.36, size * 1.36, size * 0.68,
            color::hex(0xAFCFFF, 15));
        ui.icon(icon, x - size * 0.075, y - size * 0.075,
            size * 1.15, color::hex(0x9BC9FF, 37));
    }
    ui.icon(icon, x, y, size,
        color::hex(if active { 0xF1F7FF } else { 0xC6D8EB },
            if active { 250 } else { 136 }));
}

#[allow(clippy::too_many_arguments)]
pub fn shell(
    ui: &mut Ui<'_>,
    categories: &[(&str, Icon)],
    selected: usize,
    account: &str,
    signed_in: bool,
    version: &str,
    mut target: impl FnMut([f32; 4], ShellAction),
) -> Layout {
    let (w, h) = (ui.w, ui.h);
    let c = Cross::from_screen(w, h);
    let layout = Layout::new(w, h);

    // Deliberate console-system header: light typography, no desktop bar.
    ui.text(Face::Regular, 17., 22., 14.,
        color::hex(0xECF3FC, 238), "Horizon");
    ui.text(Face::Regular, 10., 23., 42.,
        color::hex(0x9EB6D2, 176), version);
    let account_width = (w * 0.33).clamp(100., 190.);
    let account_x = w - account_width - 22.;
    let name = ui.fit_text(Face::Regular, 11., account, account_width - 14.);
    ui.text(Face::Regular, 11., account_x, 19.,
        color::hex(0xDDEAF8, 218), &name);
    if signed_in {
        ui.rounded(account_x - 11., 25., 5., 5., 2.5,
            color::hex(0x9CC5B1, 228));
    }
    target([account_x - 13., 9., account_width + 13., 45.], ShellAction::Account);
    target([20., 9., 104., 47.], ShellAction::Appearance);

    let chosen = selected.min(categories.len().saturating_sub(1));
    for (index, (fallback, _)) in categories.iter().enumerate() {
        let Some(hit) = c.category_hit(w, chosen, index) else { continue; };
        let current = chosen == index;
        let cx = hit[0] + hit[2] * 0.5;
        let icon = CATEGORY_ICONS.get(index).copied().unwrap_or(Icon::XmbSystem);
        glyph(ui, icon, cx, c.category_y, if current { 39. } else { 27. }, current);
        if current {
            // In the real XMB, only the focused category carries a caption.
            let label = CATEGORY_LABELS.get(index).copied().unwrap_or(fallback);
            let size = if c.portrait { 13. } else { 14. };
            let name = ui.fit_text(Face::Regular, size, label,
                if c.portrait { 115. } else { 150. });
            let tw = ui.measure(Face::Regular, size, &name);
            ui.text(Face::Regular, size, cx - tw * 0.5,
                c.category_y + 30., color::hex(0xEFF6FF, 232), &name);
            ui.rounded(cx - 1.5, c.category_y + 24., 3., 3., 1.5,
                color::hex(0xD6EDFF, 84));
        }
        target(hit, ShellAction::Page(index));
    }
    layout
}

fn context_for(action: HomeAction, view: &HomeView<'_>) -> (&'static str, &'static str, &'static str) {
    match action {
        HomeAction::Play => ("MINECRAFT", "JAVA EDITION 26.3",
            if view.launching { "INICIANDO JOGO" } else if view.installed {
                "PRONTO PARA JOGAR"
            } else { "INSTALAÇÃO NECESSÁRIA" }),
        HomeAction::Profile => ("SESSÃO", "PERFIL DE JOGO", "TROCAR PERFIL"),
        HomeAction::Mods => ("EXTENSÕES", "MODS", "GERENCIAR CONTEÚDO"),
        HomeAction::Packs => ("CONTEÚDO", "RECURSOS VISUAIS", "PACOTES INSTALADOS"),
        HomeAction::Shaders => ("ILUMINAÇÃO", "SHADER PACKS", "PERSONALIZAR VISUAIS"),
        HomeAction::Files => ("ARQUIVOS", "BIBLIOTECA LOCAL", "ABRIR GERENCIADOR"),
        HomeAction::Skins => ("IDENTIDADE", "SKINS E CAPAS", "PERSONALIZAR JOGADOR"),
        HomeAction::Settings | HomeAction::Appearance =>
            ("SISTEMA", "CONFIGURAÇÕES", "AJUSTAR HORIZON"),
    }
}

pub fn home(
    ui: &mut Ui<'_>,
    bounds: [f32; 4],
    view: &HomeView<'_>,
    selected: usize,
    mut target: impl FnMut([f32; 4], HomeAction),
) {
    let c = Cross::from_screen(ui.w, ui.h);
    let selected = selected.min(HOME_ACTIONS.len() - 1);
    let (category, title, subtitle) = context_for(HOME_ACTIONS[selected], view);

    // Context is pushed into the right-side air in landscape. Portrait uses
    // the same cross-axis grammar but the footer carries compact context.
    if !c.portrait {
        let context_x = (ui.w * 0.745).min(ui.w - 153.);
        let max_width = (ui.w - context_x - 15.).max(35.);
        glyph(ui, icon_for_home(HOME_ACTIONS[selected]),
            context_x + 31., c.item_y + 6., 56., false);
        ui.text(Face::Regular, 9.5, context_x, c.item_y + 48.,
            color::hex(0xB0C9E4, 166), category);
        let title = ui.fit_text(Face::Regular, 15., title, max_width);
        ui.text(Face::Regular, 15., context_x, c.item_y + 67.,
            color::hex(0xE8F3FD, 224), &title);
        let subtitle = ui.fit_text(Face::Regular, 10., subtitle, max_width);
        ui.text(Face::Regular, 10., context_x, c.item_y + 99.,
            color::hex(0xB8D6EE, 177), &subtitle);
    } else {
        let status = ui.fit_text(Face::Regular, 10.5, subtitle,
            (bounds[2] - 12.).max(50.));
        ui.text(Face::Regular, 10.5, bounds[0] + 6., c.bottom + 10.,
            color::hex(0xB4CEE5, 193), &status);
    }

    for (i, action) in HOME_ACTIONS.into_iter().enumerate() {
        let Some(r) = c.item_hit(bounds, selected, i) else { continue; };
        let chosen = i == selected;
        let cx = c.x;
        let cy = r[1] + 23.;
        glyph(ui, icon_for_home(action), cx, cy,
            if chosen { 35. } else { 25. }, chosen);
        let size = if chosen { 21. } else { 16. };
        let label = if i == 0 && !view.installed {
            "Instalar Minecraft"
        } else if i == 0 && view.launching {
            "Iniciando Minecraft"
        } else {
            HOME_LABELS[i]
        };
        let fitted = ui.fit_text(Face::Regular, size, label, (r[2] - 63.).max(20.));
        if chosen {
            // One-pixel spectral glow behind the selected text, no card.
            ui.text(Face::Regular, size, r[0] + 57.5, r[1] + 9.5,
                color::hex(0x8CB7E5, 28), &fitted);
        }
        ui.text(Face::Regular, size, r[0] + 57.,
            r[1] + if chosen { 9. } else { 12. },
            color::hex(if chosen { 0xF4F8FF } else { 0xBFD2E4 },
                if chosen { 249 } else { 152 }), &fitted);
        target(r, action);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn category_focus_moves_without_clipping_or_overlapping_touch_targets() {
        for (w, h) in [(340., 640.), (410., 780.), (760., 410.),
            (900., 480.), (1600., 900.)] {
            let c = Cross::from_screen(w, h);
            for selected in 0..7 {
                let hit: Vec<_> = (0..7).filter_map(|i| c.category_hit(w, selected, i)).collect();
                assert!(!hit.is_empty());
                assert!(hit.iter().all(|r| r[2] >= 48. && r[3] >= 48.));
                assert!(hit.iter().all(|r| r[0] >= 8. && r[0]+r[2] <= w-8.));
                assert!(hit.windows(2).all(|p| p[0][0]+p[0][2] < p[1][0]));
            }
        }
    }
    #[test]
    fn focused_home_action_lines_up_with_horizontal_xmb_cross() {
        for (w,h) in [(340.,640.),(410.,780.),(760.,410.),
            (900.,480.),(1600.,900.)] {
            let c=Cross::from_screen(w,h);
            let bounds=[if c.portrait {16.} else {24.},76.,
                w-if c.portrait {36.} else {48.},h-132.];
            for selected in 0..HOME_ACTIONS.len() {
                let r=c.item_hit(bounds,selected,selected).expect("selected item");
                assert!((r[0]+28.-c.x).abs()<0.1, "axis {w}x{h}: {r:?}");
                assert!(r[1]>=c.category_y+46.);
                assert!(r[2]>=48. && r[0]+r[2] <= w);
            }
        }
    }
    #[test]
    fn each_home_action_is_reachable_and_uses_its_own_icon() {
        let mut model=Navigation::default();
        assert_eq!(model.step_home(-200),HomeAction::Play);
        assert_eq!(model.step_home(200),HomeAction::Settings);
        for action in HOME_ACTIONS {
            model.focus_home(action);
            assert_eq!(model.selected_action(),action);
        }
        let icons:std::collections::HashSet<_> = HOME_ACTIONS
            .iter().copied().map(icon_for_home).collect();
        assert_eq!(icons.len(),7);
    }
}
