//! Geometry for the Old School console dashboard, independent of drawing and input.
//! Changes here must preserve the eight Home actions and their static hitboxes.

#[derive(Clone, Copy, Debug)]
pub(crate) struct ConsoleDashboardLayout {
    pub hero: [f32; 4],
    /// Mods, Packs, Shaders, My Files, Skins, Settings.
    pub shortcuts: [[f32; 4]; 6],
}

impl ConsoleDashboardLayout {
    pub fn new(bounds: [f32; 4]) -> Self {
        let [x, y, w, h] = bounds;
        let gap = 9.0;
        // A wide console gets a hero beside the mosaic. Compact screens
        // stack them; both use the SAME original actions and hitbox targets.
        let (hero, right) = if w < 520.0 {
            let hero_h = (h * 0.42).clamp(150.0, 226.0).min((h - 120.0).max(0.0));
            ([x, y, w, hero_h], [x, y + hero_h + gap, w, (h - hero_h - gap).max(0.0)])
        } else {
            let hero_w = (w * 0.54).max(0.0);
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
        // Interleaved two-column feature tiles reproduce the varied rhythm
        // of a real console dashboard instead of six equal 3x2 app cards.
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
