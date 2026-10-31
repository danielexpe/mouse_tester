use std::io;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

static TRIGGERED: AtomicBool = AtomicBool::new(false);
static LAST_SIGNAL: AtomicI32 = AtomicI32::new(0);

extern "C" fn handler(signal: i32) {
    LAST_SIGNAL.store(signal, Ordering::SeqCst);
    TRIGGERED.store(true, Ordering::SeqCst);
}

pub fn install() -> io::Result<()> {
    for signal in [libc::SIGINT, libc::SIGTERM, libc::SIGHUP] {
        // SAFETY: the handler only writes atomics, which is async-signal-safe.
        let previous = unsafe { libc::signal(signal, handler as *const () as libc::sighandler_t) };
        if previous == libc::SIG_ERR {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}

pub fn triggered() -> bool {
    TRIGGERED.load(Ordering::SeqCst)
}

pub fn last_signal() -> Option<i32> {
    let signal = LAST_SIGNAL.load(Ordering::SeqCst);
    if signal == 0 {
        None
    } else {
        Some(signal)
    }
}

pub fn clear() {
    LAST_SIGNAL.store(0, Ordering::SeqCst);
    TRIGGERED.store(false, Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_resets_flag() {
        TRIGGERED.store(true, Ordering::SeqCst);
        LAST_SIGNAL.store(libc::SIGTERM, Ordering::SeqCst);
        clear();
        assert!(!triggered());
        assert_eq!(last_signal(), None);
    }
}
