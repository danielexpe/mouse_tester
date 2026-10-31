use ratatui::style::Color;

use crate::app::App;
use crate::core::codes::{
    short_label, BTN_BACK, BTN_EXTRA, BTN_FORWARD, BTN_LEFT, BTN_MIDDLE, BTN_RIGHT, BTN_SIDE,
    BTN_TASK,
};
use crate::core::effects::{Edge, EffectKind};
use crate::core::geometry::{Bounds, Point};
use crate::core::metrics::Confidence;
use crate::core::modes::{ScrollStep, TargetKind, TargetResult};
use crate::core::scroll::ScrollAxis;
use crate::i18n::{tr, Key};
use crate::render::raster::Frame;
use crate::state::Mode;

const BUTTONS: [u16; 8] = [
    BTN_LEFT,
    BTN_RIGHT,
    BTN_MIDDLE,
    BTN_SIDE,
    BTN_EXTRA,
    BTN_FORWARD,
    BTN_BACK,
    BTN_TASK,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Rect {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl Rect {
    fn inset(self, amount: i32) -> Self {
        Self {
            x: self.x + amount,
            y: self.y + amount,
            w: (self.w - amount * 2).max(0),
            h: (self.h - amount * 2).max(0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Areas {
    header: Rect,
    canvas: Rect,
    side: Rect,
    footer: Rect,
    scale: i32,
}

fn areas(width: i32, height: i32) -> Areas {
    let scale = (height / 360).clamp(1, 4);
    let margin = (scale * 8).max(12);
    let header = (7 * scale * 3 + margin).min(height / 4);
    let footer = (7 * scale + margin).min(height / 6);
    let gap = margin;
    let content_y = header + margin;
    let content_h = (height - content_y - footer - margin).max(80);

    if width >= 1280 {
        let side_w = ((width as f32 * 0.29) as i32).clamp(320, 560);
        let canvas_w = (width - margin * 2 - side_w - gap).max(120);
        Areas {
            header: Rect {
                x: margin,
                y: margin,
                w: width - margin * 2,
                h: header,
            },
            canvas: Rect {
                x: margin,
                y: content_y,
                w: canvas_w,
                h: content_h,
            },
            side: Rect {
                x: margin + canvas_w + gap,
                y: content_y,
                w: side_w,
                h: content_h,
            },
            footer: Rect {
                x: margin,
                y: height - footer - margin,
                w: width - margin * 2,
                h: footer,
            },
            scale,
        }
    } else {
        let canvas_h = (content_h as f32 * 0.58) as i32;
        let side_y = content_y + canvas_h + gap;
        Areas {
            header: Rect {
                x: margin,
                y: margin,
                w: width - margin * 2,
                h: header,
            },
            canvas: Rect {
                x: margin,
                y: content_y,
                w: width - margin * 2,
                h: canvas_h,
            },
            side: Rect {
                x: margin,
                y: side_y,
                w: width - margin * 2,
                h: (height - side_y - footer - margin).max(40),
            },
            footer: Rect {
                x: margin,
                y: height - footer - margin,
                w: width - margin * 2,
                h: footer,
            },
            scale,
        }
    }
}

pub fn render(app: &App, width: u32, height: u32) -> Frame {
    let theme = app.theme;
    let mut frame = Frame::new(width, height, rgba(theme.bg));
    let areas = areas(width as i32, height as i32);

    draw_header(&mut frame, app, areas.header, areas.scale);
    draw_canvas(&mut frame, app, areas.canvas, areas.scale);
    draw_side(&mut frame, app, areas.side, areas.scale);
    draw_footer(&mut frame, app, areas.footer, areas.scale);
    frame
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect, scale: i32) {
    let theme = app.theme;
    let pad = scale * 6;
    frame.fill_rect(area.x, area.y, area.w, area.h, rgba(theme.surface));
    frame.stroke_rect(area.x, area.y, area.w, area.h, rgba(theme.border));

    let title_scale = scale + 1;
    frame.text(
        area.x + pad,
        area.y + pad,
        "MOUSE TEST",
        rgba(theme.accent),
        title_scale,
    );
    frame.text(
        area.x + pad + 11 * 6 * title_scale,
        area.y + pad + title_scale,
        &format!("V{}", env!("CARGO_PKG_VERSION")),
        rgba(theme.muted),
        scale,
    );
    frame.text(
        area.x + pad + 18 * 6 * title_scale,
        area.y + pad + title_scale,
        &format!("FASE {}", phase_label(app)),
        rgba(theme.warning),
        scale,
    );

    let mode = mode_label(app);
    frame.text(
        area.x + area.w - pad - mode.len() as i32 * 6 * scale,
        area.y + pad + title_scale,
        mode,
        rgba(theme.text),
        scale,
    );

    let device = app
        .session
        .primary_device()
        .map(|device| device.name.as_str())
        .unwrap_or(tr(app.lang, Key::NoDevice));
    let source = if app.degraded {
        "TUI-MOUSE"
    } else {
        app.session
            .last_source
            .map(|source| source.label())
            .unwrap_or("evdev")
    };
    let line = sanitize(&format!(
        "{}: {}   {}: {}   {}: FB",
        tr(app.lang, Key::Device),
        truncate(device, 36),
        tr(app.lang, Key::Source),
        source,
        tr(app.lang, Key::Backend)
    ));
    let y = area.y + pad + title_scale * 8 + scale * 5;
    frame.text(area.x + pad, y, &line, rgba(theme.muted), scale);

    if let Some(message) = banner_text(app) {
        frame.text(
            area.x + pad,
            y + 8 * scale + scale * 3,
            &truncate(&message, ((area.w - pad * 2) / (6 * scale)).max(8) as usize),
            rgba(theme.danger),
            scale,
        );
    }
}

fn banner_text(app: &App) -> Option<String> {
    if let Some(message) = &app.device_message {
        return Some(sanitize(message));
    }
    if let Some(message) = &app.banner {
        return Some(sanitize(message));
    }
    if app.degraded {
        return Some(tr(app.lang, Key::BannerDegraded).to_string());
    }
    None
}

fn draw_canvas(frame: &mut Frame, app: &App, area: Rect, scale: i32) {
    let theme = app.theme;
    let now = app.now_us;
    frame.fill_rect(area.x, area.y, area.w, area.h, rgba(theme.surface));
    frame.stroke_rect(area.x, area.y, area.w, area.h, rgba(theme.border));
    let pad = scale * 5;
    frame.text(
        area.x + pad,
        area.y + pad,
        "MOTION",
        rgba(theme.muted),
        scale,
    );

    let inner = area.inset(pad + 7 * scale);
    if inner.w <= 8 || inner.h <= 8 {
        return;
    }
    let bounds = Bounds::new(1344, 756).with_margin(0);
    let sx = inner.w as f32 / bounds.width as f32;
    let sy = inner.h as f32 / bounds.height as f32;
    let scale_factor = sx.min(sy);
    let map = |point: Point| canvas_point(inner, bounds, point);

    for index in 1..4 {
        let fraction = index as f32 / 4.0;
        let x = inner.x + (inner.w as f32 * fraction) as i32;
        let y = inner.y + (inner.h as f32 * fraction) as i32;
        frame.line(x, inner.y, x, inner.y + inner.h, rgba(theme.grid), 1);
        frame.line(inner.x, y, inner.x + inner.w, y, rgba(theme.grid), 1);
    }

    if app.session.mode == Mode::Targets {
        draw_targets(frame, app, map, scale_factor, now);
    }

    if !app.flags.no_trail {
        let trail: Vec<(i32, i32)> = app
            .session
            .pointer
            .trail()
            .iter()
            .map(|point| map(point.pos))
            .collect();
        for pair in trail.windows(2) {
            frame.line(
                pair[0].0,
                pair[0].1,
                pair[1].0,
                pair[1].1,
                rgba(theme.accent),
                (2.0 * scale_factor).max(1.0) as i32,
            );
        }
    }

    if !app.flags.no_animations {
        for effect in app.session.effects.iter() {
            let progress = effect.progress(now);
            let (x, y) = map(effect.pos);
            match effect.kind {
                EffectKind::Ripple { code, radius_scale } => {
                    frame.ring(
                        x,
                        y,
                        ((1.5 + progress * 10.0) * radius_scale * scale_factor).max(1.0) as i32,
                        rgba(theme.button(code)),
                        2,
                    );
                }
                EffectKind::DoubleRipple { code } => {
                    frame.ring(
                        x,
                        y,
                        ((1.5 + progress * 10.0) * scale_factor).max(1.0) as i32,
                        rgba(theme.button(code)),
                        2,
                    );
                    frame.ring(
                        x,
                        y,
                        ((1.0 + progress * 0.7 * 8.0) * scale_factor).max(1.0) as i32,
                        rgba(theme.button(code)),
                        1,
                    );
                }
                EffectKind::WheelPulse {
                    axis,
                    direction,
                    pulses,
                    hires_scale,
                } => {
                    for index in 0..pulses {
                        let offset = (f32::from(index) + 1.0) * 3.0 * hires_scale * scale_factor;
                        let signed = offset * f32::from(direction);
                        let (x2, y2) = match axis {
                            ScrollAxis::Vertical => (x, y - signed as i32),
                            ScrollAxis::Horizontal => (x + signed as i32, y),
                        };
                        frame.line(x, y, x2, y2, rgba(theme.warning), 2);
                    }
                }
                EffectKind::EdgeFlash { edge } => {
                    let color = rgba(theme.danger);
                    match edge {
                        Edge::Left => {
                            frame.line(inner.x, inner.y, inner.x, inner.y + inner.h, color, 2)
                        }
                        Edge::Right => frame.line(
                            inner.x + inner.w,
                            inner.y,
                            inner.x + inner.w,
                            inner.y + inner.h,
                            color,
                            2,
                        ),
                        Edge::Top => {
                            frame.line(inner.x, inner.y, inner.x + inner.w, inner.y, color, 2)
                        }
                        Edge::Bottom => frame.line(
                            inner.x,
                            inner.y + inner.h,
                            inner.x + inner.w,
                            inner.y + inner.h,
                            color,
                            2,
                        ),
                    }
                }
                EffectKind::DragRelease { start, end } => {
                    let (x1, y1) = map(start);
                    let (x2, y2) = map(end);
                    frame.line(x1, y1, x2, y2, rgba(theme.drag), 2);
                }
            }
        }
    }

    let (cursor_x, cursor_y) = map(app.session.pointer.pos());
    let cursor_radius = (6.0 * scale_factor).max(3.0) as i32;
    frame.ring(cursor_x, cursor_y, cursor_radius, rgba(theme.accent), 2);
    frame.disc(
        cursor_x,
        cursor_y,
        (2.0 * scale_factor).max(1.0) as i32,
        rgba(theme.accent),
    );
}

fn canvas_point(inner: Rect, bounds: Bounds, point: Point) -> (i32, i32) {
    let sx = inner.w as f32 / bounds.width as f32;
    let sy = inner.h as f32 / bounds.height as f32;
    (
        inner.x + (point.x * sx) as i32,
        inner.y + (point.y * sy) as i32,
    )
}

fn draw_targets(
    frame: &mut Frame,
    app: &App,
    map: impl Fn(Point) -> (i32, i32),
    scale_factor: f32,
    now: u64,
) {
    let theme = app.theme;
    let challenge = &app.session.targets;
    if !challenge.is_done() {
        let target = challenge.target();
        let (x, y) = map(target.center);
        let radius = (target.radius * scale_factor).max(4.0) as i32;
        let color = match target.kind {
            TargetKind::Click => rgba(theme.accent),
            TargetKind::Drag => rgba(theme.drag),
            TargetKind::DoubleClick => rgba(theme.warning),
        };
        frame.ring(x, y, radius, color, 3);
        if target.kind == TargetKind::DoubleClick {
            frame.ring(x, y, (radius as f32 * 0.62) as i32, color, 2);
        }
        let arm = (radius as f32 * 1.25) as i32;
        let gap = (radius as f32 * 0.35) as i32;
        frame.line(x - arm, y, x - gap, y, color, 2);
        frame.line(x + gap, y, x + arm, y, color, 2);
        frame.line(x, y - arm, x, y - gap, color, 2);
        frame.line(x, y + gap, x, y + arm, color, 2);
    }
    if challenge.result_visible(now) {
        if let Some(pos) = challenge.last_eval_pos() {
            let (x, y) = map(pos);
            let color = match challenge.last_result() {
                Some(TargetResult::Hit) => rgba(theme.success),
                Some(TargetResult::Miss) | None => rgba(theme.danger),
            };
            let radius = (challenge.target().radius * scale_factor * 1.35).max(4.0) as i32;
            frame.ring(x, y, radius, color, 2);
        }
    }
}

fn draw_side(frame: &mut Frame, app: &App, area: Rect, scale: i32) {
    let theme = app.theme;
    frame.fill_rect(area.x, area.y, area.w, area.h, rgba(theme.surface));
    frame.stroke_rect(area.x, area.y, area.w, area.h, rgba(theme.border));
    let pad = scale * 6;
    let line_height = 7 * scale + scale * 3;
    let max_chars = ((area.w - pad * 2) / (6 * scale)).max(4) as usize;
    let mut lines: Vec<(String, Color)> = Vec::new();

    match app.session.mode {
        Mode::Buttons => {
            let (done, total) = app.session.buttons_challenge.progress();
            lines.push((
                format!("{} {done}/{total}", tr(app.lang, Key::Buttons)),
                theme.accent,
            ));
            if let Some(code) = app.session.buttons_challenge.current() {
                lines.push((
                    tr(app.lang, Key::PressButton)
                        .replace("{button}", &crate::core::codes::label(code)),
                    theme.text,
                ));
            } else {
                lines.push((tr(app.lang, Key::AllButtonsSeen).to_string(), theme.success));
            }
        }
        Mode::Scroll => {
            let (done, total) = app.session.scroll_challenge.progress();
            lines.push((
                format!("{} {done}/{total}", tr(app.lang, Key::Scroll)),
                theme.accent,
            ));
            if let Some(step) = app.session.scroll_challenge.current() {
                let direction = match step {
                    ScrollStep::Up => tr(app.lang, Key::ScrollStepUp),
                    ScrollStep::Down => tr(app.lang, Key::ScrollStepDown),
                    ScrollStep::Left => tr(app.lang, Key::ScrollStepLeft),
                    ScrollStep::Right => tr(app.lang, Key::ScrollStepRight),
                };
                lines.push((
                    tr(app.lang, Key::ScrollPrompt).replace("{direction}", direction),
                    theme.text,
                ));
            } else {
                lines.push((tr(app.lang, Key::ScrollDone).to_string(), theme.success));
            }
        }
        Mode::Targets => {
            let challenge = &app.session.targets;
            lines.push((
                format!(
                    "{} {}/{}",
                    tr(app.lang, Key::Targets),
                    challenge.round(),
                    challenge.total()
                ),
                theme.accent,
            ));
            if challenge.is_done() {
                lines.push((tr(app.lang, Key::TargetsDone).to_string(), theme.success));
            } else {
                let prompt = match challenge.kind() {
                    TargetKind::Click => tr(app.lang, Key::TargetsClick),
                    TargetKind::Drag => tr(app.lang, Key::TargetsDrag),
                    TargetKind::DoubleClick => tr(app.lang, Key::TargetsDouble),
                };
                lines.push((prompt.to_string(), theme.text));
            }
            if challenge.result_visible(app.now_us) {
                let feedback = match challenge.last_result() {
                    Some(TargetResult::Hit) => tr(app.lang, Key::TargetsHit),
                    Some(TargetResult::Miss) | None => tr(app.lang, Key::TargetsMiss),
                };
                lines.push((feedback.to_string(), theme.text));
            }
            let error = challenge
                .last_error()
                .map(|error| format!("{error:.1}"))
                .unwrap_or_else(|| tr(app.lang, Key::NotAvailable).to_string());
            let time = challenge
                .last_duration_us()
                .map(|duration| format!("{:.0} MS", duration as f64 / 1000.0))
                .unwrap_or_else(|| tr(app.lang, Key::NotAvailable).to_string());
            lines.push((
                format!(
                    "{} {:.0}%  {} {}",
                    tr(app.lang, Key::TargetsAccuracy),
                    challenge.accuracy() * 100.0,
                    tr(app.lang, Key::TargetsError),
                    challenge.misses()
                ),
                theme.text,
            ));
            lines.push((
                format!(
                    "{} {error}  {} {time}",
                    tr(app.lang, Key::TargetsError),
                    tr(app.lang, Key::TargetsTime)
                ),
                theme.muted,
            ));
        }
        Mode::Dashboard | Mode::Polling => {}
    }

    lines.push((String::new(), theme.muted));
    lines.push((tr(app.lang, Key::Buttons).to_string(), theme.accent));
    for code in BUTTONS {
        let runtime = app.session.buttons.runtime(code);
        let count = runtime.map(|runtime| runtime.press_count).unwrap_or(0);
        let pressed = app.session.buttons.is_pressed(code);
        lines.push((
            format!(
                "{} {} {:>4}",
                short_label(code),
                if pressed { "ON" } else { "--" },
                count
            ),
            if pressed {
                theme.button(code)
            } else {
                theme.muted
            },
        ));
    }

    lines.push((String::new(), theme.muted));
    lines.push((tr(app.lang, Key::Motion).to_string(), theme.accent));
    let pointer = &app.session.pointer;
    let (raw_x, raw_y) = pointer.raw_total();
    lines.push((
        format!(
            "{} {:.0},{:.0}",
            tr(app.lang, Key::Position),
            pointer.pos().x,
            pointer.pos().y
        ),
        theme.text,
    ));
    lines.push((format!("RAW {raw_x:+} {raw_y:+}"), theme.text));
    lines.push((
        format!(
            "{} {:.0} C/S",
            tr(app.lang, Key::Speed),
            pointer.speed_counts_s()
        ),
        theme.text,
    ));
    lines.push((
        format!(
            "{} V{:+} H{:+}",
            tr(app.lang, Key::Scroll),
            app.session.scroll.vertical_ticks(),
            app.session.scroll.horizontal_ticks()
        ),
        theme.text,
    ));

    lines.push((String::new(), theme.muted));
    lines.push((tr(app.lang, Key::Polling).to_string(), theme.accent));
    let metrics = &app.session.metrics;
    let rate = metrics
        .hz_estimate()
        .map(|hz| format!("{hz:.0} HZ"))
        .unwrap_or_else(|| tr(app.lang, Key::NotAvailable).to_string());
    let jitter = metrics
        .jitter_us()
        .map(|jitter| format!("{jitter} US"))
        .unwrap_or_else(|| tr(app.lang, Key::NotAvailable).to_string());
    let confidence = match metrics.confidence() {
        Confidence::Low => tr(app.lang, Key::ConfidenceLow),
        Confidence::Medium => tr(app.lang, Key::ConfidenceMedium),
        Confidence::High => tr(app.lang, Key::ConfidenceHigh),
    };
    lines.push((format!("{} {rate}", tr(app.lang, Key::Rate)), theme.text));
    lines.push((
        format!("{} {jitter}", tr(app.lang, Key::Jitter)),
        theme.text,
    ));
    lines.push((
        format!("{} {}", tr(app.lang, Key::Confidence), confidence),
        theme.text,
    ));
    lines.push((
        format!("{} {}", tr(app.lang, Key::Reports), metrics.reports()),
        theme.text,
    ));

    if app.flags.raw_log {
        lines.push((String::new(), theme.muted));
        lines.push((tr(app.lang, Key::Log).to_string(), theme.accent));
        for entry in app.session.log.iter().rev().take(6).rev() {
            lines.push((format!("{:?}", entry.kind), theme.muted));
        }
    }

    let mut y = area.y + pad;
    for (text, color) in lines {
        if y + 7 * scale > area.y + area.h - pad {
            break;
        }
        frame.text(
            area.x + pad,
            y,
            &truncate(&sanitize(&text), max_chars),
            rgba(color),
            scale,
        );
        y += if text.is_empty() {
            scale * 2
        } else {
            line_height
        };
    }
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect, scale: i32) {
    let theme = app.theme;
    frame.fill_rect(area.x, area.y, area.w, area.h, rgba(theme.surface));
    frame.text(
        area.x + scale * 6,
        area.y + (area.h - 7 * scale) / 2,
        &truncate(
            &sanitize(tr(app.lang, Key::Footer)),
            ((area.w - scale * 12) / (6 * scale)).max(8) as usize,
        ),
        rgba(theme.muted),
        scale,
    );
}

fn phase_label(_app: &App) -> &'static str {
    "4"
}

fn mode_label(app: &App) -> &'static str {
    match app.session.mode {
        Mode::Dashboard => tr(app.lang, Key::Dashboard),
        Mode::Buttons => tr(app.lang, Key::Buttons),
        Mode::Scroll => tr(app.lang, Key::Scroll),
        Mode::Polling => tr(app.lang, Key::Polling),
        Mode::Targets => tr(app.lang, Key::Targets),
    }
}

fn sanitize(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for ch in value.chars() {
        let mapped = match ch {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'Á' | 'À' | 'Â' | 'Ã' | 'Ä' => 'A',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'É' | 'È' | 'Ê' | 'Ë' => 'E',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'Í' | 'Ì' | 'Î' | 'Ï' => 'I',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => 'O',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'Ú' | 'Ù' | 'Û' | 'Ü' => 'U',
            'ç' => 'c',
            'Ç' => 'C',
            'ñ' => 'n',
            'Ñ' => 'N',
            'µ' => 'u',
            '—' | '–' => '-',
            '…' => '.',
            _ => ch,
        };
        output.push(mapped);
    }
    output
}

fn truncate(value: &str, limit: usize) -> String {
    let mut result: String = value.chars().take(limit).collect();
    if value.chars().count() > limit {
        if limit > 0 {
            result.pop();
        }
        result.push('.');
    }
    result
}

pub fn rgba(color: Color) -> [u8; 4] {
    match color {
        Color::Rgb(r, g, b) => [r, g, b, 0xff],
        Color::Black => [0, 0, 0, 0xff],
        Color::Red => [205, 49, 49, 0xff],
        Color::Green => [13, 188, 121, 0xff],
        Color::Yellow => [229, 229, 16, 0xff],
        Color::Blue => [36, 114, 200, 0xff],
        Color::Magenta => [188, 63, 188, 0xff],
        Color::Cyan => [17, 168, 205, 0xff],
        Color::Gray => [118, 118, 118, 0xff],
        Color::DarkGray => [102, 102, 102, 0xff],
        Color::LightRed => [241, 76, 76, 0xff],
        Color::LightGreen => [35, 209, 139, 0xff],
        Color::LightYellow => [245, 245, 67, 0xff],
        Color::LightBlue => [59, 142, 234, 0xff],
        Color::LightMagenta => [214, 112, 214, 0xff],
        Color::LightCyan => [41, 184, 219, 0xff],
        Color::White => [229, 229, 229, 0xff],
        Color::Indexed(index) => indexed(index),
        Color::Reset => [0x0b, 0x0f, 0x14, 0xff],
    }
}

fn indexed(index: u8) -> [u8; 4] {
    if index < 16 {
        let basic = [
            [0, 0, 0],
            [205, 49, 49],
            [13, 188, 121],
            [229, 229, 16],
            [36, 114, 200],
            [188, 63, 188],
            [17, 168, 205],
            [229, 229, 229],
            [102, 102, 102],
            [241, 76, 76],
            [35, 209, 139],
            [245, 245, 67],
            [59, 142, 234],
            [214, 112, 214],
            [41, 184, 219],
            [255, 255, 255],
        ];
        let [r, g, b] = basic[index as usize];
        return [r, g, b, 0xff];
    }
    if index >= 232 {
        let value = 8 + (index - 232) * 10;
        return [value, value, value, 0xff];
    }
    let value = index - 16;
    let r = value / 36;
    let g = (value % 36) / 6;
    let b = value % 6;
    let channel = |value: u8| if value == 0 { 0 } else { 55 + value * 40 };
    [channel(r), channel(g), channel(b), 0xff]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{App, AppConfig, UiFlags};
    use crate::core::geometry::Bounds;
    use crate::i18n::Lang;
    use crate::theme::{ColorTier, Theme};

    fn app(mode: Mode, lang: Lang, tier: ColorTier) -> App {
        App::new(AppConfig {
            bounds: Bounds::new(1344, 756),
            lang,
            theme: Theme::new(tier),
            flags: UiFlags::default(),
            initial_mode: mode,
            seed: 7,
            demo_seed: Some(7),
            controller: None,
            grab: false,
            grab_force: false,
        })
    }

    #[test]
    fn renders_full_hd_frame() {
        let app = app(Mode::Dashboard, Lang::PtBr, ColorTier::TrueColor);
        let frame = render(&app, 1920, 1080);
        assert_eq!(frame.width(), 1920);
        assert_eq!(frame.height(), 1080);
        assert!(frame
            .pixels()
            .chunks(4)
            .any(|pixel| pixel != [0x0b, 0x0f, 0x14, 0xff]));
    }

    #[test]
    fn renders_every_mode_and_language() {
        for mode in Mode::ALL {
            for lang in [Lang::PtBr, Lang::En] {
                let mut app = app(mode, lang, ColorTier::TrueColor);
                app.update(1_000_000);
                let frame = render(&app, 1366, 768);
                assert_eq!(frame.width(), 1366);
                assert_eq!(frame.height(), 768);
            }
        }
    }

    #[test]
    fn renders_mono_without_rgb_tokens() {
        let app = app(Mode::Targets, Lang::PtBr, ColorTier::Mono);
        let frame = render(&app, 1024, 768);
        assert!(frame.pixels().chunks(4).any(|pixel| pixel[3] == 0xff));
    }

    #[test]
    fn sanitizes_ui_text() {
        assert_eq!(sanitize("Posição — µs…"), "Posicao - us.");
    }

    #[test]
    fn truncates_with_marker() {
        assert_eq!(truncate("MOUSE TEST", 8), "MOUSE T.");
        assert_eq!(truncate("OK", 8), "OK");
    }

    #[test]
    fn indexed_palette_covers_cube_and_gray() {
        assert_eq!(indexed(196), [255, 0, 0, 0xff]);
        assert_eq!(indexed(232), [8, 8, 8, 0xff]);
        assert_eq!(indexed(255), [238, 238, 238, 0xff]);
    }

    #[test]
    fn canvas_y_follows_pointer_downward() {
        let inner = Rect {
            x: 10,
            y: 20,
            w: 1344,
            h: 756,
        };
        let bounds = Bounds::new(1344, 756).with_margin(0);
        let (_, top) = canvas_point(inner, bounds, Point::new(0.0, 0.0));
        let (_, bottom) = canvas_point(inner, bounds, Point::new(0.0, 756.0));
        assert_eq!(top, 20);
        assert_eq!(bottom, 776);
    }

    #[test]
    fn layout_switches_between_wide_and_narrow() {
        let wide = areas(1920, 1080);
        assert!(wide.canvas.x + wide.canvas.w < wide.side.x);
        let narrow = areas(1024, 768);
        assert!(narrow.side.y > narrow.canvas.y + narrow.canvas.h);
    }
}
