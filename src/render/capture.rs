use std::io;
use std::path::Path;

use crate::util::rng::SplitMix64;

pub const CAPTURE_WIDTH: u32 = 1344;
pub const CAPTURE_HEIGHT: u32 = 756;

const BG: [u8; 4] = [0x0b, 0x0f, 0x14, 0xff];
const SURFACE: [u8; 4] = [0x12, 0x18, 0x21, 0xff];
const SURFACE_ELEVATED: [u8; 4] = [0x1a, 0x23, 0x30, 0xff];
const BORDER: [u8; 4] = [0x2a, 0x36, 0x46, 0xff];
const GRID: [u8; 4] = [0x1e, 0x28, 0x36, 0xff];
const TEXT: [u8; 4] = [0xe6, 0xed, 0xf3, 0xff];
const MUTED: [u8; 4] = [0x9a, 0xa7, 0xb6, 0xff];
const ACCENT: [u8; 4] = [0x38, 0xbd, 0xf8, 0xff];
const SUCCESS: [u8; 4] = [0x34, 0xd3, 0x99, 0xff];
const WARNING: [u8; 4] = [0xfb, 0xbf, 0x24, 0xff];
const VIOLET: [u8; 4] = [0xa7, 0x8b, 0xfa, 0xff];
const PINK: [u8; 4] = [0xf4, 0x72, 0xb6, 0xff];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl Frame {
    pub fn new(width: u32, height: u32, color: [u8; 4]) -> Self {
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);
        for _ in 0..(width * height) {
            pixels.extend_from_slice(&color);
        }
        Self {
            width,
            height,
            pixels,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: [u8; 4]) {
        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = (x + width).min(self.width as i32);
        let y1 = (y + height).min(self.height as i32);
        if x0 >= x1 || y0 >= y1 {
            return;
        }
        for py in y0..y1 {
            for px in x0..x1 {
                self.set(px, py, color);
            }
        }
    }

    pub fn stroke_rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: [u8; 4]) {
        self.fill_rect(x, y, width, 1, color);
        self.fill_rect(x, y + height - 1, width, 1, color);
        self.fill_rect(x, y, 1, height, color);
        self.fill_rect(x + width - 1, y, 1, height, color);
    }

    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: [u8; 4], thickness: i32) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let (mut x, mut y) = (x0, y0);
        loop {
            self.fill_rect(
                x - thickness / 2,
                y - thickness / 2,
                thickness,
                thickness,
                color,
            );
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn disc(&mut self, cx: i32, cy: i32, radius: i32, color: [u8; 4]) {
        for y in -radius..=radius {
            for x in -radius..=radius {
                if x * x + y * y <= radius * radius {
                    self.set(cx + x, cy + y, color);
                }
            }
        }
    }

    pub fn ring(&mut self, cx: i32, cy: i32, radius: i32, color: [u8; 4], thickness: i32) {
        let outer = radius * radius;
        let inner = (radius - thickness).max(0);
        let inner = inner * inner;
        for y in -radius..=radius {
            for x in -radius..=radius {
                let distance = x * x + y * y;
                if distance <= outer && distance >= inner {
                    self.set(cx + x, cy + y, color);
                }
            }
        }
    }

    pub fn text(&mut self, x: i32, y: i32, value: &str, color: [u8; 4], scale: i32) {
        let mut cursor = x;
        for ch in value.chars() {
            let glyph = glyph(ch);
            for (row, bits) in glyph.iter().enumerate() {
                for column in 0..5 {
                    if bits & (1 << (4 - column)) != 0 {
                        self.fill_rect(
                            cursor + column * scale,
                            y + row as i32 * scale,
                            scale,
                            scale,
                            color,
                        );
                    }
                }
            }
            cursor += 6 * scale;
        }
    }

    fn set(&mut self, x: i32, y: i32, color: [u8; 4]) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        let index = ((y as u32 * self.width + x as u32) * 4) as usize;
        self.pixels[index..index + 4].copy_from_slice(&color);
    }
}

