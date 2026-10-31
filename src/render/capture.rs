use std::io;
use std::path::Path;

pub use crate::render::raster::encode_png;
use crate::render::raster::Frame;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::raster::glyph;

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
