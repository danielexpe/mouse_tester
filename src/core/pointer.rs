use std::collections::VecDeque;

use crate::core::geometry::{Bounds, Point, Vec2};

pub const TRAIL_CAPACITY: usize = 96;
pub const TRAIL_WINDOW_US: u64 = 600_000;
pub const IDLE_SPEED_US: u64 = 250_000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrailPoint {
    pub pos: Point,
    pub at_us: u64,
    pub speed: f32,
}

#[derive(Clone, Debug)]
pub struct PointerState {
    bounds: Bounds,
    pos: Point,
    prev_pos: Point,
    raw_total_x: i64,
    raw_total_y: i64,
    distance_counts: f64,
    velocity: Vec2,
    speed_counts_s: f32,
    max_speed_counts_s: f32,
    trail: VecDeque<TrailPoint>,
    last_move_us: Option<u64>,
    clamped: bool,
}

impl PointerState {
    pub fn new(bounds: Bounds) -> Self {
        let center = bounds.center();
        Self {
            bounds,
            pos: center,
            prev_pos: center,
            raw_total_x: 0,
            raw_total_y: 0,
            distance_counts: 0.0,
            velocity: Vec2::ZERO,
            speed_counts_s: 0.0,
            max_speed_counts_s: 0.0,
            trail: VecDeque::with_capacity(TRAIL_CAPACITY),
            last_move_us: None,
            clamped: false,
        }
    }

    pub fn bounds(&self) -> Bounds {
        self.bounds
    }

    pub fn set_bounds(&mut self, bounds: Bounds) {
        self.bounds = bounds;
        let (pos, clamped) = bounds.clamp(self.pos);
        self.pos = pos;
        self.prev_pos = pos;
        self.clamped = clamped;
        self.trail.clear();
    }

    pub fn pos(&self) -> Point {
        self.pos
    }

    pub fn prev_pos(&self) -> Point {
        self.prev_pos
    }

    pub fn raw_total(&self) -> (i64, i64) {
        (self.raw_total_x, self.raw_total_y)
    }

    pub fn distance_counts(&self) -> f64 {
        self.distance_counts
    }

    pub fn velocity(&self) -> Vec2 {
        self.velocity
    }

    pub fn speed_counts_s(&self) -> f32 {
        self.speed_counts_s
    }

    pub fn max_speed_counts_s(&self) -> f32 {
        self.max_speed_counts_s
    }

    pub fn trail(&self) -> &VecDeque<TrailPoint> {
        &self.trail
    }

    pub fn is_clamped(&self) -> bool {
        self.clamped
    }

    pub fn apply_move(&mut self, dx: i32, dy: i32, now_us: u64) {
        if dx == 0 && dy == 0 {
            return;
        }

        let dx = i64::from(dx);
        let dy = i64::from(dy);
        self.raw_total_x = self.raw_total_x.saturating_add(dx);
        self.raw_total_y = self.raw_total_y.saturating_add(dy);

        let distance = (dx as f64).hypot(dy as f64);
        self.distance_counts += distance;

        let dt_us = self
            .last_move_us
            .map(|last| now_us.saturating_sub(last).clamp(1_000, IDLE_SPEED_US))
            .unwrap_or(16_000);
        let dt_s = dt_us as f32 / 1_000_000.0;
        let inst_velocity = Vec2::new(dx as f32 / dt_s, dy as f32 / dt_s);
        let inst_speed = inst_velocity.length();

        if self.last_move_us.is_some() {
            self.velocity.x = self.velocity.x * 0.75 + inst_velocity.x * 0.25;
            self.velocity.y = self.velocity.y * 0.75 + inst_velocity.y * 0.25;
            self.speed_counts_s = self.speed_counts_s * 0.75 + inst_speed * 0.25;
        } else {
            self.velocity = inst_velocity;
            self.speed_counts_s = inst_speed;
        }
        self.max_speed_counts_s = self.max_speed_counts_s.max(self.speed_counts_s);

        self.prev_pos = self.pos;
        let (pos, clamped) = self
            .bounds
            .clamp(Point::new(self.pos.x + dx as f32, self.pos.y + dy as f32));
        self.pos = pos;
        self.clamped = clamped;
        self.last_move_us = Some(now_us);
        self.push_trail(now_us);
    }

