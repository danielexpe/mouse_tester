use std::process::Command;

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_mousescope"))
}

#[test]
fn list_devices_reports_mock_device() {
    let output = binary().arg("--list-devices").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("/dev/input/mock1"));
    assert!(stdout.contains("mouse"));
}

#[test]
fn framebuffer_backend_is_reported_as_unavailable() {
    let output = binary().arg("--backend").arg("fb").output().unwrap();
    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Phase 4"));
}

#[test]
fn tui_requires_a_tty() {
    let output = binary().arg("--demo").output().unwrap();
    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("TTY"));
}

#[test]
fn version_is_available() {
    let output = binary().arg("--version").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("mousescope"));
}
