use std::ffi::CStr;
use std::fs::{File, OpenOptions};
use std::io;
use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::sync::atomic::{AtomicI32, Ordering};

use crossterm::cursor::{Hide, Show};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use thiserror::Error;

use crate::render::raster::Frame;

pub const FBIOGET_VSCREENINFO: u64 = 0x4600;
pub const FBIOGET_FSCREENINFO: u64 = 0x4602;
pub const FBIOPUT_VSCREENINFO: u64 = 0x4601;
pub const FBIOPAN_DISPLAY: u64 = 0x4606;
pub const FBIOBLANK: u64 = 0x4611;
pub const FB_BLANK_UNBLANK: u64 = 0;
pub const KDGETMODE: u64 = 0x4b3b;
pub const KDSETMODE: u64 = 0x4b3a;
pub const KD_TEXT: i32 = 0;
pub const KD_GRAPHICS: i32 = 1;

static ACTIVE_TTY: AtomicI32 = AtomicI32::new(-1);
static SAVED_KD_MODE: AtomicI32 = AtomicI32::new(KD_TEXT);

#[derive(Debug, Error)]
pub enum FbError {
    #[error("framebuffer I/O: {0}")]
    Io(#[from] io::Error),
    #[error("unsupported framebuffer: {0}")]
    Unsupported(String),
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FbBitfield {
    pub offset: u32,
    pub length: u32,
    pub msb_right: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FbVarScreenInfo {
    pub xres: u32,
    pub yres: u32,
    pub xres_virtual: u32,
    pub yres_virtual: u32,
    pub xoffset: u32,
    pub yoffset: u32,
    pub bits_per_pixel: u32,
    pub grayscale: u32,
    pub red: FbBitfield,
    pub green: FbBitfield,
    pub blue: FbBitfield,
    pub transp: FbBitfield,
    pub nonstd: u32,
    pub activate: u32,
    pub height: u32,
    pub width: u32,
    pub accel_flags: u32,
    pub pixclock: u32,
    pub left_margin: u32,
    pub right_margin: u32,
    pub upper_margin: u32,
    pub lower_margin: u32,
    pub hsync_len: u32,
    pub vsync_len: u32,
    pub sync: u32,
    pub vmode: u32,
    pub rotate: u32,
    pub colorspace: u32,
    pub reserved: [u32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FbFixScreenInfo {
    pub id: [u8; 16],
    pub smem_start: usize,
    pub smem_len: u32,
    pub fb_type: u32,
    pub type_aux: u32,
    pub visual: u32,
    pub xpanstep: u16,
    pub ypanstep: u16,
    pub ywrapstep: u16,
    pub line_length: u32,
    pub mmio_start: usize,
    pub mmio_len: u32,
    pub accel: u32,
    pub capabilities: u16,
    pub reserved: [u16; 2],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PixelFormat {
    pub bpp: u8,
    pub red: FbBitfield,
    pub green: FbBitfield,
    pub blue: FbBitfield,
}

impl PixelFormat {
    pub fn bytes_per_pixel(self) -> u32 {
        u32::from(self.bpp / 8)
    }

    pub fn pack(self, color: [u8; 4]) -> ([u8; 4], usize) {
        let r = u32::from(color[0]);
        let g = u32::from(color[1]);
        let b = u32::from(color[2]);
        let value = ((r >> (8 - self.red.length)) << self.red.offset)
            | ((g >> (8 - self.green.length)) << self.green.offset)
            | ((b >> (8 - self.blue.length)) << self.blue.offset);
        match self.bpp {
            32 => (value.to_le_bytes(), 4),
            24 => {
                let bytes = value.to_le_bytes();
                ([bytes[0], bytes[1], bytes[2], 0], 3)
            }
            16 => {
                let bytes = (value as u16).to_le_bytes();
                ([bytes[0], bytes[1], 0, 0], 2)
            }
            _ => ([0; 4], 0),
        }
    }
}

pub fn validate(var: &FbVarScreenInfo, fix: &FbFixScreenInfo) -> Result<PixelFormat, FbError> {
    if var.xres == 0 || var.yres == 0 {
        return Err(FbError::Unsupported("resolution is zero".to_string()));
    }
    if var.xres_virtual < var.xres || var.yres_virtual < var.yres {
        return Err(FbError::Unsupported(
            "virtual resolution is smaller than visible resolution".to_string(),
        ));
    }
    if var.yoffset > var.yres_virtual.saturating_sub(var.yres) {
        return Err(FbError::Unsupported(format!(
            "yoffset {} exceeds visible window",
            var.yoffset
        )));
    }
    if !matches!(var.bits_per_pixel, 16 | 24 | 32) {
        return Err(FbError::Unsupported(format!(
            "{} bits per pixel is not supported",
            var.bits_per_pixel
        )));
    }
    let format = PixelFormat {
        bpp: var.bits_per_pixel as u8,
        red: var.red,
        green: var.green,
        blue: var.blue,
    };
    let bytes_per_pixel = format.bytes_per_pixel();
    let minimum_stride = var.xres.saturating_mul(bytes_per_pixel);
    if fix.line_length < minimum_stride {
        return Err(FbError::Unsupported(format!(
            "line_length {} is smaller than {}",
            fix.line_length, minimum_stride
        )));
    }
    let visible_rows = var.yoffset.saturating_add(var.yres);
    let needed = u64::from(fix.line_length) * u64::from(visible_rows);
    if u64::from(fix.smem_len) < needed {
        return Err(FbError::Unsupported(format!(
            "smem_len {} is smaller than {}",
            fix.smem_len, needed
        )));
    }
    for field in [format.red, format.green, format.blue] {
        if field.length == 0 || field.offset + field.length > var.bits_per_pixel {
            return Err(FbError::Unsupported("invalid color bitfield".to_string()));
        }
    }
    Ok(format)
}

pub fn can_pan(var: &FbVarScreenInfo, fix: &FbFixScreenInfo, format: PixelFormat) -> bool {
    if var.yres_virtual < var.yres.saturating_mul(2) {
        return false;
    }
    let needed = u64::from(fix.line_length)
        .saturating_mul(u64::from(var.yres))
        .saturating_mul(2);
    u64::from(fix.smem_len) >= needed && format.bytes_per_pixel() > 0
}

pub fn next_offset(current: u32, yres: u32) -> u32 {
    if current == 0 {
        yres
    } else {
        0
    }
}

pub fn write_frame_to(
    buffer: &mut [u8],
    stride: u32,
    format: PixelFormat,
    frame: &Frame,
    yoffset: u32,
) -> Result<(), FbError> {
    let bytes_per_pixel = format.bytes_per_pixel();
    if bytes_per_pixel == 0 {
        return Err(FbError::Unsupported("invalid pixel format".to_string()));
    }
    if stride < frame.width() * bytes_per_pixel {
        return Err(FbError::Unsupported(
            "stride is smaller than the frame row".to_string(),
        ));
    }
    for y in 0..frame.height() {
        let destination_row = yoffset.saturating_add(y);
        let start = destination_row as usize * stride as usize;
        let end = start + frame.width() as usize * bytes_per_pixel as usize;
        if end > buffer.len() {
            return Err(FbError::Unsupported(
                "framebuffer mapping is too small".to_string(),
            ));
        }
        let source_start = y as usize * frame.width() as usize * 4;
        for x in 0..frame.width() as usize {
            let pixel = [
                frame.pixels()[source_start + x * 4],
                frame.pixels()[source_start + x * 4 + 1],
                frame.pixels()[source_start + x * 4 + 2],
                frame.pixels()[source_start + x * 4 + 3],
            ];
            let (packed, length) = format.pack(pixel);
            let offset = start + x * length;
            buffer[offset..offset + length].copy_from_slice(&packed[..length]);
        }
    }
    Ok(())
}

pub struct FbDevice {
    file: File,
    var: FbVarScreenInfo,
    fix: FbFixScreenInfo,
    map: *mut u8,
    map_len: usize,
    format: PixelFormat,
    pan: bool,
    current_offset: u32,
}

impl FbDevice {
    pub fn open(path: &Path) -> Result<Self, FbError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_CLOEXEC)
            .open(path)?;
        let fd = file.as_raw_fd();
        let mut var = FbVarScreenInfo::default();
        let mut fix = FbFixScreenInfo::default();
        ioctl(fd, FBIOGET_VSCREENINFO, &mut var)?;
        ioctl(fd, FBIOGET_FSCREENINFO, &mut fix)?;
        let format = validate(&var, &fix)?;
        let map_len = fix.smem_len as usize;
        if map_len == 0 {
            return Err(FbError::Unsupported("smem_len is zero".to_string()));
        }
        // SAFETY: mapping the framebuffer memory reported by the driver as read/write shared.
        let map = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                map_len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            )
        };
        if map == libc::MAP_FAILED {
            return Err(FbError::Io(io::Error::last_os_error()));
        }

        let pan = can_pan(&var, &fix, format);
        let current_offset = if pan { 0 } else { var.yoffset };
        let mut device = Self {
            file,
            var,
            fix,
            map: map.cast(),
            map_len,
            format,
            pan,
            current_offset,
        };
        let _ = device.unblank();
        if device.pan && device.set_offset(0).is_err() {
            device.pan = false;
            device.current_offset = device.var.yoffset;
        }
        Ok(device)
    }

    pub fn xres(&self) -> u32 {
        self.var.xres
    }

    pub fn yres(&self) -> u32 {
        self.var.yres
    }

    pub fn stride(&self) -> u32 {
        self.fix.line_length
    }

    pub fn format(&self) -> PixelFormat {
        self.format
    }

    pub fn can_pan(&self) -> bool {
        self.pan
    }

    pub fn var(&self) -> &FbVarScreenInfo {
        &self.var
    }

    pub fn fix(&self) -> &FbFixScreenInfo {
        &self.fix
    }

    pub fn unblank(&mut self) -> Result<(), FbError> {
        let fd = self.file.as_raw_fd();
        // SAFETY: fd is valid and FBIOBLANK takes the blanking level as an integer argument.
        let result = unsafe { libc::ioctl(fd, FBIOBLANK as _, FB_BLANK_UNBLANK) };
        if result < 0 {
            Err(FbError::Io(io::Error::last_os_error()))
        } else {
            Ok(())
        }
    }

    pub fn present(&mut self, frame: &Frame) -> Result<(), FbError> {
        if frame.width() != self.xres() || frame.height() != self.yres() {
            return Err(FbError::Unsupported(
                "frame size does not match the framebuffer".to_string(),
            ));
        }
        if self.pan {
            let target = next_offset(self.current_offset, self.var.yres);
            self.write_at(frame, target)?;
            match self.set_offset(target) {
                Ok(()) => {
                    self.current_offset = target;
                    Ok(())
                }
                Err(_) => {
                    self.pan = false;
                    self.current_offset = 0;
                    self.write_at(frame, 0)
                }
            }
        } else {
            self.write_at(frame, self.current_offset)
        }
    }

    fn write_at(&mut self, frame: &Frame, yoffset: u32) -> Result<(), FbError> {
        let stride = self.fix.line_length;
        let format = self.format;
        let buffer = self.buffer_mut();
        write_frame_to(buffer, stride, format, frame, yoffset)
    }

    fn set_offset(&mut self, yoffset: u32) -> Result<(), FbError> {
        self.var.yoffset = yoffset;
        ioctl(
            self.file.as_raw_fd(),
            FBIOPAN_DISPLAY,
            &mut self.var as *mut FbVarScreenInfo,
        )
    }

    fn buffer_mut(&mut self) -> &mut [u8] {
        // SAFETY: map/map_len come from a successful mmap of the framebuffer and stay valid
        // for the lifetime of FbDevice. The returned slice does not outlive `&mut self`.
        unsafe { std::slice::from_raw_parts_mut(self.map, self.map_len) }
    }
}

impl Drop for FbDevice {
    fn drop(&mut self) {
        if self.pan {
            let _ = self.set_offset(0);
        }
        // SAFETY: map/map_len came from mmap in `open` and are not used after this point.
        unsafe {
            libc::munmap(self.map.cast(), self.map_len);
        }
    }
}

pub struct FbConsole {
    tty: File,
}

impl FbConsole {
    pub fn enter() -> io::Result<Self> {
        let tty = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_CLOEXEC)
            .open("/dev/tty")?;
        let fd = tty.as_raw_fd();
        let mut mode: i32 = 0;
        // SAFETY: fd is /dev/tty and KDGETMODE writes an integer mode.
        unsafe {
            if libc::ioctl(fd, KDGETMODE as _, &mut mode) < 0 {
                return Err(io::Error::last_os_error());
            }
        }
        SAVED_KD_MODE.store(mode, Ordering::SeqCst);
        ACTIVE_TTY.store(fd, Ordering::SeqCst);

        // SAFETY: dup2 duplicates /dev/tty onto stdin so crossterm reads the console keyboard.
        unsafe {
            if libc::dup2(fd, libc::STDIN_FILENO) < 0 {
                ACTIVE_TTY.store(-1, Ordering::SeqCst);
                return Err(io::Error::last_os_error());
            }
        }
        enable_raw_mode()?;
        // SAFETY: fd is /dev/tty and KDSETMODE expects an integer mode.
        unsafe {
            if libc::ioctl(fd, KDSETMODE as _, KD_GRAPHICS) < 0 {
                let _ = disable_raw_mode();
                ACTIVE_TTY.store(-1, Ordering::SeqCst);
                return Err(io::Error::last_os_error());
            }
        }
        let _ = execute!(std::io::stdout(), Hide);
        Ok(Self { tty })
    }

    fn fd(&self) -> RawFd {
        self.tty.as_raw_fd()
    }
}

impl Drop for FbConsole {
    fn drop(&mut self) {
        let fd = self.fd();
        let saved = SAVED_KD_MODE.load(Ordering::SeqCst);
        // SAFETY: fd is /dev/tty and KDSETMODE expects an integer mode.
        unsafe {
            libc::ioctl(fd, KDSETMODE as _, saved);
        }
        let _ = execute!(std::io::stdout(), Show);
        let _ = disable_raw_mode();
        ACTIVE_TTY.store(-1, Ordering::SeqCst);
    }
}

pub fn restore_console() {
    let fd = ACTIVE_TTY.swap(-1, Ordering::SeqCst);
    if fd < 0 {
        return;
    }
    let saved = SAVED_KD_MODE.load(Ordering::SeqCst);
    // SAFETY: fd was stored by FbConsole::enter and remains valid while the guard is alive.
    unsafe {
        libc::ioctl(fd, KDSETMODE as _, saved);
    }
    let _ = execute!(std::io::stdout(), Show);
    let _ = disable_raw_mode();
}

pub fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_console();
        default_hook(info);
    }));
}

