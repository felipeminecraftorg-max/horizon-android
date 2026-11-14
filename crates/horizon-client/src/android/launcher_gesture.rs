//! Track total drag distance separately from per-event scroll deltas.
/// One pointer owns a gesture. A release can activate only the same target
/// that received its press; a second finger cannot replace or finish it.
pub(super) struct TapGate<T> {
    active: Option<(u64, Option<T>)>,
}
impl<T> Default for TapGate<T> {
    fn default() -> Self {
        Self { active: None }
    }
}
impl<T: Copy + PartialEq> TapGate<T> {
    pub(super) fn start(&mut self, id: u64, target: Option<T>) -> bool {
        if self.active.is_some() {
            return false;
        }
        self.active = Some((id, target));
        true
    }
    pub(super) fn owns(&self, id: u64) -> bool {
        self.active.is_some_and(|(owner, _)| owner == id)
    }
    pub(super) fn finish(&mut self, id: u64, target: Option<T>, moved: bool) -> bool {
        if !self.owns(id) {
            return false;
        }
        let (_, pressed) = self.active.take().unwrap();
        !moved && pressed.is_some() && pressed == target
    }
    pub(super) fn pressed(&self) -> Option<T> {
        self.active.and_then(|(_, target)| target)
    }
    pub(super) fn cancel(&mut self) {
        self.active = None;
    }
}
/// Scroll only enough to reveal the selected row, retaining the current offset.
pub(super) fn reveal(scroll: f32, row: [f32; 4], view: [f32; 3]) -> f32 {
    let [top, bottom, maximum] = view;
    let adjustment = if row[1] < top {
        row[1] - top
    } else if row[1] + row[3] > bottom {
        row[1] + row[3] - bottom
    } else {
        0.0
    };
    (scroll + adjustment).clamp(0.0, maximum.max(0.0))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PageSwipe {
    Previous,
    Next,
}

/// A horizontally swipable navigation *strip*, deliberately not the scrolling
/// content underneath it. Taps, player skin rotation and vertical catalog
/// scrolling must never be misinterpreted as changes of console channel.
pub(super) fn console_nav_zone(point: [f32; 2], viewport: [f32; 2]) -> bool {
    let [w, h] = viewport;
    if w <= 0. || h <= 0. { return false; }
    let [x, y] = point;
    if !(16. ..= w - 16.).contains(&x) { return false; }
    if h > w || w < 620. {
        (h - 82. ..= h - 8.).contains(&y)
    } else {
        (69. ..= 125.).contains(&y)
    }
}

pub(super) struct Drag {
    origin: [f64; 2],
    previous_y: f64,
    pub(super) moved: bool,
}

impl Drag {
    pub(super) fn new(x: f64, y: f64) -> Self {
        Self {
            origin: [x, y],
            previous_y: y,
            moved: false,
        }
    }

    pub(super) fn origin_y(&self) -> f64 {
        self.origin[1]
    }

    pub(super) fn originated_in_console_nav(
        &self,
        density: f64,
        viewport: [f32; 2],
    ) -> bool {
        let d = density.max(0.01);
        console_nav_zone([ (self.origin[0] / d) as f32,
                           (self.origin[1] / d) as f32 ], viewport)
    }

    /// Complete a horizontal tab swipe on release. Minimum 56dp and strong
    /// directional dominance prevent diagonal list scroll from changing page.
    /// The event owner is still enforced by TapGate in the launcher.
    pub(super) fn release_console_swipe(
        &self,
        x: f64,
        y: f64,
        density: f64,
        viewport: [f32; 2],
    ) -> Option<PageSwipe> {
        if !self.originated_in_console_nav(density, viewport) {
            return None;
        }
        let d = density.max(0.01);
        let dx = (x - self.origin[0]) / d;
        let dy = (y - self.origin[1]) / d;
        if dx.abs() < 56. || dx.abs() < dy.abs() * 1.6 {
            return None;
        }
        Some(if dx > 0. { PageSwipe::Previous } else { PageSwipe::Next })
    }

    pub(super) fn update(&mut self, x: f64, y: f64, density: f64) -> f32 {
        let density = density.max(0.01);
        self.moved |= (x - self.origin[0]).hypot(y - self.origin[1]) > 12.0 * density;
        let delta = if self.moved {
            ((self.previous_y - y) / density) as f32
        } else {
            0.0
        };
        self.previous_y = y;
        delta
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_release_never_clicks_another_button_or_an_empty_press() {
        let mut gate = TapGate::default();
        gate.start(1, Some('a'));
        assert!(!gate.finish(1, Some('b'), false));
        gate.start(1, None);
        assert!(!gate.finish(1, Some('a'), false));
        gate.start(1, Some('a'));
        assert!(!gate.finish(1, Some('a'), true));
        gate.start(1, Some('a'));
        assert!(gate.finish(1, Some('a'), false));
        assert!(!gate.finish(1, Some('a'), false));
    }
    #[test]
    fn secondary_fingers_and_cancellation_do_not_activate() {
        let mut gate = TapGate::default();
        assert!(gate.start(10, Some('a')));
        assert!(!gate.start(11, Some('b')));
        assert!(!gate.finish(11, Some('a'), false));
        assert!(gate.owns(10));
        gate.cancel();
        assert!(!gate.finish(10, Some('a'), false));
    }

    #[test]
    fn focus_scroll_reveals_rows_in_both_directions_and_stops_at_bounds() {
        assert_eq!(
            reveal(0.0, [0.0, 350.0, 100.0, 60.0], [100.0, 400.0, 500.0]),
            10.0
        );
        assert_eq!(
            reveal(200.0, [0.0, 80.0, 100.0, 60.0], [100.0, 400.0, 500.0]),
            180.0
        );
        assert_eq!(
            reveal(20.0, [0.0, 150.0, 100.0, 60.0], [100.0, 400.0, 500.0]),
            20.0
        );
        assert_eq!(
            reveal(490.0, [0.0, 700.0, 100.0, 60.0], [100.0, 400.0, 500.0]),
            500.0
        );
    }

    #[test]
    fn slow_vertical_drag_crosses_the_total_threshold() {
        let mut drag = Drag::new(100.0, 300.0);
        for y in [297.0, 294.0, 291.0, 288.0] {
            assert_eq!(drag.update(100.0, y, 1.0), 0.0);
        }
        assert_eq!(drag.update(100.0, 285.0, 1.0), 3.0);
        assert!(drag.moved);
        assert_eq!(drag.origin_y(), 300.0);
        assert_eq!(drag.update(100.0, 290.0, 1.0), -5.0);
    }

    #[test]
    fn console_swipes_only_change_page_on_tab_strip() {
        let mut nav = Drag::new(240., 91.);
        nav.update(130., 94., 1.);
        assert_eq!(nav.release_console_swipe(130., 94., 1., [900., 480.]),
            Some(PageSwipe::Next));
        let mut left = Drag::new(240., 90.);
        left.update(350., 90., 1.);
        assert_eq!(left.release_console_swipe(350., 90., 1., [900., 480.]),
            Some(PageSwipe::Previous));
        let mut content = Drag::new(280., 220.);
        content.update(160., 220., 1.);
        assert_eq!(content.release_console_swipe(160., 220., 1., [900., 480.]), None);
        let mut vertical = Drag::new(240., 88.);
        vertical.update(185., 155., 1.);
        assert_eq!(vertical.release_console_swipe(185., 155., 1., [900., 480.]), None);
        let short = Drag::new(240., 88.);
        assert_eq!(short.release_console_swipe(198., 88., 1., [900., 480.]), None);
    }

    #[test]
    fn console_navigation_strip_respects_portrait_and_density() {
        let drag = Drag::new(180. * 2., 736. * 2.);
        assert!(drag.originated_in_console_nav(2., [360., 800.]));
        assert_eq!(drag.release_console_swipe(80. * 2., 732. * 2., 2.,
            [360., 800.]), Some(PageSwipe::Next));
        assert!(!console_nav_zone([180., 400.], [360., 800.]));
        assert!(!console_nav_zone([2., 736.], [360., 800.]));
    }

    #[test]
    fn density_scales_threshold_and_scroll_together() {
        let mut drag = Drag::new(200.0, 600.0);
        assert_eq!(drag.update(200.0, 580.0, 2.0), 0.0);
        assert_eq!(drag.update(200.0, 570.0, 2.0), 5.0);
        assert!(drag.moved);
    }
}
