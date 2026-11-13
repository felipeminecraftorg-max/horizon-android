//! Horizon XMB Mobile: independent spatial navigation and home compositor.
//!
//! The Old School experience is rebuilt from the XMB interaction grammar:
//! a horizontal current-category axis crossing a vertical focused-item axis.
//! This code intentionally does NOT call the previous Old School dashboard,
//! Material cards, glass panels, old chrome wallpaper, or console icon family.
//! The lightweight vector language below is original to Horizon.
//!
//! Part 1: production shell and Home layout. Subsequent parts add WGSL waves,
//! richer iconography, contextual content surfaces and inertial touch navigation.
use super::{color, Face, Icon, Ui};
use super::launcher_visual::{HomeAction, HomeView, Layout, ShellAction};

pub const HOME_ACTIONS: [HomeAction; 8] = [
    HomeAction::Play,
    HomeAction::Profile,
    HomeAction::Mods,
    HomeAction::Packs,
    HomeAction::Shaders,
    HomeAction::Files,
    HomeAction::Skins,
    HomeAction::Settings,
];

const HOME_LABELS: [&str; 8] = [
    "Minecraft 26.3",
    "Selecionar perfil",
    "Mods",
    "Pacotes de recursos",
    "Shaders",
    "Meus arquivos",
    "Skins",
    "Configurações",
];

#[derive(Clone, Debug)]
pub struct Navigation {
    selected_home: usize,
}
impl Default for Navigation {
    fn default() -> Self {
        Self { selected_home: 0 }
    }
}
impl Navigation {
    pub fn selected_home(&self) -> usize {
        self.selected_home
    }
    pub fn step_home(&mut self, delta: i32) -> HomeAction {
        self.selected_home = self.selected_home
            .saturating_add_signed(delta as isize)
            .min(HOME_ACTIONS.len() - 1);
        HOME_ACTIONS[self.selected_home]
    }
    pub fn focus_home(&mut self, action: HomeAction) {
        if let Some(index) = HOME_ACTIONS.iter().position(|item| *item == action) {
            self.selected_home = index;
        }
    }
    pub fn selected_action(&self) -> HomeAction {
        HOME_ACTIONS[self.selected_home]
    }
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
            x: if portrait { w * 0.47 } else { w * 0.395 },
            category_y,
            item_y: (if portrait { h * 0.37 } else { h * 0.47 })
                .max(category_y + if portrait { 95. } else { 76. })
                .min(h - 100.),
            category_step: if portrait { 79. } else { 112. },
            item_step: if portrait { 57. } else { 54. },
            bottom: h - if portrait { 48. } else { 57. },
        }
    }

    pub fn category_hit(self, w: f32, selected: usize, index: usize) -> Option<[f32; 4]> {
        let cx = self.x + (index as f32 - selected as f32) * self.category_step;
        let rect = [cx - 27., self.category_y - 27., 54., 62.];
        (rect[0] >= 8. && rect[0] + rect[2] <= w - 8.).then_some(rect)
    }

    pub fn item_hit(self, bounds: [f32; 4], selected: usize, index: usize) -> Option<[f32; 4]> {
        let y = self.item_y + (index as f32 - selected as f32) * self.item_step - 6.;
        let first = (self.category_y + 46.).max(bounds[1]);
        let last = bounds[1] + bounds[3];
        let bottom = self.bottom.min(last);
        let rect = [
            bounds[0] + 6.,
            y,
            (bounds[2] * if self.portrait { 0.97 } else { 0.64 }).max(48.).min(bounds[2] - 12.),
            48.,
        ];
        (rect[1] >= first && rect[1] + rect[3] <= bottom).then_some(rect)
    }
}

/// Truly native single-pass GPU wallpaper. It is part of the original
/// Horizon XMB, not a resized bitmap or a fallback to the old dashboard.
/// Pass a frozen phase in battery/reduced-motion mode to save energy.
pub fn background(ui: &mut Ui<'_>, phase_seconds: f32, detail: f32) {
    ui.xmb_wallpaper(phase_seconds, detail);
}

#[derive(Clone, Copy)]
enum Symbol {
    Game,
    Extensions,
    Archive,
    Light,
    Storage,
    Sliders,
    Identity,
}

fn symbol_for_category(index: usize) -> Symbol {
    match index {
        0 => Symbol::Game,
        1 => Symbol::Extensions,
        2 => Symbol::Archive,
        3 => Symbol::Light,
        4 => Symbol::Storage,
        5 => Symbol::Sliders,
        _ => Symbol::Identity,
    }
}
fn symbol_for_home(action: HomeAction) -> Symbol {
    match action {
        HomeAction::Play => Symbol::Game,
        HomeAction::Profile | HomeAction::Skins => Symbol::Identity,
        HomeAction::Mods => Symbol::Extensions,
        HomeAction::Packs => Symbol::Archive,
        HomeAction::Shaders => Symbol::Light,
        HomeAction::Files => Symbol::Storage,
        HomeAction::Settings | HomeAction::Appearance => Symbol::Sliders,
    }
}

