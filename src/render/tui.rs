use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line as TextLine, Span};
use ratatui::widgets::canvas::{Canvas, Circle, Line as CanvasLine, Points};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;
use crate::core::codes::{
    label, short_label, BTN_BACK, BTN_EXTRA, BTN_FORWARD, BTN_LEFT, BTN_MIDDLE, BTN_RIGHT,
    BTN_SIDE, BTN_TASK,
};
use crate::core::effects::EffectKind;
use crate::core::metrics::Confidence;
use crate::core::scroll::ScrollAxis;
use crate::i18n::{tr, Key};
use crate::render::layout::{compute, LayoutKind};
use crate::state::{LogKind, Mode};
use crate::theme::Theme;

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

pub fn render(frame: &mut Frame, app: &App) {
    let (kind, areas) = compute(frame.area());

    if kind == LayoutKind::TooSmall {
        render_too_small(frame, app, areas.header);
        return;
    }

    render_header(frame, app, areas.header, kind);

    match kind {
        LayoutKind::Wide | LayoutKind::Normal => {
            render_canvas(frame, app, areas.canvas);
            if let Some(side) = areas.side {
                render_side(frame, app, side);
            }
        }
        LayoutKind::Compact => {
            render_canvas(frame, app, areas.canvas);
            if let Some(lower) = areas.lower {
                render_compact_panels(frame, app, lower);
            }
        }
        LayoutKind::Minimal => render_minimal(frame, app, areas.canvas),
        LayoutKind::TooSmall => {}
    }

    render_footer(frame, app, areas.footer);

    if app.show_help {
        render_help(frame, app);
    } else if app.flags.raw_log {
        render_log(frame, app);
    }
}

fn render_header(frame: &mut Frame, app: &App, area: Rect, kind: LayoutKind) {
    let theme = app.theme;
    let device = app
        .session
        .primary_device()
        .map(|device| device.name.as_str())
        .unwrap_or_else(|| tr(app.lang, Key::NoDevice));
    let source = app
        .session
        .last_source
        .map(|source| source.label())
        .unwrap_or("mock");
    let mode = mode_label(app.lang, app.session.mode);

    let title = TextLine::from(vec![
        Span::styled(
            tr(app.lang, Key::AppTitle),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("  v{}  ", env!("CARGO_PKG_VERSION"))),
        Span::styled(tr(app.lang, Key::Phase), Style::default().fg(theme.warning)),
        Span::raw("  "),
        Span::styled(mode, Style::default().fg(theme.text)),
    ]);

    let details = TextLine::from(vec![
        Span::styled(
            format!("{}: ", tr(app.lang, Key::Device)),
            Style::default().fg(theme.muted),
        ),
        Span::raw(device),
        Span::raw("  |  "),
        Span::styled(
            format!("{}: ", tr(app.lang, Key::Source)),
            Style::default().fg(theme.muted),
        ),
        Span::raw(source),
        Span::raw("  |  "),
        Span::styled(
            format!("{}: ", tr(app.lang, Key::Backend)),
            Style::default().fg(theme.muted),
        ),
        Span::raw("TUI"),
    ]);

    let mut lines = vec![title, details];
    if matches!(kind, LayoutKind::Wide | LayoutKind::Normal) {
        let hint = if app.demo_active {
            tr(app.lang, Key::DemoHint)
        } else {
            tr(app.lang, Key::PhaseOneHint)
        };
        lines.push(TextLine::from(Span::styled(
            hint,
            Style::default().fg(theme.muted),
        )));
    }

    let paragraph = Paragraph::new(lines)
        .style(Style::default().bg(theme.surface).fg(theme.text))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(theme.border)),
        );
    frame.render_widget(paragraph, area);
}

