use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::core::codes::{
    ABS_X, ABS_Y, BTN_LEFT, BTN_TOOL_FINGER, BTN_TOOL_PEN, BTN_TOUCH, BTN_TRIGGER,
    INPUT_PROP_BUTTONPAD, INPUT_PROP_DIRECT, KEY_A, KEY_Z, REL_HWHEEL, REL_HWHEEL_HI_RES,
    REL_WHEEL, REL_WHEEL_HI_RES, REL_X, REL_Y,
};

pub type DeviceId = u32;

pub const TUI_MOUSE_ID: DeviceId = DeviceId::MAX;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceClass {
    Mouse,
    Touchpad,
    Tablet,
    Keyboard,
    Joystick,
    Combo,
    Unknown,
}

impl DeviceClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mouse => "mouse",
            Self::Touchpad => "touchpad",
            Self::Tablet => "tablet",
            Self::Keyboard => "keyboard",
            Self::Joystick => "joystick",
            Self::Combo => "combo",
            Self::Unknown => "unknown",
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Mouse => "class.mouse",
            Self::Touchpad => "class.touchpad",
            Self::Tablet => "class.tablet",
            Self::Keyboard => "class.keyboard",
            Self::Joystick => "class.joystick",
            Self::Combo => "class.combo",
            Self::Unknown => "class.unknown",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Capabilities {
    pub keys: BTreeSet<u16>,
    pub rel: BTreeSet<u16>,
    pub abs: BTreeSet<u16>,
    pub props: BTreeSet<u16>,
}

impl Capabilities {
    pub fn has_rel_xy(&self) -> bool {
        self.rel.contains(&REL_X) && self.rel.contains(&REL_Y)
    }

    pub fn has_abs_xy(&self) -> bool {
        self.abs.contains(&ABS_X) && self.abs.contains(&ABS_Y)
    }

    pub fn has_left_button(&self) -> bool {
        self.keys.contains(&BTN_LEFT)
    }

    pub fn has_keyboard(&self) -> bool {
        self.keys.contains(&KEY_A) && self.keys.contains(&KEY_Z)
    }

    pub fn mouse_like(&self) -> bool {
        self.has_rel_xy() && self.has_left_button()
    }

    pub fn touchpad_like(&self) -> bool {
        self.has_abs_xy()
            && (self.keys.contains(&BTN_TOOL_FINGER)
                || self.keys.contains(&BTN_TOUCH)
                || self.props.contains(&INPUT_PROP_BUTTONPAD))
    }

    pub fn tablet_like(&self) -> bool {
        self.has_abs_xy()
            && (self.props.contains(&INPUT_PROP_DIRECT) || self.keys.contains(&BTN_TOOL_PEN))
    }

    pub fn joystick_like(&self) -> bool {
        self.has_abs_xy() && self.keys.contains(&BTN_TRIGGER)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceInfo {
    pub id: DeviceId,
    pub path: PathBuf,
    pub name: String,
    pub phys: Option<String>,
    pub uniq: Option<String>,
    pub vendor: u16,
    pub product: u16,
    pub class: DeviceClass,
    pub capabilities: Capabilities,
    pub readable: bool,
}

impl DeviceInfo {
    pub fn is_pointing(&self) -> bool {
        matches!(
            self.class,
            DeviceClass::Mouse | DeviceClass::Combo | DeviceClass::Touchpad | DeviceClass::Tablet
        ) || self.capabilities.mouse_like()
    }

    pub fn tui_mouse() -> Self {
        let mut capabilities = Capabilities::default();
        capabilities.keys.extend([BTN_LEFT, 0x111, 0x112]);
        capabilities
            .rel
            .extend([REL_WHEEL, REL_WHEEL_HI_RES, REL_HWHEEL, REL_HWHEEL_HI_RES]);

        Self {
            id: TUI_MOUSE_ID,
            path: PathBuf::from("tui://mouse"),
            name: "Terminal mouse".to_string(),
            phys: None,
            uniq: None,
            vendor: 0,
            product: 0,
            class: DeviceClass::Mouse,
            capabilities,
            readable: true,
        }
    }

    pub fn mock(id: DeviceId) -> Self {
        let mut capabilities = Capabilities::default();
        capabilities.rel.extend([
            REL_X,
            REL_Y,
            REL_WHEEL,
            REL_WHEEL_HI_RES,
            REL_HWHEEL,
            REL_HWHEEL_HI_RES,
        ]);
        capabilities
            .keys
            .extend([BTN_LEFT, 0x111, 0x112, 0x113, 0x114, 0x115, 0x116, 0x117]);

        Self {
            id,
            path: PathBuf::from(format!("/dev/input/mock{id}")),
            name: "MouseScope Mock Mouse".to_string(),
            phys: Some("mock/0".to_string()),
            uniq: None,
            vendor: 0x1209,
            product: 0x0001,
            class: DeviceClass::Mouse,
            capabilities,
            readable: true,
        }
    }
}

pub fn classify(capabilities: &Capabilities) -> DeviceClass {
    if capabilities.mouse_like() && capabilities.has_keyboard() {
        DeviceClass::Combo
    } else if capabilities.mouse_like() {
        DeviceClass::Mouse
    } else if capabilities.touchpad_like() {
        DeviceClass::Touchpad
    } else if capabilities.tablet_like() {
        DeviceClass::Tablet
    } else if capabilities.joystick_like() {
        DeviceClass::Joystick
    } else if capabilities.has_keyboard() {
        DeviceClass::Keyboard
    } else {
        DeviceClass::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::codes::BTN_SIDE;

    #[test]
    fn classifies_mouse() {
        let device = DeviceInfo::mock(1);
        assert_eq!(classify(&device.capabilities), DeviceClass::Mouse);
        assert!(device.capabilities.mouse_like());
    }

    #[test]
    fn classifies_combo() {
        let mut device = DeviceInfo::mock(2);
        device.capabilities.keys.extend([KEY_A, KEY_Z]);
        assert_eq!(classify(&device.capabilities), DeviceClass::Combo);
    }

    #[test]
    fn classifies_touchpad_without_mouse_buttons() {
        let mut capabilities = Capabilities::default();
        capabilities.abs.extend([ABS_X, ABS_Y]);
        capabilities
            .keys
            .extend([BTN_TOOL_FINGER, BTN_LEFT, BTN_SIDE]);
        capabilities.props.insert(INPUT_PROP_BUTTONPAD);
        assert_eq!(classify(&capabilities), DeviceClass::Touchpad);
    }

    #[test]
    fn keeps_unknown_buttons() {
        let device = DeviceInfo::mock(3);
        assert!(device.capabilities.keys.contains(&BTN_SIDE));
    }
}
