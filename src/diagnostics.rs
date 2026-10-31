use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::Path;

use crate::input::evdev::discover;
use crate::platform::evdev::is_readable;

pub fn write_report(path: &Path) -> io::Result<()> {
    fs::write(path, render_report())
}

pub fn render_report() -> String {
    let mut out = String::new();
    let _ = writeln!(out, "mousescope diagnostics");
    let _ = writeln!(out, "version={}", env!("CARGO_PKG_VERSION"));
    let _ = writeln!(out, "backend=tui");
    let _ = writeln!(out, "session={}", env_or("XDG_SESSION_TYPE", "unknown"));
    let _ = writeln!(out, "desktop={}", env_or("XDG_CURRENT_DESKTOP", "unknown"));
    let _ = writeln!(out, "term={}", env_or("TERM", "unknown"));
    let _ = writeln!(out, "lang={}", env_or("LANG", "unknown"));
    let _ = writeln!(out, "tty={}", tty());
    let _ = writeln!(out, "groups={}", groups());
    let _ = writeln!(out, "os={}", os_release());

    let fb = Path::new("/dev/fb0");
    let _ = writeln!(out, "fb.device={}", fb.display());
    let _ = writeln!(out, "fb.exists={}", fb.exists());
    let _ = writeln!(out, "fb.readable={}", is_readable(fb));
    let _ = writeln!(out, "fb.name={}", sysfs("/sys/class/graphics/fb0/name"));
    let _ = writeln!(
        out,
        "fb.virtual_size={}",
        sysfs("/sys/class/graphics/fb0/virtual_size")
    );
    let _ = writeln!(
        out,
        "fb.bits_per_pixel={}",
        sysfs("/sys/class/graphics/fb0/bits_per_pixel")
    );
    let _ = writeln!(out, "fb.stride={}", sysfs("/sys/class/graphics/fb0/stride"));
    let _ = writeln!(out, "fb.blank={}", sysfs("/sys/class/graphics/fb0/blank"));

    let discovery = discover();
    let _ = writeln!(out, "discovery.source={:?}", discovery.source);
    let _ = writeln!(out, "input.devices={}", discovery.devices.len());
    for (index, device) in discovery.devices.iter().enumerate() {
        let _ = writeln!(out, "device.{index}.path={}", device.path.display());
        let _ = writeln!(out, "device.{index}.name={}", device.name);
        let _ = writeln!(out, "device.{index}.class={}", device.class.as_str());
        let _ = writeln!(
            out,
            "device.{index}.ids={:04x}:{:04x}",
            device.vendor, device.product
        );
        let _ = writeln!(
            out,
            "device.{index}.phys={}",
            device.phys.as_deref().unwrap_or("")
        );
        let _ = writeln!(
            out,
            "device.{index}.uniq={}",
            device.uniq.as_deref().unwrap_or("")
        );
        let _ = writeln!(out, "device.{index}.readable={}", device.readable);
        let _ = writeln!(
            out,
            "device.{index}.keys={}",
            device.capabilities.keys.len()
        );
        let _ = writeln!(out, "device.{index}.rel={}", device.capabilities.rel.len());
        let _ = writeln!(out, "device.{index}.abs={}", device.capabilities.abs.len());
    }
    for warning in &discovery.warnings {
        let _ = writeln!(out, "warning={warning}");
    }
    let _ = writeln!(out, "notes=no keyboard events are read or stored");
    out
}

fn env_or(key: &str, fallback: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| fallback.to_string())
}

fn tty() -> String {
    fs::read_link("/proc/self/fd/0")
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

fn groups() -> String {
    fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|text| {
            text.lines()
                .find(|line| line.starts_with("Groups:"))
                .map(|line| line.trim_start_matches("Groups:").trim().to_string())
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn os_release() -> String {
    let text = fs::read_to_string("/etc/os-release").unwrap_or_default();
    for line in text.lines() {
        if let Some(value) = line.strip_prefix("PRETTY_NAME=") {
            return value.trim_matches('"').to_string();
        }
    }
    "unknown".to_string()
}

fn sysfs(path: &str) -> String {
    fs::read_to_string(path)
        .map(|value| value.trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_has_expected_sections() {
        let report = render_report();
        assert!(report.contains("mousescope diagnostics"));
        assert!(report.contains("version="));
        assert!(report.contains("input.devices="));
        assert!(report.contains("notes="));
    }
}