pub fn render_capture(seed: u64) -> Frame {
    let mut frame = Frame::new(CAPTURE_WIDTH, CAPTURE_HEIGHT, BG);
    let mut rng = SplitMix64::new(seed);

    frame.fill_rect(40, 28, 1264, 76, SURFACE);
    frame.stroke_rect(40, 28, 1264, 76, BORDER);
    frame.text(64, 44, "MOUSE TEST", ACCENT, 5);
    frame.text(64, 82, "DETERMINISTIC CAPTURE 1344X756", MUTED, 2);
    frame.text(1130, 44, "PHASE 3", WARNING, 3);
    frame.text(1130, 76, "SEED", MUTED, 2);
    let seed_text = format!("{seed}");
    frame.text(1200, 76, &seed_text, TEXT, 2);

    frame.fill_rect(40, 124, 884, 552, SURFACE);
    frame.stroke_rect(40, 124, 884, 552, BORDER);
    frame.text(62, 142, "MOTION", MUTED, 2);

    for index in 1..12 {
        let x = 40 + index * 74;
        frame.line(x, 124, x, 676, GRID, 1);
    }
    for index in 1..7 {
        let y = 124 + index * 79;
        frame.line(40, y, 924, y, GRID, 1);
    }

    let mut points = Vec::new();
    for _ in 0..7 {
        points.push((rng.range_i32(300, 780), rng.range_i32(220, 560)));
    }
    for pair in points.windows(2) {
        frame.line(pair[0].0, pair[0].1, pair[1].0, pair[1].1, ACCENT, 3);
    }
    for point in &points {
        frame.disc(point.0, point.1, 3, ACCENT);
    }

    let cursor_x = 700;
    let cursor_y = 360;
    frame.ring(cursor_x, cursor_y, 62, ACCENT, 3);
    frame.ring(cursor_x, cursor_y, 30, ACCENT, 2);
    frame.disc(cursor_x, cursor_y, 5, ACCENT);

    let click_x = 520;
    let click_y = 470;
    frame.ring(click_x, click_y, 44, SUCCESS, 4);
    frame.ring(click_x, click_y, 66, SUCCESS, 2);
    frame.line(click_x - 70, click_y, click_x - 36, click_y, SUCCESS, 2);
    frame.line(click_x + 36, click_y, click_x + 70, click_y, SUCCESS, 2);
    frame.line(click_x, click_y - 70, click_x, click_y - 36, SUCCESS, 2);
    frame.line(click_x, click_y + 36, click_x, click_y + 70, SUCCESS, 2);

    frame.fill_rect(948, 124, 356, 552, SURFACE);
    frame.stroke_rect(948, 124, 356, 552, BORDER);
    let mut y = 148;
    let rows: [(&str, [u8; 4]); 15] = [
        ("DEVICE  DEMO MOUSE", TEXT),
        ("SOURCE  EVDEV", TEXT),
        ("", MUTED),
        ("BUTTONS", MUTED),
        ("L  24    R  12    M  6", SUCCESS),
        ("", MUTED),
        ("SCROLL  V +3  H 0", TEXT),
        ("", MUTED),
        ("RATE  1000 HZ", TEXT),
        ("JITTER  42 US", TEXT),
        ("CONFIDENCE  HIGH", TEXT),
        ("", MUTED),
        ("TARGETS  3/9", ACCENT),
        ("ACCURACY  92 PERCENT", TEXT),
        ("LAST CLICK  HIT", SUCCESS),
    ];
    for (text, color) in rows {
        frame.text(972, y, text, color, 2);
        y += if text.is_empty() { 14 } else { 30 };
    }

    frame.fill_rect(972, 520, 308, 72, SURFACE_ELEVATED);
    frame.stroke_rect(972, 520, 308, 72, BORDER);
    frame.text(990, 536, "TRAIL  CLICK RING", MUTED, 2);
    frame.text(990, 562, "CURSOR  TARGET READY", TEXT, 2);

    frame.fill_rect(972, 612, 308, 44, SURFACE_ELEVATED);
    frame.stroke_rect(972, 612, 308, 44, BORDER);
    frame.disc(996, 634, 5, VIOLET);
    frame.text(1012, 626, "EXTRA BUTTON", TEXT, 2);
    frame.disc(1196, 634, 5, PINK);
    frame.text(1212, 626, "PINK", TEXT, 2);

    frame.fill_rect(40, 692, 1264, 36, SURFACE);
    frame.text(64, 704, "Q QUIT   TAB MODE   T DEVICE   F1 HELP", MUTED, 2);
    frame.text(1120, 704, "FASE 3", WARNING, 2);

    frame
}

