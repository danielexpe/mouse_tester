use std::path::PathBuf;

use clap::{Parser, ValueEnum};

use crate::i18n::Lang;
use crate::state::Mode;
use crate::theme::ColorTier;

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum BackendArg {
    Auto,
    Tui,
    Fb,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ColorArg {
    Auto,
    Truecolor,
    #[value(name = "256")]
    Ansi256,
    #[value(name = "16")]
    Ansi16,
    Never,
}

impl ColorArg {
    pub fn tier(self) -> ColorTier {
        match self {
            Self::Auto | Self::Truecolor => ColorTier::TrueColor,
            Self::Ansi256 => ColorTier::Ansi256,
            Self::Ansi16 => ColorTier::Ansi16,
            Self::Never => ColorTier::Mono,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum LangArg {
    #[value(name = "pt-br", alias = "pt")]
    PtBr,
    En,
}

impl LangArg {
    pub fn lang(self) -> Lang {
        match self {
            Self::PtBr => Lang::PtBr,
            Self::En => Lang::En,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum TuiMouseArg {
    Auto,
    On,
    Off,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ModeArg {
    Dashboard,
    Buttons,
    Scroll,
    Polling,
    Targets,
}

impl ModeArg {
    pub fn mode(self) -> Mode {
        match self {
            Self::Dashboard => Mode::Dashboard,
            Self::Buttons => Mode::Buttons,
            Self::Scroll => Mode::Scroll,
            Self::Polling => Mode::Polling,
            Self::Targets => Mode::Targets,
        }
    }
}

#[derive(Clone, Debug, Parser)]
#[command(
    name = "mousescope",
    version,
    about = "Visual mouse tester for the Linux console"
)]
pub struct Cli {
    #[arg(long, value_enum, default_value_t = BackendArg::Auto)]
    pub backend: BackendArg,

    #[arg(long, value_enum, default_value_t = ModeArg::Dashboard)]
    pub mode: ModeArg,

    #[arg(long)]
    pub list_devices: bool,

    #[arg(long)]
    pub all_devices: bool,

    #[arg(long)]
    pub diagnostics: Option<PathBuf>,

    #[arg(
        long,
        value_name = "PATH",
        help = "Write a deterministic PNG frame and exit"
    )]
    pub capture_frame: Option<PathBuf>,

    #[arg(long)]
    pub device: Option<String>,

    #[arg(long)]
    pub grab: bool,

    #[arg(long)]
    pub grab_force: bool,

    #[arg(long)]
    pub require_evdev: bool,

    #[arg(long, value_enum, default_value_t = TuiMouseArg::Auto)]
    pub tui_mouse: TuiMouseArg,

    #[arg(long, value_enum)]
    pub lang: Option<LangArg>,

    #[arg(long, default_value_t = 0)]
    pub seed: u64,

    #[arg(long, value_enum, default_value_t = ColorArg::Auto)]
    pub color: ColorArg,

    #[arg(long)]
    pub no_animations: bool,

    #[arg(long)]
    pub no_trail: bool,

    #[arg(long)]
    pub raw_log: bool,

    #[arg(long, help = "Run a synthetic input demo (Phase 1)")]
    pub demo: bool,
}

impl Cli {
    pub fn language(&self) -> Lang {
        self.lang.map(LangArg::lang).unwrap_or_else(Lang::from_env)
    }

    pub fn color_tier(&self) -> ColorTier {
        if std::env::var_os("NO_COLOR").is_some() {
            ColorTier::Mono
        } else {
            self.color.tier()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_tui_dashboard() {
        let cli = Cli::try_parse_from(["mousescope"]).unwrap();
        assert_eq!(cli.backend, BackendArg::Auto);
        assert_eq!(cli.mode, ModeArg::Dashboard);
        assert!(!cli.demo);
        assert_eq!(cli.color, ColorArg::Auto);
    }

    #[test]
    fn parses_explicit_values() {
        let cli = Cli::try_parse_from([
            "mousescope",
            "--backend",
            "fb",
            "--mode",
            "scroll",
            "--lang",
            "en",
            "--color",
            "256",
            "--no-animations",
            "--no-trail",
            "--raw-log",
            "--demo",
            "--seed",
            "42",
        ])
        .unwrap();
        assert_eq!(cli.backend, BackendArg::Fb);
        assert_eq!(cli.mode, ModeArg::Scroll);
        assert_eq!(cli.language(), Lang::En);
        assert_eq!(cli.color_tier(), ColorTier::Ansi256);
        assert!(cli.no_animations);
        assert!(cli.no_trail);
        assert!(cli.raw_log);
        assert!(cli.demo);
        assert_eq!(cli.seed, 42);
    }

    #[test]
    fn accepts_pt_alias() {
        let cli = Cli::try_parse_from(["mousescope", "--lang", "pt"]).unwrap();
        assert_eq!(cli.language(), Lang::PtBr);
    }

    #[test]
    fn never_forces_mono() {
        let cli = Cli::try_parse_from(["mousescope", "--color", "never"]).unwrap();
        assert_eq!(cli.color_tier(), ColorTier::Mono);
    }
}