    pub fn set_normalized(&mut self, x: f32, y: f32, now_us: u64) {
        let x = x.clamp(0.0, 1.0);
        let y = y.clamp(0.0, 1.0);
        self.prev_pos = self.pos;
        let target = Point::new(
            self.bounds.min_x() + x * (self.bounds.max_x() - self.bounds.min_x()),
            self.bounds.min_y() + y * (self.bounds.max_y() - self.bounds.min_y()),
        );
        let (pos, clamped) = self.bounds.clamp(target);
        self.pos = pos;
        self.clamped = clamped;
        self.last_move_us = Some(now_us);
        self.push_trail(now_us);
    }

    pub fn recenter(&mut self) {
        self.pos = self.bounds.center();
        self.prev_pos = self.pos;
        self.velocity = Vec2::ZERO;
        self.speed_counts_s = 0.0;
        self.trail.clear();
        self.clamped = false;
    }

    pub fn reset(&mut self) {
        self.raw_total_x = 0;
        self.raw_total_y = 0;
        self.distance_counts = 0.0;
        self.max_speed_counts_s = 0.0;
        self.last_move_us = None;
        self.recenter();
    }

    pub fn tick(&mut self, now_us: u64) {
        while let Some(front) = self.trail.front() {
            if now_us.saturating_sub(front.at_us) > TRAIL_WINDOW_US {
                self.trail.pop_front();
            } else {
                break;
            }
        }

        if let Some(last) = self.last_move_us {
            if now_us.saturating_sub(last) > IDLE_SPEED_US {
                self.speed_counts_s = 0.0;
                self.velocity = Vec2::ZERO;
            }
        }
    }

    fn push_trail(&mut self, now_us: u64) {
        self.trail.push_back(TrailPoint {
            pos: self.pos,
            at_us: now_us,
            speed: self.speed_counts_s,
        });
        while self.trail.len() > TRAIL_CAPACITY {
            self.trail.pop_front();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pointer() -> PointerState {
        PointerState::new(Bounds::new(1000, 800).with_margin(0))
    }

    #[test]
    fn accumulates_raw_counts_and_distance() {
        let mut pointer = pointer();
        pointer.apply_move(10, -5, 1_000);
        pointer.apply_move(5, 5, 17_000);
        assert_eq!(pointer.raw_total(), (15, 0));
        let expected = 125.0f64.sqrt() + 50.0f64.sqrt();
        assert!((pointer.distance_counts() - expected).abs() < 1e-6);
    }

    #[test]
    fn clamps_at_edges_but_keeps_raw_counts() {
        let mut pointer = pointer();
        pointer.apply_move(5000, 5000, 1_000);
        assert!(pointer.is_clamped());
        assert_eq!(pointer.pos(), Point::new(1000.0, 800.0));
        assert_eq!(pointer.raw_total(), (5000, 5000));
    }

    #[test]
    fn trail_is_capped_and_pruned() {
        let mut pointer = pointer();
        for i in 0..200 {
            pointer.apply_move(1, 0, i * 16_000);
        }
        assert!(pointer.trail().len() <= TRAIL_CAPACITY);
        pointer.tick(200 * 16_000 + TRAIL_WINDOW_US + 1);
        assert!(pointer.trail().is_empty());
    }

    #[test]
    fn speed_goes_idle_after_timeout() {
        let mut pointer = pointer();
        pointer.apply_move(10, 0, 1_000);
        assert!(pointer.speed_counts_s() > 0.0);
        pointer.tick(1_000 + IDLE_SPEED_US + 1);
        assert_eq!(pointer.speed_counts_s(), 0.0);
        assert_eq!(pointer.velocity(), Vec2::ZERO);
    }

    #[test]
    fn reset_clears_totals_and_recenters() {
        let mut pointer = pointer();
        pointer.apply_move(20, 10, 1_000);
        pointer.reset();
        assert_eq!(pointer.raw_total(), (0, 0));
        assert_eq!(pointer.pos(), Point::new(500.0, 400.0));
        assert!(pointer.trail().is_empty());
    }

    #[test]
    fn normalized_position_maps_into_bounds() {
        let mut pointer = PointerState::new(Bounds::new(100, 50).with_margin(0));
        pointer.set_normalized(1.0, 1.0, 1_000);
        assert_eq!(pointer.pos(), Point::new(100.0, 50.0));
        pointer.set_normalized(-1.0, -1.0, 2_000);
        assert_eq!(pointer.pos(), Point::new(0.0, 0.0));
    }
}
