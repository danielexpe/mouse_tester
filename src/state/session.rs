use std::collections::VecDeque;

use crate::core::buttons::ButtonState;
use crate::core::device::{Capabilities, DeviceClass, DeviceId, DeviceInfo};
use crate::core::effects::{Edge, Effects};
use crate::core::geometry::Bounds;
use crate::core::metrics::Metrics;
use crate::core::modes::{ButtonsChallenge, ScrollChallenge};
use crate::core::pointer::PointerState;
use crate::core::scroll::{ScrollState, WheelFrame};
use crate::input::{InputEvent, InputKind, InputSource};
use crate::state::log::{LogEntry, LogKind, LOG_CAPACITY};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Dashboard,
    Buttons,
    Scroll,
    Polling,
    Targets,
}

impl Mode {
    pub const ALL: [Mode; 5] = [
        Mode::Dashboard,
        Mode::Buttons,
        Mode::Scroll,
        Mode::Polling,
        Mode::Targets,
    ];

    pub fn next(self) -> Self {
        let index = Self::ALL.iter().position(|mode| *mode == self).unwrap_or(0);
        Self::ALL[(index + 1) % Self::ALL.len()]
    }

    pub fn previous(self) -> Self {
        let index = Self::ALL.iter().position(|mode| *mode == self).unwrap_or(0);
        Self::ALL[(index + Self::ALL.len() - 1) % Self::ALL.len()]
    }

