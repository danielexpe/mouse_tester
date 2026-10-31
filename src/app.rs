use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::core::device::DeviceInfo;
use crate::core::geometry::Bounds;
use crate::i18n::Lang;
use crate::input::{DemoSource, InputController, InputEvent};
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
    ToggleDevices,
    DeviceUp,
    DeviceDown,
    DeviceConfirm,
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
        KeyCode::Char('t' | 'T') => Some(Action::ToggleDevices),
        KeyCode::Up => Some(Action::DeviceUp),
        KeyCode::Down => Some(Action::DeviceDown),
        KeyCode::Enter => Some(Action::DeviceConfirm),
        KeyCode::F(2) => Some(Action::ToggleLang),
        KeyCode::F(1) | KeyCode::Char('?') => Some(Action::ToggleHelp),
        _ => None,
    }
}

#[derive(Debug)]
pub struct AppConfig {
    pub bounds: Bounds,
    pub lang: Lang,
    pub theme: Theme,
    pub flags: UiFlags,
    pub initial_mode: Mode,
    pub demo_seed: Option<u64>,
    pub controller: Option<InputController>,
    pub grab: bool,
    pub grab_force: bool,
}

#[derive(Debug)]
pub struct App {
    pub session: Session,
    pub lang: Lang,
    pub theme: Theme,
    pub flags: UiFlags,
    pub show_help: bool,
    pub quit: bool,
    pub now_us: u64,
    pub demo_active: bool,
    pub controller: Option<InputController>,
    pub degraded: bool,
    pub banner: Option<String>,
    pub show_devices: bool,
    pub device_index: usize,
    pub device_message: Option<String>,
    pub grab: bool,
    pub grab_force: bool,
    demo: Option<DemoSource>,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        let mut session = Session::new(config.bounds);
        let demo = if let Some(seed) = config.demo_seed {
            session.add_device(DeviceInfo::mock(1));
            Some(DemoSource::new(1, seed))
        } else {
            None
        };

        let controller = if demo.is_some() {
            None
        } else {
            config.controller
        };
        let mut degraded = false;
        let mut banner = None;
        if let Some(controller) = &controller {
            let status = controller.status().clone();
            session.sync_devices(status.devices, status.primary);
            degraded = controller.is_degraded();
            banner = status.message;
        }

        Self {
            session,
            lang: config.lang,
            theme: config.theme,
            flags: config.flags,
            show_help: false,
            quit: false,
            now_us: 0,
            demo_active: demo.is_some(),
            controller,
            degraded,
            banner,
            show_devices: false,
            device_index: 0,
            device_message: None,
            grab: config.grab,
            grab_force: config.grab_force,
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

        if let Some(controller) = &mut self.controller {
            controller.update();
            let events = controller.poll();
            for event in &events {
                self.session.handle_input(event);
            }
            let status = controller.status().clone();
            self.session.sync_devices(status.devices, status.primary);
            self.degraded = controller.is_degraded();
            self.banner = status.message;
        }

        self.session.tick(now_us);
    }

    pub fn handle_input_events(&mut self, events: &[InputEvent]) {
        for event in events {
            self.session.handle_input(event);
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if self.show_devices {
            match key.code {
                KeyCode::Esc => {
                    self.show_devices = false;
                    return;
                }
                KeyCode::Char('t' | 'T') => {
                    self.show_devices = false;
                    return;
                }
                KeyCode::Up => {
                    self.move_device(-1);
                    return;
                }
                KeyCode::Down => {
                    self.move_device(1);
                    return;
                }
                KeyCode::Enter => {
                    self.confirm_device();
                    return;
                }
                _ => {}
            }
        }

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
            Action::ToggleDevices => {
                if self.controller.is_some() {
                    self.show_devices = !self.show_devices;
                    self.device_index = 0;
                    self.device_message = None;
                }
            }
            Action::DeviceUp => {
                if self.show_devices {
                    self.move_device(-1);
                }
            }
            Action::DeviceDown => {
                if self.show_devices {
                    self.move_device(1);
                }
            }
            Action::DeviceConfirm => {
                if self.show_devices {
                    self.confirm_device();
                }
            }
        }
    }