pub fn write_capture(path: &Path, seed: u64) -> io::Result<()> {
    let frame = render_capture(seed);
    std::fs::write(path, encode_png(&frame))
}

pub fn encode_png(frame: &Frame) -> Vec<u8> {
    let mut raw = Vec::with_capacity((frame.width as usize * 4 + 1) * frame.height as usize);
    for row in 0..frame.height {
        raw.push(0);
        let start = row as usize * frame.width as usize * 4;
        let end = start + frame.width as usize * 4;
        raw.extend_from_slice(&frame.pixels[start..end]);
    }

    let mut zlib = Vec::with_capacity(raw.len() + raw.len() / 65535 * 5 + 16);
    zlib.extend_from_slice(&[0x78, 0x01]);
    let mut offset = 0;
    while offset < raw.len() {
        let remaining = raw.len() - offset;
        let len = remaining.min(65_535);
        let final_block = offset + len == raw.len();
        zlib.push(if final_block { 0x01 } else { 0x00 });
        let len_u16 = len as u16;
        zlib.extend_from_slice(&len_u16.to_le_bytes());
        zlib.extend_from_slice(&(!len_u16).to_le_bytes());
        zlib.extend_from_slice(&raw[offset..offset + len]);
        offset += len;
    }
    zlib.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut png = Vec::new();
    png.extend_from_slice(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]);
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&frame.width.to_be_bytes());
    ihdr.extend_from_slice(&frame.height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    write_chunk(&mut png, b"IHDR", &ihdr);
    write_chunk(&mut png, b"IDAT", &zlib);
    write_chunk(&mut png, b"IEND", &[]);
    png
}

