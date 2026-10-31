pub const HIRES_PER_NOTCH: i32 = 120;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollAxis {
    Vertical,
    Horizontal,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WheelFrame {
    pub vertical_normal: i32,
    pub vertical_hires: i32,
    pub horizontal_normal: i32,
    pub horizontal_hires: i32,
}

impl WheelFrame {
    pub fn push(&mut self, axis: ScrollAxis, value: i32, hires: bool) {
        match (axis, hires) {
            (ScrollAxis::Vertical, false) => {
                self.vertical_normal = self.vertical_normal.saturating_add(value);
            }
            (ScrollAxis::Vertical, true) => {
                self.vertical_hires = self.vertical_hires.saturating_add(value);
            }
            (ScrollAxis::Horizontal, false) => {
                self.horizontal_normal = self.horizontal_normal.saturating_add(value);
            }
            (ScrollAxis::Horizontal, true) => {
                self.horizontal_hires = self.horizontal_hires.saturating_add(value);
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.vertical_normal == 0
            && self.vertical_hires == 0
            && self.horizontal_normal == 0
            && self.horizontal_hires == 0
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollUpdate {
    pub axis: ScrollAxis,
    pub notches: i32,
    pub remainder: i32,
    pub direction: i8,
    pub pulses: u8,
    pub hires_units: i32,
}

#[derive(Clone, Debug, Default)]
pub struct ScrollState {
    vertical_ticks: i64,
    horizontal_ticks: i64,
    vertical_remainder: i32,
    horizontal_remainder: i32,
    last_axis: Option<ScrollAxis>,
    last_delta: i32,
    last_at_us: Option<u64>,
}

impl ScrollState {
    pub fn apply_frame(&mut self, frame: &WheelFrame, now_us: u64) -> Vec<ScrollUpdate> {
        let mut updates = Vec::new();

        if let Some(update) = axis_update(
            ScrollAxis::Vertical,
            frame.vertical_normal,
            frame.vertical_hires,
            &mut self.vertical_remainder,
            &mut self.vertical_ticks,
        ) {
            updates.push(update);
        }

        if let Some(update) = axis_update(
            ScrollAxis::Horizontal,
            frame.horizontal_normal,
            frame.horizontal_hires,
            &mut self.horizontal_remainder,
            &mut self.horizontal_ticks,
        ) {
            updates.push(update);
        }

        if let Some(last) = updates.last() {
            self.last_axis = Some(last.axis);
            self.last_delta = last.notches;
            self.last_at_us = Some(now_us);
        }

        updates
    }

    pub fn vertical_ticks(&self) -> i64 {
        self.vertical_ticks
    }

    pub fn horizontal_ticks(&self) -> i64 {
        self.horizontal_ticks
    }

    pub fn vertical_remainder(&self) -> i32 {
        self.vertical_remainder
    }

    pub fn horizontal_remainder(&self) -> i32 {
        self.horizontal_remainder
    }

    pub fn last_axis(&self) -> Option<ScrollAxis> {
        self.last_axis
    }

    pub fn last_delta(&self) -> i32 {
        self.last_delta
    }

    pub fn last_at_us(&self) -> Option<u64> {
        self.last_at_us
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

fn axis_update(
    axis: ScrollAxis,
    normal: i32,
    hires: i32,
    remainder: &mut i32,
    ticks: &mut i64,
) -> Option<ScrollUpdate> {
    if normal == 0 && hires == 0 {
        return None;
    }

    let (notches, new_remainder) = if normal != 0 {
        (normal, 0)
    } else {
        let total = remainder.saturating_add(hires);
        (total / HIRES_PER_NOTCH, total % HIRES_PER_NOTCH)
    };

    *remainder = new_remainder;
    if notches != 0 {
        *ticks = ticks.saturating_add(i64::from(notches));
    }

    let direction = if notches != 0 {
        notches.signum()
    } else {
        hires.signum()
    };
    let pulses = if notches == 0 {
        1
    } else {
        notches.unsigned_abs().clamp(1, 3) as u8
    };

    Some(ScrollUpdate {
        axis,
        notches,
        remainder: new_remainder,
        direction: direction as i8,
        pulses,
        hires_units: hires,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_normal_notches() {
        let mut state = ScrollState::default();
        let mut frame = WheelFrame::default();
        frame.push(ScrollAxis::Vertical, 3, false);
        let update = state.apply_frame(&frame, 1_000);
        assert_eq!(update.len(), 1);
        assert_eq!(update[0].notches, 3);
        assert_eq!(state.vertical_ticks(), 3);
        assert_eq!(state.vertical_remainder(), 0);
    }

    #[test]
    fn accumulates_hi_res_until_notch() {
        let mut state = ScrollState::default();
        let mut frame = WheelFrame::default();
        frame.push(ScrollAxis::Vertical, 60, true);
        let first = state.apply_frame(&frame, 1_000);
        assert_eq!(first[0].notches, 0);
        assert_eq!(state.vertical_remainder(), 60);

        frame.clear();
        frame.push(ScrollAxis::Vertical, 60, true);
        let second = state.apply_frame(&frame, 2_000);
        assert_eq!(second[0].notches, 1);
        assert_eq!(state.vertical_ticks(), 1);
        assert_eq!(state.vertical_remainder(), 0);
    }

    #[test]
    fn normal_wins_over_hi_res_without_double_count() {
        let mut state = ScrollState::default();
        let mut frame = WheelFrame::default();
        frame.push(ScrollAxis::Vertical, 1, false);
        frame.push(ScrollAxis::Vertical, 120, true);
        let update = state.apply_frame(&frame, 1_000);
        assert_eq!(update[0].notches, 1);
        assert_eq!(state.vertical_ticks(), 1);
        assert_eq!(state.vertical_remainder(), 0);
    }

    #[test]
    fn counts_horizontal_axis() {
        let mut state = ScrollState::default();
        let mut frame = WheelFrame::default();
        frame.push(ScrollAxis::Horizontal, -2, false);
        let update = state.apply_frame(&frame, 1_000);
        assert_eq!(update[0].notches, -2);
        assert_eq!(state.horizontal_ticks(), -2);
        assert_eq!(state.horizontal_remainder(), 0);
    }

    #[test]
    fn negative_hires_produces_negative_notch() {
        let mut state = ScrollState::default();
        let mut frame = WheelFrame::default();
        frame.push(ScrollAxis::Vertical, -120, true);
        let update = state.apply_frame(&frame, 1_000);
        assert_eq!(update[0].notches, -1);
        assert_eq!(update[0].direction, -1);
    }

    #[test]
    fn reset_clears_counters() {
        let mut state = ScrollState::default();
        let mut frame = WheelFrame::default();
        frame.push(ScrollAxis::Vertical, 2, false);
        state.apply_frame(&frame, 1_000);
        state.reset();
        assert_eq!(state.vertical_ticks(), 0);
        assert!(state.last_axis().is_none());
    }
}
