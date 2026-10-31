pub mod controller;
pub mod evdev;
pub mod mock;
pub mod tui_mouse;

pub use controller::InputController;
pub use evdev::{discover, Discovery, EvdevRuntime, InputError, InputMode, InputStatus, Selection};
pub use mock::DemoSource;

use crate::core::device::{DeviceId, DeviceInfo};
use crate::core::scroll::ScrollAxis;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputSource {
    Mock,
    Evdev,
    TuiMouse,
}

impl InputSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::Mock => "mock",
            Self::Evdev => "evdev",
            Self::TuiMouse => "tui-mouse",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum InputKind {
    Move {
        dx: i32,
        dy: i32,
    },
    Absolute {
        x: f32,
        y: f32,
    },
    Button {
        code: u16,
        pressed: bool,
    },
    Wheel {
        axis: ScrollAxis,
        value: i32,
        hires: bool,
    },
    Report,
    DeviceAdded(DeviceInfo),
    DeviceRemoved(DeviceId),
    Overflow {
        dropped: u64,
    },
    SourceError(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct InputEvent {
    pub device: DeviceId,
    pub source: InputSource,
    pub time_us: u64,
    pub kind: InputKind,
}

impl InputEvent {
    pub fn new(device: DeviceId, source: InputSource, time_us: u64, kind: InputKind) -> Self {
        Self {
            device,
            source,
            time_us,
            kind,
        }
    }

    pub fn mock(device: DeviceId, time_us: u64, kind: InputKind) -> Self {
        Self::new(device, InputSource::Mock, time_us, kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_events_carry_source_and_device() {
        let event = InputEvent::mock(4, 123, InputKind::Move { dx: 1, dy: -1 });
        assert_eq!(event.device, 4);
        assert_eq!(event.source, InputSource::Mock);
        assert_eq!(event.time_us, 123);
    }
}
