use ratatui::style::Color;

use crate::core::codes::{
    BTN_BACK, BTN_EXTRA, BTN_FORWARD, BTN_LEFT, BTN_MIDDLE, BTN_RIGHT, BTN_SIDE, BTN_TASK,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorTier {
    TrueColor,
    Ansi256,
    Ansi16,
    Mono,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Theme {
    pub bg: Color,
    pub surface: Color,
    pub surface_elevated: Color,
    pub border: Color,
    pub grid: Color,
    pub text: Color,
    pub muted: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub violet: Color,
    pub pink: Color,
    pub drag: Color,
}

impl Theme {
    pub fn new(tier: ColorTier) -> Self {
        match tier {
            ColorTier::TrueColor => Self {
                bg: Color::Rgb(0x0b, 0x0f, 0x14),
                surface: Color::Rgb(0x12, 0x18, 0x21),
                surface_elevated: Color::Rgb(0x1a, 0x23, 0x30),
                border: Color::Rgb(0x2a, 0x36, 0x46),
                grid: Color::Rgb(0x1e, 0x28, 0x36),
                text: Color::Rgb(0xe6, 0xed, 0xf3),
                muted: Color::Rgb(0x9a, 0xa7, 0xb6),
                accent: Color::Rgb(0x38, 0xbd, 0xf8),
                success: Color::Rgb(0x34, 0xd3, 0x99),
                warning: Color::Rgb(0xfb, 0xbf, 0x24),
                danger: Color::Rgb(0xf8, 0x71, 0x71),
                violet: Color::Rgb(0xa7, 0x8b, 0xfa),
                pink: Color::Rgb(0xf4, 0x72, 0xb6),
                drag: Color::Rgb(0xfd, 0xe6, 0x8a),
            },
            ColorTier::Ansi256 => Self {
                bg: Color::Indexed(233),
                surface: Color::Indexed(234),
                surface_elevated: Color::Indexed(235),
                border: Color::Indexed(237),
                grid: Color::Indexed(236),
                text: Color::Indexed(255),
                muted: Color::Indexed(247),
                accent: Color::Indexed(45),
                success: Color::Indexed(48),
                warning: Color::Indexed(214),
                danger: Color::Indexed(203),
                violet: Color::Indexed(141),
                pink: Color::Indexed(212),
                drag: Color::Indexed(222),
            },
            ColorTier::Ansi16 => Self {
                bg: Color::Black,
                surface: Color::Black,
                surface_elevated: Color::DarkGray,
                border: Color::Gray,
                grid: Color::DarkGray,
                text: Color::White,
                muted: Color::Gray,
                accent: Color::Cyan,
                success: Color::Green,
                warning: Color::Yellow,
                danger: Color::Red,
                violet: Color::Magenta,
                pink: Color::Magenta,
                drag: Color::Yellow,
            },
            ColorTier::Mono => Self {
                bg: Color::Reset,
                surface: Color::Reset,
                surface_elevated: Color::Reset,
                border: Color::Gray,
                grid: Color::DarkGray,
                text: Color::White,
                muted: Color::Gray,
                accent: Color::White,
                success: Color::White,
                warning: Color::White,
                danger: Color::White,
                violet: Color::White,
                pink: Color::White,
                drag: Color::White,
            },
        }
    }

    pub fn button(self, code: u16) -> Color {
        match code {
            BTN_LEFT => self.success,
            BTN_RIGHT => self.danger,
            BTN_MIDDLE => self.warning,
            BTN_SIDE | BTN_BACK => self.violet,
            BTN_EXTRA | BTN_TASK => self.pink,
            BTN_FORWARD => self.accent,
            _ => self.muted,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::new(ColorTier::TrueColor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buttons_have_distinct_tokens() {
        let theme = Theme::default();
        assert_eq!(theme.button(BTN_LEFT), theme.success);
        assert_eq!(theme.button(BTN_RIGHT), theme.danger);
        assert_eq!(theme.button(BTN_MIDDLE), theme.warning);
    }

    #[test]
    fn mono_theme_has_no_rgb_colors() {
        let theme = Theme::new(ColorTier::Mono);
        assert_eq!(theme.accent, Color::White);
        assert_eq!(theme.bg, Color::Reset);
    }

    fn rgb(color: Color) -> (f32, f32, f32) {
        match color {
            Color::Rgb(r, g, b) => (
                f32::from(r) / 255.0,
                f32::from(g) / 255.0,
                f32::from(b) / 255.0,
            ),
            _ => (1.0, 1.0, 1.0),
        }
    }

    fn luminance(color: Color) -> f32 {
        let (r, g, b) = rgb(color);
        let channel = |value: f32| {
            if value <= 0.03928 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
    }

    fn contrast(a: Color, b: Color) -> f32 {
        let (high, low) = if luminance(a) > luminance(b) {
            (luminance(a), luminance(b))
        } else {
            (luminance(b), luminance(a))
        };
        (high + 0.05) / (low + 0.05)
    }

    #[test]
    fn truecolor_palette_meets_contrast_targets() {
        let theme = Theme::default();
        for color in [
            theme.text,
            theme.muted,
            theme.accent,
            theme.success,
            theme.warning,
            theme.danger,
            theme.violet,
            theme.pink,
            theme.drag,
        ] {
            assert!(contrast(color, theme.surface) >= 4.5);
            assert!(contrast(color, theme.bg) >= 4.5);
        }
    }
}