    pub fn devices(&self) -> &[DeviceInfo] {
        self.controller
            .as_ref()
            .map(InputController::devices)
            .unwrap_or(&self.session.devices)
    }

    pub fn selected_device(&self) -> Option<&DeviceInfo> {
        self.devices().get(self.device_index)
    }

    fn move_device(&mut self, delta: isize) {
        let len = self.devices().len();
        if len == 0 {
            self.device_index = 0;
            return;
        }
        let len = len as isize;
        self.device_index = (self.device_index as isize + delta).rem_euclid(len) as usize;
    }

    fn confirm_device(&mut self) {
        let Some(device) = self.selected_device().cloned() else {
            return;
        };
        if device
            .path
            .as_os_str()
            .to_string_lossy()
            .starts_with("tui://")
        {
            self.show_devices = false;
            return;
        }
        let Some(controller) = &mut self.controller else {
            return;
        };
        match controller.select(&device.path, self.grab, self.grab_force) {
            Ok(()) => {
                let status = controller.status().clone();
                self.session.sync_devices(status.devices, status.primary);
                self.degraded = controller.is_degraded();
                self.banner = status.message;
                self.device_message = None;
                self.show_devices = false;
            }
            Err(error) => {
                self.degraded = controller.is_degraded();
                self.device_message = Some(error.to_string());
            }
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

    fn config() -> AppConfig {
        AppConfig {
            bounds: Bounds::new(1000, 800),
            lang: Lang::PtBr,
            theme: Theme::default(),
            flags: UiFlags::default(),
            initial_mode: Mode::Dashboard,
            demo_seed: Some(1),
            controller: None,
            grab: false,
            grab_force: false,
        }
    }

    fn app() -> App {
        App::new(config())
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
        assert_eq!(
            map_key(key(KeyCode::Char('t'))),
            Some(Action::ToggleDevices)
        );
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
        let mut config = config();
        config.demo_seed = None;
        let mut app = App::new(config);
        app.update(1_000_000);
        assert_eq!(app.session.metrics.events(), 0);
        assert!(app.session.devices.is_empty());
    }

    #[test]
    fn picker_without_controller_is_ignored() {
        let mut app = app();
        app.handle_action(Action::ToggleDevices);
        assert!(!app.show_devices);
    }

    #[test]
    fn mode_challenges_advance_with_events() {
        let mut app = app();
        app.handle_action(Action::SetMode(Mode::Buttons));
        let events = [
            InputEvent::mock(
                1,
                1_000,
                crate::input::InputKind::Button {
                    code: crate::core::codes::BTN_LEFT,
                    pressed: true,
                },
            ),
            InputEvent::mock(1, 2_000, crate::input::InputKind::Report),
        ];
        app.handle_input_events(&events);
        assert!(app
            .session
            .buttons_challenge
            .seen()
            .contains(&crate::core::codes::BTN_LEFT));
    }

    #[test]
    fn device_index_wraps() {
        let mut app = app();
        app.session.devices = vec![DeviceInfo::mock(1), DeviceInfo::mock(2)];
        app.show_devices = true;
        app.move_device(-1);
        assert_eq!(app.device_index, 1);
        app.move_device(1);
        assert_eq!(app.device_index, 0);
    }

    #[test]
    fn selected_device_is_stable() {
        let app = app();
        assert_eq!(app.selected_device().map(|device| device.id), Some(1));
    }

    #[test]
    fn demo_ignores_controller() {
        let mut config = config();
        config.demo_seed = Some(7);
        let app = App::new(config);
        assert!(app.demo_active);
        assert!(app.controller.is_none());
    }
}
