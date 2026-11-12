//! Android launcher materials and shared, captureable native page components.
//! Wallpaper art is original and cached. Liquid Glass uses GPU backdrop lenses;
//! Dark Aero retains its cached wallpaper material. Neither reads pixels to the CPU.
use super::{ExperiencePreset, Face, Icon, LauncherThemeConfig, ThemePreset, Ui, active_theme, color};
use std::{cell::RefCell, collections::HashMap};

pub fn default_effects() -> bool {
    true
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct GlassTuning {
    pub opacity: f32,
    pub blur: f32,
    pub refraction: f32,
}
impl Default for GlassTuning {
    fn default() -> Self {
        Self {
            opacity: 0.08,
            blur: 2.,
            refraction: 14.,
        }
    }
}
impl GlassTuning {
    pub fn normalized(self) -> Self {
        let def = Self::default();
        let safe = |v: f32, fallback, min, max| {
            if v.is_finite() {
                v.clamp(min, max)
            } else {
                fallback
            }
        };
        Self {
            opacity: safe(self.opacity, def.opacity, 0.06, 0.5),
            blur: safe(self.blur, def.blur, 0., 10.),
            refraction: safe(self.refraction, def.refraction, 0., 24.),
        }
    }
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum Wallpaper {
    #[default]
    None,
    Aurora,
    Graphite,
    Midnight,
}
impl Wallpaper {
    pub const ALL: [Self; 4] = [Self::None, Self::Aurora, Self::Graphite, Self::Midnight];
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "Plain",
            Self::Aurora => "Aurora",
            Self::Graphite => "Graphite",
            Self::Midnight => "Midnight",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Material {
    #[default]
    Opaque,
    Glass,
    Aero,
}

const TEX_W: u32 = 512;
const TEX_H: u32 = 288;
type PaperCache = HashMap<(Wallpaper, bool), Vec<u8>>;
thread_local! { static PAPERS: RefCell<PaperCache> = RefCell::new(HashMap::new()); }

fn paper_pixel(paper: Wallpaper, x: f32, y: f32) -> [f32; 3] {
    let mut c = match paper {
        Wallpaper::Aurora => [9.0, 12.0, 26.0],
        Wallpaper::Midnight => [6.0, 16.0, 27.0],
        _ => [3.0, 4.0, 6.0],
    };
    // Broad folded ribbons. Narrow edge light belongs to the wallpaper, never text.
    for (i, (cx, cy, rx, ry)) in [
        (-0.15, 0.05, 0.95, 0.78),
        (0.87, -0.37, 0.70, 1.20),
        (1.14, 1.46, 1.20, 0.92),
    ]
    .into_iter()
    .enumerate()
    {
        let d = (((x - cx) / rx).powi(2) + ((y - cy) / ry).powi(2)).sqrt();
        let edge = (-((d - 1.0) / 0.009).powi(2)).exp();
        let broad = (-((d - 0.94) / 0.15).powi(2)).exp();
        let reflection = (0.25 + 0.75 * (x * 2.5 + y * 1.6 + i as f32).sin().abs()).powi(3);
        let tint = match paper {
            Wallpaper::Aurora => {
                if i == 1 {
                    [0.52, 0.55, 1.0]
                } else {
                    [0.70, 0.46, 1.0]
                }
            }
            Wallpaper::Midnight => [0.20, 0.65, 0.88],
            _ => [0.86, 0.90, 0.97],
        };
        // Liquid Glass should not look like a glowing screensaver behind the lenses.
        // Old School / Dark Aero keep their existing wallpaper highlights.
        let (edge_energy, broad_energy) = if paper == Wallpaper::Aurora {
            (43.0, 12.0)
        } else {
            (112.0, 37.0)
        };
        let light = edge_energy * edge * reflection + broad_energy * broad * reflection;
        for k in 0..3 {
            c[k] += tint[k] * light;
        }
    }
    c
}

fn wallpaper_pixels(paper: Wallpaper, blur: bool) -> Vec<u8> {
    let mut pixels = Vec::with_capacity((TEX_W * TEX_H * 4) as usize);
    for y in 0..TEX_H {
        for x in 0..TEX_W {
            let mut c = [0.0; 3];
            let samples: &[(f32, f32)] = if blur {
                &[
                    (0., 0.),
                    (-0.018, 0.),
                    (0.018, 0.),
                    (0., -0.032),
                    (0., 0.032),
                ]
            } else {
                &[(0., 0.)]
            };
            for (dx, dy) in samples {
                let p = paper_pixel(
                    paper,
                    x as f32 / TEX_W as f32 + dx,
                    y as f32 / TEX_H as f32 + dy,
                );
                for k in 0..3 {
                    c[k] += p[k] / samples.len() as f32;
                }
            }
            pixels.extend(c.map(|n| n.clamp(0., 255.) as u8));
            pixels.push(255);
        }
    }
    pixels
}

fn texture(ui: &mut Ui<'_>, paper: Wallpaper, blur: bool) -> Option<super::Rect> {
    PAPERS.with(|papers| {
        let mut papers = papers.borrow_mut();
        let pixels = papers
            .entry((paper, blur))
            .or_insert_with(|| wallpaper_pixels(paper, blur));
        let key = format!("launcher-wallpaper:{paper:?}:{blur}");
        super::with_cache(ui.atlas, |cache| {
            cache.image(ui.atlas, &key, TEX_W, TEX_H, pixels)
        })
    })
}

pub fn background(ui: &mut Ui<'_>, r: [f32; 4], paper: Wallpaper) {
    if let Some(t) = texture(ui, paper, false) {
        let [x, y, w, h] = r;
        ui.emit_clipped_uv(
            ui.px_x(x),
            ui.px_y(y),
            ui.px_x(x + w),
            ui.px_y(y + h),
            t.u0,
            t.v0,
            t.u1,
            t.v1,
            u32::MAX,
        );
    } else {
        ui.fill(r[0], r[1], r[2], r[3], active_theme().bg_solid);
    }
}

fn preview(ui: &mut Ui<'_>, r: [f32; 4], paper: Wallpaper, radius: f32) {
    let Some(t) = texture(ui, paper, false) else {
        return;
    };
    let [x, y, w, h] = r;
    let mut band = |top: f32, height: f32, inset: f32| {
        let u0 = t.u0 + (t.u1 - t.u0) * inset / w;
        let u1 = t.u1 - (t.u1 - t.u0) * inset / w;
        let v0 = t.v0 + (t.v1 - t.v0) * (top - y) / h;
        let v1 = t.v0 + (t.v1 - t.v0) * (top + height - y) / h;
        ui.emit_clipped_uv(
            ui.px_x(x + inset),
            ui.px_y(top),
            ui.px_x(x + w - inset),
            ui.px_y(top + height),
            u0,
            v0,
            u1,
            v1,
            u32::MAX,
        );
    };
    for i in 0..24 {
        let dh = radius / 24.;
        let d = radius - (i as f32 + 0.5) * dh;
        let inset = radius - (radius * radius - d * d).sqrt();
        band(y + i as f32 * dh, dh, inset);
        band(y + h - (i as f32 + 1.) * dh, dh, inset);
    }
    band(y + radius, h - 2. * radius, 0.);
}

/// Frosted wallpaper sampled in screen space, followed by a translucent tint
/// and a restrained edge highlight. This is a wallpaper material, not iOS's compositor.
fn sheen(ui: &mut Ui<'_>, r: [f32; 4], radius: f32, aero: bool) {
    let key = if aero {
        "launcher-material:aero"
    } else {
        "launcher-material:glass"
    };
    let tex = super::with_cache(ui.atlas, |cache| {
        if let Some(rect) = cache.images.get(key) {
            return Some(*rect);
        }
        let mut pixels = Vec::with_capacity(64 * 128 * 4);
        for y in 0..128 {
            for x in 0..64 {
                let dx = (x as f32 + 0.5 - 32.).abs() - 16.;
                let dy = (y as f32 + 0.5 - 64.).abs() - 48.;
                let coverage = (16.5 - dx.max(0.).hypot(dy.max(0.))).clamp(0., 1.);
                let top = (-(y as f32) / 21.).exp() * if aero { 86. } else { 54. };
                let bottom = if aero {
                    (-((127. - y as f32) / 12.).powi(2)).exp() * 20.
                } else {
                    0.
                };
                pixels.extend([255, 255, 255, ((top + bottom) * coverage) as u8]);
            }
        }
        cache.image(ui.atlas, key, 64, 128, &pixels)
    });
    let Some(t) = tex else {
        return;
    };
    let [x, y, w, h] = r;
    let xs = [x, x + radius, x + w - radius, x + w];
    let ys = [y, y + radius, y + h - radius, y + h];
    let us = [
        t.u0,
        t.u0 + (t.u1 - t.u0) * 0.25,
        t.u0 + (t.u1 - t.u0) * 0.75,
        t.u1,
    ];
    let vs = [
        t.v0,
        t.v0 + (t.v1 - t.v0) * 0.125,
        t.v0 + (t.v1 - t.v0) * 0.875,
        t.v1,
    ];
    for row in 0..3 {
        for col in 0..3 {
            ui.emit_clipped_uv(
                ui.px_x(xs[col]),
                ui.px_y(ys[row]),
                ui.px_x(xs[col + 1]),
                ui.px_y(ys[row + 1]),
                us[col],
                vs[row],
                us[col + 1],
                vs[row + 1],
                u32::MAX,
            );
        }
    }
}

pub fn panel(ui: &mut Ui<'_>, r: [f32; 4], radius: f32, emphasized: bool) {
    let th = active_theme();
    let [x, y, w, h] = r;
    let radius = radius.min(w / 2.).min(h / 2.).max(0.);
    if th.material == Material::Opaque {
        ui.rounded(
            x,
            y,
            w,
            h,
            radius,
            if emphasized { th.accent } else { th.surface },
        );
        return;
    }
    if th.material == Material::Glass {
        let optics = th.glass.normalized();
        // Physical optical hierarchy: interactive pills / buttons have visibly
        // curved glass, while spacious cards stay clear rather than foggy blue.
        let compact = h <= 100. || w <= 190.;
        let tint_opacity = if emphasized {
            optics.opacity * 0.54
        } else if compact {
            optics.opacity * 0.36
        } else {
            optics.opacity * 0.16
        };
        let tint = color::hex(
            if emphasized { 0xCFE4F4 } else { 0xDFECF7 },
            (tint_opacity * 255.) as u8,
        );
        let blur = optics.blur * (if compact { 0.72 } else { 0.34 });
        let refraction = (optics.refraction * (if compact { 1.24 } else { 0.62 }))
            .min(24.);
        ui.backdrop_lens(r, radius, tint, [blur, refraction]);
        return;
    }
    for i in (1..=3).rev() {
        let d = i as f32 * 1.3;
        ui.rounded(
            x - d,
            y - d + 2.,
            w + 2. * d,
            h + 2. * d,
            radius + d,
            color::hex(0, 12),
        );
    }
    if th.wallpaper != Wallpaper::None
        && let Some(t) = texture(ui, th.wallpaper, true)
    {
        // Only rounded top/bottom bands need subdivisions. The middle is one quad.
        let mut band = |top: f32, height: f32, inset: f32| {
            let left = x + inset;
            let right = x + w - inset;
            let u0 = t.u0 + (t.u1 - t.u0) * (left / ui.w).clamp(0., 1.);
            let u1 = t.u0 + (t.u1 - t.u0) * (right / ui.w).clamp(0., 1.);
            let v0 = t.v0 + (t.v1 - t.v0) * (top / ui.h).clamp(0., 1.);
            let v1 = t.v0 + (t.v1 - t.v0) * ((top + height) / ui.h).clamp(0., 1.);
            ui.emit_clipped_uv(
                ui.px_x(left),
                ui.px_y(top),
                ui.px_x(right),
                ui.px_y(top + height),
                u0,
                v0,
                u1,
                v1,
                u32::MAX,
            );
        };
        let n = radius.ceil().min(18.) as u32;
        if n > 0 {
            for j in 0..n {
                let dh = radius / n as f32;
                let d = radius - (j as f32 + 0.5) * dh;
                let inset = radius - (radius * radius - d * d).max(0.).sqrt();
                band(y + j as f32 * dh, dh, inset);
                band(y + h - (j as f32 + 1.) * dh, dh, inset);
            }
        }
        band(y + radius, (h - 2. * radius).max(0.), 0.);
    }
    let aero = th.material == Material::Aero;
    let tint = if emphasized && !aero {
        color::hex(0x7358CF, 174)
    } else if aero {
        color::hex(0x101216, 160)
    } else {
        color::hex(0x33394B, 142)
    };
    ui.rounded(x, y, w, h, radius, tint);
    sheen(ui, r, radius, aero);
    ui.stroke_rounded(
        x,
        y,
        w,
        h,
        radius,
        0.8,
        color::hex(0xDAE2F5, if aero { 105 } else { 83 }),
    );
    ui.fill(
        x + radius,
        y + 0.7,
        (w - 2. * radius).max(0.),
        0.65,
        color::hex(0xFFFFFF, 96),
    );
}

pub fn button(ui: &mut Ui<'_>, rect: [f32; 4], label: &str, icon: Option<Icon>, accent: bool) {
    // A true semicircular cap, not an old-style frosted rounded rectangle.
    panel(ui, rect, rect[3] * 0.5, accent);
    let th = active_theme();
    let fg = if accent && th.material == Material::Opaque {
        th.on_accent
    } else {
        th.text_primary
    };
    let icon_w = if icon.is_some() { 26. } else { 0. };
    let text = ui.fit_text(Face::Medium, 14., label, rect[2] - 24. - icon_w);
    let tw = ui.measure(Face::Medium, 14., &text);
    let left = rect[0] + (rect[2] - tw - icon_w) * 0.5;
    if let Some(icon) = icon {
        ui.icon(icon, left, rect[1] + (rect[3] - 19.) * 0.5, 19., fg);
    }
    ui.text(
        Face::Medium,
        14.,
        left + icon_w,
        rect[1] + (rect[3] - 20.) * 0.5,
        fg,
        &text,
    );
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HomeAction {
    Play,
    Profile,
    Appearance,
    Mods,
    Packs,
    Shaders,
    Files,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShellAction {
    Page(usize),
    Account,
    Appearance,
}

#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub portrait: bool,
    /// Console-style channel rail, selected by the Old School experience.
    pub console: bool,
    pub x: f32,
    pub width: f32,
    pub bottom: f32,
}
impl Layout {
    pub fn new(w: f32, h: f32) -> Self {
        let portrait = h > w || w < 620.;
        let console = active_theme().experience == ExperiencePreset::OldSchool;
        let x = if portrait { 16. } else if console { 230. } else { 210. };
        Self {
            portrait,
            console,
            x,
            width: (w - x - 20.).max(200.),
            bottom: if portrait { h - 84. } else { h },
        }
    }
    pub fn nav_rect(self, w: f32, h: f32, index: usize, count: usize) -> [f32; 4] {
        if self.portrait {
            let step = (w - 32.) / count as f32;
            [16. + index as f32 * step, h - 72., step, 56.]
        } else if self.console {
            // Wide metallic channels, independent of the Liquid Glass bubble rail.
            let step = ((h - 125.) / count as f32).clamp(33., 48.);
            [20., 105. + index as f32 * step, 188., (step - 3.).min(44.)]
        } else {
            [
                20.,
                76. + index as f32 * ((h - 88.) / count as f32).min(50.),
                164.,
                44.,
            ]
        }
    }
}

/// Header and adaptive navigation shared by the app and the native captures.
#[allow(clippy::too_many_arguments)]
pub fn shell(
    ui: &mut Ui<'_>,
    items: &[(&str, Icon)],
    selected: usize,
    account: &str,
    signed_in: bool,
    version: &str,
    mut target: impl FnMut([f32; 4], ShellAction),
) -> Layout {
    let th = active_theme();
    let (w, h) = (ui.w, ui.h);
    let layout = Layout::new(w, h);
    ui.horizon_logo(18., 17., 31.);
    ui.text(Face::SemiBold, 17., 58., 13., th.text_primary, "Horizon");
    ui.text(Face::Regular, 10.5, 58., 37., th.text_muted, version);
    let aw = if layout.portrait { 118. } else { 166. };
    let account_r = [w - 20. - aw, 12., aw, 44.];
    panel(ui, account_r, 22., false);
    ui.avatar_head_2d(account_r[0] + 6., 20., 28., account);
    if signed_in {
        ui.rounded(account_r[0] + 26., 40., 6., 6., 3., th.success);
    }
    let name = ui.fit_text(Face::Medium, 12., account, aw - 59.);
    ui.text(
        Face::Medium,
        12.,
        account_r[0] + 40.,
        25.,
        th.text_primary,
        &name,
    );
    ui.icon(Icon::ChevronDown, w - 40., 28., 12., th.text_muted);
    target(account_r, ShellAction::Account);
    let gear = [account_r[0] - 54., 12., 44., 44.];
    panel(ui, gear, 22., false);
    ui.icon(Icon::Gear, gear[0] + 12., 24., 20., th.text_primary);
    target(gear, ShellAction::Appearance);
    // The navigation rail is no longer one giant frosted sheet for Liquid
    // Glass. Each selected icon is an individual physical lens instead.
    if th.material != Material::Glass {
        if layout.portrait {
            panel(ui, [12., h - 78., w - 24., 68.], if layout.console { 12. } else { 23. }, false);
        } else if layout.console {
            panel(ui, [12., 72., 204., (h - 84.).max(10.)], 13., false);
            ui.text(Face::SemiBold, 10., 30., 83., th.text_muted, "SYSTEM CHANNELS");
            ui.fill(28., 101., 171., 1., color::hex(0xE1ECFA, 75));
        } else {
            panel(ui, [12., 72., 180., (h - 84.).max(10.)], 24., false);
        }
    }
    for (i, (label, icon)) in items.iter().enumerate() {
        let r = layout.nav_rect(w, h, i, items.len());
        let chosen = i == selected;
        if chosen {
            if th.material == Material::Glass {
                let bubble = if layout.portrait {
                    [r[0] + (r[2] - 36.) * 0.5, r[1] + 2., 36., 36.]
                } else {
                    [r[0] + 7., r[1] + 5., 38., 38.]
                };
                panel(ui, bubble, bubble[3] * 0.5, true);
            } else {
                panel(
                    ui,
                    [r[0] + 2., r[1] + 2., r[2] - 4., r[3] - 4.],
                    if layout.console { 10. } else if layout.portrait { 17. } else { 21. },
                    true,
                );
                if layout.console && !layout.portrait {
                    ui.fill(r[0] + 4., r[1] + 7., 2., (r[3] - 14.).max(3.), color::hex(0xF7F9FF, 195));
                }
            }
        }
        let fg = if chosen {
            th.text_primary
        } else {
            th.text_muted
        };
        if layout.portrait {
            ui.icon(
                *icon,
                r[0] + (r[2] - 20.) / 2.,
                r[1] + if th.material == Material::Glass { 10. } else { 7. },
                20.,
                fg,
            );
            let text = ui.fit_text(Face::Medium, 9., label, r[2] - 4.);
            ui.centered_fit(
                Face::Medium,
                9.,
                [r[0] + 2., r[1] + if th.material == Material::Glass { 40. } else { 32. }, r[2] - 4., 16.],
                fg,
                &text,
            );
        } else {
            ui.icon(
                *icon,
                r[0] + if th.material == Material::Glass { 16. } else { 16. },
                r[1] + if th.material == Material::Glass { 14. } else { 13. },
                20.,
                fg,
            );
            ui.text(
                Face::Medium,
                13.,
                r[0] + if th.material == Material::Glass { 55. } else { 47. },
                r[1] + 13.,
                fg,
                label,
            );
        }
        target(r, ShellAction::Page(i));
    }
    layout
}

pub struct HomeView<'a> {
    pub installed: bool,
    pub launching: bool,
    pub profile: u32,
    pub renderer: &'a str,
}


/// Old School has its own functional composition, not merely different colors.
/// A console-like two-panel dashboard keeps the same actions/profile data while
/// giving focus to the selected installation and a separate systems channel.
fn home_old_school(
    ui: &mut Ui<'_>,
    rect: [f32; 4],
    view: &HomeView<'_>,
    mut target: impl FnMut([f32; 4], HomeAction),
) {
    let [x, y, w, h] = rect;
    let th = active_theme();
    let narrow = w < 430.;
    let stage_h = if narrow { 198. } else { (h - 165.).clamp(150., 210.) };
    let side_w = if narrow { 0. } else { (w * 0.31).clamp(132., 232.) };
    let main_w = if narrow { w } else { w - side_w - 12. };
    panel(ui, [x, y, main_w, stage_h], 18., false);
    ui.fill(x + 19., y + 13., (main_w - 38.).max(0.), 1., color::hex(0xD6E4F4, 76));
    ui.text(Face::Medium, 11., x + 20., y + 20., th.text_muted, "CHANNEL 01  /  GAME");
    ui.text(
        Face::SemiBold,
        if narrow { 24. } else { 28. },
        x + 20.,
        y + 48.,
        th.text_primary,
        "Minecraft 26.3",
    );
    let status = if view.installed { "READY TO LAUNCH" } else { "INSTALLATION REQUIRED" };
    ui.text(Face::Medium, 11., x + 21., y + 91., th.text_muted, status);
    let play = [x + 18., y + stage_h - 65., main_w - 36., 49.];
    button(ui, play, if view.launching { "Getting ready..." } else if view.installed { "Start game" } else { "Install Minecraft" }, Some(Icon::Play), true);
    if !view.launching { target(play, HomeAction::Play); }

    if !narrow {
        let side_x = x + main_w + 12.;
        panel(ui, [side_x, y, side_w, stage_h], 13., false);
        ui.text(Face::Medium, 10.5, side_x + 15., y + 20., th.text_muted, "SYSTEM / PROFILE");
        ui.text(Face::Medium, 13., side_x + 15., y + 48., th.text_primary, &format!("Slot {:02}", view.profile + 1));
        let mode = ui.fit_text(Face::Regular, 11., view.renderer, side_w - 29.);
        ui.text(Face::Regular, 11., side_x + 15., y + 79., th.text_muted, &mode);
        let profile = [side_x + 12., y + stage_h - 57., side_w - 24., 42.];
        button(ui, profile, &format!("Profile {}", view.profile + 1), Some(Icon::ChevronDown), false);
        target(profile, HomeAction::Profile);
    }

    let row_y = y + stage_h + 13.;
    let cols = if narrow { 2 } else { 4 };
    let gap = 9.;
    let card_w = (w - gap * (cols - 1) as f32) / cols as f32;
    for (i, (label, icon, action)) in [
        ("Mods", Icon::Mods, HomeAction::Mods),
        ("Packs", Icon::Packs, HomeAction::Packs),
        ("Shaders", Icon::Shaders, HomeAction::Shaders),
        ("My files", Icon::Folder, HomeAction::Files),
    ].into_iter().enumerate() {
        let r = [x + (i % cols) as f32 * (card_w + gap), row_y + (i / cols) as f32 * 65., card_w, 57.];
        if r[1] + r[3] > y + h { continue; }
        panel(ui, r, 12., false);
        ui.icon(icon, r[0] + 12., r[1] + 17., 20., th.text_primary);
        let caption = ui.fit_text(Face::Medium, 12., label, (r[2] - 50.).max(25.));
        ui.text(Face::Medium, 12., r[0] + 42., r[1] + 18., th.text_primary, &caption);
        target(r, action);
    }
    if narrow {
        let profile = [x, row_y + 135., (w - 9.) / 2., 42.];
        if profile[1] + profile[3] < y + h {
            button(ui, profile, &format!("Profile {}", view.profile + 1), Some(Icon::ChevronDown), false);
            target(profile, HomeAction::Profile);
        }
    }
}

/// Used directly by the Android launcher and by the native capture harness.
pub fn home(
    ui: &mut Ui<'_>,
    rect: [f32; 4],
    view: &HomeView<'_>,
    mut target: impl FnMut([f32; 4], HomeAction),
) {
    let [x, y, w, h] = rect;
    let th = active_theme();
    if th.experience == ExperiencePreset::OldSchool {
        return home_old_school(ui, rect, view, target);
    }
    let portrait = w < 430.;
    let hero_h = if portrait {
        208.
    } else {
        (h - 174.).clamp(if w < 520. { 190. } else { 146. }, 204.)
    };
    // Realistic glass does not cover the entire page in a blue tinted sheet.
    // Content floats directly above the wallpaper. A compact status lens
    // anchors the section instead of a giant frosted content rectangle.
    if th.material != Material::Glass {
        panel(ui, [x, y, w, hero_h], 24., false);
    } else {
        let status_width = (ui.measure(Face::Regular, 12., view.renderer) + 49.)
            .clamp(82., (w - 34.).max(82.));
        let status_x = if portrait || w < 520. {
            x + 17.
        } else {
            // Keep renderer metadata in its own column on short landscape screens:
            // the previous x+17 placement overlapped "Minecraft 26.3 / Profile".
            x + w - status_width - 16.
        };
        panel(ui, [status_x, y + hero_h - 47., status_width, 37.], 18.5, false);
    }
    ui.text(
        Face::Medium,
        11.,
        x + 24.,
        y + 20.,
        th.text_muted,
        "YOUR SPACE TO PLAY",
    );
    ui.text(
        Face::SemiBold,
        if portrait { 29. } else { 31. },
        x + 24.,
        y + 44.,
        th.text_primary,
        if view.installed {
            "Ready to play."
        } else {
            "Your Java worlds."
        },
    );
    let subtitle = format!("Minecraft 26.3  ·  Profile {}", view.profile + 1);
    ui.text(
        Face::Regular,
        14.,
        x + 24.,
        y + 91.,
        th.text_muted,
        &subtitle,
    );
    let mode = ui.fit_text(Face::Regular, 12., view.renderer, w - 88.);
    let mode_width = ui.measure(Face::Regular, 12., &mode) + 49.;
    let mode_x = if th.material == Material::Glass && !portrait && w >= 520. {
        x + w - mode_width - 16.
    } else {
        x + 17.
    };
    ui.text(
        Face::Regular,
        12.,
        mode_x + 21.,
        y + hero_h - 38.,
        th.text_muted,
        &mode,
    );
    ui.rounded(mode_x + 9., y + hero_h - 30., 5., 5., 2.5, th.success);
    let customize = [x + w - 62., y + 16., 44., 44.];
    panel(ui, customize, 22., false);
    ui.icon(
        Icon::Gear,
        customize[0] + 12.,
        customize[1] + 12.,
        20.,
        th.text_primary,
    );
    target(customize, HomeAction::Appearance);
    let by = y + hero_h + 16.;
    let profile_w = if portrait { 108. } else { 136. };
    let play = [x, by, w - profile_w - 12., 54.];
    button(
        ui,
        play,
        if view.launching {
            "Getting ready…"
        } else if view.installed {
            "Play Minecraft"
        } else {
            "Install Minecraft"
        },
        Some(Icon::Play),
        true,
    );
    if !view.launching {
        target(play, HomeAction::Play);
    }
    let profile = [x + w - profile_w, by, profile_w, 54.];
    button(
        ui,
        profile,
        &format!("Profile {}", view.profile + 1),
        Some(Icon::ChevronDown),
        false,
    );
    target(profile, HomeAction::Profile);
    let qy = by + 72.;
    let cols = if portrait { 2 } else { 4 };
    let gap = 10.;
    let qw = (w - gap * (cols - 1) as f32) / cols as f32;
    for (i, (label, icon, action)) in [
        ("Mods", Icon::Mods, HomeAction::Mods),
        ("Packs", Icon::Packs, HomeAction::Packs),
        ("Shaders", Icon::Shaders, HomeAction::Shaders),
        ("My files", Icon::Folder, HomeAction::Files),
    ]
    .into_iter()
    .enumerate()
    {
        let r = [
            x + (i % cols) as f32 * (qw + gap),
            qy + (i / cols) as f32 * 76.,
            qw,
            66.,
        ];
        if r[1] + r[3] > y + h {
            continue;
        }
        if th.material == Material::Glass {
            // Each shortcut is a tangible lens. The label remains crisp
            // outside the optical surface, so glass never blurs the text.
            let bubble = [r[0] + 11., r[1] + 11., 44., 44.];
            panel(ui, bubble, 22., false);
            ui.icon(icon, bubble[0] + 13., bubble[1] + 13., 18., th.text_primary);
            let label = ui.fit_text(Face::Medium, 12., label, (r[2] - 74.).max(24.));
            ui.text(
                Face::Medium,
                12.,
                r[0] + 65.,
                r[1] + 21.,
                th.text_primary,
                &label,
            );
        } else {
            panel(ui, r, 17., false);
            ui.icon(icon, r[0] + 14., r[1] + 12., 18., th.text_muted);
            ui.text(
                Face::Medium,
                12.,
                r[0] + 14.,
                r[1] + 36.,
                th.text_primary,
                label,
            );
        }
        target(r, action);
    }
    if portrait && qy + 172. < y + h {
        let r = [x, qy + 164., w, 52.];
        button(ui, r, "Personalize launcher", Some(Icon::Gear), false);
        target(r, HomeAction::Appearance);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AppearanceAction {
    Preset(ThemePreset),
    Wallpaper(Wallpaper),
    Effects,
    Transparency,
    Blur,
    Refraction,
    Sounds,
    Motion,
    Advanced,
    Back,
}

#[allow(clippy::too_many_arguments)]
pub fn appearance_screen(
    ui: &mut Ui<'_>,
    x: f32,
    w: f32,
    bottom: f32,
    slide: f32,
    cfg: &LauncherThemeConfig,
    reduced: bool,
    scroll: f32,
    mut target: impl FnMut([f32; 4], AppearanceAction),
) -> [f32; 3] {
    let back = [x, 72. + slide, 96., 44.];
    button(ui, back, "Back", Some(Icon::ArrowLeft), false);
    target(back, AppearanceAction::Back);
    ui.text(
        Face::SemiBold,
        23.,
        x + 112.,
        78. + slide,
        active_theme().text_primary,
        "Appearance",
    );
    let top = 132. + slide;
    let end = bottom - 28.;
    let max = appearance(
        ui,
        [x, top, w, (end - top).max(1.)],
        cfg,
        reduced,
        scroll,
        target,
    );
    [top, end, max]
}

/// A compact mobile appearance page; advanced colour editing stays one tap away.
pub fn appearance(
    ui: &mut Ui<'_>,
    rect: [f32; 4],
    cfg: &LauncherThemeConfig,
    reduced: bool,
    scroll: f32,
    mut target: impl FnMut([f32; 4], AppearanceAction),
) -> f32 {
    let [x, top, w, height] = rect;
    let y = top - scroll;
    let th = active_theme();
    let narrow = w < 430.;
    let card_w = (w - 12.) / 2.;
    ui.set_clip(Some(rect));
    ui.text(
        Face::Regular,
        12.,
        x,
        y,
        th.text_muted,
        "A little more you. Only your launcher.",
    );
    for (i, preset) in [ThemePreset::LiquidGlass, ThemePreset::OldSchool]
        .into_iter()
        .enumerate()
    {
        let r = [x + i as f32 * (card_w + 12.), y + 30., card_w, 116.];
        let paper = if i == 0 {
            Wallpaper::Aurora
        } else {
            Wallpaper::Graphite
        };
        preview(ui, r, paper, 18.);
        ui.rounded(r[0], r[1], r[2], r[3], 18., color::hex(0x12151E, 95));
        if preset == ThemePreset::LiquidGlass {
            panel(ui, [r[0] + 14., r[1] + 14., 32., 32.], 16., false);
            panel(ui, [r[0] + 56., r[1] + 17., card_w - 70., 27.], 13.5, false);
        } else {
            // Console selection preview uses stacked bars and chrome channels.
            for band in 0..3 {
                let yy = r[1] + 14. + band as f32 * 13.;
                ui.rounded(r[0] + 14., yy, card_w - 28., 9., 3., color::hex(0xC6D7E8, if band == 1 { 95 } else { 30 }));
            }
        }
        ui.stroke_rounded(
            r[0],
            r[1],
            r[2],
            r[3],
            18.,
            if cfg.preset == preset { 1.8 } else { 0.7 },
            if cfg.preset == preset {
                th.accent
            } else {
                th.border
            },
        );
        ui.text(
            Face::Medium,
            14.,
            r[0] + 14.,
            r[1] + 60.,
            th.text_primary,
            preset.label(),
        );
        ui.text(
            Face::Regular,
            10.5,
            r[0] + 14.,
            r[1] + 84.,
            th.text_muted,
            if cfg.preset == preset {
                "Selected"
            } else if preset == ThemePreset::LiquidGlass {
                "Fluid lenses / Calm"
            } else {
                "Chrome / Console"
            },
        );
        if let Some(r) = ui.clip_rect(r) {
            target(r, AppearanceAction::Preset(preset));
        }
    }
    ui.text(Face::Medium, 13., x, y + 166., th.text_primary, "Wallpaper");
    let ww = (w - 24.) / 4.;
    for (i, paper) in Wallpaper::ALL.into_iter().enumerate() {
        let r = [x + i as f32 * (ww + 8.), y + 191., ww, 60.];
        if paper == Wallpaper::None {
            ui.rounded(r[0], r[1], r[2], r[3], 12., th.bg_solid);
        } else {
            preview(ui, r, paper, 12.);
        }

        ui.text(
            Face::Medium,
            10.5,
            r[0] + 8.,
            r[1] + 39.,
            th.text_primary,
            paper.label(),
        );
        ui.stroke_rounded(
            r[0],
            r[1],
            r[2],
            r[3],
            12.,
            if cfg.wallpaper == paper { 1.8 } else { 0.6 },
            if cfg.wallpaper == paper {
                th.accent
            } else {
                th.border
            },
        );
        if let Some(r) = ui.clip_rect(r) {
            target(r, AppearanceAction::Wallpaper(paper));
        }
    }
    let row_y = y + 270.;
    for (i, (title, subtitle, on, action)) in [
        (
            "Glass effects",
            if cfg.preset == ThemePreset::LiquidGlass {
                "Live backdrop, curved edges and reflections"
            } else {
                "Frosted wallpaper and edge reflections"
            },
            cfg.glass_effects,
            AppearanceAction::Effects,
        ),
        (
            "Interface sounds",
            "Only after a confirmed button tap",
            cfg.interface_sounds,
            AppearanceAction::Sounds,
        ),
        (
            "Animations",
            "Short transitions; no idle animation",
            !reduced,
            AppearanceAction::Motion,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let r = [x, row_y + i as f32 * 74., w, 66.];
        panel(ui, r, 17., false);
        ui.text(
            Face::Medium,
            14.,
            x + 16.,
            r[1] + 10.,
            th.text_primary,
            title,
        );
        let sub = ui.fit_text(Face::Regular, 11., subtitle, w - 98.);
        ui.text(Face::Regular, 11., x + 16., r[1] + 35., th.text_muted, &sub);
        ui.m3_switch(x + w - 62., r[1] + 18., on);
        if let Some(r) = ui.clip_rect(r) {
            target(r, action);
        }
    }
    let tuning_y = row_y + 232.;
    let tuning_height = if cfg.preset == ThemePreset::LiquidGlass && cfg.glass_effects {
        let optics = cfg.glass.normalized();
        for (i, (label, value, action)) in [
            (
                "Transparency",
                format!("{}%", ((1. - optics.opacity) * 100.).round() as u32),
                AppearanceAction::Transparency,
            ),
            (
                "Backdrop blur",
                if optics.blur < 0.1 {
                    "Off · lightest".into()
                } else if optics.blur < 6. {
                    "Soft".into()
                } else {
                    "Strong".into()
                },
                AppearanceAction::Blur,
            ),
            (
                "Refraction",
                if optics.refraction < 0.1 {
                    "Off".into()
                } else if optics.refraction < 18. {
                    "Natural".into()
                } else {
                    "Strong".into()
                },
                AppearanceAction::Refraction,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let r = [x, tuning_y + i as f32 * 58., w, 50.];
            panel(ui, r, 14., false);
            ui.text(
                Face::Medium,
                12.,
                x + 16.,
                r[1] + 15.,
                th.text_primary,
                label,
            );
            let text = ui.fit_text(Face::Regular, 11., &value, w * 0.40);
            let tw = ui.measure(Face::Regular, 11., &text);
            ui.text(
                Face::Regular,
                11.,
                x + w - 16. - tw,
                r[1] + 16.,
                th.text_primary,
                &text,
            );
            if let Some(r) = ui.clip_rect(r) {
                target(r, action);
            }
        }
        182.
    } else {
        0.
    };
    let cy = row_y + 240. + tuning_height;
    ui.text(Face::Medium, 13., x, cy, th.text_primary, "More palettes");
    let cols = if narrow { 2 } else { 4 };
    let cw = (w - 8. * (cols - 1) as f32) / cols as f32;
    for (i, preset) in ThemePreset::ALL
        .into_iter()
        .filter(|p| !matches!(p, ThemePreset::LiquidGlass | ThemePreset::DarkAero))
        .enumerate()
    {
        let r = [
            x + (i % cols) as f32 * (cw + 8.),
            cy + 26. + (i / cols) as f32 * 54.,
            cw,
            46.,
        ];
        button(ui, r, preset.label(), None, cfg.preset == preset);
        if let Some(r) = ui.clip_rect(r) {
            target(r, AppearanceAction::Preset(preset));
        }
    }
    let end = cy + 26. + ThemePreset::ALL.into_iter().filter(|p| !matches!(p, ThemePreset::LiquidGlass | ThemePreset::OldSchool)).count().div_ceil(cols) as f32 * 54.;
    let advanced = [x, end + 8., w, 48.];
    button(ui, advanced, "Advanced colors", Some(Icon::Gear), false);
    if let Some(r) = ui.clip_rect(advanced) {
        target(r, AppearanceAction::Advanced);
    }
    ui.set_clip(None);
    (advanced[1] + advanced[3] + 16. - y - height).max(0.)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hud::{HudAtlas, HudBatch};

    #[test]
    fn navigation_and_home_actions_stay_separate_on_phone_viewports() {
        for (w, h) in [(340., 640.), (410., 780.), (760., 410.), (980., 440.)] {
            let layout = Layout::new(w, h);
            let mut nav = Vec::new();
            for i in 0..7 {
                let r = layout.nav_rect(w, h, i, 7);
                assert!(
                    r[0] >= 0. && r[1] >= 0. && r[0] + r[2] <= w && r[1] + r[3] <= h,
                    "{w}x{h}: {r:?}"
                );
                assert!(r[2] >= 44. && r[3] >= 44.);
                for other in &nav {
                    let o: &[f32; 4] = other;
                    assert!(
                        r[0] >= o[0] + o[2]
                            || o[0] >= r[0] + r[2]
                            || r[1] >= o[1] + o[3]
                            || o[1] >= r[1] + r[3]
                    );
                }
                nav.push(r);
            }
            let atlas = HudAtlas::boot();
            let mut batch = HudBatch::new([w as u32, h as u32]);
            let mut ui = Ui::mobile(&atlas, &mut batch, (0., 0.));
            let layout = Layout::new(ui.w, ui.h);
            let mut actions = Vec::new();
            home(
                &mut ui,
                [layout.x, 76., layout.width, layout.bottom - 116.],
                &HomeView {
                    installed: true,
                    launching: false,
                    profile: 0,
                    renderer: "Vulkan",
                },
                |r, a| actions.push((r, a)),
            );
            assert!(actions.iter().any(|(_, a)| *a == HomeAction::Play));
            assert!(actions.iter().any(|(_, a)| *a == HomeAction::Profile));
            for (i, (r, _)) in actions.iter().enumerate() {
                assert!(r[0] >= layout.x && r[0] + r[2] <= ui.w && r[1] + r[3] <= layout.bottom);
                for (o, _) in actions.iter().take(i) {
                    assert!(
                        r[0] >= o[0] + o[2]
                            || o[0] >= r[0] + r[2]
                            || r[1] >= o[1] + o[3]
                            || o[1] >= r[1] + r[3]
                    );
                }
            }
        }
    }

    #[test]
    fn saved_themes_keep_wallpaper_independent_of_material_effects() {
        let mut cfg = LauncherThemeConfig::default();
        cfg.apply_preset(ThemePreset::DarkAero);
        cfg.glass_effects = false;
        let loaded: LauncherThemeConfig =
            serde_json::from_str(&serde_json::to_string(&cfg).unwrap()).unwrap();
        assert_eq!(loaded.resolve().material, Material::Opaque);
        assert_eq!(loaded.resolve().wallpaper, Wallpaper::Graphite);
        let mut legacy = serde_json::to_value(&cfg).unwrap();
        for key in ["wallpaper", "glass_effects", "interface_sounds", "glass"] {
            legacy.as_object_mut().unwrap().remove(key);
        }
        let loaded: LauncherThemeConfig = serde_json::from_value(legacy).unwrap();
        assert!(loaded.interface_sounds);
        assert_eq!(loaded.wallpaper, Wallpaper::None);
        assert_eq!(loaded.glass, GlassTuning::default());
    }

    #[test]
    fn disabling_liquid_glass_removes_backdrop_work_and_saved_optics_roundtrip() {
        let mut cfg = LauncherThemeConfig::default();
        cfg.apply_preset(ThemePreset::LiquidGlass);
        cfg.glass = GlassTuning {
            opacity: 0.08,
            blur: 0.,
            refraction: 24.,
        };
        let saved: LauncherThemeConfig =
            serde_json::from_str(&serde_json::to_string(&cfg).unwrap()).unwrap();
        assert_eq!(saved.resolve().glass, cfg.glass);
        let atlas = HudAtlas::boot();
        for enabled in [true, false] {
            cfg.glass_effects = enabled;
            set_active_for_test(&cfg);
            let mut batch = HudBatch::new([640, 360]);
            let mut ui = Ui::new(&atlas, &mut batch, (0., 0.));
            panel(&mut ui, [20., 20., 150., 50.], 18., false);
            ui.finish();
            assert_eq!(!batch.glass.is_empty(), enabled);
        }
    }

    fn set_active_for_test(cfg: &LauncherThemeConfig) {
        super::super::set_active_theme(cfg.resolve());
    }
}