fn render_canvas(frame: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme;
    let bounds = app.session.pointer.bounds();
    let width = bounds.width as f64;
    let height = bounds.height as f64;
    let now = app.now_us;
    let no_trail = app.flags.no_trail;
    let no_animations = app.flags.no_animations;

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", tr(app.lang, Key::Motion)))
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.surface));

    let canvas = Canvas::default()
        .block(block)
        .marker(ratatui::symbols::Marker::Braille)
        .x_bounds([0.0, width])
        .y_bounds([0.0, height])
        .paint(|ctx| {
            for index in 1..4 {
                let fraction = index as f64 / 4.0;
                ctx.draw(&CanvasLine {
                    x1: width * fraction,
                    y1: 0.0,
                    x2: width * fraction,
                    y2: height,
                    color: theme.grid,
                });
                ctx.draw(&CanvasLine {
                    x1: 0.0,
                    y1: height * fraction,
                    x2: width,
                    y2: height * fraction,
                    color: theme.grid,
                });
            }

            if !no_trail {
                let trail: Vec<(f64, f64)> = app
                    .session
                    .pointer
                    .trail()
                    .iter()
                    .map(|point| (point.pos.x as f64, height - point.pos.y as f64))
                    .collect();
                if trail.len() >= 2 {
                    for pair in trail.windows(2) {
                        ctx.draw(&CanvasLine {
                            x1: pair[0].0,
                            y1: pair[0].1,
                            x2: pair[1].0,
                            y2: pair[1].1,
                            color: theme.accent,
                        });
                    }
                }
                if !trail.is_empty() {
                    ctx.draw(&Points {
                        coords: &trail,
                        color: theme.accent,
                    });
                }
            }

            let cursor = app.session.pointer.pos();
            let cursor_x = cursor.x as f64;
            let cursor_y = height - cursor.y as f64;
            ctx.draw(&Circle {
                x: cursor_x,
                y: cursor_y,
                radius: 2.6,
                color: theme.accent,
            });

            if no_animations {
                return;
            }

            for effect in app.session.effects.iter() {
                let progress = effect.progress(now) as f64;
                let x = effect.pos.x as f64;
                let y = height - effect.pos.y as f64;
                match effect.kind {
                    EffectKind::Ripple { code, radius_scale } => {
                        ctx.draw(&Circle {
                            x,
                            y,
                            radius: 1.5 + progress * 10.0 * radius_scale as f64,
                            color: theme.button(code),
                        });
                    }
                    EffectKind::DoubleRipple { code } => {
                        ctx.draw(&Circle {
                            x,
                            y,
                            radius: 1.5 + progress * 10.0,
                            color: theme.button(code),
                        });
                        ctx.draw(&Circle {
                            x,
                            y,
                            radius: 1.0 + (progress * 0.7) * 8.0,
                            color: theme.button(code),
                        });
                    }
                    EffectKind::WheelPulse {
                        axis,
                        direction,
                        pulses,
                        hires_scale,
                    } => {
                        let scale = f64::from(hires_scale);
                        for index in 0..pulses {
                            let offset = (f64::from(index) + 1.0) * 3.0 * scale;
                            let signed = offset * f64::from(direction);
                            let (x1, y1, x2, y2) = match axis {
                                ScrollAxis::Vertical => (x, y, x, y - signed),
                                ScrollAxis::Horizontal => (x, y, x + signed, y),
                            };
                            ctx.draw(&CanvasLine {
                                x1,
                                y1,
                                x2,
                                y2,
                                color: theme.warning,
                            });
                        }
                    }
                    EffectKind::EdgeFlash { edge } => {
                        let (x1, y1, x2, y2) = match edge {
                            crate::core::effects::Edge::Left => (1.0, 0.0, 1.0, height),
                            crate::core::effects::Edge::Right => {
                                (width - 1.0, 0.0, width - 1.0, height)
                            }
                            crate::core::effects::Edge::Top => {
                                (0.0, height - 1.0, width, height - 1.0)
                            }
                            crate::core::effects::Edge::Bottom => (0.0, 1.0, width, 1.0),
                        };
                        ctx.draw(&CanvasLine {
                            x1,
                            y1,
                            x2,
                            y2,
                            color: theme.danger,
                        });
                    }
                    EffectKind::DragRelease { start, end } => {
                        ctx.draw(&CanvasLine {
                            x1: start.x as f64,
                            y1: height - start.y as f64,
                            x2: end.x as f64,
                            y2: height - end.y as f64,
                            color: theme.drag,
                        });
                    }
                }
            }
        });

    frame.render_widget(canvas, area);
}

fn render_side(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(45),
            Constraint::Percentage(30),
            Constraint::Percentage(25),
        ])
        .split(area);
    render_buttons(frame, app, chunks[0]);
    render_motion(frame, app, chunks[1]);
    render_metrics(frame, app, chunks[2]);
}

fn render_compact_panels(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(52), Constraint::Percentage(48)])
        .split(area);
    render_buttons(frame, app, chunks[0]);
    render_motion(frame, app, chunks[1]);
}

fn render_buttons(frame: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme;
    let mut lines = Vec::new();
    for code in BUTTONS {
        let runtime = app.session.buttons.runtime(code);
        let count = runtime.map(|runtime| runtime.press_count).unwrap_or(0);
        let pressed = app.session.buttons.is_pressed(code);
        let style = if pressed {
            Style::default()
                .fg(theme.button(code))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.muted)
        };
        lines.push(TextLine::from(vec![
            Span::styled(format!("{} ", short_label(code)), style),
            Span::raw(if pressed { "ON " } else { "-- " }),
            Span::styled(format!("{count:>4}"), Style::default().fg(theme.text)),
        ]));
    }

    let paragraph = Paragraph::new(lines).block(panel_block(theme, tr(app.lang, Key::Buttons)));
    frame.render_widget(paragraph, area);
}