    pub fn from_number(number: u8) -> Option<Self> {
        match number {
            1 => Some(Self::Dashboard),
            2 => Some(Self::Buttons),
            3 => Some(Self::Scroll),
            4 => Some(Self::Polling),
            5 => Some(Self::Targets),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Session {
    pub mode: Mode,
    pub pointer: PointerState,
    pub buttons: ButtonState,
    pub scroll: ScrollState,
    pub effects: Effects,
    pub metrics: Metrics,
    pub buttons_challenge: ButtonsChallenge,
    pub scroll_challenge: ScrollChallenge,
    pub devices: Vec<DeviceInfo>,
    pub primary: Option<DeviceId>,
    pub log: VecDeque<LogEntry>,
    pub overflow_events: u64,
    pub last_source: Option<InputSource>,
    pub last_event_us: u64,
    wheel_frame: WheelFrame,
}

impl Session {
    pub fn new(bounds: Bounds) -> Self {
        Self {
            mode: Mode::Dashboard,
            pointer: PointerState::new(bounds),
            buttons: ButtonState::default(),
            scroll: ScrollState::default(),
            effects: Effects::default(),
            metrics: Metrics::default(),
            buttons_challenge: ButtonsChallenge::new(&Capabilities::default()),
            scroll_challenge: ScrollChallenge::from_capabilities(None),
            devices: Vec::new(),
            primary: None,
            log: VecDeque::with_capacity(LOG_CAPACITY),
            overflow_events: 0,
            last_source: None,
            last_event_us: 0,
            wheel_frame: WheelFrame::default(),
        }
    }

    pub fn add_device(&mut self, info: DeviceInfo) {
        let id = info.id;
        let candidate = matches!(info.class, DeviceClass::Mouse | DeviceClass::Combo);
        self.devices.retain(|device| device.id != id);
        self.devices.push(info);
        if self.primary.is_none() && candidate {
            self.primary = Some(id);
            self.refresh_challenges();
        }
    }

    pub fn sync_devices(&mut self, devices: Vec<DeviceInfo>, primary: Option<DeviceId>) {
        let old_primary = self.primary;
        self.devices = devices;
        self.primary = primary.filter(|id| self.devices.iter().any(|device| device.id == *id));
        if self.primary != old_primary {
            self.pointer.recenter();
            self.refresh_challenges();
        }
    }

    pub fn select_primary(&mut self, id: DeviceId) {
        if self.devices.iter().any(|device| device.id == id) {
            self.primary = Some(id);
            self.pointer.recenter();
            self.refresh_challenges();
        }
    }

    pub fn primary_device(&self) -> Option<&DeviceInfo> {
        let primary = self.primary?;
        self.devices.iter().find(|device| device.id == primary)
    }

    pub fn remove_device(&mut self, id: DeviceId) {
        self.devices.retain(|device| device.id != id);
        if self.primary == Some(id) {
            self.primary = self
                .devices
                .iter()
                .find(|device| matches!(device.class, DeviceClass::Mouse | DeviceClass::Combo))
                .or_else(|| self.devices.first())
                .map(|device| device.id);
            self.pointer.recenter();
            self.refresh_challenges();
        }
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub fn handle_input(&mut self, event: &InputEvent) {
        self.last_source = Some(event.source);
        self.last_event_us = event.time_us;

        match &event.kind {
            InputKind::Move { dx, dy } => {
                self.metrics.record_event();
                self.pointer.apply_move(*dx, *dy, event.time_us);
                self.buttons.move_cursor(self.pointer.pos());
                self.push_log(event, LogKind::Move { dx: *dx, dy: *dy });
            }
            InputKind::Absolute { x, y } => {
                self.metrics.record_event();
                self.pointer.set_normalized(*x, *y, event.time_us);
                self.buttons.move_cursor(self.pointer.pos());
            }
            InputKind::Button { code, pressed } => {
                self.metrics.record_event();
                let update = self
                    .buttons
                    .apply(*code, *pressed, event.time_us, self.pointer.pos());
                if update.pressed {
                    if self.mode == Mode::Buttons {
                        self.buttons_challenge.observe(*code);
                    }
                    self.effects.push(Effects::button(
                        *code,
                        self.pointer.pos(),
                        event.time_us,
                        update.double_click,
                    ));
                    if self.pointer.is_clamped() {
                        let edge = self.edge_for_cursor();
                        self.effects
                            .push(Effects::edge(edge, self.pointer.pos(), event.time_us));
                    }
                }
                if let Some(drag) = update.drag_released {
                    self.effects
                        .push(Effects::drag_release(drag, event.time_us));
                }
                self.push_log(
                    event,
                    LogKind::Button {
                        code: *code,
                        pressed: *pressed,
                    },
                );
            }
            InputKind::Wheel { axis, value, hires } => {
                self.metrics.record_event();
                self.wheel_frame.push(*axis, *value, *hires);
                self.push_log(
                    event,
                    LogKind::Wheel {
                        axis: *axis,
                        value: *value,
                        hires: *hires,
                    },
                );
            }
            InputKind::Report => {
                self.metrics.record_report(event.time_us);
                let updates = self.scroll.apply_frame(&self.wheel_frame, event.time_us);
                self.wheel_frame.clear();
                for update in &updates {
                    if let Some(effect) = Effects::wheel(update, self.pointer.pos(), event.time_us)
                    {
                        self.effects.push(effect);
                    }
                }
            }
            InputKind::DeviceAdded(info) => {
                self.add_device(info.clone());
                self.push_log(event, LogKind::DeviceAdded);
            }
            InputKind::DeviceRemoved(id) => {
                self.remove_device(*id);
                self.push_log(event, LogKind::DeviceRemoved);
            }
            InputKind::Overflow { dropped } => {
                self.overflow_events = self.overflow_events.saturating_add(*dropped);
                self.metrics.record_dropped(*dropped);
                self.push_log(event, LogKind::Overflow { dropped: *dropped });
            }
            InputKind::SourceError(message) => {
                self.push_log(event, LogKind::Error(message.clone()));
            }
        }
    }

    pub fn tick(&mut self, now_us: u64) {
        self.pointer.tick(now_us);
        self.effects.tick(now_us);
    }

    pub fn reset(&mut self) {
        self.pointer.reset();
        self.buttons.reset();
        self.scroll.reset();
        self.effects.clear();
        self.metrics.reset();
        self.wheel_frame.clear();
        self.log.clear();
        self.overflow_events = 0;
        self.last_event_us = 0;
        self.refresh_challenges();
    }

    fn push_log(&mut self, event: &InputEvent, kind: LogKind) {
        if self.log.len() >= LOG_CAPACITY {
            self.log.pop_front();
        }
        self.log.push_back(LogEntry {
            at_us: event.time_us,
            device: event.device,
            kind,
        });
    }

    fn refresh_challenges(&mut self) {
        let capabilities = self
            .primary_device()
            .map(|device| device.capabilities.clone())
            .unwrap_or_default();
        self.buttons_challenge = ButtonsChallenge::new(&capabilities);
        self.scroll_challenge = ScrollChallenge::from_capabilities(Some(&capabilities));
    }

    fn edge_for_cursor(&self) -> Edge {
        let pos = self.pointer.pos();
        let bounds = self.pointer.bounds();
        let left = (pos.x - bounds.min_x()).abs();
        let right = (pos.x - bounds.max_x()).abs();
        let top = (pos.y - bounds.min_y()).abs();
        let bottom = (pos.y - bounds.max_y()).abs();
        let min = left.min(right).min(top).min(bottom);
        if min == left {
            Edge::Left
        } else if min == right {
            Edge::Right
        } else if min == top {
            Edge::Top
        } else {
            Edge::Bottom
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::codes::BTN_LEFT;
    use crate::core::scroll::ScrollAxis;
    use crate::input::mock;

    fn session_with_mock() -> Session {
        let mut session = Session::new(Bounds::new(1000, 800).with_margin(0));
        session.add_device(DeviceInfo::mock(1));
        session
    }

    #[test]
    fn handles_move_and_updates_pointer() {
        let mut session = session_with_mock();
        session.handle_input(&mock::movement(1, 1_000, 10, 3));
        assert_eq!(session.pointer.raw_total(), (10, 3));
        assert_eq!(session.metrics.events(), 1);
    }

    #[test]
    fn handles_button_and_spawns_effect() {
        let mut session = session_with_mock();
        session.handle_input(&mock::button(1, 1_000, BTN_LEFT, true));
        assert!(session.buttons.is_pressed(BTN_LEFT));
        assert_eq!(session.effects.len(), 1);
    }

    #[test]
    fn wheel_is_applied_on_report() {
        let mut session = session_with_mock();
        session.handle_input(&mock::wheel(1, 1_000, ScrollAxis::Vertical, 2, false));
        assert_eq!(session.scroll.vertical_ticks(), 0);
        session.handle_input(&mock::report(1, 1_000));
        assert_eq!(session.scroll.vertical_ticks(), 2);
        assert_eq!(session.effects.len(), 1);
    }

    #[test]
    fn overflow_is_counted() {
        let mut session = session_with_mock();
        session.handle_input(&InputEvent::mock(
            1,
            1_000,
            InputKind::Overflow { dropped: 7 },
        ));
        assert_eq!(session.overflow_events, 7);
        assert_eq!(session.metrics.dropped(), 7);
    }

    #[test]
    fn removing_primary_selects_next_mouse() {
        let mut session = session_with_mock();
        session.add_device(DeviceInfo::mock(2));
        session.remove_device(1);
        assert_eq!(session.primary, Some(2));
    }

    #[test]
    fn reset_keeps_devices_and_primary() {
        let mut session = session_with_mock();
        session.handle_input(&mock::movement(1, 1_000, 10, 0));
        session.reset();
        assert_eq!(session.primary, Some(1));
        assert_eq!(session.devices.len(), 1);
        assert_eq!(session.pointer.raw_total(), (0, 0));
    }

    #[test]
    fn mode_cycles() {
        assert_eq!(Mode::Dashboard.next(), Mode::Buttons);
        assert_eq!(Mode::Dashboard.previous(), Mode::Targets);
        assert_eq!(Mode::from_number(3), Some(Mode::Scroll));
        assert_eq!(Mode::from_number(9), None);
    }
}