pub fn is_vt() -> bool {
    current_tty_name().is_some_and(|name| is_vt_name(&name))
}

pub fn current_tty_name() -> Option<String> {
    let name = tty_name(libc::STDIN_FILENO).or_else(|| {
        let tty = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/tty")
            .ok()?;
        tty_name(tty.as_raw_fd())
    });
    let active = std::fs::read_to_string("/sys/class/tty/tty0/active").ok();
    resolve_tty_name(name, active.as_deref())
}

fn resolve_tty_name(name: Option<String>, active: Option<&str>) -> Option<String> {
    match name.as_deref() {
        Some(name) if name != "/dev/tty" => Some(name.to_string()),
        Some(_) => {
            let active = active?.trim();
            if active.is_empty() {
                None
            } else {
                Some(format!("/dev/{active}"))
            }
        }
        None => None,
    }
}

fn tty_name(fd: RawFd) -> Option<String> {
    // SAFETY: ttyname is called with a valid fd and the returned pointer is copied immediately.
    unsafe {
        let pointer = libc::ttyname(fd);
        if pointer.is_null() {
            return None;
        }
        Some(CStr::from_ptr(pointer).to_string_lossy().to_string())
    }
}

pub fn is_vt_name(name: &str) -> bool {
    name.strip_prefix("/dev/tty")
        .is_some_and(|rest| !rest.is_empty() && rest.chars().all(|ch| ch.is_ascii_digit()))
}