/// Original ultra-light XMB pictograms built directly from primitives.
/// No previous Old School vector assets or Sony-owned icons are loaded.
fn symbol(ui: &mut Ui<'_>, glyph: Symbol, cx: f32, cy: f32, s: f32, ink: u32) {
    let x = cx - s * 0.5;
    let y = cy - s * 0.5;
    let t = (s * 0.066).max(1.);
    match glyph {
        Symbol::Game => {
            // Two mirrored trigger arms and tiny thumb accents: controller silhouette.
            ui.stroke_rounded(x + s * 0.09, y + s * 0.30, s * 0.82, s * 0.51,
                s * 0.19, t, ink);
            ui.fill(x + s * 0.27, y + s * 0.45, s * 0.20, t, ink);
            ui.fill(x + s * 0.35 - t * 0.5, y + s * 0.37, t, s * 0.20, ink);
            ui.rounded(x + s * 0.68, y + s * 0.43, t * 1.55, t * 1.55, t * 0.8, ink);
            ui.rounded(x + s * 0.75, y + s * 0.55, t * 1.55, t * 1.55, t * 0.8, ink);
        }
        Symbol::Extensions => {
            let d = s * 0.22;
            for (col, row) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                ui.stroke_rounded(
                    x + s * (0.19 + col as f32 * 0.38),
                    y + s * (0.19 + row as f32 * 0.38),
                    d, d, t * 1.1, t, ink,
                );
            }
        }
        Symbol::Archive => {
            for i in 0..3 {
                let offset = i as f32 * s * 0.115;
                ui.stroke_rounded(x + s * 0.22 - offset * 0.32,
                    y + s * 0.17 + offset,
                    s * 0.58, s * 0.22,
                    t * 0.9, t, ink);
            }
        }
        Symbol::Light => {
            ui.stroke_rounded(x + s * 0.30, y + s * 0.29, s * 0.40, s * 0.40,
                s * 0.21, t, ink);
            for (ax, ay, bw, bh) in [
                (0.48, 0.04, 0.04, 0.16), (0.48, 0.80, 0.04, 0.16),
                (0.04, 0.48, 0.16, 0.04), (0.80, 0.48, 0.16, 0.04),
            ] {
                ui.fill(x + s * ax, y + s * ay, s * bw, s * bh, ink);
            }
        }
        Symbol::Storage => {
            ui.stroke_rounded(x + s * 0.13, y + s * 0.30, s * 0.74, s * 0.48,
                t, t, ink);
            ui.fill(x + s * 0.17, y + s * 0.24, s * 0.30, t * 1.1, ink);
        }
        Symbol::Sliders => {
            for (line, dot) in [(0.25, 0.34), (0.5, 0.67), (0.75, 0.44)] {
                ui.fill(x + s * 0.15, y + s * line, s * 0.70, t, ink);
                ui.rounded(x + s * dot - t * 0.5,
                    y + s * line - t * 1.3, t * 2.0, t * 3.5,
                    t, ink);
            }
        }
        Symbol::Identity => {
            ui.stroke_rounded(x + s * 0.39, y + s * 0.16, s * 0.22, s * 0.23,
                s * 0.11, t, ink);
            ui.stroke_rounded(x + s * 0.22, y + s * 0.57, s * 0.56, s * 0.26,
                s * 0.13, t, ink);
        }
    }
}

/// Real XMB horizontal category world. No background to each tab; neighbor
/// categories continue off-screen and only the centered one owns the focus.
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
    let w = ui.w;
    let h = ui.h;
    let cross = Cross::from_screen(w, h);
    let layout = Layout::new(w, h);
    ui.text(Face::Regular, 19., 21., 16.,
        color::hex(0xE9ECF7, 245), "Horizon");
    ui.text(Face::Regular, 9.5, 23., 42.,
        color::hex(0xAABBD9, 153), version);
    let account_width = (w * 0.35).clamp(100., 178.);
    let account_x = w - account_width - 20.;
    let account_name = ui.fit_text(Face::Regular, 11., account, account_width - 14.);
    ui.text(Face::Regular, 11., account_x, 20.,
        color::hex(0xDDE5F2, 214), &account_name);
    if signed_in {
        ui.rounded(account_x - 11., 25., 5., 5., 2.5,
            color::hex(0xB5DDC8, 226));
    }
    target([account_x - 13., 9., account_width + 13., 45.], ShellAction::Account);
    // Access theme settings without a second app-like nav bar or chrome control.
    target([21., 10., 105., 49.], ShellAction::Appearance);

    let chosen = selected.min(categories.len().saturating_sub(1));
    for (index, (label, _legacy_icon)) in categories.iter().enumerate() {
        let Some(hit) = cross.category_hit(w, chosen, index) else { continue; };
        let current = chosen == index;
        let ink = color::hex(0xEEF3FF, if current { 255 } else { 143 });
        let glyph_size = if current { 33. } else { 25. };
        let center_x = hit[0] + hit[2] * 0.5;
        symbol(ui, symbol_for_category(index), center_x,
            cross.category_y, glyph_size, ink);
        if current {
            let name = match index {
                0 => "Jogo", 1 => "Mods", 2 => "Packs", 3 => "Shaders",
                4 => "Arquivos", 5 => "Sistema", _ => "Skins",
            };
            let name = if categories.len() == 7 { name } else { label };
            let size = 13.;
            let tw = ui.measure(Face::Regular, size, name);
            ui.text(Face::Regular, size, center_x - tw * 0.5,
                cross.category_y + 26., color::hex(0xF0F4FF, 232), name);
            // Light comes from under the icon; no tab background, no underline.
            ui.rounded(center_x - 2., cross.category_y + 13., 4., 4., 2.,
                color::hex(0xDCEAFF, 74));
        }
        target(hit, ShellAction::Page(index));
    }
    layout
}

