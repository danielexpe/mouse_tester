use std::collections::BTreeSet;

use crate::core::codes::{REL_HWHEEL, REL_HWHEEL_HI_RES, REL_WHEEL, REL_WHEEL_HI_RES};
use crate::core::device::Capabilities;
use crate::core::scroll::ScrollAxis;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScrollStep {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Debug, Default)]
pub struct ScrollChallenge {
    steps: Vec<ScrollStep>,
    seen: BTreeSet<ScrollStep>,
}

impl ScrollChallenge {
    pub fn from_capabilities(capabilities: Option<&Capabilities>) -> Self {
        let mut steps = Vec::new();
        match capabilities {
            Some(capabilities) => {
                let vertical = capabilities.rel.contains(&REL_WHEEL)
                    || capabilities.rel.contains(&REL_WHEEL_HI_RES);
                let horizontal = capabilities.rel.contains(&REL_HWHEEL)
                    || capabilities.rel.contains(&REL_HWHEEL_HI_RES);
                if vertical {
                    steps.extend([ScrollStep::Up, ScrollStep::Down]);
                }
                if horizontal {
                    steps.extend([ScrollStep::Left, ScrollStep::Right]);
                }
            }
            None => steps.extend([
                ScrollStep::Up,
                ScrollStep::Down,
                ScrollStep::Left,
                ScrollStep::Right,
            ]),
        }
        if steps.is_empty() {
            steps.extend([ScrollStep::Up, ScrollStep::Down]);
        }
        Self {
            steps,
            seen: BTreeSet::new(),
        }
    }

    pub fn observe(&mut self, axis: ScrollAxis, direction: i8) {
        if direction == 0 {
            return;
        }
        let step = match (axis, direction > 0) {
            (ScrollAxis::Vertical, true) => ScrollStep::Up,
            (ScrollAxis::Vertical, false) => ScrollStep::Down,
            (ScrollAxis::Horizontal, true) => ScrollStep::Right,
            (ScrollAxis::Horizontal, false) => ScrollStep::Left,
        };
        if self.steps.contains(&step) {
            self.seen.insert(step);
        }
    }

    pub fn steps(&self) -> &[ScrollStep] {
        &self.steps
    }

    pub fn seen(&self) -> &BTreeSet<ScrollStep> {
        &self.seen
    }

    pub fn current(&self) -> Option<ScrollStep> {
        self.steps
            .iter()
            .copied()
            .find(|step| !self.seen.contains(step))
    }

    pub fn progress(&self) -> (usize, usize) {
        (self.seen.len(), self.steps.len())
    }

    pub fn is_done(&self) -> bool {
        self.steps.iter().all(|step| self.seen.contains(step))
    }

    pub fn reset(&mut self) {
        self.seen.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capabilities(rel: &[u16]) -> Capabilities {
        let mut capabilities = Capabilities::default();
        capabilities.rel.extend(rel.iter().copied());
        capabilities
    }

    #[test]
    fn vertical_only_device_has_two_steps() {
        let challenge = ScrollChallenge::from_capabilities(Some(&capabilities(&[REL_WHEEL])));
        assert_eq!(challenge.steps(), &[ScrollStep::Up, ScrollStep::Down]);
    }

    #[test]
    fn full_device_has_four_steps() {
        let challenge =
            ScrollChallenge::from_capabilities(Some(&capabilities(&[REL_WHEEL, REL_HWHEEL])));
        assert_eq!(
            challenge.steps(),
            &[
                ScrollStep::Up,
                ScrollStep::Down,
                ScrollStep::Left,
                ScrollStep::Right
            ]
        );
    }

    #[test]
    fn degraded_mode_assumes_four_steps() {
        let challenge = ScrollChallenge::from_capabilities(None);
        assert_eq!(challenge.progress(), (0, 4));
    }

    #[test]
    fn observes_directions() {
        let mut challenge =
            ScrollChallenge::from_capabilities(Some(&capabilities(&[REL_WHEEL, REL_HWHEEL])));
        challenge.observe(ScrollAxis::Vertical, 1);
        challenge.observe(ScrollAxis::Vertical, -1);
        challenge.observe(ScrollAxis::Horizontal, 1);
        challenge.observe(ScrollAxis::Horizontal, -1);
        assert!(challenge.is_done());
        assert_eq!(challenge.progress(), (4, 4));
    }

    #[test]
    fn ignores_direction_zero() {
        let mut challenge = ScrollChallenge::from_capabilities(None);
        challenge.observe(ScrollAxis::Vertical, 0);
        assert_eq!(challenge.progress(), (0, 4));
    }

    #[test]
    fn reset_keeps_steps() {
        let mut challenge = ScrollChallenge::from_capabilities(None);
        challenge.observe(ScrollAxis::Vertical, 1);
        challenge.reset();
        assert_eq!(challenge.progress(), (0, 4));
    }
}