fn render_motion(frame: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme;
    let pointer = &app.session.pointer;
    let (raw_x, raw_y) = pointer.raw_total();
    let last_axis = app
        .session
        .scroll
        .last_axis()
        .map(|axis| match axis {
            ScrollAxis::Vertical => "V",
            ScrollAxis::Horizontal => "H",
        })
        .unwrap_or("-");

    let lines = vec![
        TextLine::from(format!(
            "{} {:.0},{:.0}",
            tr(app.lang, Key::Position),
            pointer.pos().x,
            pointer.pos().y
        )),
        TextLine::from(format!("raw {raw_x:+} {raw_y:+}")),
        TextLine::from(format!(
            "{} {:.0} c/s",
            tr(app.lang, Key::Speed),
            pointer.speed_counts_s()
        )),
        TextLine::from(format!(
            "{} {:.0}",
            tr(app.lang, Key::Distance),
            pointer.distance_counts()
        )),
        TextLine::from(format!(
            "{} V{:+} H{:+} last {last_axis}{:+}",
            tr(app.lang, Key::Scroll),
            app.session.scroll.vertical_ticks(),
            app.session.scroll.horizontal_ticks(),
            app.session.scroll.last_delta()
        )),
        TextLine::from(format!(
            "{} {}",
            tr(app.lang, Key::Events),
            app.session.metrics.events()
        )),
    ];

    let paragraph = Paragraph::new(lines)
        .block(panel_block(theme, tr(app.lang, Key::Motion)))
        .style(Style::default().fg(theme.text));
    frame.render_widget(paragraph, area);
}

fn render_metrics(frame: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme;
    let metrics = &app.session.metrics;
    let rate = metrics
        .hz_estimate()
        .map(|hz| format!("{hz:.0} Hz"))
        .unwrap_or_else(|| tr(app.lang, Key::NotAvailable).to_string());
    let jitter = metrics
        .jitter_us()
        .map(|jitter| format!("{jitter} µs"))
        .unwrap_or_else(|| tr(app.lang, Key::NotAvailable).to_string());
    let confidence = match metrics.confidence() {
        Confidence::Low => tr(app.lang, Key::ConfidenceLow),
        Confidence::Medium => tr(app.lang, Key::ConfidenceMedium),
        Confidence::High => tr(app.lang, Key::ConfidenceHigh),
    };

    let lines = vec![
        TextLine::from(format!("{} {rate}", tr(app.lang, Key::Rate))),
        TextLine::from(format!("{} {jitter}", tr(app.lang, Key::Jitter))),
        TextLine::from(format!(
            "{} {}",
            tr(app.lang, Key::Samples),
            metrics.sample_count()
        )),
        TextLine::from(format!("{} {}", tr(app.lang, Key::Confidence), confidence)),
        TextLine::from(format!(
            "{} {}",
            tr(app.lang, Key::Reports),
            metrics.reports()
        )),
    ];

    let paragraph = Paragraph::new(lines)
        .block(panel_block(theme, tr(app.lang, Key::Polling)))
        .style(Style::default().fg(theme.text));
    frame.render_widget(paragraph, area);
}

fn render_minimal(frame: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme;
    let pointer = &app.session.pointer;
    let (raw_x, raw_y) = pointer.raw_total();
    let pressed = app
        .session
        .buttons
        .pressed()
        .iter()
        .map(|code| short_label(*code))
        .collect::<Vec<_>>()
        .join(" ");
    let rate = app
        .session
        .metrics
        .hz_estimate()
        .map(|hz| format!("{hz:.0} Hz"))
        .unwrap_or_else(|| "—".to_string());

    let lines = vec![
        TextLine::from(format!(
            "{} x{:.0} y{:.0}  raw {raw_x:+} {raw_y:+}",
            tr(app.lang, Key::Position),
            pointer.pos().x,
            pointer.pos().y
        )),
        TextLine::from(format!(
            "{} {:.0} c/s  {} {rate}",
            tr(app.lang, Key::Speed),
            pointer.speed_counts_s(),
            tr(app.lang, Key::Rate)
        )),
        TextLine::from(format!(
            "{} V{:+} H{:+}",
            tr(app.lang, Key::Scroll),
            app.session.scroll.vertical_ticks(),
            app.session.scroll.horizontal_ticks()
        )),
        TextLine::from(if pressed.is_empty() {
            format!("{} --", tr(app.lang, Key::Buttons))
        } else {
            format!("{} {pressed}", tr(app.lang, Key::Buttons))
        }),
    ];

    let paragraph = Paragraph::new(lines)
        .block(panel_block(theme, tr(app.lang, Key::Motion)))
        .style(Style::default().fg(theme.text));
    frame.render_widget(paragraph, area);
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let paragraph = Paragraph::new(tr(app.lang, Key::Footer))
        .style(Style::default().fg(app.theme.muted))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(app.theme.border)),
        );
    frame.render_widget(paragraph, area);
}

