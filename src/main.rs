use std::io::{stdout, Write};
use std::process::ExitCode;
use std::time::{Duration, Instant};

use clap::Parser;
use crossterm::event::{self, Event};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use mousescope::app::{App, UiFlags};
use mousescope::cli::{BackendArg, Cli};
use mousescope::core::device::{DeviceClass, DeviceInfo};
use mousescope::core::geometry::Bounds;
use mousescope::platform::{install_panic_hook, TuiGuard};
use mousescope::theme::Theme;

#[derive(Debug, thiserror::Error)]
enum RunError {
    #[error("backend unavailable: {0}")]
    Backend(String),
    #[error("TUI unavailable: {0}")]
    Tui(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl RunError {
    fn exit_code(&self) -> u8 {
        match self {
            Self::Backend(_) | Self::Tui(_) => 3,
            Self::Io(_) => 1,
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("mousescope: {error}");
            ExitCode::from(error.exit_code())
        }
    }
}

fn run(cli: Cli) -> Result<(), RunError> {
    if cli.list_devices {
        list_devices(cli.all_devices)?;
        return Ok(());
    }

    match cli.backend {
        BackendArg::Fb => Err(RunError::Backend(
            "the framebuffer backend is planned for Phase 4; use --backend tui".to_string(),
        )),
        BackendArg::Auto | BackendArg::Tui => run_tui(cli),
    }
}

fn list_devices(all_devices: bool) -> Result<(), RunError> {
    let device = DeviceInfo::mock(1);
    println!("PATH\tCLASS\tNAME\tPERM");
    println!(
        "{}\t{}\t{}\tok",
        device.path.display(),
        class_name(device.class),
        device.name
    );
    if all_devices {
        println!("# Phase 1 exposes only the mock device; evdev discovery arrives in Phase 2");
    }
    stdout().flush()?;
    Ok(())
}

fn class_name(class: DeviceClass) -> &'static str {
    match class {
        DeviceClass::Mouse => "mouse",
        DeviceClass::Touchpad => "touchpad",
        DeviceClass::Tablet => "tablet",
        DeviceClass::Keyboard => "keyboard",
        DeviceClass::Joystick => "joystick",
        DeviceClass::Combo => "combo",
        DeviceClass::Unknown => "unknown",
    }
}

fn run_tui(cli: Cli) -> Result<(), RunError> {
    install_panic_hook();
    let _guard = TuiGuard::enter().map_err(|error| RunError::Tui(error.to_string()))?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend).map_err(|error| RunError::Tui(error.to_string()))?;

    let flags = UiFlags {
        no_animations: cli.no_animations,
        no_trail: cli.no_trail,
        raw_log: cli.raw_log,
    };
    let mut app = App::new(
        Bounds::new(1344, 756),
        cli.language(),
        Theme::new(cli.color_tier()),
        flags,
        cli.mode.mode(),
        if cli.demo { Some(cli.seed) } else { None },
    );

    let started = Instant::now();
    loop {
        let now_us = started.elapsed().as_micros() as u64;
        app.update(now_us);
        terminal
            .draw(|frame| mousescope::render::render(frame, &app))
            .map_err(RunError::Io)?;

        if app.quit {
            break;
        }

        let timeout = if app.demo_active {
            Duration::from_millis(16)
        } else {
            Duration::from_millis(100)
        };
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => app.handle_key(key),
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
    }

    Ok(())
}
