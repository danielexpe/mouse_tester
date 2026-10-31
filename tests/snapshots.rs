use std::fs;
use std::path::PathBuf;

use mouse_test::app::{App, AppConfig, UiFlags};
use mouse_test::core::geometry::Bounds;
use mouse_test::i18n::Lang;
use mouse_test::render::render;
use mouse_test::state::Mode;
use mouse_test::theme::{ColorTier, Theme};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn app(lang: Lang, tier: ColorTier) -> App {
    App::new(AppConfig {
        bounds: Bounds::new(1344, 756),
        lang,
        theme: Theme::new(tier),
        flags: UiFlags::default(),
        initial_mode: Mode::Targets,
        seed: 7,
        demo_seed: None,
        controller: None,
        grab: false,
        grab_force: false,
    })
}

fn buffer_text(width: u16, height: u16, app: &App) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| render(frame, app)).unwrap();
    let buffer = terminal.backend().buffer();
    let mut text = String::new();
    for row in 0..height {
        for column in 0..width {
            let cell = &buffer.content[(row as usize) * (width as usize) + column as usize];
            text.push_str(cell.symbol());
        }
        text.push('\n');
    }
    text
}

fn snapshot(name: &str, width: u16, height: u16, mut app: App) {
    app.update(1_000_000);
    let text = buffer_text(width, height, &app);
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("snapshots");
    path.push(format!("{name}.txt"));

    if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, &text).unwrap();
        return;
    }

    let expected = fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "missing snapshot {} ({error}); run with UPDATE_SNAPSHOTS=1",
            path.display()
        )
    });
    assert_eq!(text, expected, "snapshot mismatch: {}", path.display());
}

#[test]
fn targets_pt_wide() {
    snapshot(
        "targets_pt_wide",
        120,
        36,
        app(Lang::PtBr, ColorTier::TrueColor),
    );
}

#[test]
fn targets_en_normal() {
    snapshot(
        "targets_en_normal",
        100,
        30,
        app(Lang::En, ColorTier::TrueColor),
    );
}

#[test]
fn targets_mono_normal() {
    snapshot(
        "targets_mono_normal",
        100,
        30,
        app(Lang::PtBr, ColorTier::Mono),
    );
}

#[test]
fn degraded_dashboard_pt_wide() {
    let mut app = app(Lang::PtBr, ColorTier::TrueColor);
    app.session.set_mode(Mode::Dashboard);
    app.degraded = true;
    snapshot("degraded_dashboard_pt_wide", 120, 36, app);
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[test]
fn capture_png_matches_golden_hash() {
    let png =
        mouse_test::render::capture::encode_png(&mouse_test::render::capture::render_capture(42));
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("snapshots");
    path.push("capture_seed42.fnv");
    let value = format!("{:016x}\n", fnv1a64(&png));

    if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, &value).unwrap();
        return;
    }

    let expected = fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "missing capture snapshot {} ({error}); run with UPDATE_SNAPSHOTS=1",
            path.display()
        )
    });
    assert_eq!(value, expected, "capture snapshot mismatch");
}
