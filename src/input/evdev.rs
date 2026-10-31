use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TrySendError};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::core::codes::is_mouse_button;
use crate::core::device::{classify, Capabilities, DeviceClass, DeviceId, DeviceInfo};
use crate::core::scroll::ScrollAxis;
use crate::input::{InputEvent, InputKind, InputSource};
use crate::platform::evdev::{
    is_readable, poll_fd, poll_readable, AbsInfo, EvdevDevice, RawInputEvent, ABS_X, ABS_Y, EV_ABS,
    EV_KEY, EV_REL, EV_SYN, REL_HWHEEL, REL_HWHEEL_HI_RES, REL_WHEEL, REL_WHEEL_HI_RES, REL_X,
    REL_Y, SYN_DROPPED, SYN_REPORT,
};

const SCAN_INTERVAL: Duration = Duration::from_secs(1);
const POLL_TIMEOUT_MS: i32 = 100;
const CHANNEL_CAPACITY: usize = 8192;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Selection {
    Auto,
    Path(PathBuf),
    All,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputMode {
    Evdev,
    Waiting,
}

#[derive(Clone, Debug)]
pub struct InputStatus {
    pub mode: InputMode,
    pub devices: Vec<DeviceInfo>,
    pub primary: Option<DeviceId>,
    pub message: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum InputError {
    #[error("permission denied for {path}; add the user to the input group and log in again")]
    Permission { path: PathBuf },
    #[error("no compatible mouse found")]
    NoDevice,
    #[error("grab refused for combo device {path}")]
    ComboGrab { path: PathBuf },
    #[error("device {path} is not a pointing device")]
    Unsupported { path: PathBuf },
    #[error("evdev I/O error: {0}")]
    Io(#[from] io::Error),
}

#[derive(Clone, Debug, Default)]
pub struct Discovery {
    pub devices: Vec<DeviceInfo>,
    pub source: DiscoverySource,
    pub warnings: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DiscoverySource {
    #[default]
    None,
    ProcBusInput,
    Sysfs,
}

pub fn discover() -> Discovery {
    match fs::read_to_string("/proc/bus/input/devices") {
        Ok(text) => {
            let mut devices = parse_proc_devices(&text);
            finish_discovery(&mut devices, DiscoverySource::ProcBusInput)
        }
        Err(error) => {
            let mut discovery = discover_sysfs();
            discovery
                .warnings
                .push(format!("cannot read /proc/bus/input/devices: {error}"));
            discovery
        }
    }
}

pub fn parse_proc_devices(text: &str) -> Vec<DeviceInfo> {
    let mut devices = Vec::new();
    for block in text.split("\n\n") {
        if block.trim().is_empty() {
            continue;
        }
        if let Some(device) = parse_block(block) {
            devices.push(device);
        }
    }
    devices
}

fn parse_block(block: &str) -> Option<DeviceInfo> {
    let mut name = String::from("unknown");
    let mut phys = None;
    let mut uniq = None;
    let mut vendor = 0;
    let mut product = 0;
    let mut path = None;
    let mut caps = Capabilities::default();

    for line in block.lines() {
        if let Some(value) = line.strip_prefix("I:") {
            for part in value.split_whitespace() {
                if let Some((key, raw)) = part.split_once('=') {
                    let parsed = u16::from_str_radix(raw, 16).unwrap_or(0);
                    match key {
                        "Vendor" => vendor = parsed,
                        "Product" => product = parsed,
                        _ => {}
                    }
                }
            }
        } else if let Some(value) = line.strip_prefix("N: Name=") {
            name = value.trim_matches('"').to_string();
        } else if let Some(value) = line.strip_prefix("P: Phys=") {
            phys = Some(value.trim().to_string());
        } else if let Some(value) = line.strip_prefix("U: Uniq=") {
            let value = value.trim();
            if !value.is_empty() {
                uniq = Some(value.to_string());
            }
        } else if let Some(value) = line.strip_prefix("H: Handlers=") {
            for handler in value.split_whitespace() {
                if handler.starts_with("event") {
                    path = Some(PathBuf::from("/dev/input").join(handler));
                    break;
                }
            }
        } else if let Some(value) = line.strip_prefix("B: PROP=") {
            caps.props = parse_bitmap(value);
        } else if let Some(value) = line.strip_prefix("B: KEY=") {
            caps.keys = parse_bitmap(value);
        } else if let Some(value) = line.strip_prefix("B: REL=") {
            caps.rel = parse_bitmap(value);
        } else if let Some(value) = line.strip_prefix("B: ABS=") {
            caps.abs = parse_bitmap(value);
        }
    }

    let path = path?;
    let class = classify(&caps);
    Some(DeviceInfo {
        id: 0,
        path,
        name,
        phys,
        uniq,
        vendor,
        product,
        class,
        capabilities: caps,
        readable: false,
    })
}

fn parse_bitmap(value: &str) -> BTreeSet<u16> {
    let words: Vec<&str> = value.split_whitespace().collect();
    let mut set = BTreeSet::new();
    for (word_index, word) in words.iter().rev().enumerate() {
        let word = u64::from_str_radix(word, 16).unwrap_or(0);
        for bit in 0..64 {
            if word & (1u64 << bit) != 0 {
                if let Ok(code) = u16::try_from(word_index * 64 + bit) {
                    set.insert(code);
                }
            }
        }
    }
    set
}

fn finish_discovery(devices: &mut [DeviceInfo], source: DiscoverySource) -> Discovery {
    devices.sort_by(|left, right| {
        class_rank(left.class)
            .cmp(&class_rank(right.class))
            .then_with(|| left.path.cmp(&right.path))
    });
    for (index, device) in devices.iter_mut().enumerate() {
        device.id = index as DeviceId + 1;
        device.readable = is_readable(&device.path);
    }
    Discovery {
        devices: devices.to_vec(),
        source,
        warnings: Vec::new(),
    }
}

fn class_rank(class: DeviceClass) -> u8 {
    match class {
        DeviceClass::Mouse | DeviceClass::Combo => 0,
        DeviceClass::Touchpad => 1,
        DeviceClass::Tablet => 2,
        DeviceClass::Keyboard => 3,
        DeviceClass::Joystick => 4,
        DeviceClass::Unknown => 5,
    }
}

fn discover_sysfs() -> Discovery {
    let mut devices = Vec::new();
    let mut warnings = Vec::new();
    let Ok(entries) = fs::read_dir("/sys/class/input") else {
        return Discovery {
            devices,
            source: DiscoverySource::None,
            warnings,
        };
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with("event") {
            continue;
        }
        let base = entry.path().join("device");
        let device_name = read_sysfs(&base.join("name")).unwrap_or_else(|| "unknown".to_string());
        let vendor = read_hex(&base.join("id/vendor"));
        let product = read_hex(&base.join("id/product"));
        let phys = read_sysfs(&base.join("phys"));
        let uniq = read_sysfs(&base.join("uniq"));
        let path = PathBuf::from("/dev/input").join(name.as_ref());
        let class = DeviceClass::Unknown;
        let readable = is_readable(&path);
        let mut capabilities = Capabilities::default();
        if readable {
            if let Ok(device) = EvdevDevice::open(&path) {
                if let Ok(caps) = device.capabilities() {
                    capabilities = capabilities_from_platform(caps);
                }
            }
        }
        devices.push(DeviceInfo {
            id: 0,
            path,
            name: device_name,
            phys: phys.filter(|value| !value.is_empty()),
            uniq: uniq.filter(|value| !value.is_empty()),
            vendor,
            product,
            class,
            capabilities,
            readable,
        });
    }

    let mut discovery = finish_discovery(&mut devices, DiscoverySource::Sysfs);
    discovery.warnings.append(&mut warnings);
    discovery
}

fn read_sysfs(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn read_hex(path: &Path) -> u16 {
    read_sysfs(path)
        .and_then(|value| u16::from_str_radix(value.trim_start_matches("0x"), 16).ok())
        .unwrap_or(0)
}

fn capabilities_from_platform(caps: crate::platform::evdev::DeviceCaps) -> Capabilities {
    Capabilities {
        keys: caps.keys,
        rel: caps.rel,
        abs: caps.abs,
        props: caps.props,
    }
}

#[derive(Debug)]
pub struct EvdevRuntime {
    receiver: Receiver<InputEvent>,
    status: Arc<Mutex<InputStatus>>,
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
}

impl EvdevRuntime {
    pub fn start(selection: Selection, grab: bool, grab_force: bool) -> Result<Self, InputError> {
        let mut discovery = discover();
        let mut registry = BTreeMap::new();
        let mut next_id = 1;
        assign_ids(&mut discovery.devices, &mut registry, &mut next_id);

        let initial_paths = choose_initial_paths(&discovery.devices, &selection)?;
        let mut open = BTreeMap::new();
        for path in initial_paths {
            let info = discovery
                .devices
                .iter()
                .find(|device| device.path == path)
                .cloned()
                .ok_or(InputError::NoDevice)?;
            let device = open_device(&info, grab, grab_force)?;
            open.insert(path, device);
        }

        let primary = open.values().next().map(|device| device.info.id);
        let status = Arc::new(Mutex::new(InputStatus {
            mode: if open.is_empty() {
                InputMode::Waiting
            } else {
                InputMode::Evdev
            },
            devices: discovery.devices,
            primary,
            message: None,
        }));

        let (tx, receiver) = sync_channel(CHANNEL_CAPACITY);
        let stop = Arc::new(AtomicBool::new(false));
        let reader = Reader {
            open,
            registry,
            next_id,
            selection,
            grab,
            grab_force,
            status: Arc::clone(&status),
            sender: EventSender::new(tx),
            stop: Arc::clone(&stop),
        };
        let join = thread::Builder::new()
            .name("mouse_tester-evdev".to_string())
            .spawn(move || reader.run())
            .map_err(InputError::Io)?;

        Ok(Self {
            receiver,
            status,
            stop,
            join: Some(join),
        })
    }

    pub fn poll(&self) -> Vec<InputEvent> {
        self.receiver.try_iter().collect()
    }

    pub fn status(&self) -> InputStatus {
        self.status
            .lock()
            .map(|status| status.clone())
            .unwrap_or(InputStatus {
                mode: InputMode::Waiting,
                devices: Vec::new(),
                primary: None,
                message: Some("input status unavailable".to_string()),
            })
    }

    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

impl Drop for EvdevRuntime {
    fn drop(&mut self) {
        self.stop();
    }
}

struct Reader {
    open: BTreeMap<PathBuf, OpenDevice>,
    registry: BTreeMap<PathBuf, DeviceId>,
    next_id: DeviceId,
    selection: Selection,
    grab: bool,
    grab_force: bool,
    status: Arc<Mutex<InputStatus>>,
    sender: EventSender,
    stop: Arc<AtomicBool>,
}

impl Reader {
    fn run(mut self) {
        let mut last_scan = Instant::now()
            .checked_sub(SCAN_INTERVAL)
            .unwrap_or_else(Instant::now);
        while !self.stop.load(Ordering::SeqCst) {
            if last_scan.elapsed() >= SCAN_INTERVAL {
                self.rescan();
                last_scan = Instant::now();
            }

            let paths: Vec<PathBuf> = self.open.keys().cloned().collect();
            let mut fds: Vec<libc::pollfd> = self
                .open
                .values()
                .map(|device| poll_fd(device.device.raw_fd()))
                .collect();

            if fds.is_empty() {
                thread::sleep(Duration::from_millis(POLL_TIMEOUT_MS as u64));
                continue;
            }

            match poll_readable(&mut fds, POLL_TIMEOUT_MS) {
                Ok(0) => {}
                Ok(_) => {
                    let mut remove = Vec::new();
                    for (index, fd) in fds.iter().enumerate() {
                        if fd.revents & (libc::POLLIN | libc::POLLHUP | libc::POLLERR) == 0 {
                            continue;
                        }
                        let Some(path) = paths.get(index) else {
                            continue;
                        };
                        let Some(device) = self.open.get_mut(path) else {
                            continue;
                        };
                        if device.read_and_emit(&mut self.sender).is_err() {
                            remove.push(path.clone());
                        }
                        if fd.revents & (libc::POLLHUP | libc::POLLERR) != 0 {
                            remove.push(path.clone());
                        }
                    }
                    remove.sort();
                    remove.dedup();
                    for path in remove {
                        self.close_path(&path);
                    }
                }
                Err(error) => {
                    self.set_message(Some(format!("evdev poll failed: {error}")));
                    thread::sleep(Duration::from_millis(POLL_TIMEOUT_MS as u64));
                }
            }
        }
    }

    fn rescan(&mut self) {
        let mut discovery = discover();
        assign_ids(
            &mut discovery.devices,
            &mut self.registry,
            &mut self.next_id,
        );

        let pointing: Vec<DeviceInfo> = discovery
            .devices
            .iter()
            .filter(|device| is_pointing(device))
            .cloned()
            .collect();
        let mouse_like: Vec<DeviceInfo> = discovery
            .devices
            .iter()
            .filter(|device| is_mouse_like(device))
            .cloned()
            .collect();
        let readable_mouse: Vec<DeviceInfo> = mouse_like
            .iter()
            .filter(|device| device.readable)
            .cloned()
            .collect();
        let readable_pointing: Vec<DeviceInfo> = pointing
            .iter()
            .filter(|device| device.readable)
            .cloned()
            .collect();

        let selection = self.selection.clone();
        let mut switch_to_auto = false;
        let desired = match &selection {
            Selection::All => readable_mouse
                .iter()
                .map(|device| device.path.clone())
                .collect(),
            Selection::Path(path) => {
                if readable_pointing.iter().any(|device| &device.path == path) {
                    vec![path.clone()]
                } else {
                    switch_to_auto = true;
                    readable_mouse
                        .first()
                        .map(|device| vec![device.path.clone()])
                        .unwrap_or_default()
                }
            }
            Selection::Auto => {
                let keep_current = self
                    .open
                    .values()
                    .next()
                    .map(|device| device.info.path.clone())
                    .filter(|path| readable_mouse.iter().any(|device| &device.path == path));
                if let Some(path) = keep_current {
                    vec![path]
                } else {
                    readable_mouse
                        .first()
                        .map(|device| vec![device.path.clone()])
                        .unwrap_or_default()
                }
            }
        };
        if switch_to_auto {
            self.selection = Selection::Auto;
        }

        let desired_set: BTreeSet<PathBuf> = desired.iter().cloned().collect();
        let current: Vec<PathBuf> = self.open.keys().cloned().collect();
        for path in current {
            if !desired_set.contains(&path) {
                self.close_path(&path);
            }
        }

        for path in desired {
            if self.open.contains_key(&path) {
                continue;
            }
            let Some(info) = discovery
                .devices
                .iter()
                .find(|device| device.path == path)
                .cloned()
            else {
                continue;
            };
            match open_device(&info, self.grab, self.grab_force) {
                Ok(device) => {
                    self.sender.send(InputEvent::new(
                        info.id,
                        InputSource::Evdev,
                        now_micros(),
                        InputKind::DeviceAdded(info.clone()),
                    ));
                    self.open.insert(path, device);
                }
                Err(error) => {
                    self.set_message(Some(format!(
                        "cannot open {}: {error}",
                        info.path.display()
                    )));
                }
            }
        }

        let primary = self.open.values().next().map(|device| device.info.id);
        let message = if self.open.is_empty() && !readable_mouse.is_empty() {
            Some("waiting for a readable mouse".to_string())
        } else if self.open.is_empty() && mouse_like.iter().any(|device| !device.readable) {
            Some("mouse found but permission denied; see help".to_string())
        } else {
            None
        };
        if let Ok(mut status) = self.status.lock() {
            status.mode = if self.open.is_empty() {
                InputMode::Waiting
            } else {
                InputMode::Evdev
            };
            status.devices = discovery.devices;
            status.primary = primary;
            status.message = message;
        }
    }

    fn close_path(&mut self, path: &Path) {
        if let Some(device) = self.open.remove(path) {
            self.sender.send(InputEvent::new(
                device.info.id,
                InputSource::Evdev,
                now_micros(),
                InputKind::DeviceRemoved(device.info.id),
            ));
        }
    }

    fn set_message(&self, message: Option<String>) {
        if let Ok(mut status) = self.status.lock() {
            status.message = message;
        }
    }
}

fn choose_initial_paths(
    devices: &[DeviceInfo],
    selection: &Selection,
) -> Result<Vec<PathBuf>, InputError> {
    let mouse_like: Vec<&DeviceInfo> = devices
        .iter()
        .filter(|device| is_mouse_like(device))
        .collect();
    let readable_mouse: Vec<&DeviceInfo> = mouse_like
        .iter()
        .copied()
        .filter(|device| device.readable)
        .collect();

    match selection {
        Selection::All => {
            if readable_mouse.is_empty() {
                if let Some(denied) = mouse_like.iter().find(|device| !device.readable) {
                    return Err(InputError::Permission {
                        path: denied.path.clone(),
                    });
                }
            }
            Ok(readable_mouse
                .iter()
                .map(|device| device.path.clone())
                .collect())
        }
        Selection::Path(path) => {
            let Some(device) = devices.iter().find(|device| &device.path == path) else {
                return Err(InputError::NoDevice);
            };
            if !is_pointing(device) {
                return Err(InputError::Unsupported { path: path.clone() });
            }
            if !device.readable {
                return Err(InputError::Permission { path: path.clone() });
            }
            Ok(vec![path.clone()])
        }
        Selection::Auto => {
            if let Some(device) = readable_mouse.first() {
                Ok(vec![device.path.clone()])
            } else if let Some(device) = mouse_like.first() {
                Err(InputError::Permission {
                    path: device.path.clone(),
                })
            } else {
                Ok(Vec::new())
            }
        }
    }
}

fn assign_ids(
    devices: &mut [DeviceInfo],
    registry: &mut BTreeMap<PathBuf, DeviceId>,
    next_id: &mut DeviceId,
) {
    for device in devices {
        let id = *registry.entry(device.path.clone()).or_insert_with(|| {
            let id = *next_id;
            *next_id = next_id.saturating_add(1);
            id
        });
        device.id = id;
    }
}

fn is_pointing(device: &DeviceInfo) -> bool {
    device.is_pointing()
}

fn is_mouse_like(device: &DeviceInfo) -> bool {
    matches!(device.class, DeviceClass::Mouse | DeviceClass::Combo)
        || device.capabilities.mouse_like()
}

fn open_device(info: &DeviceInfo, grab: bool, grab_force: bool) -> Result<OpenDevice, InputError> {
    if grab && !grab_force && info.class == DeviceClass::Combo {
        return Err(InputError::ComboGrab {
            path: info.path.clone(),
        });
    }

    let device = EvdevDevice::open(&info.path).map_err(|error| {
        if error.kind() == io::ErrorKind::PermissionDenied {
            InputError::Permission {
                path: info.path.clone(),
            }
        } else {
            InputError::Io(error)
        }
    })?;
    let _ = device.set_clock_monotonic();
    if grab {
        let _ = device.grab(true);
    }

    let (abs_x, abs_y) = if matches!(info.class, DeviceClass::Touchpad | DeviceClass::Tablet) {
        (device.abs_info(ABS_X).ok(), device.abs_info(ABS_Y).ok())
    } else {
        (None, None)
    };

    Ok(OpenDevice {
        info: info.clone(),
        device,
        pressed: BTreeSet::new(),
        abs_x,
        abs_y,
        abs_value_x: None,
        abs_value_y: None,
        last_abs: None,
    })
}

struct OpenDevice {
    info: DeviceInfo,
    device: EvdevDevice,
    pressed: BTreeSet<u16>,
    abs_x: Option<AbsInfo>,
    abs_y: Option<AbsInfo>,
    abs_value_x: Option<i32>,
    abs_value_y: Option<i32>,
    last_abs: Option<(f32, f32)>,
}

impl OpenDevice {
    fn read_and_emit(&mut self, sender: &mut EventSender) -> io::Result<()> {
        let mut events = Vec::new();
        self.device.read_events(&mut events)?;
        for event in events {
            if !self.handle_raw(event, sender) {
                break;
            }
        }
        Ok(())
    }

    fn handle_raw(&mut self, raw: RawInputEvent, sender: &mut EventSender) -> bool {
        let device = self.info.id;
        let time = raw.time_us();
        let source = InputSource::Evdev;

        match raw.kind {
            EV_KEY if is_mouse_button(raw.code) => {
                if raw.value == 2 {
                    return true;
                }
                let pressed = raw.value != 0;
                if pressed {
                    self.pressed.insert(raw.code);
                } else {
                    self.pressed.remove(&raw.code);
                }
                sender.send(InputEvent::new(
                    device,
                    source,
                    time,
                    InputKind::Button {
                        code: raw.code,
                        pressed,
                    },
                ))
            }
            EV_REL => match raw.code {
                REL_X => sender.send(InputEvent::new(
                    device,
                    source,
                    time,
                    InputKind::Move {
                        dx: raw.value,
                        dy: 0,
                    },
                )),
                REL_Y => sender.send(InputEvent::new(
                    device,
                    source,
                    time,
                    InputKind::Move {
                        dx: 0,
                        dy: raw.value,
                    },
                )),
                REL_WHEEL | REL_WHEEL_HI_RES => sender.send(InputEvent::new(
                    device,
                    source,
                    time,
                    InputKind::Wheel {
                        axis: ScrollAxis::Vertical,
                        value: raw.value,
                        hires: raw.code == REL_WHEEL_HI_RES,
                    },
                )),
                REL_HWHEEL | REL_HWHEEL_HI_RES => sender.send(InputEvent::new(
                    device,
                    source,
                    time,
                    InputKind::Wheel {
                        axis: ScrollAxis::Horizontal,
                        value: raw.value,
                        hires: raw.code == REL_HWHEEL_HI_RES,
                    },
                )),
                _ => true,
            },
            EV_ABS => {
                if raw.code == ABS_X {
                    self.abs_value_x = Some(raw.value);
                } else if raw.code == ABS_Y {
                    self.abs_value_y = Some(raw.value);
                }
                true
            }
            EV_SYN if raw.code == SYN_REPORT => {
                if let Some((x, y)) = self.pending_absolute() {
                    sender.send(InputEvent::new(
                        device,
                        source,
                        time,
                        InputKind::Absolute { x, y },
                    ));
                }
                sender.send(InputEvent::new(device, source, time, InputKind::Report))
            }
            EV_SYN if raw.code == SYN_DROPPED => {
                self.resync_buttons(sender, time);
                sender.send(InputEvent::new(
                    device,
                    source,
                    time,
                    InputKind::SourceError("evdev: SYN_DROPPED; state resynchronized".to_string()),
                ))
            }
            _ => true,
        }
    }

    fn pending_absolute(&mut self) -> Option<(f32, f32)> {
        let x = self.abs_value_x?;
        let y = self.abs_value_y?;
        let x_info = self.abs_x?;
        let y_info = self.abs_y?;
        let x = normalize_axis(x, x_info)?;
        let y = normalize_axis(y, y_info)?;
        if self.last_abs == Some((x, y)) {
            None
        } else {
            self.last_abs = Some((x, y));
            Some((x, y))
        }
    }

    fn resync_buttons(&mut self, sender: &mut EventSender, time: u64) {
        let Ok(state) = self.device.key_state() else {
            return;
        };
        let state: BTreeSet<u16> = state
            .into_iter()
            .filter(|code| is_mouse_button(*code))
            .collect();
        for code in self.pressed.difference(&state).copied().collect::<Vec<_>>() {
            sender.send(InputEvent::new(
                self.info.id,
                InputSource::Evdev,
                time,
                InputKind::Button {
                    code,
                    pressed: false,
                },
            ));
        }
        for code in state.difference(&self.pressed).copied().collect::<Vec<_>>() {
            sender.send(InputEvent::new(
                self.info.id,
                InputSource::Evdev,
                time,
                InputKind::Button {
                    code,
                    pressed: true,
                },
            ));
        }
        self.pressed = state;
    }
}

fn normalize_axis(value: i32, info: AbsInfo) -> Option<f32> {
    let span = info.maximum.saturating_sub(info.minimum);
    if span <= 0 {
        return None;
    }
    let offset = value.saturating_sub(info.minimum);
    Some((offset as f32 / span as f32).clamp(0.0, 1.0))
}

struct EventSender {
    tx: SyncSender<InputEvent>,
    dropped: u64,
}

impl EventSender {
    fn new(tx: SyncSender<InputEvent>) -> Self {
        Self { tx, dropped: 0 }
    }

    fn send(&mut self, event: InputEvent) -> bool {
        if self.dropped > 0 {
            let overflow = InputEvent::new(
                event.device,
                event.source,
                event.time_us,
                InputKind::Overflow {
                    dropped: self.dropped,
                },
            );
            if self.tx.try_send(overflow).is_ok() {
                self.dropped = 0;
            }
        }
        match self.tx.try_send(event) {
            Ok(()) => true,
            Err(TrySendError::Full(_)) => {
                self.dropped = self.dropped.saturating_add(1);
                true
            }
            Err(TrySendError::Disconnected(_)) => false,
        }
    }
}

fn now_micros() -> u64 {
    static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_micros() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::codes::{
        BTN_EXTRA, BTN_LEFT, BTN_RIGHT, INPUT_PROP_BUTTONPAD, KEY_A, KEY_Z, REL_X, REL_Y,
    };
    use crate::platform::evdev::{ABS_X, ABS_Y, REL_WHEEL};

    const PROC_MOUSE: &str = r#"I: Bus=0003 Vendor=046d Product=c08b Version=0110
N: Name="Logitech G Pro"
P: Phys=usb-0000:00:14.0-1/input0
S: Sysfs=/devices/pci0000:00/0000:00:14.0/usb1/1-1/input/input5
U: Uniq=
H: Handlers=mouse0 event5
B: PROP=0
B: EV=17
B: KEY=1f0000 0 0 0 0
B: REL=903
B: MSC=10
"#;

    #[test]
    fn parses_proc_mouse_and_capabilities() {
        let devices = parse_proc_devices(PROC_MOUSE);
        assert_eq!(devices.len(), 1);
        let device = &devices[0];
        assert_eq!(device.name, "Logitech G Pro");
        assert_eq!(device.path, PathBuf::from("/dev/input/event5"));
        assert_eq!(device.vendor, 0x046d);
        assert_eq!(device.product, 0xc08b);
        assert_eq!(device.class, DeviceClass::Mouse);
        assert!(device.capabilities.keys.contains(&BTN_LEFT));
        assert!(device.capabilities.keys.contains(&BTN_RIGHT));
        assert!(device.capabilities.keys.contains(&BTN_EXTRA));
        assert!(device.capabilities.rel.contains(&REL_X));
        assert!(device.capabilities.rel.contains(&REL_Y));
        assert!(device.capabilities.rel.contains(&REL_WHEEL));
    }

    #[test]
    fn parses_combo_and_ignores_keyboard_only_devices() {
        let combo = r#"I: Bus=0003 Vendor=1bcf Product=08a0 Version=0110
N: Name="HID Keyboard Mouse"
P: Phys=usb-1/input0
H: Handlers=sysrq kbd mouse0 event4
B: PROP=0
B: EV=12001f
B: KEY=1f0000 0 0 0 100040000000
B: REL=903
"#;
        let devices = parse_proc_devices(combo);
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].class, DeviceClass::Combo);
    }

    #[test]
    fn keeps_events_without_mouse_buttons() {
        let text = r#"I: Bus=0019 Vendor=0000 Product=0001 Version=0000
N: Name="Power Button"
P: Phys=PNP0C0C/button/input0
H: Handlers=kbd event0
B: PROP=0
B: EV=3
B: KEY=8000 10000000000000 0
"#;
        let devices = parse_proc_devices(text);
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].class, DeviceClass::Unknown);
        assert!(!devices[0].capabilities.keys.is_empty());
    }

    #[test]
    fn parses_touchpad_absolute_properties() {
        let text = r#"I: Bus=0011 Vendor=0002 Product=0007 Version=0000
N: Name="SynPS/2 Synaptics TouchPad"
P: Phys=isa0060/serio1/input0
H: Handlers=mouse1 event6
B: PROP=5
B: EV=b
B: KEY=e520 10000 0 0 0 0
B: ABS=260800000000003
"#;
        let devices = parse_proc_devices(text);
        assert_eq!(devices[0].class, DeviceClass::Touchpad);
        assert!(devices[0]
            .capabilities
            .props
            .contains(&INPUT_PROP_BUTTONPAD));
        assert!(devices[0].capabilities.abs.contains(&ABS_X));
        assert!(devices[0].capabilities.abs.contains(&ABS_Y));
    }

    #[test]
    fn keyboard_bitmap_is_parsed_without_reading_it() {
        let text = r#"I: Bus=0003 Vendor=1a2c Product=6004 Version=0110
N: Name="SEMICO USB Keyboard"
H: Handlers=sysrq kbd event5
B: PROP=0
B: KEY=100040000000
B: REL=0
"#;
        let devices = parse_proc_devices(text);
        assert!(devices[0].capabilities.keys.contains(&KEY_A));
        assert!(devices[0].capabilities.keys.contains(&KEY_Z));
        assert_eq!(devices[0].class, DeviceClass::Keyboard);
    }

    #[test]
    fn normalize_axis_handles_ranges() {
        let info = AbsInfo {
            minimum: 0,
            maximum: 100,
            ..AbsInfo::default()
        };
        assert_eq!(normalize_axis(50, info), Some(0.5));
        assert_eq!(normalize_axis(1000, info), Some(1.0));
        assert_eq!(normalize_axis(-10, info), Some(0.0));
        assert_eq!(
            normalize_axis(
                10,
                AbsInfo {
                    minimum: 5,
                    maximum: 5,
                    ..AbsInfo::default()
                }
            ),
            None
        );
    }

    #[test]
    fn auto_does_not_select_touchpad_without_mouse() {
        let mut touchpad = parse_proc_devices(
            r#"I: Bus=0011 Vendor=0002 Product=0007 Version=0000
N: Name="Touchpad"
H: Handlers=event6
B: PROP=5
B: EV=b
B: KEY=e520 10000
B: ABS=260800000000003
"#,
        )
        .remove(0);
        touchpad.readable = true;
        let selected = choose_initial_paths(&[touchpad], &Selection::Auto).unwrap();
        assert!(selected.is_empty(), "{selected:?}");
    }

    #[test]
    fn explicit_touchpad_is_allowed() {
        let mut touchpad = parse_proc_devices(
            r#"I: Bus=0011 Vendor=0002 Product=0007 Version=0000
N: Name="Touchpad"
H: Handlers=event6
B: PROP=5
B: EV=b
B: KEY=e520 10000
B: ABS=260800000000003
"#,
        )
        .remove(0);
        touchpad.readable = true;
        let path = touchpad.path.clone();
        let selected = choose_initial_paths(&[touchpad], &Selection::Path(path.clone())).unwrap();
        assert_eq!(selected, vec![path]);
    }

    #[test]
    fn explicit_missing_device_is_an_error() {
        let result = EvdevRuntime::start(
            Selection::Path(PathBuf::from("/dev/input/mouse_tester-no-such-device")),
            false,
            false,
        );
        assert!(matches!(result, Err(InputError::NoDevice)));
    }

    #[test]
    fn pointing_filter_includes_mouse_and_touchpad() {
        let mouse = parse_proc_devices(PROC_MOUSE).remove(0);
        assert!(is_pointing(&mouse));
        let touchpad = parse_proc_devices(
            r#"I: Bus=0011 Vendor=0002 Product=0007 Version=0000
N: Name="Touchpad"
H: Handlers=event6
B: PROP=5
B: EV=b
B: KEY=e520 10000
B: ABS=260800000000003
"#,
        )
        .remove(0);
        assert!(is_pointing(&touchpad));
    }
}
