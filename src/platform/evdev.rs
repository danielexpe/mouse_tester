use std::collections::BTreeSet;
use std::ffi::CString;
use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::{Path, PathBuf};

pub const EV_SYN: u16 = 0x00;
pub const EV_KEY: u16 = 0x01;
pub const EV_REL: u16 = 0x02;
pub const EV_ABS: u16 = 0x03;

pub const SYN_REPORT: u16 = 0x00;
pub const SYN_DROPPED: u16 = 0x03;

pub const REL_X: u16 = 0x00;
pub const REL_Y: u16 = 0x01;
pub const REL_HWHEEL: u16 = 0x06;
pub const REL_WHEEL: u16 = 0x08;
pub const REL_WHEEL_HI_RES: u16 = 0x0b;
pub const REL_HWHEEL_HI_RES: u16 = 0x0c;

pub const ABS_X: u16 = 0x00;
pub const ABS_Y: u16 = 0x01;

const IOC_READ: u64 = 2;
const IOC_WRITE: u64 = 1;
const KEY_BYTES: usize = 96;
const REL_BYTES: usize = 8;
const ABS_BYTES: usize = 8;
const PROP_BYTES: usize = 8;

const fn ioc(direction: u64, kind: u64, number: u64, size: u64) -> libc::c_ulong {
    ((direction << 30) | (size << 16) | (kind << 8) | number) as libc::c_ulong
}

const fn eviocgbit_request(event_type: u16, length: usize) -> libc::c_ulong {
    ioc(
        IOC_READ,
        b'E' as u64,
        0x20 + event_type as u64,
        length as u64,
    )
}

const fn eviocgname_request(length: usize) -> libc::c_ulong {
    ioc(IOC_READ, b'E' as u64, 0x06, length as u64)
}

const fn eviocgphys_request(length: usize) -> libc::c_ulong {
    ioc(IOC_READ, b'E' as u64, 0x07, length as u64)
}

const fn eviocguniq_request(length: usize) -> libc::c_ulong {
    ioc(IOC_READ, b'E' as u64, 0x08, length as u64)
}

const fn eviocgprop_request(length: usize) -> libc::c_ulong {
    ioc(IOC_READ, b'E' as u64, 0x09, length as u64)
}

const fn eviocgkey_request(length: usize) -> libc::c_ulong {
    ioc(IOC_READ, b'E' as u64, 0x18, length as u64)
}

const fn eviocgabs_request(axis: u16) -> libc::c_ulong {
    ioc(IOC_READ, b'E' as u64, 0x40 + axis as u64, 24)
}

const EVIOCGID_REQUEST: libc::c_ulong = ioc(IOC_READ, b'E' as u64, 0x02, 8);
const EVIOCGRAB_REQUEST: libc::c_ulong = ioc(IOC_WRITE, b'E' as u64, 0x90, 4);
const EVIOCSCLOCKID_REQUEST: libc::c_ulong = ioc(IOC_WRITE, b'E' as u64, 0xa0, 4);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RawInputEvent {
    pub sec: i64,
    pub usec: i64,
    pub kind: u16,
    pub code: u16,
    pub value: i32,
}

