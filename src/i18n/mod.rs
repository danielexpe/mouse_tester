#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    PtBr,
    En,
}

impl Lang {
    pub fn from_env() -> Self {
        let value = std::env::var("LC_ALL")
            .or_else(|_| std::env::var("LC_MESSAGES"))
            .or_else(|_| std::env::var("LANG"))
            .unwrap_or_default()
            .to_ascii_lowercase();
        if value.starts_with("en") {
            Self::En
        } else {
            Self::PtBr
        }
    }

    pub fn tag(self) -> &'static str {
        match self {
            Self::PtBr => "pt-BR",
            Self::En => "en",
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            Self::PtBr => Self::En,
            Self::En => Self::PtBr,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    AppTitle,
    Subtitle,
    Phase,
    Backend,
    Source,
    Device,
    Mode,
    Motion,
    Position,
    Speed,
    Distance,
    Events,
    Reports,
    Rate,
    Jitter,
    Samples,
    Confidence,
    Buttons,
    Scroll,
    Polling,
    Targets,
    Dashboard,
    Help,
    Footer,
    Log,
    LogEmpty,
    Waiting,
    NoDevice,
    MockSource,
    PhaseOneHint,
    DemoHint,
    ResetHint,
    ToggleLogHint,
    ToggleLangHint,
    HelpHint,
    QuitHint,
    ConfidenceLow,
    ConfidenceMedium,
    ConfidenceHigh,
    NotAvailable,
    HelpTitle,
    HelpBody,
    TooSmall,
    Devices,
    DevicePickerHint,
    DeviceOk,
    DeviceDenied,
    DevicePrimary,
    PressButton,
    AllButtonsSeen,
    ScrollStepUp,
    ScrollStepDown,
    ScrollStepLeft,
    ScrollStepRight,
    ScrollPrompt,
    ScrollDone,
    BannerDegraded,
    BannerPermission,
    BannerWaiting,
    Histogram,
}

pub fn tr(lang: Lang, key: Key) -> &'static str {
    match (lang, key) {
        (Lang::PtBr, Key::AppTitle) => "MouseScope",
        (Lang::En, Key::AppTitle) => "MouseScope",
        (Lang::PtBr, Key::Subtitle) => "Laboratório visual de mouse",
        (Lang::En, Key::Subtitle) => "Visual mouse lab",
        (Lang::PtBr, Key::Phase) => "Fase 2",
        (Lang::En, Key::Phase) => "Phase 2",
        (Lang::PtBr, Key::Backend) => "Backend",
        (Lang::En, Key::Backend) => "Backend",
        (Lang::PtBr, Key::Source) => "Fonte",
        (Lang::En, Key::Source) => "Source",
        (Lang::PtBr, Key::Device) => "Dispositivo",
        (Lang::En, Key::Device) => "Device",
        (Lang::PtBr, Key::Mode) => "Modo",
        (Lang::En, Key::Mode) => "Mode",
        (Lang::PtBr, Key::Motion) => "Movimento",
        (Lang::En, Key::Motion) => "Motion",
        (Lang::PtBr, Key::Position) => "Posição",
        (Lang::En, Key::Position) => "Position",
        (Lang::PtBr, Key::Speed) => "Velocidade",
        (Lang::En, Key::Speed) => "Speed",
        (Lang::PtBr, Key::Distance) => "Distância",
        (Lang::En, Key::Distance) => "Distance",
        (Lang::PtBr, Key::Events) => "Eventos",
        (Lang::En, Key::Events) => "Events",
        (Lang::PtBr, Key::Reports) => "Relatórios",
        (Lang::En, Key::Reports) => "Reports",
        (Lang::PtBr, Key::Rate) => "Taxa",
        (Lang::En, Key::Rate) => "Rate",
        (Lang::PtBr, Key::Jitter) => "Jitter",
        (Lang::En, Key::Jitter) => "Jitter",
        (Lang::PtBr, Key::Samples) => "Amostras",
        (Lang::En, Key::Samples) => "Samples",
        (Lang::PtBr, Key::Confidence) => "Confiança",
        (Lang::En, Key::Confidence) => "Confidence",
        (Lang::PtBr, Key::Buttons) => "Botões",
        (Lang::En, Key::Buttons) => "Buttons",
        (Lang::PtBr, Key::Scroll) => "Scroll",
        (Lang::En, Key::Scroll) => "Scroll",
        (Lang::PtBr, Key::Polling) => "Taxa",
        (Lang::En, Key::Polling) => "Rate",
        (Lang::PtBr, Key::Targets) => "Alvos",
        (Lang::En, Key::Targets) => "Targets",
        (Lang::PtBr, Key::Dashboard) => "Painel",
        (Lang::En, Key::Dashboard) => "Dashboard",
        (Lang::PtBr, Key::Help) => "Ajuda",
        (Lang::En, Key::Help) => "Help",
        (Lang::PtBr, Key::Footer) => {
            "1-5 modo  TAB alterna  R reset  L log  F2 idioma  F1 ajuda  Q sair"
        }
        (Lang::En, Key::Footer) => {
            "1-5 mode  TAB cycle  R reset  L log  F2 language  F1 help  Q quit"
        }
        (Lang::PtBr, Key::Log) => "Log bruto",
        (Lang::En, Key::Log) => "Raw log",
        (Lang::PtBr, Key::LogEmpty) => "sem eventos",
        (Lang::En, Key::LogEmpty) => "no events",
        (Lang::PtBr, Key::Waiting) => "Aguardando mouse",
        (Lang::En, Key::Waiting) => "Waiting for mouse",
        (Lang::PtBr, Key::NoDevice) => "Nenhum dispositivo",
        (Lang::En, Key::NoDevice) => "No device",
        (Lang::PtBr, Key::MockSource) => "mock",
        (Lang::En, Key::MockSource) => "mock",
        (Lang::PtBr, Key::PhaseOneHint) => "Fase 1: domínio conectado; evdev chega na Fase 2",
        (Lang::En, Key::PhaseOneHint) => "Phase 1: domain wired; evdev arrives in Phase 2",
        (Lang::PtBr, Key::DemoHint) => "demo sintética ativa (--demo)",
        (Lang::En, Key::DemoHint) => "synthetic demo active (--demo)",
        (Lang::PtBr, Key::ResetHint) => "R reinicia contadores",
        (Lang::En, Key::ResetHint) => "R resets counters",
        (Lang::PtBr, Key::ToggleLogHint) => "L mostra o log bruto",
        (Lang::En, Key::ToggleLogHint) => "L shows the raw log",
        (Lang::PtBr, Key::ToggleLangHint) => "F2 alterna o idioma",
        (Lang::En, Key::ToggleLangHint) => "F2 toggles language",
        (Lang::PtBr, Key::HelpHint) => "F1 abre a ajuda",
        (Lang::En, Key::HelpHint) => "F1 opens help",
        (Lang::PtBr, Key::QuitHint) => "Q, Esc ou Ctrl+C para sair",
        (Lang::En, Key::QuitHint) => "Q, Esc or Ctrl+C to quit",
        (Lang::PtBr, Key::ConfidenceLow) => "baixa",
        (Lang::En, Key::ConfidenceLow) => "low",
        (Lang::PtBr, Key::ConfidenceMedium) => "média",
        (Lang::En, Key::ConfidenceMedium) => "medium",
        (Lang::PtBr, Key::ConfidenceHigh) => "alta",
        (Lang::En, Key::ConfidenceHigh) => "high",
        (Lang::PtBr, Key::NotAvailable) => "—",
        (Lang::En, Key::NotAvailable) => "—",
        (Lang::PtBr, Key::HelpTitle) => "Ajuda",
        (Lang::En, Key::HelpTitle) => "Help",
        (Lang::PtBr, Key::HelpBody) => {
            "A navegação é só por teclado; o mouse é a entrada sob teste.\n\
             1-5 escolhe o modo, TAB alterna, R reinicia, L abre o log.\n\
             F2 alterna pt-BR/en, F1 fecha esta ajuda, Q sai.\n\
             A Fase 2 usa evdev; sem permissão, o mouse do terminal cobre o básico."
        }
        (Lang::En, Key::HelpBody) => {
            "Navigation is keyboard-only; the mouse is the input under test.\n\
             1-5 selects a mode, TAB cycles, R resets, L opens the log.\n\
             F2 toggles pt-BR/en, F1 closes this help, Q quits.\n\
             Phase 2 uses evdev; without permission, the terminal mouse covers the basics."
        }
        (Lang::PtBr, Key::TooSmall) => "Terminal pequeno: redimensione para pelo menos 60x20.",
        (Lang::En, Key::TooSmall) => "Terminal too small: resize to at least 60x20.",
        (Lang::PtBr, Key::Devices) => "Dispositivos",
        (Lang::En, Key::Devices) => "Devices",
        (Lang::PtBr, Key::DevicePickerHint) => "T fecha  setas escolhem  Enter aplica",
        (Lang::En, Key::DevicePickerHint) => "T closes  arrows select  Enter applies",
        (Lang::PtBr, Key::DeviceOk) => "ok",
        (Lang::En, Key::DeviceOk) => "ok",
        (Lang::PtBr, Key::DeviceDenied) => "sem permissão",
        (Lang::En, Key::DeviceDenied) => "denied",
        (Lang::PtBr, Key::DevicePrimary) => "primário",
        (Lang::En, Key::DevicePrimary) => "primary",
        (Lang::PtBr, Key::PressButton) => "Pressione: {button}",
        (Lang::En, Key::PressButton) => "Press: {button}",
        (Lang::PtBr, Key::AllButtonsSeen) => "Todos os botões vistos",
        (Lang::En, Key::AllButtonsSeen) => "All buttons seen",
        (Lang::PtBr, Key::ScrollStepUp) => "cima",
        (Lang::En, Key::ScrollStepUp) => "up",
        (Lang::PtBr, Key::ScrollStepDown) => "baixo",
        (Lang::En, Key::ScrollStepDown) => "down",
        (Lang::PtBr, Key::ScrollStepLeft) => "esquerda",
        (Lang::En, Key::ScrollStepLeft) => "left",
        (Lang::PtBr, Key::ScrollStepRight) => "direita",
        (Lang::En, Key::ScrollStepRight) => "right",
        (Lang::PtBr, Key::ScrollPrompt) => "Role para {direction}",
        (Lang::En, Key::ScrollPrompt) => "Scroll {direction}",
        (Lang::PtBr, Key::ScrollDone) => "Scroll concluído",
        (Lang::En, Key::ScrollDone) => "Scroll complete",
        (Lang::PtBr, Key::BannerDegraded) => "Modo degradado: mouse do terminal",
        (Lang::En, Key::BannerDegraded) => "Degraded mode: terminal mouse",
        (Lang::PtBr, Key::BannerPermission) => {
            "Permissão negada em /dev/input; veja a ajuda ou use o mouse do terminal"
        }
        (Lang::En, Key::BannerPermission) => {
            "Permission denied on /dev/input; see help or use the terminal mouse"
        }
        (Lang::PtBr, Key::BannerWaiting) => "Aguardando um mouse legível",
        (Lang::En, Key::BannerWaiting) => "Waiting for a readable mouse",
        (Lang::PtBr, Key::Histogram) => "Histograma",
        (Lang::En, Key::Histogram) => "Histogram",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggles_language() {
        assert_eq!(Lang::PtBr.toggle(), Lang::En);
        assert_eq!(Lang::En.toggle(), Lang::PtBr);
    }

    #[test]
    fn every_key_has_a_translation() {
        for key in [
            Key::AppTitle,
            Key::Footer,
            Key::HelpBody,
            Key::TooSmall,
            Key::MockSource,
        ] {
            assert!(!tr(Lang::PtBr, key).is_empty());
            assert!(!tr(Lang::En, key).is_empty());
        }
    }
}
