use std::collections::{BTreeMap, BTreeSet};

use crate::core::codes::BTN_LEFT;
use crate::core::geometry::Point;

pub const DOUBLE_CLICK_US: u64 = 400_000;
pub const DRAG_THRESHOLD: f32 = 4.0;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ButtonRuntime {
    pub code: u16,
    pub pressed: bool,
    pub press_count: u32,
    pub last_press_us: Option<u64>,
    pub last_release_us: Option<u64>,
    pub last_duration_us: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DragState {
    pub start: Point,
    pub current: Point,
    pub start_us: u64,
    pub moved: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DragSummary {
    pub start: Point,
    pub end: Point,
    pub duration_us: u64,
    pub moved: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ButtonUpdate {
    pub code: u16,
    pub pressed: bool,
    pub double_click: bool,
    pub duration_us: Option<u64>,
    pub drag_released: Option<DragSummary>,
}

#[derive(Clone, Debug)]
pub struct ButtonState {
    buttons: BTreeMap<u16, ButtonRuntime>,
    pressed: BTreeSet<u16>,
    double_click_us: u64,
    drag: Option<DragState>,
}

impl Default for ButtonState {
    fn default() -> Self {
        Self {
            buttons: BTreeMap::new(),
            pressed: BTreeSet::new(),
            double_click_us: DOUBLE_CLICK_US,
            drag: None,
        }
    }
}

impl ButtonState {
    pub fn new(double_click_us: u64) -> Self {
        Self {
            double_click_us,
            ..Self::default()
        }
    }

    pub fn apply(&mut self, code: u16, pressed: bool, now_us: u64, cursor: Point) -> ButtonUpdate {
        if pressed {
            self.press(code, now_us, cursor)
        } else {
            self.release(code, now_us, cursor)
        }
    }

    pub fn move_cursor(&mut self, cursor: Point) {
        if let Some(drag) = &mut self.drag {
            drag.current = cursor;
            if drag.start.distance(cursor) >= DRAG_THRESHOLD {
                drag.moved = true;
            }
        }
    }

    pub fn is_pressed(&self, code: u16) -> bool {
        self.pressed.contains(&code)
    }

    pub fn pressed(&self) -> &BTreeSet<u16> {
        &self.pressed
    }

    pub fn buttons(&self) -> &BTreeMap<u16, ButtonRuntime> {
        &self.buttons
    }

    pub fn runtime(&self, code: u16) -> Option<&ButtonRuntime> {
        self.buttons.get(&code)
    }

    pub fn drag(&self) -> Option<DragState> {
        self.drag
    }

    pub fn reset(&mut self) {
        self.buttons.clear();
        self.pressed.clear();
        self.drag = None;
    }

    fn press(&mut self, code: u16, now_us: u64, cursor: Point) -> ButtonUpdate {
        if self.pressed.contains(&code) {
            return ButtonUpdate {
                code,
                pressed: true,
                double_click: false,
                duration_us: None,
                drag_released: None,
            };
        }

        let double_click = self
            .buttons
            .get(&code)
            .and_then(|runtime| runtime.last_release_us)
            .is_some_and(|released| now_us.saturating_sub(released) <= self.double_click_us);

        let runtime = self.buttons.entry(code).or_insert(ButtonRuntime {
            code,
            ..ButtonRuntime::default()
        });
        runtime.pressed = true;
        runtime.press_count = runtime.press_count.saturating_add(1);
        runtime.last_press_us = Some(now_us);
        self.pressed.insert(code);

        if code == BTN_LEFT {
            self.drag = Some(DragState {
                start: cursor,
                current: cursor,
                start_us: now_us,
                moved: false,
            });
        }

        ButtonUpdate {
            code,
            pressed: true,
            double_click,
            duration_us: None,
            drag_released: None,
        }
    }

    fn release(&mut self, code: u16, now_us: u64, cursor: Point) -> ButtonUpdate {
        if !self.pressed.remove(&code) {
            return ButtonUpdate {
                code,
                pressed: false,
                double_click: false,
                duration_us: None,
                drag_released: None,
            };
        }

        let duration_us = self
            .buttons
            .get(&code)
            .and_then(|runtime| runtime.last_press_us)
            .map(|pressed| now_us.saturating_sub(pressed));

        if let Some(runtime) = self.buttons.get_mut(&code) {
            runtime.pressed = false;
            runtime.last_release_us = Some(now_us);
            runtime.last_duration_us = duration_us;
        }

        let drag_released = if code == BTN_LEFT {
            self.drag.take().map(|mut drag| {
                drag.current = cursor;
                drag.moved = drag.moved || drag.start.distance(cursor) >= DRAG_THRESHOLD;
                DragSummary {
                    start: drag.start,
                    end: drag.current,
                    duration_us: now_us.saturating_sub(drag.start_us),
                    moved: drag.moved,
                }
            })
        } else {
            None
        };

        ButtonUpdate {
            code,
            pressed: false,
            double_click: false,
            duration_us,
            drag_released,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::codes::{BTN_RIGHT, BTN_SIDE};

    fn cursor() -> Point {
        Point::new(100.0, 100.0)
    }

    #[test]
    fn counts_presses_and_measures_duration() {
        let mut buttons = ButtonState::default();
        buttons.apply(BTN_LEFT, true, 1_000, cursor());
        let update = buttons.apply(BTN_LEFT, false, 151_000, cursor());
        assert_eq!(update.duration_us, Some(150_000));
        assert!(!buttons.is_pressed(BTN_LEFT));
        assert_eq!(buttons.runtime(BTN_LEFT).unwrap().press_count, 1);
    }

    #[test]
    fn detects_double_click_window() {
        let mut buttons = ButtonState::default();
        buttons.apply(BTN_LEFT, true, 1_000, cursor());
        buttons.apply(BTN_LEFT, false, 50_000, cursor());
        let update = buttons.apply(BTN_LEFT, true, 300_000, cursor());
        assert!(update.double_click);
    }

    #[test]
    fn ignores_double_click_outside_window() {
        let mut buttons = ButtonState::default();
        buttons.apply(BTN_LEFT, true, 1_000, cursor());
        buttons.apply(BTN_LEFT, false, 50_000, cursor());
        let update = buttons.apply(BTN_LEFT, true, 600_000, cursor());
        assert!(!update.double_click);
    }

    #[test]
    fn ignores_duplicate_press() {
        let mut buttons = ButtonState::default();
        buttons.apply(BTN_RIGHT, true, 1_000, cursor());
        buttons.apply(BTN_RIGHT, true, 2_000, cursor());
        assert_eq!(buttons.runtime(BTN_RIGHT).unwrap().press_count, 1);
    }

    #[test]
    fn tracks_drag_release() {
        let mut buttons = ButtonState::default();
        buttons.apply(BTN_LEFT, true, 1_000, Point::new(10.0, 10.0));
        buttons.move_cursor(Point::new(30.0, 10.0));
        let update = buttons.apply(BTN_LEFT, false, 101_000, Point::new(30.0, 10.0));
        let drag = update.drag_released.unwrap();
        assert!(drag.moved);
        assert_eq!(drag.duration_us, 100_000);
        assert_eq!(drag.start, Point::new(10.0, 10.0));
        assert_eq!(drag.end, Point::new(30.0, 10.0));
    }

    #[test]
    fn release_without_press_is_safe() {
        let mut buttons = ButtonState::default();
        let update = buttons.apply(BTN_SIDE, false, 1_000, cursor());
        assert!(!update.pressed);
        assert!(update.duration_us.is_none());
    }
}
