//! Horizon Jade Home geometry: an NXE-style hero beside a Metro-style utility mosaic.
//! All six destinations plus Play/Profile retain fixed input rectangles.

#[derive(Clone, Copy, Debug)]
pub(crate) struct ConsoleDashboardLayout {
    pub hero: [f32; 4],
    /// Mods, Packs, Shaders, My Files, Skins, Settings.
    pub shortcuts: [[f32; 4]; 6],
}

impl ConsoleDashboardLayout {
    pub fn new(bounds: [f32; 4]) -> Self {
        let [x, y, w, h] = bounds;
        let gap = if w < 520.0 { 8.0 } else { 10.0 };
        // A wide console gets a hero beside the mosaic. Compact screens
        // stack them; both use the SAME original actions and hitbox targets.
        let (hero, right) = if w < 520.0 {
            let hero_h = (h * 0.40).clamp(150.0, 240.0).min((h - 166.0).max(0.0));
            ([x, y, w, hero_h], [x, y + hero_h + gap, w, (h - hero_h - gap).max(0.0)])
        } else {
            let hero_w = (w * 0.56).min((w - 255.0).max(0.0));
            ([x, y, hero_w, h], [x + hero_w + gap, y, (w - hero_w - gap).max(0.0), h])
        };
        let [rx, ry, rw, rh] = right;
        let column = ((rw - 2.0 * gap) / 3.0).max(0.0);
        let row = ((rh - 2.0 * gap) / 3.0).max(0.0);
        let tile = |col: usize, row_idx: usize, span: usize| [
            rx + col as f32 * (column + gap),
            ry + row_idx as f32 * (row + gap),
            column * span as f32 + gap * (span - 1) as f32,
            row,
        ];
        // Alternating wide utility tiles retain visual hierarchy and avoid
        // six indistinguishable buttons. Geometry is independent of theme.
        Self {
            hero,
            shortcuts: [
                tile(0, 0, 2), // Mods
                tile(2, 0, 1), // Packs
                tile(0, 1, 1), // Shaders
                tile(1, 1, 2), // My Files
                tile(0, 2, 1), // Skins
                tile(1, 2, 2), // Settings
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ConsoleDashboardLayout;

    #[test]
    fn every_home_target_stays_inside_and_never_overlaps() {
        for bounds in [
            [16.0, 76.0, 308.0, 440.0],
            [16.0, 76.0, 378.0, 580.0],
            [24.0, 130.0, 712.0, 202.0],
            [24.0, 130.0, 852.0, 272.0],
            [24.0, 130.0, 1250.0, 550.0],
        ] {
            let layout = ConsoleDashboardLayout::new(bounds);
            let mut all = vec![layout.hero];
            all.extend(layout.shortcuts);
            for (i, r) in all.iter().enumerate() {
                assert!(r[2] >= 48.0 && r[3] >= 44.0, "touch target: {r:?}");
                assert!(r[0] >= bounds[0] && r[1] >= bounds[1], "origin: {r:?}");
                assert!(r[0] + r[2] <= bounds[0] + bounds[2] + 0.01);
                assert!(r[1] + r[3] <= bounds[1] + bounds[3] + 0.01);
                for earlier in &all[..i] {
                    assert!(r[0] >= earlier[0] + earlier[2]
                        || earlier[0] >= r[0] + r[2]
                        || r[1] >= earlier[1] + earlier[3]
                        || earlier[1] >= r[1] + r[3], "overlap: {earlier:?}, {r:?}");
                }
            }
        }
    }
}
