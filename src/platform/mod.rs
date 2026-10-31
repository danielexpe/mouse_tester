pub mod evdev;
pub mod tty;

pub use tty::{install_panic_hook, restore_terminal, set_mouse_capture, TuiGuard};
