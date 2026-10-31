use std::process::Command;

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_mouse_test"))
}

#[test]
fn list_devices_prints_a_table() {
    let output = binary().arg("--list-devices").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PATH"));
    assert!(stdout.contains("CLASS"));
    assert!(stdout.contains("NAME"));
    assert!(stdout.contains("PERM"));
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
fn all_devices_flag_lists_keyboards_too() {
    let output = binary()
        .arg("--list-devices")
        .arg("--all-devices")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PATH"));
}

#[test]
fn version_is_available() {
    let output = binary().arg("--version").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("mouse_test"));
}

#[test]
fn capture_frame_writes_png_without_tty() {
    let path = std::env::temp_dir().join(format!(
        "mouse_test-capture-{}-{}.png",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = std::fs::remove_file(&path);
    let output = binary()
        .arg("--capture-frame")
        .arg(&path)
        .arg("--seed")
        .arg("42")
        .output()
        .unwrap();
    assert!(output.status.success());
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(
        &bytes[..8],
        &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]
    );
    assert_eq!(u32::from_be_bytes(bytes[16..20].try_into().unwrap()), 1344);
    assert_eq!(u32::from_be_bytes(bytes[20..24].try_into().unwrap()), 756);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn capture_frame_invalid_path_exits_one() {
    let output = binary()
        .arg("--capture-frame")
        .arg("/proc/mouse_test/denied.png")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
}
