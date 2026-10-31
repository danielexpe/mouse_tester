use std::io::{stdout, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use clap::Parser;
use crossterm::event::{self, Event};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use mouse_tester::app::{App, AppConfig, UiFlags};
use mouse_tester::cli::{BackendArg, Cli, TuiMouseArg};
use mouse_tester::core::device::DeviceInfo;
use mouse_tester::core::geometry::Bounds;
use mouse_tester::input::{discover, InputController, InputError, Selection};
use mouse_tester::platform::fb::{self, FbConsole, FbDevice, FbError};
use mouse_tester::platform::signals;
use mouse_tester::platform::{install_panic_hook, set_mouse_capture, TuiGuard};
use mouse_tester::theme::Theme;

#[derive(Debug, thiserror::Error)]
enum RunError {
    #[error("backend unavailable: {0}")]
    Backend(String),
    #[error("TUI unavailable: {0}")]
    Tui(String),
    #[error("input unavailable: {0}")]
    Input(String),
    #[error("framebuffer permission: {0}")]
    FbPermission(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl RunError {
    fn exit_code(&self) -> u8 {
        match self {
            Self::Backend(_) | Self::Tui(_) => 3,
            Self::Input(_) | Self::FbPermission(_) => 4,
            Self::Io(_) => 1,
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("mouse_tester: {error}");
            ExitCode::from(error.exit_code())
        }
    }
}

fn run(cli: Cli) -> Result<(), RunError> {
    if cli.list_devices {
        list_devices(cli.all_devices)?;
        return Ok(());
    }

    if let Some(path) = &cli.diagnostics {
        mouse_tester::diagnostics::write_report(path)?;
        return Ok(());
    }

    if let Some(path) = &cli.capture_frame {
        mouse_tester::render::capture::write_capture(path, cli.seed)?;
        return Ok(());
    }

    match cli.backend {
        BackendArg::Fb => run_fb(cli),
        BackendArg::Auto | BackendArg::Tui => run_tui(cli),
    }
}

fn map_fb_error(error: FbError) -> RunError {
    match &error {
        FbError::Io(io_error) if io_error.kind() == std::io::ErrorKind::PermissionDenied => {
            RunError::FbPermission(error.to_string())
        }
        _ => RunError::Backend(error.to_string()),
    }
}

fn run_fb(cli: Cli) -> Result<(), RunError> {
    if !fb::is_vt() {
        return Err(RunError::Backend(
            "--backend fb requires a local text VT (/dev/ttyN); run it from tty2..tty63"
                .to_string(),
        ));
    }

    let controller = create_evdev_controller(&cli)?;
    let mut device = FbDevice::open(&cli.fb_device).map_err(map_fb_error)?;
    fb::install_panic_hook();
    signals::install()?;
    signals::clear();
    let _console = FbConsole::enter().map_err(|error| {
        if error.kind() == std::io::ErrorKind::PermissionDenied {
            RunError::FbPermission(format!("/dev/tty: {error}"))
        } else {
            RunError::Backend(format!("/dev/tty: {error}"))
        }
    })?;
    device.unblank().map_err(map_fb_error)?;

    let flags = UiFlags {
        no_animations: cli.no_animations,
        no_trail: cli.no_trail,
        raw_log: cli.raw_log,
    };
    let mut app = App::new(AppConfig {
        bounds: Bounds::new(1344, 756),
        lang: cli.language(),
        theme: Theme::new(cli.color_tier()),
        flags,
        initial_mode: cli.mode.mode(),
        seed: cli.seed,
        demo_seed: if cli.demo { Some(cli.seed) } else { None },
        controller,
        grab: cli.grab,
        grab_force: cli.grab_force,
    });

    let started = Instant::now();
    loop {
        if signals::triggered() {
            break;
        }
        let now_us = started.elapsed().as_micros() as u64;
        app.update(now_us);
        let frame = mouse_tester::render::fb::render(&app, device.xres(), device.yres());
        device.present(&frame).map_err(map_fb_error)?;

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

fn list_devices(all_devices: bool) -> Result<(), RunError> {
    let discovery = discover();
    println!("PATH\tCLASS\tNAME\tPERM");
    for device in &discovery.devices {
        if !all_devices && !device.is_pointing() {
            continue;
        }
        println!(
            "{}\t{}\t{}\t{}",
            device.path.display(),
            device.class.as_str(),
            device.name,
            if device.readable { "ok" } else { "denied" }
        );
    }
    if discovery.devices.is_empty() {
        eprintln!("mouse_tester: no input devices found in /proc/bus/input/devices");
    }
    for warning in &discovery.warnings {
        eprintln!("mouse_tester: {warning}");
    }
    stdout().flush()?;
    Ok(())
}

fn run_tui(cli: Cli) -> Result<(), RunError> {
    let controller = create_controller(&cli)?;
    install_panic_hook();
    signals::install()?;
    signals::clear();
    let _guard = TuiGuard::enter().map_err(|error| RunError::Tui(error.to_string()))?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend).map_err(|error| RunError::Tui(error.to_string()))?;

    let flags = UiFlags {
        no_animations: cli.no_animations,
        no_trail: cli.no_trail,
        raw_log: cli.raw_log,
    };
    let mut app = App::new(AppConfig {
        bounds: Bounds::new(1344, 756),
        lang: cli.language(),
        theme: Theme::new(cli.color_tier()),
        flags,
        initial_mode: cli.mode.mode(),
        seed: cli.seed,
        demo_seed: if cli.demo { Some(cli.seed) } else { None },
        controller,
        grab: cli.grab,
        grab_force: cli.grab_force,
    });

    let started = Instant::now();
    let mut mouse_capture = false;
    loop {
        if signals::triggered() {
            break;
        }
        let now_us = started.elapsed().as_micros() as u64;
        app.update(now_us);

        let want_mouse = app.degraded && cli.tui_mouse != TuiMouseArg::Off;
        if want_mouse != mouse_capture {
            set_mouse_capture(want_mouse);
            mouse_capture = want_mouse;
        }

        terminal
            .draw(|frame| mouse_tester::render::render(frame, &app))
            .map_err(RunError::Io)?;

        if app.quit {
            break;
        }

        let timeout = if app.demo_active || want_mouse {
            Duration::from_millis(16)
        } else {
            Duration::from_millis(100)
        };
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => app.handle_key(key),
                Event::Mouse(mouse) if mouse_capture => {
                    let size = terminal.size()?;
                    let events = mouse_tester::input::tui_mouse::map_mouse_event(
                        mouse,
                        size.width,
                        size.height,
                        now_us,
                    );
                    app.handle_input_events(&events);
                }
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
    }

    if mouse_capture {
        set_mouse_capture(false);
    }

    Ok(())
}

fn create_controller(cli: &Cli) -> Result<Option<InputController>, RunError> {
    if cli.demo {
        return Ok(None);
    }

    if cli.tui_mouse == TuiMouseArg::On {
        if cli.require_evdev {
            return Err(RunError::Input(
                "--require-evdev conflicts with --tui-mouse on".to_string(),
            ));
        }
        return Ok(Some(InputController::degraded(
            "terminal mouse requested".to_string(),
        )));
    }

    let discovery = discover();
    let selection = selection_from_cli(cli, &discovery.devices)?;
    match InputController::start(selection, cli.grab, cli.grab_force) {
        Ok(controller) => {
            if cli.require_evdev && controller.status().primary.is_none() {
                return Err(RunError::Input(
                    "evdev required but no readable mouse was found".to_string(),
                ));
            }
            Ok(Some(controller))
        }
        Err(error) => {
            let can_fallback = matches!(error, InputError::Permission { .. })
                && cli.tui_mouse != TuiMouseArg::Off
                && !cli.require_evdev;
            if can_fallback {
                Ok(Some(InputController::degraded(error.to_string())))
            } else {
                Err(RunError::Input(error.to_string()))
            }
        }
    }
}

fn create_evdev_controller(cli: &Cli) -> Result<Option<InputController>, RunError> {
    if cli.demo {
        return Ok(None);
    }
    let discovery = discover();
    let selection = selection_from_cli(cli, &discovery.devices)?;
    match InputController::start(selection, cli.grab, cli.grab_force) {
        Ok(controller) if controller.status().primary.is_some() => Ok(Some(controller)),
        Ok(_) => Err(RunError::Input(
            "no readable mouse found for the framebuffer backend".to_string(),
        )),
        Err(error) => Err(RunError::Input(error.to_string())),
    }
}

fn selection_from_cli(cli: &Cli, devices: &[DeviceInfo]) -> Result<Selection, RunError> {
    let Some(selector) = cli.device.as_deref() else {
        return Ok(Selection::Auto);
    };
    match selector {
        "auto" => return Ok(Selection::Auto),
        "all" => return Ok(Selection::All),
        _ => {}
    }

    let path = if selector.contains('/') {
        PathBuf::from(selector)
    } else {
        let wanted = selector.to_ascii_lowercase();
        devices
            .iter()
            .find(|device| device.name.eq_ignore_ascii_case(selector))
            .or_else(|| {
                devices
                    .iter()
                    .find(|device| device.name.to_ascii_lowercase().contains(&wanted))
            })
            .map(|device| device.path.clone())
            .ok_or_else(|| RunError::Input(format!("device not found: {selector}")))?
    };
    Ok(Selection::Path(path))
}
