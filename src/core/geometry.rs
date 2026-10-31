#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn length(self) -> f32 {
        self.x.hypot(self.y)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance(self, other: Self) -> f32 {
        (self.x - other.x).hypot(self.y - other.y)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds {
    pub width: u32,
    pub height: u32,
    pub margin: u32,
}

impl Bounds {
    pub const fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            margin: 12,
        }
    }

    pub const fn with_margin(self, margin: u32) -> Self {
        Self { margin, ..self }
    }

    pub fn min_x(self) -> f32 {
        self.margin as f32
    }

    pub fn min_y(self) -> f32 {
        self.margin as f32
    }

    pub fn max_x(self) -> f32 {
        (self.width.saturating_sub(self.margin)) as f32
    }

    pub fn max_y(self) -> f32 {
        (self.height.saturating_sub(self.margin)) as f32
    }

    pub fn center(self) -> Point {
        Point::new(self.width as f32 / 2.0, self.height as f32 / 2.0)
    }

    pub fn clamp(self, point: Point) -> (Point, bool) {
        let min_x = self.min_x().min(self.max_x());
        let min_y = self.min_y().min(self.max_y());
        let x = point.x.clamp(min_x, self.max_x().max(min_x));
        let y = point.y.clamp(min_y, self.max_y().max(min_y));
        let clamped = (x - point.x).abs() > f32::EPSILON || (y - point.y).abs() > f32::EPSILON;
        (Point::new(x, y), clamped)
    }
}

impl Default for Bounds {
    fn default() -> Self {
        Self::new(1344, 756)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_to_margin() {
        let bounds = Bounds::new(100, 80).with_margin(10);
        let (point, clamped) = bounds.clamp(Point::new(-5.0, 200.0));
        assert!(clamped);
        assert_eq!(point, Point::new(10.0, 70.0));
    }

    #[test]
    fn leaves_inside_point_untouched() {
        let bounds = Bounds::new(100, 80).with_margin(10);
        let (point, clamped) = bounds.clamp(Point::new(50.0, 40.0));
        assert!(!clamped);
        assert_eq!(point, Point::new(50.0, 40.0));
    }

    #[test]
    fn center_is_mid_screen() {
        let bounds = Bounds::new(100, 80);
        assert_eq!(bounds.center(), Point::new(50.0, 40.0));
    }
}
