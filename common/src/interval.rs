use core::ops::Add;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Interval {
    pub min: f32,
    pub max: f32,
}

impl Interval {
    pub const EMPTY: Interval = Interval::new(f32::INFINITY, f32::NEG_INFINITY);
    pub const UNIVERSE: Interval = Interval::new(f32::NEG_INFINITY, f32::INFINITY);
    pub const fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }

    pub fn merge(self, other: Interval) -> Interval {
        Interval {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    pub fn expand(self, delta: f32) -> Self {
        let padding = delta / 2.0;
        Self::new(self.min - padding, self.max + padding)
    }
    pub fn size(self) -> f32 {
        self.max - self.min
    }

    pub fn contains(self, x: f32) -> bool {
        (self.min..=self.max).contains(&x)
    }

    pub fn surrounds(self, x: f32) -> bool {
        self.min < x && x < self.max
    }

    pub fn clamp(self, x: f32) -> f32 {
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

impl Add<f32> for Interval {
    type Output = Interval;

    fn add(self, rhs: f32) -> Interval {
        Interval::new(self.min + rhs, self.max + rhs)
    }
}
