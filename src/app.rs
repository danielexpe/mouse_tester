use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::core::device::DeviceInfo;
use crate::core::geometry::Bounds;
use crate::i18n::Lang;
use crate::input::DemoSource;
use crate::state::{Mode, Session};
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UiFlags {
    pub no_animations: bool,
    pub no_trail: bool,
    pub raw_log: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Quit,
    NextMode,
    PreviousMode,
    SetMode(Mode),
    Reset,
    ToggleLog,
    ToggleLang,
    ToggleHelp,
}

pub fn map_key(key: KeyEvent) -> Option<Action> {
    if key.kind != KeyEventKind::Press {
        return None;
    }

    if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c')) {
        return Some(Action::Quit);
    }

    match key.code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc => Some(Action::Quit),
        KeyCode::Char('1') => Some(Action::SetMode(Mode::Dashboard)),
        KeyCode::Char('2') => Some(Action::SetMode(Mode::Buttons)),
        KeyCode::Char('3') => Some(Action::SetMode(Mode::Scroll)),
        KeyCode::Char('4') => Some(Action::SetMode(Mode::Polling)),
        KeyCode::Char('5') => Some(Action::SetMode(Mode::Targets)),
        KeyCode::Tab => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                Some(Action::PreviousMode)
            } else {
                Some(Action::NextMode)
            }
        }
        KeyCode::BackTab => Some(Action::PreviousMode),
        KeyCode::Char('r' | 'R') => Some(Action::Reset),
        KeyCode::Char('l' | 'L') => Some(Action::ToggleLog),
        KeyCode::F(2) => Some(Action::ToggleLang),
        KeyCode::F(1) | KeyCode::Char('?') => Some(Action::ToggleHelp),
        _ => None,
    }
}

#[derive(Clone, Debug)]
pub struct App {
    pub session: Session,
    pub lang: Lang,
    pub theme: Theme,
    pub flags: UiFlags,
    pub show_help: bool,
    pub quit: bool,
    pub now_us: u64,
    pub demo_active: bool,
    demo: Option<DemoSource>,
}

impl App {
    pub fn new(
        bounds: Bounds,
        lang: Lang,
        theme: Theme,
        flags: UiFlags,
        initial_mode: Mode,
        demo_seed: Option<u64>,
    ) -> Self {
        let mut session = Session::new(bounds);
        session.add_device(DeviceInfo::mock(1));
        session.set_mode(initial_mode);
        let demo = demo_seed.map(|seed| DemoSource::new(1, seed));

        Self {
            session,
            lang,
            theme,
            flags,
            show_help: false,
            quit: false,
            now_us: 0,
            demo_active: demo.is_some(),
            demo,
        }
    }

    pub fn update(&mut self, now_us: u64) {
        self.now_us = now_us;
        if let Some(demo) = &mut self.demo {
            let events = demo.poll(now_us);
            for event in &events {
                self.session.handle_input(event);
            }
        }
        self.session.tick(now_us);
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if let Some(action) = map_key(key) {
            self.handle_action(action);
        }
    }

    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.quit = true,
            Action::NextMode => self.session.set_mode(self.session.mode.next()),
            Action::PreviousMode => self.session.set_mode(self.session.mode.previous()),
            Action::SetMode(mode) => self.session.set_mode(mode),
            Action::Reset => self.session.reset(),
            Action::ToggleLog => self.flags.raw_log = !self.flags.raw_log,
            Action::ToggleLang => self.lang = self.lang.toggle(),
            Action::ToggleHelp => self.show_help = !self.show_help,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEvent;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn app() -> App {
        App::new(
            Bounds::new(1000, 800),
            Lang::PtBr,
            Theme::default(),
            UiFlags::default(),
            Mode::Dashboard,
            Some(1),
        )
    }

    #[test]
    fn maps_quit_keys() {
        assert_eq!(map_key(key(KeyCode::Char('q'))), Some(Action::Quit));
        assert_eq!(map_key(key(KeyCode::Esc)), Some(Action::Quit));
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            Some(Action::Quit)
        );
    }

    #[test]
    fn maps_modes_and_help() {
        assert_eq!(
            map_key(key(KeyCode::Char('3'))),
            Some(Action::SetMode(Mode::Scroll))
        );
        assert_eq!(map_key(key(KeyCode::F(1))), Some(Action::ToggleHelp));
        assert_eq!(map_key(key(KeyCode::F(2))), Some(Action::ToggleLang));
        assert_eq!(map_key(key(KeyCode::Tab)), Some(Action::NextMode));
    }

    #[test]
    fn handles_actions() {
        let mut app = app();
        app.handle_action(Action::NextMode);
        assert_eq!(app.session.mode, Mode::Buttons);
        app.handle_action(Action::ToggleLang);
        assert_eq!(app.lang, Lang::En);
        app.handle_action(Action::ToggleLog);
        assert!(app.flags.raw_log);
        app.handle_action(Action::Quit);
        assert!(app.quit);
    }

    #[test]
    fn demo_feeds_session() {
        let mut app = app();
        app.update(1_000_000);
        assert!(app.session.metrics.events() > 0);
        assert!(app.session.metrics.reports() > 0);
        assert!(!app.session.log.is_empty());
    }

    #[test]
    fn no_demo_leaves_session_idle() {
        let mut app = App::new(
            Bounds::new(1000, 800),
            Lang::PtBr,
            Theme::default(),
            UiFlags::default(),
            Mode::Dashboard,
            None,
        );
        app.update(1_000_000);
        assert_eq!(app.session.metrics.events(), 0);
        assert_eq!(app.session.primary, Some(1));
    }
}
