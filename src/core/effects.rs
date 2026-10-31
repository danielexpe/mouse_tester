use std::collections::VecDeque;

use crate::core::buttons::DragSummary;
use crate::core::geometry::Point;
use crate::core::scroll::{ScrollAxis, ScrollUpdate};

pub const MAX_EFFECTS: usize = 128;
pub const RIPPLE_US: u64 = 400_000;
pub const SIDE_RIPPLE_US: u64 = 320_000;
pub const WHEEL_PULSE_US: u64 = 250_000;
pub const EDGE_FLASH_US: u64 = 150_000;
pub const DRAG_RELEASE_US: u64 = 300_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EffectKind {
    Ripple {
        code: u16,
        radius_scale: f32,
    },
    DoubleRipple {
        code: u16,
    },
    WheelPulse {
        axis: ScrollAxis,
        direction: i8,
        pulses: u8,
        hires_scale: f32,
    },
    EdgeFlash {
        edge: Edge,
    },
    DragRelease {
        start: Point,
        end: Point,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Effect {
    pub kind: EffectKind,
    pub pos: Point,
    pub start_us: u64,
    pub duration_us: u64,
    pub intensity: f32,
}

impl Effect {
    pub fn progress(&self, now_us: u64) -> f32 {
        if self.duration_us == 0 {
            return 1.0;
        }
        let elapsed = now_us.saturating_sub(self.start_us);
        (elapsed as f32 / self.duration_us as f32).clamp(0.0, 1.0)
    }

    pub fn is_expired(&self, now_us: u64) -> bool {
        now_us >= self.start_us.saturating_add(self.duration_us)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Effects {
    items: VecDeque<Effect>,
}

impl Effects {
    pub fn button(code: u16, pos: Point, now_us: u64, double_click: bool) -> Effect {
        let side_button = !matches!(code, 0x110..=0x112);
        Effect {
            kind: if double_click {
                EffectKind::DoubleRipple { code }
            } else {
                EffectKind::Ripple {
                    code,
                    radius_scale: if side_button { 0.7 } else { 1.0 },
                }
            },
            pos,
            start_us: now_us,
            duration_us: if side_button {
                SIDE_RIPPLE_US
            } else {
                RIPPLE_US
            },
            intensity: 1.0,
        }
    }

    pub fn wheel(update: &ScrollUpdate, pos: Point, now_us: u64) -> Option<Effect> {
        if update.notches == 0 && update.hires_units == 0 {
            return None;
        }
        Some(Effect {
            kind: EffectKind::WheelPulse {
                axis: update.axis,
                direction: update.direction,
                pulses: update.pulses,
                hires_scale: if update.notches == 0 { 0.5 } else { 1.0 },
            },
            pos,
            start_us: now_us,
            duration_us: WHEEL_PULSE_US,
            intensity: if update.notches == 0 { 0.5 } else { 1.0 },
        })
    }

    pub fn edge(edge: Edge, pos: Point, now_us: u64) -> Effect {
        Effect {
            kind: EffectKind::EdgeFlash { edge },
            pos,
            start_us: now_us,
            duration_us: EDGE_FLASH_US,
            intensity: 1.0,
        }
    }

    pub fn drag_release(drag: DragSummary, now_us: u64) -> Effect {
        Effect {
            kind: EffectKind::DragRelease {
                start: drag.start,
                end: drag.end,
            },
            pos: drag.end,
            start_us: now_us,
            duration_us: DRAG_RELEASE_US,
            intensity: 1.0,
        }
    }

    pub fn push(&mut self, effect: Effect) {
        while self.items.len() >= MAX_EFFECTS {
            self.items.pop_front();
        }
        self.items.push_back(effect);
    }

    pub fn tick(&mut self, now_us: u64) {
        while self
            .items
            .front()
            .is_some_and(|effect| effect.is_expired(now_us))
        {
            self.items.pop_front();
        }
        self.items.retain(|effect| !effect.is_expired(now_us));
    }

    pub fn iter(&self) -> impl Iterator<Item = &Effect> {
        self.items.iter()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::codes::BTN_LEFT;
    use crate::core::scroll::ScrollUpdate;

    fn wheel_update(notches: i32, hires_units: i32) -> ScrollUpdate {
        ScrollUpdate {
            axis: ScrollAxis::Vertical,
            notches,
            remainder: 0,
            direction: notches.signum() as i8,
            pulses: notches.unsigned_abs().clamp(1, 3) as u8,
            hires_units,
        }
    }

    #[test]
    fn effect_expires() {
        let effect = Effects::button(BTN_LEFT, Point::new(1.0, 2.0), 1_000, false);
        assert_eq!(effect.progress(1_000), 0.0);
        assert_eq!(effect.progress(1_000 + RIPPLE_US / 2), 0.5);
        assert!(effect.is_expired(1_000 + RIPPLE_US));
    }

    #[test]
    fn caps_active_effects() {
        let mut effects = Effects::default();
        for i in 0..(MAX_EFFECTS + 10) {
            effects.push(Effects::button(
                BTN_LEFT,
                Point::new(i as f32, 0.0),
                i as u64,
                false,
            ));
        }
        assert_eq!(effects.len(), MAX_EFFECTS);
    }

    #[test]
    fn tick_removes_expired() {
        let mut effects = Effects::default();
        effects.push(Effects::button(BTN_LEFT, Point::new(0.0, 0.0), 0, false));
        effects.tick(RIPPLE_US);
        assert!(effects.is_empty());
    }

    #[test]
    fn wheel_without_motion_is_ignored() {
        assert!(Effects::wheel(&wheel_update(0, 0), Point::new(0.0, 0.0), 0).is_none());
    }

    #[test]
    fn hi_res_wheel_gets_smaller_intensity() {
        let effect = Effects::wheel(&wheel_update(0, 60), Point::new(0.0, 0.0), 0).unwrap();
        assert_eq!(effect.intensity, 0.5);
    }

    #[test]
    fn drag_release_keeps_endpoints() {
        let effect = Effects::drag_release(
            DragSummary {
                start: Point::new(1.0, 1.0),
                end: Point::new(5.0, 5.0),
                duration_us: 10,
                moved: true,
            },
            0,
        );
        match effect.kind {
            EffectKind::DragRelease { start, end } => {
                assert_eq!(start, Point::new(1.0, 1.0));
                assert_eq!(end, Point::new(5.0, 5.0));
            }
            _ => panic!("wrong effect"),
        }
    }
}
