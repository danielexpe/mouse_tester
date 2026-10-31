use std::io::{self, stdin, stdout, IsTerminal};
use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};

use crossterm::cursor::{Hide, Show};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};

static ACTIVE: AtomicBool = AtomicBool::new(false);

pub struct TuiGuard;

impl TuiGuard {
    pub fn enter() -> io::Result<Self> {
        if !stdin().is_terminal() || !stdout().is_terminal() {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "stdin and stdout must be a TTY",
            ));
        }

        enable_raw_mode()?;
        if let Err(error) = execute!(stdout(), EnterAlternateScreen, Hide) {
            let _ = disable_raw_mode();
            return Err(error);
        }

        ACTIVE.store(true, Ordering::SeqCst);
        Ok(Self)
    }
}

impl Drop for TuiGuard {
    fn drop(&mut self) {
        restore_terminal();
    }
}

pub fn install_panic_hook() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        restore_terminal();
        default_hook(info);
    }));
}

pub fn restore_terminal() {
    if !ACTIVE.swap(false, Ordering::SeqCst) {
        return;
    }
    let _ = disable_raw_mode();
    let _ = execute!(stdout(), DisableMouseCapture, Show, LeaveAlternateScreen);
}

pub fn set_mouse_capture(enabled: bool) {
    if enabled {
        let _ = execute!(stdout(), EnableMouseCapture);
    } else {
        let _ = execute!(stdout(), DisableMouseCapture);
    }
}
