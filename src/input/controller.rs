use std::path::Path;

use crate::core::device::{DeviceClass, DeviceId, DeviceInfo};
use crate::input::evdev::{discover, EvdevRuntime, InputError, InputMode, InputStatus, Selection};
use crate::input::InputEvent;
use crate::platform::evdev::is_readable;

#[derive(Debug)]
pub struct InputController {
    runtime: Option<EvdevRuntime>,
    status: InputStatus,
    degraded: bool,
}

impl InputController {
    pub fn start(selection: Selection, grab: bool, grab_force: bool) -> Result<Self, InputError> {
        let runtime = EvdevRuntime::start(selection, grab, grab_force)?;
        let status = runtime.status();
        Ok(Self {
            runtime: Some(runtime),
            status,
            degraded: false,
        })
    }

    pub fn degraded(message: String) -> Self {
        let mut discovery = discover();
        let tui_mouse = DeviceInfo::tui_mouse();
        let primary = tui_mouse.id;
        discovery.devices.push(tui_mouse);
        Self {
            runtime: None,
            status: InputStatus {
                mode: InputMode::Waiting,
                devices: discovery.devices,
                primary: Some(primary),
                message: Some(message),
            },
            degraded: true,
        }
    }

    pub fn update(&mut self) {
        if let Some(runtime) = &self.runtime {
            self.status = runtime.status();
        }
    }

    pub fn poll(&self) -> Vec<InputEvent> {
        self.runtime
            .as_ref()
            .map(EvdevRuntime::poll)
            .unwrap_or_default()
    }

    pub fn status(&self) -> &InputStatus {
        &self.status
    }

    pub fn devices(&self) -> &[DeviceInfo] {
        &self.status.devices
    }

    pub fn primary(&self) -> Option<DeviceId> {
        self.status.primary
    }

    pub fn is_degraded(&self) -> bool {
        self.degraded
    }

    pub fn select(&mut self, path: &Path, grab: bool, grab_force: bool) -> Result<(), InputError> {
        let discovery = discover();
        let Some(device) = discovery.devices.iter().find(|device| device.path == path) else {
            return Err(InputError::NoDevice);
        };
        if !matches!(
            device.class,
            DeviceClass::Mouse | DeviceClass::Combo | DeviceClass::Touchpad | DeviceClass::Tablet
        ) {
            return Err(InputError::Unsupported {
                path: path.to_path_buf(),
            });
        }
        if !device.readable || !is_readable(path) {
            return Err(InputError::Permission {
                path: path.to_path_buf(),
            });
        }

        if let Some(mut runtime) = self.runtime.take() {
            runtime.stop();
        }

        match EvdevRuntime::start(Selection::Path(path.to_path_buf()), grab, grab_force) {
            Ok(runtime) => {
                self.status = runtime.status();
                self.runtime = Some(runtime);
                self.degraded = false;
                Ok(())
            }
            Err(error) => {
                self.status = InputStatus {
                    mode: InputMode::Waiting,
                    devices: discovery.devices,
                    primary: None,
                    message: Some(error.to_string()),
                };
                self.degraded = true;
                Err(error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::device::TUI_MOUSE_ID;

    #[test]
    fn degraded_controller_exposes_terminal_mouse() {
        let controller = InputController::degraded("test".to_string());
        assert!(controller.is_degraded());
        assert_eq!(controller.primary(), Some(TUI_MOUSE_ID));
        assert!(controller
            .devices()
            .iter()
            .any(|device| device.id == TUI_MOUSE_ID));
    }
}
