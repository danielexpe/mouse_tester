pub const BTN_0: u16 = 0x100;
pub const BTN_1: u16 = 0x101;
pub const BTN_2: u16 = 0x102;
pub const BTN_3: u16 = 0x103;
pub const BTN_4: u16 = 0x104;
pub const BTN_5: u16 = 0x105;
pub const BTN_6: u16 = 0x106;
pub const BTN_7: u16 = 0x107;
pub const BTN_8: u16 = 0x108;
pub const BTN_9: u16 = 0x109;

pub const BTN_LEFT: u16 = 0x110;
pub const BTN_RIGHT: u16 = 0x111;
pub const BTN_MIDDLE: u16 = 0x112;
pub const BTN_SIDE: u16 = 0x113;
pub const BTN_EXTRA: u16 = 0x114;
pub const BTN_FORWARD: u16 = 0x115;
pub const BTN_BACK: u16 = 0x116;
pub const BTN_TASK: u16 = 0x117;

pub const BTN_TRIGGER_HAPPY_FIRST: u16 = 0x2c0;
pub const BTN_TRIGGER_HAPPY_LAST: u16 = 0x2e7;

pub const REL_X: u16 = 0x00;
pub const REL_Y: u16 = 0x01;
pub const REL_HWHEEL: u16 = 0x06;
pub const REL_WHEEL: u16 = 0x08;
pub const REL_WHEEL_HI_RES: u16 = 0x0b;
pub const REL_HWHEEL_HI_RES: u16 = 0x0c;

pub const ABS_X: u16 = 0x00;
pub const ABS_Y: u16 = 0x01;

pub const KEY_A: u16 = 30;
pub const KEY_Z: u16 = 44;

pub const BTN_TOOL_PEN: u16 = 0x140;
pub const BTN_TOOL_FINGER: u16 = 0x145;
pub const BTN_TOUCH: u16 = 0x14a;

pub const BTN_TRIGGER: u16 = 0x120;

pub const INPUT_PROP_POINTER: u16 = 0x00;
pub const INPUT_PROP_DIRECT: u16 = 0x01;
pub const INPUT_PROP_BUTTONPAD: u16 = 0x02;

pub fn is_mouse_button(code: u16) -> bool {
    (BTN_0..=BTN_9).contains(&code)
        || (BTN_LEFT..=BTN_TASK).contains(&code)
        || (BTN_TRIGGER_HAPPY_FIRST..=BTN_TRIGGER_HAPPY_LAST).contains(&code)
}

fn known_short_label(code: u16) -> Option<&'static str> {
    match code {
        BTN_LEFT => Some("[L]"),
        BTN_RIGHT => Some("[R]"),
        BTN_MIDDLE => Some("[M]"),
        BTN_SIDE => Some("[S1]"),
        BTN_EXTRA => Some("[S2]"),
        BTN_FORWARD => Some("[F]"),
        BTN_BACK => Some("[B]"),
        BTN_TASK => Some("[T]"),
        BTN_0 => Some("[0]"),
        BTN_1 => Some("[1]"),
        BTN_2 => Some("[2]"),
        BTN_3 => Some("[3]"),
        BTN_4 => Some("[4]"),
        BTN_5 => Some("[5]"),
        BTN_6 => Some("[6]"),
        BTN_7 => Some("[7]"),
        BTN_8 => Some("[8]"),
        BTN_9 => Some("[9]"),
        _ => None,
    }
}

pub fn short_label(code: u16) -> &'static str {
    known_short_label(code).unwrap_or("[?]")
}

pub fn label(code: u16) -> String {
    match known_short_label(code) {
        Some(label) => label.to_string(),
        None => format!("[0x{code:03X}]"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_mouse_buttons() {
        assert!(is_mouse_button(BTN_LEFT));
        assert!(is_mouse_button(BTN_SIDE));
        assert!(is_mouse_button(BTN_TRIGGER_HAPPY_FIRST));
        assert!(!is_mouse_button(KEY_A));
        assert!(!is_mouse_button(BTN_TOOL_FINGER));
    }

    #[test]
    fn falls_back_to_hex_label() {
        assert_eq!(label(BTN_LEFT), "[L]");
        assert_eq!(label(0x2c3), "[0x2C3]");
    }
}
