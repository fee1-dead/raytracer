#[derive(Clone, Copy)]
pub struct Interval {
    pub min: f64,
    pub max: f64,
}

impl Interval {
    pub const EMPTY: Interval = Interval::new(f64::INFINITY, f64::NEG_INFINITY);
    pub const UNIVERSE: Interval = Interval::new(f64::NEG_INFINITY, f64::INFINITY);
    pub const fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    pub fn merge(self, other: Interval) -> Interval {
        Interval {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    pub fn expand(self, delta: f64) -> Self {
        let padding = delta / 2.0;
        Self::new(self.min - padding, self.max + padding)
    }
    pub fn size(self) -> f64 {
        self.max - self.min
    }

    pub fn contains(self, x: f64) -> bool {
        (self.min..=self.max).contains(&x)
    }

    pub fn surrounds(self, x: f64) -> bool {
        self.min < x && x < self.max
    }

    pub fn clamp(self, x: f64) -> f64 {
        if x < self.min {
            self.min
        } else if x > self.max {
            self.max
        } else {
            x
        }
    }
}

impl Default for Interval {
    fn default() -> Self {
        Self::EMPTY
    }
}