fn write_chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let mut crc_input = Vec::with_capacity(4 + data.len());
    crc_input.extend_from_slice(kind);
    crc_input.extend_from_slice(data);
    out.extend_from_slice(&crc32(&crc_input).to_be_bytes());
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in data {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

fn adler32(data: &[u8]) -> u32 {
    const MOD: u32 = 65_521;
    let mut a = 1u32;
    let mut b = 0u32;
    for byte in data {
        a = (a + u32::from(*byte)) % MOD;
        b = (b + a) % MOD;
    }
    (b << 16) | a
}

fn glyph(ch: char) -> [u8; 7] {
    match ch.to_ascii_uppercase() {
        'A' => [0x0e, 0x11, 0x11, 0x1f, 0x11, 0x11, 0x11],
        'B' => [0x1e, 0x11, 0x11, 0x1e, 0x11, 0x11, 0x1e],
        'C' => [0x0e, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0e],
        'D' => [0x1e, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1e],
        'E' => [0x1f, 0x10, 0x10, 0x1e, 0x10, 0x10, 0x1f],
        'F' => [0x1f, 0x10, 0x10, 0x1e, 0x10, 0x10, 0x10],
        'G' => [0x0e, 0x11, 0x10, 0x17, 0x11, 0x11, 0x0e],
        'H' => [0x11, 0x11, 0x11, 0x1f, 0x11, 0x11, 0x11],
        'I' => [0x0e, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0e],
        'J' => [0x07, 0x02, 0x02, 0x02, 0x02, 0x12, 0x0c],
        'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1f],
        'M' => [0x11, 0x1b, 0x15, 0x15, 0x11, 0x11, 0x11],
        'N' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11],
        'O' => [0x0e, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0e],
        'P' => [0x1e, 0x11, 0x11, 0x1e, 0x10, 0x10, 0x10],
        'Q' => [0x0e, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0d],
        'R' => [0x1e, 0x11, 0x11, 0x1e, 0x14, 0x12, 0x11],
        'S' => [0x0f, 0x10, 0x10, 0x0e, 0x01, 0x01, 0x1e],
        'T' => [0x1f, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0e],
        'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0a, 0x04],
        'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x15, 0x0a],
        'X' => [0x11, 0x11, 0x0a, 0x04, 0x0a, 0x11, 0x11],
        'Y' => [0x11, 0x11, 0x0a, 0x04, 0x04, 0x04, 0x04],
        'Z' => [0x1f, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1f],
        '0' => [0x0e, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0e],
        '1' => [0x04, 0x0c, 0x04, 0x04, 0x04, 0x04, 0x0e],
        '2' => [0x0e, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1f],
        '3' => [0x1f, 0x02, 0x04, 0x02, 0x01, 0x11, 0x0e],
        '4' => [0x02, 0x06, 0x0a, 0x12, 0x1f, 0x02, 0x02],
        '5' => [0x1f, 0x10, 0x1e, 0x01, 0x01, 0x11, 0x0e],
        '6' => [0x06, 0x08, 0x10, 0x1e, 0x11, 0x11, 0x0e],
        '7' => [0x1f, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        '8' => [0x0e, 0x11, 0x11, 0x0e, 0x11, 0x11, 0x0e],
        '9' => [0x0e, 0x11, 0x11, 0x0f, 0x01, 0x02, 0x0c],
        '.' => [0, 0, 0, 0, 0, 0x06, 0x06],
        ',' => [0, 0, 0, 0, 0x06, 0x06, 0x04],
        ':' => [0, 0x06, 0x06, 0, 0x06, 0x06, 0],
        '+' => [0, 0x04, 0x04, 0x1f, 0x04, 0x04, 0],
        '-' => [0, 0, 0, 0x1f, 0, 0, 0],
        '/' => [0x01, 0x02, 0x02, 0x04, 0x08, 0x08, 0x10],
        '[' => [0x0e, 0x08, 0x08, 0x08, 0x08, 0x08, 0x0e],
        ']' => [0x0e, 0x02, 0x02, 0x02, 0x02, 0x02, 0x0e],
        '!' => [0x04, 0x04, 0x04, 0x04, 0x04, 0, 0x04],
        '?' => [0x0e, 0x11, 0x01, 0x02, 0x04, 0, 0x04],
        '_' => [0, 0, 0, 0, 0, 0, 0x1f],
        _ => [0, 0, 0, 0, 0, 0, 0],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_has_expected_dimensions() {
        let frame = render_capture(7);
        assert_eq!(frame.width(), CAPTURE_WIDTH);
        assert_eq!(frame.height(), CAPTURE_HEIGHT);
        assert_eq!(
            frame.pixels().len(),
            (CAPTURE_WIDTH * CAPTURE_HEIGHT * 4) as usize
        );
    }

    #[test]
    fn capture_is_deterministic() {
        let a = encode_png(&render_capture(42));
        let b = encode_png(&render_capture(42));
        assert_eq!(a, b);
    }

    #[test]
    fn different_seeds_diverge() {
        let a = encode_png(&render_capture(1));
        let b = encode_png(&render_capture(2));
        assert_ne!(a, b);
    }

    #[test]
    fn png_signature_and_chunks_are_valid() {
        let png = encode_png(&render_capture(3));
        assert_eq!(&png[..8], &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]);
        let mut offset = 8;
        let mut chunks = Vec::new();
        while offset < png.len() {
            let len = u32::from_be_bytes(png[offset..offset + 4].try_into().unwrap()) as usize;
            let kind = &png[offset + 4..offset + 8];
            chunks.push((kind.to_vec(), len));
            offset += 12 + len;
        }
        assert_eq!(chunks[0].0, b"IHDR");
        assert_eq!(chunks[1].0, b"IDAT");
        assert_eq!(chunks.last().unwrap().0, b"IEND");
    }

    #[test]
    fn frame_draws_non_background_pixels() {
        let frame = render_capture(9);
        assert!(frame.pixels().chunks(4).any(|pixel| pixel != BG));
    }

    #[test]
    fn glyphs_cover_capture_text() {
        let text = "MOUSE TEST DETERMINISTIC CAPTURE 1344X756 PHASE 3 SEED MOTION DEVICE DEMO MOUSE SOURCE EVDEV";
        for ch in text.chars().filter(|ch| !ch.is_whitespace()) {
            assert_ne!(glyph(ch), [0; 7], "missing glyph for {ch}");
        }
    }
}