#[allow(dead_code)]
fn ioctl<T>(fd: RawFd, request: u64, argument: *mut T) -> Result<(), FbError> {
    // SAFETY: callers pass a valid fd and a pointer to a structure matching the ioctl request.
    unsafe {
        if libc::ioctl(fd, request as _, argument) < 0 {
            Err(FbError::Io(io::Error::last_os_error()))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(offset: u32, length: u32) -> FbBitfield {
        FbBitfield {
            offset,
            length,
            msb_right: 0,
        }
    }

    fn var_1080p() -> FbVarScreenInfo {
        FbVarScreenInfo {
            xres: 1920,
            yres: 1080,
            xres_virtual: 1920,
            yres_virtual: 2160,
            yoffset: 0,
            bits_per_pixel: 32,
            red: field(16, 8),
            green: field(8, 8),
            blue: field(0, 8),
            transp: field(24, 8),
            ..FbVarScreenInfo::default()
        }
    }

    fn fix_1080p() -> FbFixScreenInfo {
        FbFixScreenInfo {
            smem_len: 1920 * 4 * 2160,
            line_length: 1920 * 4,
            ..FbFixScreenInfo::default()
        }
    }

    #[test]
    fn abi_sizes_match_x86_64_linux() {
        assert_eq!(std::mem::size_of::<FbVarScreenInfo>(), 160);
        assert_eq!(std::mem::size_of::<FbFixScreenInfo>(), 80);
    }

    #[test]
    fn validates_32_bpp() {
        let format = validate(&var_1080p(), &fix_1080p()).unwrap();
        assert_eq!(format.bpp, 32);
        assert!(can_pan(&var_1080p(), &fix_1080p(), format));
    }

    #[test]
    fn validates_24_and_16_bpp() {
        let mut var = var_1080p();
        var.bits_per_pixel = 24;
        var.red = field(16, 8);
        var.green = field(8, 8);
        var.blue = field(0, 8);
        let mut fix = fix_1080p();
        fix.line_length = 1920 * 3;
        fix.smem_len = 1920 * 3 * 2160;
        assert_eq!(validate(&var, &fix).unwrap().bpp, 24);

        var.bits_per_pixel = 16;
        var.red = field(11, 5);
        var.green = field(5, 6);
        var.blue = field(0, 5);
        fix.line_length = 1920 * 2;
        fix.smem_len = 1920 * 2 * 2160;
        assert_eq!(validate(&var, &fix).unwrap().bpp, 16);
    }

    #[test]
    fn rejects_unsupported_formats() {
        let mut var = var_1080p();
        var.bits_per_pixel = 8;
        assert!(validate(&var, &fix_1080p()).is_err());

        let mut var = var_1080p();
        var.xres_virtual = 100;
        assert!(validate(&var, &fix_1080p()).is_err());

        let mut var = var_1080p();
        var.yoffset = 5000;
        assert!(validate(&var, &fix_1080p()).is_err());

        let mut fix = fix_1080p();
        fix.line_length = 100;
        assert!(validate(&var_1080p(), &fix).is_err());

        let mut fix = fix_1080p();
        fix.smem_len = 100;
        assert!(validate(&var_1080p(), &fix).is_err());

        let mut var = var_1080p();
        var.red = field(30, 8);
        assert!(validate(&var, &fix_1080p()).is_err());
    }

    #[test]
    fn disables_pan_without_double_buffer() {
        let mut var = var_1080p();
        var.yres_virtual = var.yres;
        let mut fix = fix_1080p();
        fix.smem_len = fix.line_length * var.yres;
        let format = validate(&var, &fix).unwrap();
        assert!(!can_pan(&var, &fix, format));
    }

    #[test]
    fn pan_offset_alternates() {
        assert_eq!(next_offset(0, 1080), 1080);
        assert_eq!(next_offset(1080, 1080), 0);
        assert_eq!(next_offset(42, 1080), 0);
    }

    #[test]
    fn packs_32_24_and_16_bpp() {
        let format32 = PixelFormat {
            bpp: 32,
            red: field(16, 8),
            green: field(8, 8),
            blue: field(0, 8),
        };
        assert_eq!(
            format32.pack([0x12, 0x34, 0x56, 0xff]),
            ([0x56, 0x34, 0x12, 0x00], 4)
        );

        let format24 = PixelFormat {
            bpp: 24,
            ..format32
        };
        assert_eq!(
            format24.pack([0x12, 0x34, 0x56, 0xff]),
            ([0x56, 0x34, 0x12, 0x00], 3)
        );

        let format16 = PixelFormat {
            bpp: 16,
            red: field(11, 5),
            green: field(5, 6),
            blue: field(0, 5),
        };
        assert_eq!(
            format16.pack([0xf8, 0x00, 0x00, 0xff]),
            ([0x00, 0xf8, 0x00, 0x00], 2)
        );
    }

    #[test]
    fn writes_rows_with_stride_padding() {
        let frame = Frame::new(2, 1, [1, 2, 3, 255]);
        let format = PixelFormat {
            bpp: 32,
            red: field(16, 8),
            green: field(8, 8),
            blue: field(0, 8),
        };
        let mut buffer = vec![0u8; 3 * 4 + 4];
        write_frame_to(&mut buffer, 12, format, &frame, 0).unwrap();
        assert_eq!(&buffer[0..4], &[3, 2, 1, 0]);
        assert_eq!(&buffer[4..8], &[3, 2, 1, 0]);
        assert_eq!(&buffer[8..12], &[0, 0, 0, 0]);
    }

    #[test]
    fn writes_a_full_hd_frame() {
        let frame = Frame::new(1920, 1080, [0x12, 0x34, 0x56, 0xff]);
        let format = PixelFormat {
            bpp: 32,
            red: field(16, 8),
            green: field(8, 8),
            blue: field(0, 8),
        };
        let mut buffer = vec![0u8; 1920 * 4 * 1080];
        write_frame_to(&mut buffer, 1920 * 4, format, &frame, 0).unwrap();
        assert_eq!(&buffer[0..4], &[0x56, 0x34, 0x12, 0x00]);
        assert_eq!(&buffer[buffer.len() - 4..], &[0x56, 0x34, 0x12, 0x00]);
    }

    #[test]
    fn detects_virtual_terminals() {
        assert!(is_vt_name("/dev/tty2"));
        assert!(!is_vt_name("/dev/pts/0"));
        assert!(!is_vt_name("/dev/tty"));
    }

    #[test]
    fn resolves_generic_tty_to_active_vt() {
        assert_eq!(
            resolve_tty_name(Some("/dev/tty3".to_string()), Some("tty3\n")),
            Some("/dev/tty3".to_string())
        );
        assert_eq!(
            resolve_tty_name(Some("/dev/tty".to_string()), Some("tty3\n")),
            Some("/dev/tty3".to_string())
        );
        assert_eq!(resolve_tty_name(None, Some("tty3\n")), None);
        assert_eq!(
            resolve_tty_name(Some("/dev/pts/0".to_string()), Some("tty3\n")),
            Some("/dev/pts/0".to_string())
        );
        assert_eq!(resolve_tty_name(Some("/dev/tty".to_string()), None), None);
    }
}
