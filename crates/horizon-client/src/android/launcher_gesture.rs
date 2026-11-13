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

    pub(super) fn origin_x(&self) -> f64 {
        self.origin[0]
    }

    pub(super) fn origin_y(&self) -> f64 {
        self.origin[1]
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

/// The axis-lock decision for PS3-style cross navigation is deliberately
/// independent of frame rate, scroll offsets and Android pixel density:
/// callers pass the final drag vector in dp.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum XmbSwipe {
    Category(isize),
    Item(i32),
}

pub(super) fn xmb_swipe(dx: f32, dy: f32, on_home: bool) -> Option<XmbSwipe> {
    if !dx.is_finite() || !dy.is_finite() {
        return None;
    }
    if dx.abs() >= 38. && dx.abs() > dy.abs() * 1.3 {
        return Some(XmbSwipe::Category(if dx < 0. { 1 } else { -1 }));
    }
    if on_home && dy.abs() >= 38. && dy.abs() > dx.abs() * 1.3 {
        let steps = ((dy.abs() / 66.).round() as i32).clamp(1, 3);
        return Some(XmbSwipe::Item(if dy < 0. { steps } else { -steps }));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xmb_swipes_lock_axis_avoid_diagonals_and_cap_steps() {
        assert_eq!(xmb_swipe(-90., 9., true), Some(XmbSwipe::Category(1)));
        assert_eq!(xmb_swipe(90., 9., true), Some(XmbSwipe::Category(-1)));
        assert_eq!(xmb_swipe(9., -64., true), Some(XmbSwipe::Item(1)));
        assert_eq!(xmb_swipe(9., 150., true), Some(XmbSwipe::Item(-2)));
        assert_eq!(xmb_swipe(0., -1000., true), Some(XmbSwipe::Item(3)));
        assert_eq!(xmb_swipe(0., -100., false), None);
        assert_eq!(xmb_swipe(-35., 3., true), None);
        assert_eq!(xmb_swipe(-80., 79., true), None);
        assert_eq!(xmb_swipe(f32::NAN, 0., true), None);
    }

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
    fn density_scales_threshold_and_scroll_together() {
        let mut drag = Drag::new(200.0, 600.0);
        assert_eq!(drag.update(200.0, 580.0, 2.0), 0.0);
        assert_eq!(drag.update(200.0, 570.0, 2.0), 5.0);
        assert!(drag.moved);
    }
}
