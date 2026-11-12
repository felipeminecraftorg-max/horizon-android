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
            opacity: 0.06,
            blur: 1.0,
            refraction: 20.,
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


// Small, original Minecraft-inspired landscapes, painted once in the native
// atlas. No external CDN, copyrighted game screenshot, or Android asset request.
const SCENE_W: u32 = 384;
const SCENE_H: u32 = 200;
thread_local! {
    static SCENES: RefCell<HashMap<bool, Vec<u8>>> = RefCell::new(HashMap::new());
}
fn scene_pixels(sunset: bool) -> Vec<u8> {
    let mut pixels = Vec::with_capacity((SCENE_W * SCENE_H * 4) as usize);
    let mix = |a: f32, b: f32, v: f32| a + (b - a) * v.clamp(0., 1.);
    for y in 0..SCENE_H {
        for x in 0..SCENE_W {
            // Quantization gives the landscape the deliberately blocky silhouette
            // of a voxel game, even when displayed as a soft photographic hero.
            let bx = (x / 4) as f32 * 4.;
            let by = (y / 4) as f32 * 4.;
            let u = bx / SCENE_W as f32;
            let v = by / SCENE_H as f32;
            let noise = (((x / 4).wrapping_mul(15427)
                ^ (y / 4).wrapping_mul(64901)
                ^ 0x4D3B19) % 31) as f32 - 15.;
            let top = if sunset { [37., 39., 78.] } else { [30., 61., 82.] };
            let low = if sunset { [243., 163., 123.] } else { [136., 168., 174.] };
            let sky_t = (v * 1.4).clamp(0., 1.);
            let mut rgb = [
                mix(top[0], low[0], sky_t),
                mix(top[1], low[1], sky_t),
                mix(top[2], low[2], sky_t),
            ];
            // Striped block-clouds, intentionally subtle in the dark variant.
            let cloud = (u * 11. + v * 4.).sin() + (u * 21. - v * 5.).cos() * 0.5;
            if v > 0.12 && v < 0.47 && cloud > 1.22 {
                for component in &mut rgb { *component = mix(*component, 210., 0.17); }
            }
            let sun_x = if sunset { 0.71 } else { 0.79 };
            let sun_y = if sunset { 0.53 } else { 0.47 };
            let sun_dist = ((u - sun_x).powi(2) * 1.2 + (v - sun_y).powi(2) * 3.).sqrt();
            let warm = (1. - sun_dist / 0.18).clamp(0., 1.);
            for k in 0..3 {
                rgb[k] = mix(rgb[k], [250., 216., 175.][k], warm * 0.68);
            }
            let ridge = 0.48
                + (u * 10.1).sin() * 0.047
                + (u * 29.7).sin() * 0.019
                - (u - 0.77).max(0.).powi(2) * 2.4;
            let ridge = (ridge * 40.).floor() / 40.;
            if v > ridge {
                rgb = if sunset {
                    [65. - 13. * v, 81. - 12. * v, 106. - 14. * v]
                } else {
                    [46. - 13. * v, 78. - 18. * v, 85. - 12. * v]
                };
            }
            let grass = 0.74 + (u * 11.).sin() * 0.028
                + (u * 37.).sin() * 0.013
                - (u - 0.3).powi(2) * 0.08;
            if v > grass {
                rgb = if sunset { [46., 85., 47.] } else { [30., 73., 51.] };
                let soil = ((v - grass) * 10.).floor() as i32 % 5;
                if soil == 3 { rgb = [44., 53., 45.]; }
                if (v - grass) < 0.025 { rgb = [70., 121., 54.]; }
            }
            // Layered square-canopy trees and dark thin trunks.
            for &cx in &[0.08_f32, 0.19, 0.31, 0.46, 0.60, 0.89] {
                let cy = grass + (cx * 11.).sin() * 0.01;
                let th = 0.10 + (cx * 19.).sin().abs() * 0.05;
                let trunk_top = cy - th;
                if (u - cx).abs() < 0.010 && v >= trunk_top && v < cy + 0.04 {
                    rgb = [48., 44., 35.];
                }
                let du = (u - cx).abs();
                if (du < 0.055 && v > trunk_top - 0.070 && v < trunk_top + 0.045)
                    || (du < 0.038 && v > trunk_top - 0.095 && v < trunk_top + 0.065)
                {
                    rgb = if sunset { [37., 68., 47.] } else { [30., 77., 56.] };
                    rgb[1] += noise.abs() * 0.55;
                }
            }
            // Quiet reflective inlet; the reflection stretches toward the sun.
            let inlet = v > 0.755 && u > 0.49 && u < 0.88
                && v < 0.94 + (u * 22.).sin() * 0.02;
            if inlet {
                rgb = if sunset { [43., 78., 102.] } else { [26., 69., 83.] };
                let ripple = ((v * 96. + u * 22.).sin().abs() * 0.5
                    + (u * 44.).cos().abs() * 0.3).clamp(0., 1.);
                let glint = (1. - (u - sun_x).abs() * 5.0).clamp(0., 1.);
                rgb[0] += ripple * glint * if sunset { 95. } else { 39. };
                rgb[1] += ripple * glint * 70.;
                rgb[2] += ripple * glint * 47.;
            }
            for k in 0..3 {
                rgb[k] = (rgb[k] + noise * if v > ridge { 0.20 } else { 0.07 })
                    .clamp(0., 255.);
                pixels.push(rgb[k] as u8);
            }
            pixels.push(255);
        }
    }
    pixels
}
fn scene_texture(ui: &mut Ui<'_>, sunset: bool) -> Option<super::Rect> {
    SCENES.with(|scenes| {
        let mut scenes = scenes.borrow_mut();
        let pixels = scenes.entry(sunset).or_insert_with(|| scene_pixels(sunset));
        let key = if sunset { "launcher-wallpaper:scene:console" }
            else { "launcher-wallpaper:scene:liquid" };
        super::with_cache(ui.atlas, |cache| {
            cache.image(ui.atlas, key, SCENE_W, SCENE_H, pixels)
        })
    })
}
/// Draw a cached original world-like scene inside a rounded rectangle.
/// Rounded pixels are clipped by symmetric bands, without covering text.
fn scene(ui: &mut Ui<'_>, r: [f32; 4], corner: f32, sunset: bool) {
    let Some(t) = scene_texture(ui, sunset) else { return; };
    let [x, y, w, h] = r;
    if w <= 0. || h <= 0. { return; }
    let corner = corner.min(w * 0.5).min(h * 0.5);
    let mut band = |top: f32, height: f32, inset: f32| {
        let u0 = t.u0 + (t.u1 - t.u0) * (inset / w);
        let u1 = t.u1 - (t.u1 - t.u0) * (inset / w);
        let v0 = t.v0 + (t.v1 - t.v0) * ((top - y) / h);
        let v1 = t.v0 + (t.v1 - t.v0) * ((top + height - y) / h);
        ui.emit_clipped_uv(
            ui.px_x(x + inset), ui.px_y(top), ui.px_x(x + w - inset),
            ui.px_y(top + height), u0, v0, u1, v1, u32::MAX,
        );
    };
    for i in 0..12 {
        let dh = corner / 12.;
        let d = corner - (i as f32 + 0.5) * dh;
        let inset = corner - (corner * corner - d * d).max(0.).sqrt();
        band(y + i as f32 * dh, dh, inset);
        band(y + h - (i as f32 + 1.) * dh, dh, inset);
    }
    band(y + corner, (h - 2. * corner).max(0.), 0.);
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
    if th.experience == ExperiencePreset::OldSchool && th.material == Material::Aero
        && w > 285. && h > 138.
    {
        // Large panels are a *content* layer, never a mirror of previously
        // rendered controls. This removes repeated ghost labels near edges.
        ui.rounded(x, y, w, h, radius, color::hex(0x142033, 112));
        ui.stroke_rounded(x, y, w, h, radius, 0.8,
            color::hex(0xE4E9F6, 75));
        return;
    }
    if th.experience == ExperiencePreset::OldSchool && th.material == Material::Aero {
        // Actual GPU-composited chrome: WGSL samples the live framebuffer and
        // evaluates the PS3 pearl surface from the exported Glass Lab profile.
        // The controls are not opaque rounded rectangles with fake highlights.
        let optics = th.glass.normalized();
        let compact = h <= 100. || w <= 190.;
        ui.backdrop_lens(r, radius, color::hex(0xFFFFFF, 0), [
            if compact { 0.05 } else { 0.02 },
            if compact { optics.refraction * 0.48 } else { optics.refraction * 0.27 },
        ]);
        return;
    }
    if th.material == Material::Glass && w > 300. && h > 145. {
        // A spacious photographic/content surface should not have the same
        // heavy lens extrusion as a push button or slider handle.
        ui.rounded(x, y, w, h, radius, color::hex(0x101D29, 100));
        ui.stroke_rounded(x, y, w, h, radius, 0.9,
            color::hex(0xDFEBF5, 92));
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
    if th.experience == ExperiencePreset::OldSchool {
        // The console style has metal insets and a bottom chrome reflection.
        // These are real per-component details, not a tinted glass skin.
        ui.fill(x + radius, y + h - 2., (w - 2. * radius).max(0.), 1., color::hex(0xCFD9E6, if emphasized { 94 } else { 42 }));
        ui.fill(x + radius, y + 2., (w - 2. * radius).max(0.), 0.7, color::hex(0xF7FAFF, if emphasized { 125 } else { 66 }));
    }
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
    let th = active_theme();
    panel(ui, rect, if th.experience == ExperiencePreset::OldSchool { 10. } else { rect[3] * 0.5 }, accent);
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

/// Glass Lab V7 components adapted to the native Horizon HUD.
/// These are actual interactive surfaces: callers keep their existing hitboxes,
/// state and touch handlers rather than replacing them with visual mockups.
/// The material in panel() is rasterized by the WGSL backdrop-lens pipeline.
pub fn component_card(ui: &mut Ui<'_>, rect: [f32; 4], emphasized: bool) {
    let th = active_theme();
    let [x, y, w, h] = rect;
    if th.experience == ExperiencePreset::OldSchool {
        // A wide settings/catalog card should remain readable, not repeat
        // previously composited text as a giant refractive lens.
        ui.rounded(x, y, w, h, 12., color::hex(0x101927, if emphasized { 192 } else { 174 }));
        ui.stroke_rounded(x, y, w, h, 12., 0.75, color::hex(0xC8DAF0, 65));
        ui.fill(x + 13., y + 2., (w - 26.).max(0.), 0.8,
            color::hex(0xEFF5FF, 62));
    } else if th.experience == ExperiencePreset::LiquidGlass {
        ui.rounded(x, y, w, h, 15., color::hex(0x162A32, if emphasized { 105 } else { 77 }));
        ui.stroke_rounded(x, y, w, h, 15., 0.8, color::hex(0xE2F7FF, 84));
    } else {
        ui.rounded(x, y, w, h, 14., th.surface);
    }
}

/// V7 pill/chip: translucent text-bearing object, small enough for GPU optics.
pub fn component_chip(ui: &mut Ui<'_>, rect: [f32; 4], selected: bool) {
    let th = active_theme();
    let [x, y, w, h] = rect;
    if th.experience == ExperiencePreset::Standard {
        ui.rounded(x, y, w, h, h / 2., if selected { th.accent } else { th.surface });
    } else {
        // The chip's backdrop is a single real lens; the label is emitted
        // afterwards so glyphs aren't sampled twice by adjacent lenses.
        panel(ui, rect, if th.experience == ExperiencePreset::OldSchool { 9. } else { h / 2. }, selected);
        if selected {
            ui.stroke_rounded(x + 1., y + 1., w - 2., h - 2.,
                if th.experience == ExperiencePreset::OldSchool { 8. } else { h / 2. - 1. },
                0.85, color::hex(0xF1F5FF, 137));
        }
    }
}

/// Slider uses the same fraction as the native settings gesture handler.
/// Its thumb is intentionally legible (satin white / aluminium), while the
/// track stays partially transparent. Works on both Android touch and mouse.
pub fn component_slider(ui: &mut Ui<'_>, rect: [f32; 4], value: f32, value_label: &str) {
    let th = active_theme();
    let [x, y, w, h] = rect;
    let old = th.experience == ExperiencePreset::OldSchool;
    panel(ui, rect, if old { 10. } else { h * 0.5 }, false);
    let caption = ui.fit_text(Face::SemiBold, 12., value_label, w - 16.);
    ui.centered_fit(Face::SemiBold, 12.,
        [x + 6., y + 2., w - 12., 18.], color::hex(0xF0F5FC, 245), &caption);
    let bar_x = x + 10.;
    let bar_y = y + h - 12.;
    let bar_w = (w - 20.).max(24.);
    let t = value.clamp(0., 1.);
    ui.rounded(bar_x, bar_y, bar_w, 5., 2.5,
        color::hex(if old { 0x71849B } else { 0xDCE9F5 }, 83));
    ui.rounded(bar_x, bar_y, (bar_w * t).max(2.), 5., 2.5,
        color::hex(if old { 0xCAD7EA } else { 0xEDF9FF }, if old { 185 } else { 128 }));
    let thumb_w = if old { 9. } else { 11. };
    let thumb_x = (bar_x + bar_w * t - thumb_w * 0.5)
        .clamp(bar_x - 1., bar_x + bar_w - thumb_w + 1.);
    ui.rounded(thumb_x, bar_y - 4., thumb_w, 13.,
        if old { 3.5 } else { thumb_w * 0.5 },
        color::hex(if old { 0xE1E7EF } else { 0xF7FCFF }, 229));
    ui.stroke_rounded(thumb_x, bar_y - 4., thumb_w, 13.,
        if old { 3.5 } else { thumb_w * 0.5 },
        0.8, color::hex(0xFFFFFF, 155));
}

/// Spring-state keyed by the switch's actual screen position. Controls are
/// instant in logic and hit testing; only the visible thumb eases into place.
struct V7SwitchAnimation {
    position: f32,
    velocity: f32,
    target: f32,
    last_frame: std::time::Instant,
}
std::thread_local! {
    static V7_SWITCH_ANIMATIONS:
        std::cell::RefCell<std::collections::HashMap<(i32, i32, u8), V7SwitchAnimation>>
        = std::cell::RefCell::new(std::collections::HashMap::new());
}
pub fn switch_motion_active() -> bool {
    V7_SWITCH_ANIMATIONS.with(|cache| cache.borrow().values().any(|animation| {
        animation.last_frame.elapsed().as_secs_f32() < 0.07
            && ((animation.position - animation.target).abs() > 0.008
                || animation.velocity.abs() > 0.02)
    }))
}
fn switch_thumb_fraction(x: f32, y: f32, enabled: bool, style: ExperiencePreset) -> f32 {
    let target = if enabled { 1. } else { 0. };
    if super::motion::reduced_motion() {
        return target;
    }
    let style_id = match style {
        ExperiencePreset::OldSchool => 1,
        ExperiencePreset::LiquidGlass => 2,
        ExperiencePreset::Standard => 0,
    };
    let key = ((x * 2.).round() as i32, (y * 2.).round() as i32, style_id);
    V7_SWITCH_ANIMATIONS.with(|cache| {
        let mut cache = cache.borrow_mut();
        // Prevent long-lived saved page geometries growing the map forever.
        if cache.len() > 400 {
            cache.retain(|_, a| a.last_frame.elapsed().as_secs() < 15);
        }
        let now = std::time::Instant::now();
        let motion = cache.entry(key).or_insert_with(|| V7SwitchAnimation {
            position: target,
            velocity: 0.,
            target,
            last_frame: now,
        });
        let dt = now.duration_since(motion.last_frame).as_secs_f32().min(0.08);
        motion.last_frame = now;
        motion.target = target;
        let (position, velocity) = super::launcher_preset_pack::spring_position(
            style, motion.position, motion.velocity, target, dt,
        );
        motion.position = position.clamp(-0.08, 1.08);
        motion.velocity = velocity.clamp(-10., 10.);
        motion.position
    })
}

/// Toggle: translucent GPU track and an opaque touch-readable thumb, animated
/// by the same physics that controls the selected navigation bubble.
pub fn component_switch(ui: &mut Ui<'_>, x: f32, y: f32, on: bool) {
    let th = active_theme();
    let old = th.experience == ExperiencePreset::OldSchool;
    let rect = [x, y, 52., 28.];
    panel(ui, rect, if old { 10. } else { 14. }, on);
    let position = switch_thumb_fraction(x, y, on, th.experience);
    let travel = 4. + 22. * position;
    let thumb_d = 20.;
    ui.rounded(x + travel, y + 4., thumb_d, thumb_d,
        if old { 6. } else { 10. },
        color::hex(if old { 0xEBEEF3 } else { 0xF7FBFF }, 236));
    if position > 0.70 {
        ui.icon(Icon::Check, x + travel + 4., y + 8., 12.,
            color::hex(if old { 0x253243 } else { 0x3E6872 }, 245));
    }
}

/// Track containing *one* active lens; all tab labels remain crisp. The focus
/// fraction may be supplied by a spring; touch targets are registered by caller.
pub fn component_tabs(ui: &mut Ui<'_>, rect: [f32; 4],
    labels: [&str; 2], focused: f32) {
    let th = active_theme();
    let [x, y, w, h] = rect;
    let old = th.experience == ExperiencePreset::OldSchool;
    ui.rounded(x, y, w, h, if old { 11. } else { h / 2. },
        color::hex(if old { 0x171C2D } else { 0xB9D5DE }, if old { 144 } else { 34 }));
    let part_w = w / 2.;
    let fx = x + part_w * focused.clamp(0., 1.);
    panel(ui, [fx + 2., y + 2., part_w - 4., h - 4.],
        if old { 9. } else { (h - 4.) / 2. }, true);
    for (i, label) in labels.into_iter().enumerate() {
        let selected = (focused - i as f32).abs() < 0.5;
        ui.centered_fit(Face::Medium, 13.,
            [x + i as f32 * part_w + 5., y + 2., part_w - 10., h - 4.],
            if selected { th.text_primary } else { th.text_muted }, label);
    }
}

/// Sidebar selected bubble shares the same V7 optical surface as controls;
/// scrolling and navigation never draw glyphs into the lens framebuffer.
pub fn component_circular_icon(ui: &mut Ui<'_>, rect: [f32; 4], icon: Icon) {
    let th = active_theme();
    panel(ui, rect, rect[3] / 2., true);
    let size = (rect[3] * 0.46).clamp(16., 24.);
    ui.icon(icon, rect[0] + (rect[2] - size) / 2.,
        rect[1] + (rect[3] - size) / 2., size, th.text_primary);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HomeAction {
    Play,
    Profile,
    Appearance,
    Settings,
    Mods,
    Packs,
    Shaders,
    Files,
    Skins,
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
        let experience = active_theme().experience;
        let console = experience == ExperiencePreset::OldSchool;
        let x = if portrait {
            16.
        } else if console {
            24. // Full-width console dashboard beneath the horizontal tabs.
        } else if experience == ExperiencePreset::LiquidGlass {
            119. // Narrow icon rail, like the modern reference.
        } else {
            210.
        };
        Self {
            portrait,
            console,
            x,
            width: (w - x - if console { 24. } else { 20. }).max(200.),
            bottom: if portrait { h - 84. } else { h },
        }
    }
    pub fn nav_rect(self, w: f32, h: f32, index: usize, count: usize) -> [f32; 4] {
        if self.portrait {
            let step = (w - 32.) / count as f32;
            [16. + index as f32 * step, h - 72., step, 56.]
        } else if self.console {
            let step = (w - 48.) / count.max(1) as f32;
            [24. + index as f32 * step, 73., step, 44.]
        } else if active_theme().experience == ExperiencePreset::LiquidGlass {
            let step = ((h - 114.) / count.max(1) as f32).clamp(44., 52.);
            [32., 90. + index as f32 * step, 55., 44.]
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


/// Late-2000s console interface: no sidebar, no oversized rounded capsules.
/// Tabs form a real horizontal navigation row; home supplies a tile dashboard.
#[allow(clippy::too_many_arguments)]
fn shell_console(
    ui: &mut Ui<'_>,
    items: &[(&str, Icon)],
    selected: usize,
    account: &str,
    signed_in: bool,
    version: &str,
    focus_x: f32,
    mut target: impl FnMut([f32; 4], ShellAction),
) -> Layout {
    let th = active_theme();
    let (w, h) = (ui.w, ui.h);
    let layout = Layout::new(w, h);
    ui.horizon_logo(24., 15., 30.);
    ui.text(Face::SemiBold, 20., 62., 12., th.text_primary, "Horizon");
    ui.text(Face::Regular, 9.5, 62., 38., th.text_muted, version);

    let aw = (w * 0.22).clamp(138., 196.);
    let account_r = [w - aw - 66., 13., aw, 39.];
    ui.rounded(account_r[0], account_r[1], aw, 39., 9., color::hex(0xECF3FF, 27));
    ui.stroke_rounded(account_r[0], account_r[1], aw, 39., 9., 0.9, color::hex(0xFFFFFF, 85));
    ui.avatar_head_2d(account_r[0] + 6., 19., 27., account);
    if signed_in {
        ui.rounded(account_r[0] + 29., 40., 5., 5., 2.5, th.success);
    }
    let name = ui.fit_text(Face::Medium, 12., account, aw - 51.);
    ui.text(Face::Medium, 12., account_r[0] + 42., 24., th.text_primary, &name);
    target(account_r, ShellAction::Account);
    let settings = [w - 53., 13., 38., 39.];
    ui.icon(Icon::Gear, settings[0] + 10., settings[1] + 9., 19., th.text_primary);
    target(settings, ShellAction::Appearance);
    ui.fill(24., 117., w - 48., 1., color::hex(0xD4D8EB, 72));

    // A physically animated lens follows the selected console tab. The
    // underlying hitboxes remain fixed, so focus and touch never lag.
    if !items.is_empty() {
        let step = (w - 48.) / items.len() as f32;
        let fx = focus_x.clamp(24., w - 24. - step);
        panel(ui, [fx + 10., 76., (step - 20.).max(22.), 34.], 12., false);
        ui.fill(fx + step * 0.28, 114., step * 0.44, 1.4,
            color::hex(0xEEF2FF, 208));
    }
    for (i, (label, _icon)) in items.iter().enumerate() {
        let tab = layout.nav_rect(w, h, i, items.len());
        let chosen = i == selected;
        let fitted = ui.fit_text(Face::Medium, 13., label, tab[2] - 12.);
        let tw = ui.measure(Face::Medium, 13., &fitted);
        ui.text(
            Face::Medium, 13.,
            tab[0] + (tab[2] - tw) * 0.5,
            tab[1] + 8.,
            if chosen { th.text_primary } else { th.text_muted },
            &fitted,
        );
        target(tab, ShellAction::Page(i));
    }
    layout
}

/// Header and adaptive navigation shared by the app and the native captures.
/// Backward-compatible non-animated entry, used by the standalone UI harness.
#[allow(clippy::too_many_arguments)]
pub fn shell(
    ui: &mut Ui<'_>,
    items: &[(&str, Icon)],
    selected: usize,
    account: &str,
    signed_in: bool,
    version: &str,
    target: impl FnMut([f32; 4], ShellAction),
) -> Layout {
    let layout = Layout::new(ui.w, ui.h);
    let index = selected.min(items.len().saturating_sub(1));
    let target_rect = layout.nav_rect(ui.w, ui.h, index, items.len().max(1));
    shell_animated(ui, items, selected, account, signed_in, version,
        [target_rect[0], target_rect[1]], target)
}

/// Modern glass and console channels each have an animated selected component.
/// The physical lens moves independently of immutable target hitboxes.
#[allow(clippy::too_many_arguments)]
pub fn shell_animated(
    ui: &mut Ui<'_>,
    items: &[(&str, Icon)],
    selected: usize,
    account: &str,
    signed_in: bool,
    version: &str,
    focus: [f32; 2],
    mut target: impl FnMut([f32; 4], ShellAction),
) -> Layout {
    let th = active_theme();
    let (w, h) = (ui.w, ui.h);
    let layout = Layout::new(w, h);
    if layout.console && !layout.portrait {
        return shell_console(ui, items, selected, account, signed_in, version, focus[0], target);
    }
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
    // Liquid Glass: an entirely different narrow icon dock. Keep tab content
    // outside the glass layer; the icons and their hitboxes remain crisp.
    if th.experience == ExperiencePreset::LiquidGlass && !layout.portrait {
        panel(ui, [19., 83., 81., (h - 99.).max(80.)], 31., false);
        let selected_rect = layout.nav_rect(w, h, selected, items.len().max(1));
        let selected_y = focus[1].clamp(90., (h - 61.).max(90.));
        panel(ui, [selected_rect[0] + 3., selected_y + 1., selected_rect[2] - 6., 42.], 18., true);
        for (i, (_, icon)) in items.iter().enumerate() {
            let r = layout.nav_rect(w, h, i, items.len());
            ui.icon(
                *icon, r[0] + (r[2] - 21.) / 2., r[1] + 11., 21.,
                if i == selected { th.text_primary } else { th.text_muted },
            );
            target(r, ShellAction::Page(i));
        }
        return layout;
    }
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
    // Genuine console tile dashboard: game cover occupies the left half and
    // the functional shortcuts form a 3x2 grid, not cards below a modern hero.
    let poster_w = if narrow { w } else { w * 0.505 };
    let poster_h = if narrow { (h * 0.49).max(173.) } else { h };
    let poster = [x, y, poster_w, poster_h];
    // Drop the placeholder voxel image. Real GPU chrome samples the Graphite wallpaper.
    panel(ui, poster, 16., false);
    // Satin dark content inset, lit by the current wallpaper.
    ui.stroke_rounded(x, y, poster_w, poster_h, 16., 0.8, color::hex(0xE2E5FA, 80));
    ui.rounded(x + 15., y + 16., (poster_w - 30.).min(330.), 104., 12.,
        color::hex(0x0A1020, 111));
    ui.text(Face::SemiBold, if narrow { 23. } else { 31. },
        x + 26., y + 22., color::hex(0xFFFFFF, 255), "MINECRAFT");
    ui.text(Face::Medium, 16., x + 27., y + 64., color::hex(0xE4E6F5, 243), "26.3");
    let small = if view.installed { "Ready to play" } else { "Installation required" };
    ui.text(Face::Regular, 12.5, x + 27., y + 95., color::hex(0xFAFAFF, 236), small);
    // A small profile selector belongs to the poster, not the utility cards.
    let profile = [x + poster_w - 119., y + 13., 105., 37.];
    panel(ui, profile, 9., false);
    ui.text(Face::Medium, 11., profile[0] + 9., profile[1] + 11.,
        color::hex(0xF2F6FF, 245), &format!("Profile {}", view.profile + 1));
    target(profile, HomeAction::Profile);

    let play = [x + 12., y + poster_h - 67., poster_w - 24., 54.];
    panel(ui, play, 13., true);
    ui.rounded(play[0] + 1., play[1] + 1., play[2] - 2., play[3] - 2.,
        12., color::hex(0x587B61, 48));
    // Cool satin edge: dark-chrome surface stays faithful to the user's V7 material.
    ui.fill(play[0] + 12., play[1] + 3., play[2] - 24., 1.,
        color::hex(0xE9F1FA, 102));
    ui.icon(Icon::Play, play[0] + 19., play[1] + 14., 27., color::hex(0xFFFFFF, 255));
    let label = if view.launching { "Getting ready..." } else if view.installed {
        "Play Minecraft"
    } else { "Install Minecraft" };
    let fitted = ui.fit_text(Face::Medium, 22., label, poster_w - 66.);
    ui.text(Face::Medium, 22., play[0] + 57., play[1] + 15.,
        color::hex(0xFFFFFF, 255), &fitted);
    if !view.launching { target(play, HomeAction::Play); }

    let gap = 7.;
    let cols = if narrow { 2 } else { 3 };
    let tx = if narrow { x } else { x + poster_w + 10. };
    let ty = if narrow { y + poster_h + gap } else { y };
    let available = if narrow { w } else { w - poster_w - 10. };
    let card_w = (available - gap * (cols - 1) as f32) / cols as f32;
    let card_h = if narrow { ((h - poster_h - 3. * gap) / 3.).max(39.) }
        else { (h - gap) * 0.5 };
    // A tint is a small *translucent* reflection in the chrome, not a solid
    // Xbox-style painted tile. Maintain bright typography over each finish.
    let shortcuts = [
        ("Mods", Icon::Mods, HomeAction::Mods, 0x986C9A_u32),
        ("Packs", Icon::Packs, HomeAction::Packs, 0x537DB2_u32),
        ("Shaders", Icon::Shaders, HomeAction::Shaders, 0xB18A57_u32),
        ("My files", Icon::Folder, HomeAction::Files, 0x5A8DA2_u32),
        ("Skins", Icon::Person, HomeAction::Skins, 0x619B7A_u32),
        ("Settings", Icon::Gear, HomeAction::Settings, 0x777E9A_u32),
    ];
    for (i, (name, icon, action, tint)) in shortcuts.into_iter().enumerate() {
        let row = (i / cols) as f32;
        let col = (i % cols) as f32;
        let card = [tx + col * (card_w + gap), ty + row * (card_h + gap), card_w, card_h];
        if card[1] + card[3] > y + h + 1. { continue; }
        panel(ui, card, 12., false);
        ui.rounded(card[0] + 1., card[1] + 1., card_w - 2., card_h - 2.,
            11., color::hex(tint, 47));
        // The narrow silver seam is a design accent, not another lens surface.
        ui.fill(card[0] + 12., card[1] + 3., (card_w - 24.).max(0.), 1.,
            color::hex(0xFFFFFF, 44));
        ui.stroke_rounded(card[0], card[1], card_w, card_h, 11., 0.7,
            color::hex(tint, 126));
        let icon_size = if narrow { 23. } else { (card_h * 0.30).min(36.) };
        ui.icon(icon, card[0] + (card_w - icon_size) * 0.5,
            card[1] + (card_h * 0.23).min(35.), icon_size, color::hex(0xF5F7FF, 243));
        let font = if narrow { 13. } else { 16. };
        ui.centered_fit(Face::Medium, font,
            [card[0] + 4., card[1] + card_h - if narrow { 26. } else { 38. }, card_w - 8., 29.],
            color::hex(0xF4F5FB, 255), name);
        target(card, action);
    }
}

/// Clean modern Liquid Glass: a single wide image and six simple translucent
/// shortcuts, with real refracting surfaces only where there are controls.
fn home_liquid(
    ui: &mut Ui<'_>,
    rect: [f32; 4],
    view: &HomeView<'_>,
    mut target: impl FnMut([f32; 4], HomeAction),
) {
    let [x, y, w, h] = rect;
    let th = active_theme();
    let gap = 10.;
    let tiles_h = (h * 0.25).clamp(58., 95.);
    let hero_h = (h - tiles_h - 14.).max(145.);
    let hero = [x, y, w, hero_h];

    // No synthetic pixel-art backdrop: true refraction samples the live theme wallpaper.
    // Background imagery remains visible. The lens is light and only bends at
    // the edge; central text is never baked into the background texture.
    panel(ui, hero, 25., false);
    // Soft left-side scrim (not another frosted panel) preserves contrast.
    for i in 0..8 {
        let fraction = i as f32 / 8.;
        ui.fill(x + 6. + fraction * w * 0.51, y + 18.,
            w * 0.068, hero_h - 36.,
            color::hex(0x071321, ((1. - fraction) * 27.) as u8));
    }

    // Icon tile plus title and clear status.
    let icon_r = [x + 25., y + 44., 75., 75.];
    ui.rounded(icon_r[0], icon_r[1], 75., 75., 18., color::hex(0x121D22, 165));
    ui.border(icon_r[0], icon_r[1], 75., 75., 18., color::hex(0xE3F7FF, 67));
    ui.icon(Icon::Gamepad, icon_r[0] + 21., icon_r[1] + 21., 33.,
        color::hex(0xE9F5F7, 250));
    ui.text(Face::SemiBold, 30., x + 117., y + 48., color::hex(0xFFFFFF, 255),
        "Minecraft 26.3");
    let ready = if view.installed { "Ready to play" } else { "Installation required" };
    ui.icon(if view.installed { Icon::Check } else { Icon::Installed },
        x + 120., y + 94., 18., if view.installed { th.success } else { th.text_muted });
    ui.text(Face::Medium, 13., x + 147., y + 97.,
        color::hex(0xEDF6FA, 243), ready);

    // The main action and profile picker live inside the hero, as in the
    // submitted mockup, with a real gap and disjoint touch areas.
    let buttons_y = y + hero_h - 67.;
    let play_width = (w * 0.35).clamp(193., 310.);
    let profile_width = (w * 0.26).clamp(137., 222.);
    let play = [x + 25., buttons_y, play_width, 49.];
    let profile = [play[0] + play_width + 14., buttons_y, profile_width, 49.];
    panel(ui, play, 24.5, true);
    ui.icon(Icon::Play, play[0] + 17., play[1] + 14., 21., color::hex(0xFFFFFF, 255));
    let title = if view.launching { "Getting ready..." } else if view.installed {
        "Play Minecraft"
    } else { "Install Minecraft" };
    let title = ui.fit_text(Face::Medium, 14., title, play_width - 55.);
    ui.text(Face::Medium, 14., play[0] + 48., play[1] + 15.,
        color::hex(0xFFFFFF, 255), &title);
    if !view.launching { target(play, HomeAction::Play); }
    panel(ui, profile, 24.5, false);
    ui.icon(Icon::ChevronDown, profile[0] + profile_width - 27.,
        profile[1] + 16., 15., th.text_primary);
    let label = ui.fit_text(Face::Medium, 12.,
        &format!("Profile {}", view.profile + 1), profile_width - 48.);
    ui.text(Face::Medium, 12., profile[0] + 17., profile[1] + 17., th.text_primary, &label);
    target(profile, HomeAction::Profile);

    let renderer = ui.fit_text(Face::Regular, 10., view.renderer, 152.);
    let rw = (ui.measure(Face::Regular, 10., &renderer) + 22.).min(w * 0.22);
    let render_x = x + w - rw - 19.;
    // This label never overlaps the Minecraft title or the launch buttons.
    ui.text(Face::Regular, 10., render_x, y + 15., color::hex(0xEBF7FB, 194), &renderer);

    let tiles_y = y + hero_h + 14.;
    let tile_w = (w - gap * 5.) / 6.;
    for (i, (label, icon, action)) in [
        ("Mods", Icon::Mods, HomeAction::Mods),
        ("Packs", Icon::Packs, HomeAction::Packs),
        ("Shaders", Icon::Shaders, HomeAction::Shaders),
        ("My files", Icon::Folder, HomeAction::Files),
        ("Settings", Icon::Gear, HomeAction::Settings),
        ("Skins", Icon::Person, HomeAction::Skins),
    ].into_iter().enumerate() {
        let r = [x + i as f32 * (tile_w + gap), tiles_y, tile_w, tiles_h];
        // Independent, restrained physical controls; no stacked nested glass.
        panel(ui, r, 20., false);
        let size = (tiles_h * 0.38).min(31.);
        ui.icon(icon, r[0] + (tile_w - size) * 0.5,
            r[1] + tiles_h * 0.22, size, th.text_primary);
        ui.centered_fit(Face::Medium, 13.0,
            [r[0] + 4., r[1] + tiles_h - 27., tile_w - 8., 22.],
            th.text_primary, label);
        target(r, action);
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
    if th.experience == ExperiencePreset::LiquidGlass && w >= 620. {
        return home_liquid(ui, rect, view, target);
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
        .filter(|p| !matches!(p, ThemePreset::LiquidGlass | ThemePreset::OldSchool))
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
    fn both_landscape_mockups_keep_all_six_tiles_and_disjoint_hitboxes() {
        for preset in [ThemePreset::OldSchool, ThemePreset::LiquidGlass] {
            let mut cfg = LauncherThemeConfig::default();
            cfg.apply_preset(preset);
            set_active_for_test(&cfg);
            for (width, height) in [(760., 410.), (900., 480.)] {
                let layout = Layout::new(width, height);
                let y = if layout.console { 130. } else { 76. };
                let reserved = if layout.console { 35. } else { 40. };
                let bounds = [layout.x, y, layout.width, height - y - reserved];
                let atlas = HudAtlas::boot();
                let mut batch = HudBatch::new([width as u32, height as u32]);
                let mut ui = Ui::new(&atlas, &mut batch, (0., 0.));
                let mut actions = Vec::new();
                home(
                    &mut ui,
                    bounds,
                    &HomeView { installed: true, launching: false, profile: 0,
                        renderer: "Vulkan / compatibility" },
                    |r, action| actions.push((r, action)),
                );
                assert!(actions.len() >= 8, "{preset:?} {width}x{height}");
                for wanted in [
                    HomeAction::Play, HomeAction::Profile, HomeAction::Mods,
                    HomeAction::Packs, HomeAction::Shaders, HomeAction::Files,
                    HomeAction::Settings, HomeAction::Skins,
                ] {
                    assert!(actions.iter().any(|(_, action)| *action == wanted),
                        "{preset:?} {width}x{height} {wanted:?}");
                }
                for (i, (r, _)) in actions.iter().enumerate() {
                    assert!(r[0] >= bounds[0] && r[1] >= bounds[1]);
                    assert!(r[0] + r[2] <= bounds[0] + bounds[2] + 0.1);
                    assert!(r[1] + r[3] <= bounds[1] + bounds[3] + 0.1);
                    for (other, _) in actions.iter().take(i) {
                        assert!(r[0] >= other[0] + other[2]
                            || other[0] >= r[0] + r[2]
                            || r[1] >= other[1] + other[3]
                            || other[1] >= r[1] + r[3],
                            "overlap {preset:?} {width}x{height}: {r:?} and {other:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn reference_experiences_have_independent_navigation_and_hero_art() {
        let mut cfg = LauncherThemeConfig::default();
        cfg.apply_preset(ThemePreset::LiquidGlass);
        set_active_for_test(&cfg);
        let liquid = Layout::new(900., 480.);
        assert!(!liquid.console);
        assert_eq!(liquid.x, 119.);
        assert!(liquid.nav_rect(900., 480., 0, 7)[2] < 65.);
        let liquid_scene = scene_pixels(false);
        assert_eq!(liquid_scene.len(), (SCENE_W * SCENE_H * 4) as usize);
        cfg.apply_preset(ThemePreset::OldSchool);
        set_active_for_test(&cfg);
        let old = Layout::new(900., 480.);
        assert!(old.console);
        assert_eq!(old.x, 24.);
        assert_eq!(old.nav_rect(900., 480., 0, 7)[1], 73.);
        assert_eq!(old.nav_rect(900., 480., 0, 7)[3], 44.);
        let old_scene = scene_pixels(true);
        assert_eq!(old_scene.len(), liquid_scene.len());
        assert_ne!(liquid_scene, old_scene);
    }

    #[test]
    fn old_school_changes_real_layout_without_losing_home_actions() {
        let mut cfg = LauncherThemeConfig::default();
        cfg.apply_preset(ThemePreset::OldSchool);
        assert_eq!(cfg.resolve().experience, ExperiencePreset::OldSchool);
        assert_eq!(cfg.resolve().material, Material::Aero);
        assert_eq!(cfg.resolve().wallpaper, Wallpaper::Graphite);
        set_active_for_test(&cfg);
        for (w, h) in [(340., 640.), (900., 480.)] {
            let layout = Layout::new(w, h);
            assert!(layout.console);
            let atlas = HudAtlas::boot();
            let mut batch = HudBatch::new([w as u32, h as u32]);
            let mut ui = Ui::new(&atlas, &mut batch, (0., 0.));
            let mut actions = Vec::new();
            home(
                &mut ui,
                [layout.x, 76., layout.width, layout.bottom - 116.],
                &HomeView {
                    installed: true,
                    launching: false,
                    profile: 0,
                    renderer: "Vulkan / compatibility",
                },
                |r, a| actions.push((r, a)),
            );
            for required in [
                HomeAction::Play,
                HomeAction::Profile,
                HomeAction::Mods,
                HomeAction::Packs,
                HomeAction::Shaders,
                HomeAction::Files,
            ] {
                assert!(actions.iter().any(|(_, actual)| *actual == required), "{w}x{h}: {required:?}");
            }
            for (i, (r, _)) in actions.iter().enumerate() {
                assert!(r[0] >= layout.x && r[0] + r[2] <= ui.w);
                assert!(r[1] >= 76. && r[1] + r[3] <= layout.bottom);
                for (o, _) in actions.iter().take(i) {
                    assert!(
                        r[0] >= o[0] + o[2] || o[0] >= r[0] + r[2] ||
                        r[1] >= o[1] + o[3] || o[1] >= r[1] + r[3],
                        "{w}x{h}: action overlap {r:?} / {o:?}"
                    );
                }
            }
        }
        cfg.apply_preset(ThemePreset::LiquidGlass);
        set_active_for_test(&cfg);
        assert_eq!(cfg.resolve().experience, ExperiencePreset::LiquidGlass);
        assert!(!Layout::new(900., 480.).console);
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
