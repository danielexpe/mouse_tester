use crate::core::device::DeviceId;
use crate::core::scroll::ScrollAxis;

pub const LOG_CAPACITY: usize = 200;

#[derive(Clone, Debug, PartialEq)]
pub enum LogKind {
    Move {
        dx: i32,
        dy: i32,
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
    DeviceAdded,
    DeviceRemoved,
    Overflow {
        dropped: u64,
    },
    Error(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct LogEntry {
    pub at_us: u64,
    pub device: DeviceId,
    pub kind: LogKind,
}
