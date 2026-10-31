use crate::core::buttons::DragSummary;
use crate::core::geometry::{Bounds, Point};
use crate::util::rng::SplitMix64;

pub const TARGET_RADIUS: f32 = 42.0;
pub const TOTAL_TARGETS: u32 = 9;
pub const RESULT_VISIBLE_US: u64 = 600_000;

const TARGET_KINDS: [TargetKind; 3] =
    [TargetKind::Click, TargetKind::Drag, TargetKind::DoubleClick];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetKind {
    Click,
    Drag,
    DoubleClick,
}

impl TargetKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Click => "click",
            Self::Drag => "drag",
            Self::DoubleClick => "double-click",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Target {
    pub kind: TargetKind,
    pub center: Point,
    pub radius: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetResult {
    Hit,
    Miss,
}

#[derive(Clone, Debug)]
pub struct TargetsChallenge {
    seed: u64,
    rng: SplitMix64,
    bounds: Bounds,
    target: Target,
    round: u32,
    hits: u32,
    misses: u32,
    error_samples: u32,
    total_error: f32,
    total_duration_us: u64,
    last_duration_us: Option<u64>,
    last_error: Option<f32>,
    last_result: Option<TargetResult>,
    last_result_us: u64,
    last_eval_pos: Option<Point>,
    round_started_us: u64,
    press_pos: Option<Point>,
    drag_armed: bool,
    awaiting_second_click: bool,
    last_center: Option<Point>,
}

impl TargetsChallenge {
    pub fn new(seed: u64, bounds: Bounds) -> Self {
        let mut challenge = Self {
            seed,
            rng: SplitMix64::new(seed),
            bounds,
            target: Target {
                kind: TargetKind::Click,
                center: bounds.center(),
                radius: TARGET_RADIUS,
            },
            round: 0,
            hits: 0,
            misses: 0,
            error_samples: 0,
            total_error: 0.0,
            total_duration_us: 0,
            last_duration_us: None,
            last_error: None,
            last_result: None,
            last_result_us: 0,
            last_eval_pos: None,
            round_started_us: 0,
            press_pos: None,
            drag_armed: false,
            awaiting_second_click: false,
            last_center: None,
        };
        challenge.target = challenge.place_target(TargetKind::Click);
        challenge
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn bounds(&self) -> Bounds {
        self.bounds
    }

    pub fn target(&self) -> Target {
        self.target
    }

    pub fn kind(&self) -> TargetKind {
        self.target.kind
    }

    pub fn round(&self) -> u32 {
        self.round
    }

    pub fn total(&self) -> u32 {
        TOTAL_TARGETS
    }

    pub fn hits(&self) -> u32 {
        self.hits
    }

    pub fn misses(&self) -> u32 {
        self.misses
    }

    pub fn is_done(&self) -> bool {
        self.round >= TOTAL_TARGETS
    }

    pub fn last_duration_us(&self) -> Option<u64> {
        self.last_duration_us
    }

    pub fn last_error(&self) -> Option<f32> {
        self.last_error
    }

    pub fn last_result(&self) -> Option<TargetResult> {
        self.last_result
    }

    pub fn last_eval_pos(&self) -> Option<Point> {
        self.last_eval_pos
    }

    pub fn result_visible(&self, now_us: u64) -> bool {
        self.last_result.is_some()
            && now_us.saturating_sub(self.last_result_us) <= RESULT_VISIBLE_US
    }

    pub fn accuracy(&self) -> f32 {
        let attempts = self.hits + self.misses;
        if attempts == 0 {
            0.0
        } else {
            self.hits as f32 / attempts as f32
        }
    }

    pub fn mean_error(&self) -> Option<f32> {
        if self.error_samples == 0 {
            None
        } else {
            Some(self.total_error / self.error_samples as f32)
        }
    }

    pub fn mean_duration_us(&self) -> Option<u64> {
        if self.hits == 0 {
            None
        } else {
            Some(self.total_duration_us / u64::from(self.hits))
        }
    }

    pub fn observe_press(&mut self, pos: Point, now_us: u64, double_click: bool) {
        if self.is_done() {
            return;
        }
        self.press_pos = Some(pos);
        let error = pos.distance(self.target.center);

        match self.target.kind {
            TargetKind::Click => {
                if error <= self.target.radius {
                    self.finish(true, error, now_us, pos);
                } else {
                    self.miss(error, now_us, pos);
                }
            }
            TargetKind::Drag => {
                if error <= self.target.radius {
                    self.drag_armed = true;
                } else {
                    self.drag_armed = false;
                    self.miss(error, now_us, pos);
                }
            }
            TargetKind::DoubleClick => {
                if error > self.target.radius {
                    self.awaiting_second_click = false;
                    self.miss(error, now_us, pos);
                } else if double_click && self.awaiting_second_click {
                    self.finish(true, error, now_us, pos);
                } else {
                    self.awaiting_second_click = true;
                }
            }
        }
    }

    pub fn observe_release(&mut self, now_us: u64, drag: Option<&DragSummary>) {
        if self.is_done() || self.target.kind != TargetKind::Drag || !self.drag_armed {
            return;
        }
        self.drag_armed = false;
        let press = self.press_pos;
        let Some(press) = press else {
            return;
        };
        let error = press.distance(self.target.center);
        if drag.is_some_and(|drag| drag.moved) {
            self.finish(true, error, now_us, press);
        } else {
            self.miss(error, now_us, press);
        }
    }

    pub fn restart(&mut self) {
        let seed = self.seed;
        let bounds = self.bounds;
        *self = Self::new(seed, bounds);
    }

    pub fn set_bounds(&mut self, bounds: Bounds) {
        if self.bounds == bounds {
            return;
        }
        self.bounds = bounds;
        self.target = self.place_target(self.target.kind);
    }

    fn finish(&mut self, hit: bool, error: f32, now_us: u64, pos: Point) {
        if !hit {
            self.miss(error, now_us, pos);
            return;
        }

        self.hits = self.hits.saturating_add(1);
        self.round = self.round.saturating_add(1);
        self.error_samples = self.error_samples.saturating_add(1);
        self.total_error += error;
        let duration = now_us.saturating_sub(self.round_started_us);
        self.total_duration_us = self.total_duration_us.saturating_add(duration);
        self.last_duration_us = Some(duration);
        self.last_error = Some(error);
        self.last_result = Some(TargetResult::Hit);
        self.last_result_us = now_us;
        self.last_eval_pos = Some(pos);
        self.drag_armed = false;
        self.awaiting_second_click = false;
        self.press_pos = None;
        self.advance(now_us);
    }

    fn miss(&mut self, error: f32, now_us: u64, pos: Point) {
        self.misses = self.misses.saturating_add(1);
        self.error_samples = self.error_samples.saturating_add(1);
        self.total_error += error;
        self.last_error = Some(error);
        self.last_result = Some(TargetResult::Miss);
        self.last_result_us = now_us;
        self.last_eval_pos = Some(pos);
    }

    fn advance(&mut self, now_us: u64) {
        self.round_started_us = now_us;
        if self.is_done() {
            return;
        }
        let kind = TARGET_KINDS[(self.round as usize) % TARGET_KINDS.len()];
        self.target = self.place_target(kind);
    }

    fn place_target(&mut self, kind: TargetKind) -> Target {
        let margin = TARGET_RADIUS + 24.0;
        let min_x = (self.bounds.min_x() + margin).min(self.bounds.max_x() - margin);
        let min_y = (self.bounds.min_y() + margin).min(self.bounds.max_y() - margin);
        let max_x = (self.bounds.max_x() - margin).max(self.bounds.min_x() + margin);
        let max_y = (self.bounds.max_y() - margin).max(self.bounds.min_y() + margin);

        let mut candidate = self.bounds.center();
        for _ in 0..8 {
            let center = Point::new(
                self.rng.range_i32(min_x as i32, max_x as i32) as f32,
                self.rng.range_i32(min_y as i32, max_y as i32) as f32,
            );
            candidate = center;
            let too_close = self
                .last_center
                .is_some_and(|last| last.distance(center) < TARGET_RADIUS * 2.5);
            if !too_close {
                break;
            }
        }

        let center = self.bounds.clamp(candidate).0;
        self.last_center = Some(center);
        Target {
            kind,
            center,
            radius: TARGET_RADIUS,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn challenge(seed: u64) -> TargetsChallenge {
        TargetsChallenge::new(seed, Bounds::new(1344, 756))
    }

    fn drag_from(start: Point, end: Point, moved: bool) -> DragSummary {
        DragSummary {
            start,
            end,
            duration_us: 100_000,
            moved,
        }
    }

    #[test]
    fn same_seed_reproduces_positions() {
        let a = challenge(7);
        let b = challenge(7);
        assert_eq!(a.target(), b.target());
        assert_eq!(a.kind(), TargetKind::Click);
    }

    #[test]
    fn different_seeds_diverge() {
        let a = challenge(1);
        let b = challenge(2);
        assert_ne!(a.target().center, b.target().center);
    }

    #[test]
    fn target_stays_inside_bounds() {
        for seed in 0..64 {
            let challenge = challenge(seed);
            let target = challenge.target();
            assert!(target.center.x >= target.radius);
            assert!(target.center.y >= target.radius);
            assert!(target.center.x <= 1344.0 - target.radius);
            assert!(target.center.y <= 756.0 - target.radius);
        }
    }

    #[test]
    fn click_inside_is_hit_and_advances() {
        let mut challenge = challenge(3);
        let center = challenge.target().center;
        challenge.observe_press(center, 1_000, false);
        assert_eq!(challenge.hits(), 1);
        assert_eq!(challenge.round(), 1);
        assert_eq!(challenge.kind(), TargetKind::Drag);
        assert_eq!(challenge.last_duration_us(), Some(1_000));
    }

    #[test]
    fn click_outside_is_miss_and_keeps_target() {
        let mut challenge = challenge(3);
        let center = challenge.target().center;
        challenge.observe_press(Point::new(center.x + 1000.0, center.y), 1_000, false);
        assert_eq!(challenge.hits(), 0);
        assert_eq!(challenge.misses(), 1);
        assert_eq!(challenge.round(), 0);
        assert_eq!(challenge.last_result(), Some(TargetResult::Miss));
    }

    #[test]
    fn drag_requires_movement() {
        let mut challenge = challenge(3);
        let click = challenge.target().center;
        challenge.observe_press(click, 1_000, false);
        assert_eq!(challenge.kind(), TargetKind::Drag);
        let center = challenge.target().center;
        challenge.observe_press(center, 2_000, false);
        challenge.observe_release(50_000, Some(&drag_from(center, center, false)));
        assert_eq!(challenge.hits(), 1);
        assert_eq!(challenge.misses(), 1);

        challenge.observe_press(center, 60_000, false);
        challenge.observe_release(
            120_000,
            Some(&drag_from(
                center,
                Point::new(center.x + 80.0, center.y),
                true,
            )),
        );
        assert_eq!(challenge.hits(), 2);
        assert_eq!(challenge.kind(), TargetKind::DoubleClick);
    }

    #[test]
    fn double_click_needs_second_click_inside() {
        let mut challenge = challenge(3);
        let click = challenge.target().center;
        challenge.observe_press(click, 1_000, false);
        let drag = challenge.target().center;
        challenge.observe_press(drag, 2_000, false);
        challenge.observe_release(
            50_000,
            Some(&drag_from(drag, Point::new(drag.x + 80.0, drag.y), true)),
        );
        assert_eq!(challenge.kind(), TargetKind::DoubleClick);
        let center = challenge.target().center;
        challenge.observe_press(center, 1_000_000, false);
        challenge.observe_release(1_040_000, None);
        challenge.observe_press(center, 1_200_000, true);
        assert_eq!(challenge.hits(), 3);
        assert_eq!(challenge.last_result(), Some(TargetResult::Hit));
    }

    #[test]
    fn metrics_report_accuracy_and_mean_error() {
        let mut challenge = challenge(3);
        let center = challenge.target().center;
        challenge.observe_press(center, 1_000, false);
        let next = challenge.target().center;
        challenge.observe_press(Point::new(next.x + 1000.0, next.y), 2_000, false);
        assert!((challenge.accuracy() - 0.5).abs() < f32::EPSILON);
        assert!(challenge.mean_error().unwrap() > 0.0);
        assert!(challenge.mean_duration_us().is_some());
    }

    #[test]
    fn covers_all_rounds_and_finishes() {
        let mut challenge = challenge(5);
        for round in 0..TOTAL_TARGETS {
            let kind = challenge.kind();
            let center = challenge.target().center;
            challenge.observe_press(center, u64::from(round) * 1_000_000 + 1_000, false);
            if kind == TargetKind::Drag {
                challenge.observe_release(
                    u64::from(round) * 1_000_000 + 50_000,
                    Some(&drag_from(
                        center,
                        Point::new(center.x + 90.0, center.y),
                        true,
                    )),
                );
            } else if kind == TargetKind::DoubleClick {
                challenge.observe_release(u64::from(round) * 1_000_000 + 40_000, None);
                challenge.observe_press(center, u64::from(round) * 1_000_000 + 120_000, true);
            }
        }
        assert!(challenge.is_done());
        assert_eq!(challenge.hits(), TOTAL_TARGETS);
    }

    #[test]
    fn restart_replays_same_seed() {
        let mut challenge = challenge(11);
        let first = challenge.target();
        challenge.observe_press(first.center, 1_000, false);
        challenge.restart();
        assert_eq!(challenge.seed(), 11);
        assert_eq!(challenge.target(), first);
        assert_eq!(challenge.round(), 0);
        assert_eq!(challenge.hits(), 0);
    }
}