/// Virtualized vertical item column. The focused item occupies a stable
/// crosshair coordinate even when navigating a long list of Home actions.
pub fn home(
    ui: &mut Ui<'_>,
    bounds: [f32; 4],
    view: &HomeView<'_>,
    selected: usize,
    mut target: impl FnMut([f32; 4], HomeAction),
) {
    let c = Cross::from_screen(ui.w, ui.h);
    let selected = selected.min(HOME_ACTIONS.len() - 1);
    let status = if view.launching { "INICIANDO" } else if view.installed {
        "PRONTO PARA JOGAR"
    } else {
        "INSTALAÇÃO NECESSÁRIA"
    };
    // Quiet context floats in empty space; no tile, hero, border, or card.
    if !c.portrait {
        let context_x = (ui.w * 0.71).min(ui.w - 133.);
        ui.text(Face::Regular, 11., context_x, c.item_y + 9.,
            color::hex(0xA9BBD6, 190), "MINECRAFT");
        ui.text(Face::Regular, 15., context_x, c.item_y + 32.,
            color::hex(0xEDF2FE, 220), "JAVA 26.3");
        ui.text(Face::Regular, 10., context_x, c.item_y + 60.,
            color::hex(0xB3D8CA, 176), status);
    } else {
        let status = ui.fit_text(Face::Regular, 10.5, status, bounds[2] - 12.);
        ui.text(Face::Regular, 10.5, bounds[0] + 5., c.bottom - 29.,
            color::hex(0xB4C8E4, 188), &status);
    }

    for (i, action) in HOME_ACTIONS.into_iter().enumerate() {
        let Some(r) = c.item_hit(bounds, selected, i) else { continue; };
        let active = i == selected;
        let glyph_x = r[0] + 27.;
        let ink = color::hex(0xF5F7FF, if active { 248 } else { 132 });
        symbol(ui, symbol_for_home(action), glyph_x,
            r[1] + 23., if active { 27. } else { 20. }, ink);
        let size = if active { 22. } else { 16. };
        let label = if i == 0 && !view.installed {
            "Instalar Minecraft"
        } else if i == 0 && view.launching {
            "Iniciando Minecraft"
        } else {
            HOME_LABELS[i]
        };
        let fitted = ui.fit_text(Face::Regular, size,
            label, (r[2] - 69.).max(18.));
        ui.text(Face::Regular, size, r[0] + 57.,
            r[1] + if active { 9. } else { 12. }, ink, &fitted);
        target(r, action);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_focus_moves_without_clipping_or_overlapping_touch_targets() {
        for (w, h) in [(340., 640.), (410., 780.), (760., 410.), (900., 480.),
            (1600., 900.)] {
            let cross = Cross::from_screen(w, h);
            for selected in 0..7 {
                let rects: Vec<_> = (0..7)
                    .filter_map(|i| cross.category_hit(w, selected, i)).collect();
                assert!(!rects.is_empty());
                assert!(rects.iter().all(|r| r[2] >= 48. && r[3] >= 48.));
                assert!(rects.iter().all(|r| r[0] >= 8. && r[0] + r[2] <= w - 8.));
                assert!(rects.windows(2).all(|p| p[0][0] + p[0][2] < p[1][0]));
            }
        }
    }

    #[test]
    fn every_home_action_becomes_focusable_even_on_short_landscape() {
        for (w, h) in [(340., 640.), (760., 410.), (1600., 900.)] {
            let c = Cross::from_screen(w, h);
            let bounds = [20., 76., w - 40., h - 132.];
            for selected in 0..HOME_ACTIONS.len() {
                assert!(c.item_hit(bounds, selected, selected).is_some(),
                    "{w}x{h}, item {selected}");
            }
        }
    }

    #[test]
    fn focus_is_clamped_and_actions_remain_stable() {
        let mut model = Navigation::default();
        assert_eq!(model.step_home(-200), HomeAction::Play);
        assert_eq!(model.step_home(200), HomeAction::Settings);
        model.focus_home(HomeAction::Shaders);
        assert_eq!(model.selected_home(), 4);
        assert_eq!(model.selected_action(), HomeAction::Shaders);
    }
}