impl RawInputEvent {
    pub fn time_us(self) -> u64 {
        let seconds = self.sec.max(0) as u64;
        let micros = self.usec.clamp(0, 999_999) as u64;
        seconds.saturating_mul(1_000_000).saturating_add(micros)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InputId {
    pub bustype: u16,
    pub vendor: u16,
    pub product: u16,
    pub version: u16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AbsInfo {
    pub value: i32,
    pub minimum: i32,
    pub maximum: i32,
    pub fuzz: i32,
    pub flat: i32,
    pub resolution: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeviceCaps {
    pub keys: BTreeSet<u16>,
    pub rel: BTreeSet<u16>,
    pub abs: BTreeSet<u16>,
    pub props: BTreeSet<u16>,
}

pub struct EvdevDevice {
    file: File,
    path: PathBuf,
    pending: Vec<u8>,
}

impl EvdevDevice {
    pub fn open(path: &Path) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK | libc::O_CLOEXEC)
            .open(path)?;
        Ok(Self {
            file,
            path: path.to_path_buf(),
            pending: Vec::with_capacity(4096),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }

    pub fn name(&self) -> io::Result<String> {
        self.ioctl_string(eviocgname_request(256))
    }

    pub fn phys(&self) -> io::Result<Option<String>> {
        self.ioctl_optional_string(eviocgphys_request(256))
    }

    pub fn uniq(&self) -> io::Result<Option<String>> {
        self.ioctl_optional_string(eviocguniq_request(256))
    }

    pub fn input_id(&self) -> io::Result<InputId> {
        let bytes = self.ioctl_array::<8>(EVIOCGID_REQUEST)?;
        Ok(parse_input_id(&bytes))
    }

    pub fn capabilities(&self) -> io::Result<DeviceCaps> {
        let keys = bitset_from_bytes(
            &self.ioctl_vec_sized(eviocgbit_request(EV_KEY, KEY_BYTES), KEY_BYTES)?,
        );
        let rel = bitset_from_bytes(
            &self.ioctl_vec_sized(eviocgbit_request(EV_REL, REL_BYTES), REL_BYTES)?,
        );
        let abs = bitset_from_bytes(
            &self.ioctl_vec_sized(eviocgbit_request(EV_ABS, ABS_BYTES), ABS_BYTES)?,
        );
        let props =
            bitset_from_bytes(&self.ioctl_vec_sized(eviocgprop_request(PROP_BYTES), PROP_BYTES)?);
        Ok(DeviceCaps {
            keys,
            rel,
            abs,
            props,
        })
    }

    pub fn abs_info(&self, axis: u16) -> io::Result<AbsInfo> {
        let bytes = self.ioctl_array::<24>(eviocgabs_request(axis))?;
        Ok(parse_abs_info(&bytes))
    }

    pub fn key_state(&self) -> io::Result<BTreeSet<u16>> {
        let bytes = self.ioctl_vec_sized(eviocgkey_request(KEY_BYTES), KEY_BYTES)?;
        Ok(bitset_from_bytes(&bytes))
    }

    pub fn set_clock_monotonic(&self) -> io::Result<()> {
        let clock = libc::CLOCK_MONOTONIC as libc::c_int;
        let result = unsafe { libc::ioctl(self.raw_fd(), EVIOCSCLOCKID_REQUEST, &clock) };
        if result < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub fn grab(&self, enabled: bool) -> io::Result<()> {
        let value: libc::c_int = i32::from(enabled);
        let result = unsafe { libc::ioctl(self.raw_fd(), EVIOCGRAB_REQUEST, &value) };
        if result < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub fn read_events(&mut self, out: &mut Vec<RawInputEvent>) -> io::Result<usize> {
        let mut buffer = [0u8; 4096];
        loop {
            match self.file.read(&mut buffer) {
                Ok(0) => break,
                Ok(count) => self.pending.extend_from_slice(&buffer[..count]),
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(error),
            }
        }
        Ok(parse_input_events(&mut self.pending, out))
    }

    fn ioctl_array<const N: usize>(&self, request: libc::c_ulong) -> io::Result<[u8; N]> {
        let mut bytes = [0u8; N];
        let result = unsafe {
            libc::ioctl(
                self.raw_fd(),
                request,
                bytes.as_mut_ptr() as *mut libc::c_void,
            )
        };
        if result < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(bytes)
        }
    }

    fn ioctl_string(&self, request: libc::c_ulong) -> io::Result<String> {
        let bytes = self.ioctl_vec_sized(request, 256)?;
        Ok(bytes_to_string(&bytes))
    }

    fn ioctl_optional_string(&self, request: libc::c_ulong) -> io::Result<Option<String>> {
        let bytes = self.ioctl_vec_sized(request, 256)?;
        let value = bytes_to_string(&bytes);
        Ok(if value.is_empty() { None } else { Some(value) })
    }

    fn ioctl_vec_sized(&self, request: libc::c_ulong, length: usize) -> io::Result<Vec<u8>> {
        let mut bytes = vec![0u8; length];
        let result = unsafe {
            libc::ioctl(
                self.raw_fd(),
                request,
                bytes.as_mut_ptr() as *mut libc::c_void,
            )
        };
        if result < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(bytes)
        }
    }
}

pub fn poll_fd(fd: RawFd) -> libc::pollfd {
    libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    }
}

pub fn poll_readable(fds: &mut [libc::pollfd], timeout_ms: i32) -> io::Result<usize> {
    let result = unsafe { libc::poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, timeout_ms) };
    if result < 0 {
        let error = io::Error::last_os_error();
        if error.kind() == io::ErrorKind::Interrupted {
            Ok(0)
        } else {
            Err(error)
        }
    } else {
        Ok(result as usize)
    }
}

pub fn is_readable(path: &Path) -> bool {
    let Ok(path) = CString::new(path.as_os_str().as_bytes()) else {
        return false;
    };
    unsafe { libc::access(path.as_ptr(), libc::R_OK) == 0 }
}

pub fn parse_input_events(pending: &mut Vec<u8>, out: &mut Vec<RawInputEvent>) -> usize {
    let mut consumed = 0;
    let mut count = 0;
    while pending.len() - consumed >= 24 {
        let bytes = &pending[consumed..consumed + 24];
        out.push(RawInputEvent {
            sec: i64::from_le_bytes(bytes[0..8].try_into().unwrap_or_default()),
            usec: i64::from_le_bytes(bytes[8..16].try_into().unwrap_or_default()),
            kind: u16::from_le_bytes(bytes[16..18].try_into().unwrap_or_default()),
            code: u16::from_le_bytes(bytes[18..20].try_into().unwrap_or_default()),
            value: i32::from_le_bytes(bytes[20..24].try_into().unwrap_or_default()),
        });
        consumed += 24;
        count += 1;
    }
    if consumed > 0 {
        pending.drain(..consumed);
    }
    count
}

pub fn parse_input_id(bytes: &[u8; 8]) -> InputId {
    InputId {
        bustype: u16::from_le_bytes([bytes[0], bytes[1]]),
        vendor: u16::from_le_bytes([bytes[2], bytes[3]]),
        product: u16::from_le_bytes([bytes[4], bytes[5]]),
        version: u16::from_le_bytes([bytes[6], bytes[7]]),
    }
}

pub fn parse_abs_info(bytes: &[u8; 24]) -> AbsInfo {
    let value = |offset: usize| {
        i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap_or_default())
    };
    AbsInfo {
        value: value(0),
        minimum: value(4),
        maximum: value(8),
        fuzz: value(12),
        flat: value(16),
        resolution: value(20),
    }
}

pub fn bitset_from_bytes(bytes: &[u8]) -> BTreeSet<u16> {
    let mut set = BTreeSet::new();
    for (index, byte) in bytes.iter().enumerate() {
        for bit in 0..8 {
            if byte & (1 << bit) != 0 {
                let code = index * 8 + bit;
                if let Ok(code) = u16::try_from(code) {
                    set.insert(code);
                }
            }
        }
    }
    set
}

fn bytes_to_string(bytes: &[u8]) -> String {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event_bytes(sec: i64, usec: i64, kind: u16, code: u16, value: i32) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&sec.to_le_bytes());
        bytes.extend_from_slice(&usec.to_le_bytes());
        bytes.extend_from_slice(&kind.to_le_bytes());
        bytes.extend_from_slice(&code.to_le_bytes());
        bytes.extend_from_slice(&value.to_le_bytes());
        bytes
    }

    #[test]
    fn parses_input_event_layout() {
        let mut pending = event_bytes(12, 34, EV_REL, REL_X, -5);
        pending.extend_from_slice(&event_bytes(12, 35, EV_SYN, SYN_REPORT, 0));
        let mut events = Vec::new();
        assert_eq!(parse_input_events(&mut pending, &mut events), 2);
        assert!(pending.is_empty());
        assert_eq!(events[0].sec, 12);
        assert_eq!(events[0].usec, 34);
        assert_eq!(events[0].kind, EV_REL);
        assert_eq!(events[0].code, REL_X);
        assert_eq!(events[0].value, -5);
        assert_eq!(events[0].time_us(), 12_000_034);
        assert_eq!(events[1].code, SYN_REPORT);
    }

    #[test]
    fn keeps_partial_event_for_next_read() {
        let full = event_bytes(1, 2, EV_KEY, 0x110, 1);
        let mut pending = full[..10].to_vec();
        let mut events = Vec::new();
        assert_eq!(parse_input_events(&mut pending, &mut events), 0);
        assert_eq!(pending.len(), 10);
        pending.extend_from_slice(&full[10..]);
        assert_eq!(parse_input_events(&mut pending, &mut events), 1);
        assert_eq!(events[0].kind, EV_KEY);
    }

    #[test]
    fn converts_bitmap_to_codes() {
        let mut bytes = vec![0u8; 74];
        bytes[0x110 / 8] |= 1 << (0x110 % 8);
        bytes[0x111 / 8] |= 1 << (0x111 % 8);
        let set = bitset_from_bytes(&bytes);
        assert!(set.contains(&0x110));
        assert!(set.contains(&0x111));
        assert!(!set.contains(&0x112));
    }

    #[test]
    fn parses_input_id_and_abs_info() {
        let id = parse_input_id(&[3, 0, 0x09, 0x12, 1, 0, 0, 0]);
        assert_eq!(id.bustype, 3);
        assert_eq!(id.vendor, 0x1209);
        assert_eq!(id.product, 1);

        let mut bytes = [0u8; 24];
        for (index, value) in [10i32, 0, 100, 1, 2, 3].iter().enumerate() {
            bytes[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
        let info = parse_abs_info(&bytes);
        assert_eq!(info.value, 10);
        assert_eq!(info.maximum, 100);
        assert_eq!(info.resolution, 3);
    }

    #[test]
    fn time_clamps_negative_values() {
        let event = RawInputEvent {
            sec: -1,
            usec: -5,
            ..RawInputEvent::default()
        };
        assert_eq!(event.time_us(), 0);
    }
}