fn render_help(frame: &mut Frame, app: &App) {
    let area = centered(frame.area(), 70, 55);
    let paragraph = Paragraph::new(tr(app.lang, Key::HelpBody))
        .block(
            panel_block(app.theme, tr(app.lang, Key::HelpTitle))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.accent)),
        )
        .style(Style::default().fg(app.theme.text))
        .wrap(Wrap { trim: false });
    frame.render_widget(Clear, area);
    frame.render_widget(paragraph, area);
}

fn render_log(frame: &mut Frame, app: &App) {
    let area = centered(frame.area(), 72, 50);
    let mut lines = Vec::new();
    for entry in app.session.log.iter().rev().take(10).rev() {
        lines.push(TextLine::from(format_log(entry)));
    }
    if lines.is_empty() {
        lines.push(TextLine::from(tr(app.lang, Key::LogEmpty)));
    }

    let paragraph = Paragraph::new(lines)
        .block(panel_block(app.theme, tr(app.lang, Key::Log)))
        .style(Style::default().fg(app.theme.text))
        .wrap(Wrap { trim: true });
    frame.render_widget(Clear, area);
    frame.render_widget(paragraph, area);
}

fn render_too_small(frame: &mut Frame, app: &App, area: Rect) {
    let paragraph = Paragraph::new(vec![
        TextLine::from(tr(app.lang, Key::TooSmall)),
        TextLine::from(tr(app.lang, Key::QuitHint)),
    ])
    .alignment(Alignment::Center)
    .style(Style::default().fg(app.theme.text).bg(app.theme.bg))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(paragraph, area);
}

fn panel_block(theme: Theme, title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} "))
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.surface))
}

fn centered(area: Rect, width_percent: u16, height_percent: u16) -> Rect {
    let width = area.width.saturating_mul(width_percent) / 100;
    let height = area.height.saturating_mul(height_percent) / 100;
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    )
}

fn mode_label(lang: crate::i18n::Lang, mode: Mode) -> &'static str {
    match mode {
        Mode::Dashboard => tr(lang, Key::Dashboard),
        Mode::Buttons => tr(lang, Key::Buttons),
        Mode::Scroll => tr(lang, Key::Scroll),
        Mode::Polling => tr(lang, Key::Polling),
        Mode::Targets => tr(lang, Key::Targets),
    }
}

fn format_log(entry: &crate::state::LogEntry) -> String {
    let device = entry.device;
    match &entry.kind {
        LogKind::Move { dx, dy } => format!("d{device} move {dx:+} {dy:+}"),
        LogKind::Button { code, pressed } => {
            format!(
                "d{device} {} {}",
                label(*code),
                if *pressed { "down" } else { "up" }
            )
        }
        LogKind::Wheel { axis, value, hires } => {
            let axis = match axis {
                ScrollAxis::Vertical => "V",
                ScrollAxis::Horizontal => "H",
            };
            let kind = if *hires { "hires" } else { "notch" };
            format!("d{device} wheel {axis} {value:+} {kind}")
        }
        LogKind::DeviceAdded => format!("d{device} added"),
        LogKind::DeviceRemoved => format!("d{device} removed"),
        LogKind::Overflow { dropped } => format!("d{device} overflow {dropped}"),
        LogKind::Error(message) => format!("d{device} error {message}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{App, UiFlags};
    use crate::core::geometry::Bounds;
    use crate::i18n::Lang;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn app() -> App {
        let mut app = App::new(
            Bounds::new(1000, 800),
            Lang::PtBr,
            Theme::default(),
            UiFlags::default(),
            Mode::Dashboard,
            Some(1),
        );
        app.update(1_000_000);
        app
    }

    fn buffer_text(terminal: &Terminal<TestBackend>) -> String {
        terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect()
    }

    #[test]
    fn renders_wide_dashboard() {
        let backend = TestBackend::new(120, 36);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = app();
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let text = buffer_text(&terminal);
        assert!(text.contains("MouseScope"));
        assert!(text.contains("Painel"));
        assert!(text.contains("Botões"));
    }

    #[test]
    fn renders_help_overlay() {
        let backend = TestBackend::new(120, 36);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = app();
        app.show_help = true;
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let text = buffer_text(&terminal);
        assert!(text.contains("Ajuda"));
        assert!(text.contains("teclado"));
    }

    #[test]
    fn renders_too_small_message() {
        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = app();
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let text = buffer_text(&terminal);
        assert!(text.contains("redimensione"));
    }
}
