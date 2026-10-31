use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

use crate::core::codes::{BTN_LEFT, BTN_MIDDLE, BTN_RIGHT};
use crate::core::device::{DeviceId, TUI_MOUSE_ID};
use crate::core::scroll::ScrollAxis;
use crate::input::{InputEvent, InputKind, InputSource};

pub const DEVICE_ID: DeviceId = TUI_MOUSE_ID;

pub fn map_mouse_event(
    event: MouseEvent,
    terminal_width: u16,
    terminal_height: u16,
    time_us: u64,
) -> Vec<InputEvent> {
    let width = terminal_width.max(1) as f32;
    let height = terminal_height.max(1) as f32;
    let x = (event.column as f32 / width).clamp(0.0, 1.0);
    let y = (event.row as f32 / height).clamp(0.0, 1.0);
    let event_for = |kind| InputEvent::new(DEVICE_ID, InputSource::TuiMouse, time_us, kind);
    let report = || event_for(InputKind::Report);

    match event.kind {
        MouseEventKind::Moved | MouseEventKind::Drag(_) => {
            vec![event_for(InputKind::Absolute { x, y })]
        }
        MouseEventKind::Down(button) | MouseEventKind::Up(button) => {
            let Some(code) = button_code(button) else {
                return Vec::new();
            };
            vec![
                event_for(InputKind::Button {
                    code,
                    pressed: matches!(event.kind, MouseEventKind::Down(_)),
                }),
                event_for(InputKind::Absolute { x, y }),
            ]
        }
        MouseEventKind::ScrollUp => vec![
            event_for(InputKind::Wheel {
                axis: ScrollAxis::Vertical,
                value: 1,
                hires: false,
            }),
            report(),
        ],
        MouseEventKind::ScrollDown => vec![
            event_for(InputKind::Wheel {
                axis: ScrollAxis::Vertical,
                value: -1,
                hires: false,
            }),
            report(),
        ],
        MouseEventKind::ScrollLeft => vec![
            event_for(InputKind::Wheel {
                axis: ScrollAxis::Horizontal,
                value: -1,
                hires: false,
            }),
            report(),
        ],
        MouseEventKind::ScrollRight => vec![
            event_for(InputKind::Wheel {
                axis: ScrollAxis::Horizontal,
                value: 1,
                hires: false,
            }),
            report(),
        ],
    }
}

fn button_code(button: MouseButton) -> Option<u16> {
    match button {
        MouseButton::Left => Some(BTN_LEFT),
        MouseButton::Right => Some(BTN_RIGHT),
        MouseButton::Middle => Some(BTN_MIDDLE),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mouse(kind: MouseEventKind, column: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind,
            column,
            row,
            modifiers: crossterm::event::KeyModifiers::NONE,
        }
    }

    #[test]
    fn maps_click_and_position() {
        let events = map_mouse_event(
            mouse(MouseEventKind::Down(MouseButton::Left), 10, 20),
            100,
            40,
            42,
        );
        assert_eq!(events.len(), 2);
        assert!(matches!(
            events[0].kind,
            InputKind::Button {
                code: BTN_LEFT,
                pressed: true
            }
        ));
        match events[1].kind {
            InputKind::Absolute { x, y } => {
                assert!((x - 0.10).abs() < 0.001);
                assert!((y - 0.50).abs() < 0.001);
            }
            _ => panic!("expected absolute"),
        }
    }

    #[test]
    fn maps_wheel_with_report() {
        let events = map_mouse_event(mouse(MouseEventKind::ScrollDown, 0, 0), 80, 24, 1);
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0].kind, InputKind::Wheel { value: -1, .. }));
        assert!(matches!(events[1].kind, InputKind::Report));
    }

    #[test]
    fn clamps_edges() {
        let events = map_mouse_event(mouse(MouseEventKind::Moved, 200, 100), 100, 40, 1);
        match events[0].kind {
            InputKind::Absolute { x, y } => {
                assert_eq!(x, 1.0);
                assert_eq!(y, 1.0);
            }
            _ => panic!("expected absolute"),
        }
    }
}
