use crate::core::codes::BTN_LEFT;
use crate::core::device::DeviceId;
use crate::core::scroll::ScrollAxis;
use crate::input::{InputEvent, InputKind};
use crate::util::SplitMix64;

pub const MOVE_INTERVAL_US: u64 = 16_000;
pub const CLICK_INTERVAL_US: u64 = 1_400_000;
pub const WHEEL_INTERVAL_US: u64 = 900_000;
pub const CLICK_HOLD_US: u64 = 90_000;

pub fn movement(device: DeviceId, time_us: u64, dx: i32, dy: i32) -> InputEvent {
    InputEvent::mock(device, time_us, InputKind::Move { dx, dy })
}

pub fn button(device: DeviceId, time_us: u64, code: u16, pressed: bool) -> InputEvent {
    InputEvent::mock(device, time_us, InputKind::Button { code, pressed })
}

pub fn wheel(
    device: DeviceId,
    time_us: u64,
    axis: ScrollAxis,
    value: i32,
    hires: bool,
) -> InputEvent {
    InputEvent::mock(device, time_us, InputKind::Wheel { axis, value, hires })
}

pub fn report(device: DeviceId, time_us: u64) -> InputEvent {
    InputEvent::mock(device, time_us, InputKind::Report)
}

#[derive(Clone, Debug)]
pub struct DemoSource {
    device: DeviceId,
    rng: SplitMix64,
    next_move_us: u64,
    next_click_us: u64,
    next_release_us: Option<u64>,
    next_wheel_us: u64,
    wheel_direction: i32,
    phase: f32,
}

impl DemoSource {
    pub fn new(device: DeviceId, seed: u64) -> Self {
        Self {
            device,
            rng: SplitMix64::new(seed),
            next_move_us: 0,
            next_click_us: CLICK_INTERVAL_US,
            next_release_us: None,
            next_wheel_us: WHEEL_INTERVAL_US / 2,
            wheel_direction: 1,
            phase: 0.0,
        }
    }

    pub fn poll(&mut self, now_us: u64) -> Vec<InputEvent> {
        let mut events = Vec::new();
        let mut budget = 128usize;

        while self.next_move_us <= now_us && budget > 0 {
            let at = self.next_move_us;
            let dx = (self.phase.sin() * 8.0).round() as i32;
            let dy = ((self.phase * 0.7).cos() * 5.0).round() as i32;
            events.push(movement(self.device, at, dx, dy));
            events.push(report(self.device, at));
            self.phase += 0.22;
            self.next_move_us = self.next_move_us.saturating_add(MOVE_INTERVAL_US);
            budget -= 1;
        }

        while self.next_click_us <= now_us && budget > 0 {
            let at = self.next_click_us;
            events.push(button(self.device, at, BTN_LEFT, true));
            events.push(report(self.device, at));
            self.next_release_us = Some(at.saturating_add(CLICK_HOLD_US));
            self.next_click_us = self
                .next_click_us
                .saturating_add(CLICK_INTERVAL_US + (self.rng.range_i32(0, 200) as u64) * 1_000);
            budget -= 1;
        }

        if let Some(release_at) = self.next_release_us {
            if release_at <= now_us {
                events.push(button(self.device, release_at, BTN_LEFT, false));
                events.push(report(self.device, release_at));
                self.next_release_us = None;
            }
        }

        while self.next_wheel_us <= now_us && budget > 0 {
            let at = self.next_wheel_us;
            events.push(wheel(
                self.device,
                at,
                ScrollAxis::Vertical,
                self.wheel_direction,
                false,
            ));
            events.push(report(self.device, at));
            self.wheel_direction = -self.wheel_direction;
            self.next_wheel_us = self.next_wheel_us.saturating_add(WHEEL_INTERVAL_US);
            budget -= 1;
        }

        events.sort_by_key(|event| event.time_us);
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_emits_events_in_time_order() {
        let mut demo = DemoSource::new(1, 42);
        let events = demo.poll(2_000_000);
        assert!(!events.is_empty());
        assert!(events
            .windows(2)
            .all(|pair| pair[0].time_us <= pair[1].time_us));
        assert!(events
            .iter()
            .any(|event| matches!(event.kind, InputKind::Move { .. })));
        assert!(events
            .iter()
            .any(|event| matches!(event.kind, InputKind::Button { .. })));
        assert!(events
            .iter()
            .any(|event| matches!(event.kind, InputKind::Wheel { .. })));
    }

    #[test]
    fn demo_is_deterministic_for_seed() {
        let mut first = DemoSource::new(1, 7);
        let mut second = DemoSource::new(1, 7);
        assert_eq!(first.poll(500_000), second.poll(500_000));
    }
}
