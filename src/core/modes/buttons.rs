use std::collections::BTreeSet;

use crate::core::codes::{
    is_mouse_button, BTN_BACK, BTN_EXTRA, BTN_FORWARD, BTN_LEFT, BTN_MIDDLE, BTN_RIGHT, BTN_SIDE,
    BTN_TASK,
};
use crate::core::device::Capabilities;

const COMMON_ORDER: [u16; 8] = [
    BTN_LEFT,
    BTN_RIGHT,
    BTN_MIDDLE,
    BTN_SIDE,
    BTN_EXTRA,
    BTN_FORWARD,
    BTN_BACK,
    BTN_TASK,
];

#[derive(Clone, Debug, Default)]
pub struct ButtonsChallenge {
    expected: Vec<u16>,
    seen: BTreeSet<u16>,
}

impl ButtonsChallenge {
    pub fn new(capabilities: &Capabilities) -> Self {
        let mut expected: Vec<u16> = COMMON_ORDER
            .iter()
            .copied()
            .filter(|code| capabilities.keys.contains(code))
            .collect();

        for code in capabilities.keys.iter().copied() {
            if is_mouse_button(code) && !expected.contains(&code) {
                expected.push(code);
            }
        }

        if expected.is_empty() {
            expected.extend([BTN_LEFT, BTN_RIGHT, BTN_MIDDLE]);
        }

        Self {
            expected,
            seen: BTreeSet::new(),
        }
    }

    pub fn observe(&mut self, code: u16) {
        if !is_mouse_button(code) {
            return;
        }
        if !self.expected.contains(&code) {
            self.expected.push(code);
        }
        self.seen.insert(code);
    }

    pub fn expected(&self) -> &[u16] {
        &self.expected
    }

    pub fn seen(&self) -> &BTreeSet<u16> {
        &self.seen
    }

    pub fn current(&self) -> Option<u16> {
        self.expected
            .iter()
            .copied()
            .find(|code| !self.seen.contains(code))
    }

    pub fn missing(&self) -> Vec<u16> {
        self.expected
            .iter()
            .copied()
            .filter(|code| !self.seen.contains(code))
            .collect()
    }

    pub fn progress(&self) -> (usize, usize) {
        (self.seen.len(), self.expected.len())
    }

    pub fn is_done(&self) -> bool {
        self.expected.iter().all(|code| self.seen.contains(code))
    }

    pub fn reset(&mut self) {
        self.seen.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::codes::BTN_FORWARD;

    fn capabilities(codes: &[u16]) -> Capabilities {
        let mut capabilities = Capabilities::default();
        capabilities.keys.extend(codes.iter().copied());
        capabilities
    }

    #[test]
    fn builds_expected_from_capabilities() {
        let challenge = ButtonsChallenge::new(&capabilities(&[BTN_LEFT, BTN_RIGHT, BTN_SIDE]));
        assert_eq!(challenge.expected(), &[BTN_LEFT, BTN_RIGHT, BTN_SIDE]);
        assert_eq!(challenge.progress(), (0, 3));
    }

    #[test]
    fn observes_and_completes() {
        let mut challenge = ButtonsChallenge::new(&capabilities(&[BTN_LEFT, BTN_RIGHT]));
        challenge.observe(BTN_LEFT);
        assert_eq!(challenge.current(), Some(BTN_RIGHT));
        challenge.observe(BTN_RIGHT);
        assert!(challenge.is_done());
        assert_eq!(challenge.progress(), (2, 2));
    }

    #[test]
    fn adds_unannounced_mouse_buttons() {
        let mut challenge = ButtonsChallenge::new(&capabilities(&[BTN_LEFT]));
        challenge.observe(BTN_FORWARD);
        assert!(challenge.expected().contains(&BTN_FORWARD));
        assert!(challenge.seen().contains(&BTN_FORWARD));
    }

    #[test]
    fn ignores_keyboard_codes() {
        let mut challenge = ButtonsChallenge::new(&capabilities(&[BTN_LEFT]));
        challenge.observe(crate::core::codes::KEY_A);
        assert_eq!(challenge.progress(), (0, 1));
    }

    #[test]
    fn falls_back_to_three_buttons() {
        let challenge = ButtonsChallenge::new(&Capabilities::default());
        assert_eq!(challenge.expected(), &[BTN_LEFT, BTN_RIGHT, BTN_MIDDLE]);
    }

    #[test]
    fn reset_keeps_expected() {
        let mut challenge = ButtonsChallenge::new(&capabilities(&[BTN_LEFT, BTN_RIGHT]));
        challenge.observe(BTN_LEFT);
        challenge.reset();
        assert_eq!(challenge.progress(), (0, 2));
    }
}
