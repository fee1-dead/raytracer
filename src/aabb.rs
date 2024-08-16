use crate::interval::Interval;
use crate::ray::Ray;
use crate::vec3::Point;

#[derive(Clone, Copy, Default)]
pub struct AxisAlignedBoundingBox {
    pub x: Interval,
    pub y: Interval,
    pub z: Interval,
}

impl AxisAlignedBoundingBox {
    pub const EMPTY: AxisAlignedBoundingBox = AxisAlignedBoundingBox {
        x: Interval::EMPTY,
        y: Interval::EMPTY,
        z: Interval::EMPTY,
    };
    pub const UNIVERSE: AxisAlignedBoundingBox = AxisAlignedBoundingBox {
        x: Interval::UNIVERSE,
        y: Interval::UNIVERSE,
        z: Interval::UNIVERSE,
    };
    fn pad_to_minimums(mut self) -> Self {
        let delta = 0.0001;
        if self.x.size() < delta {
            self.x = self.x.expand(delta);
        }
        if self.y.size() < delta {
            self.y = self.y.expand(delta);
        }
        if self.z.size() < delta {
            self.z = self.z.expand(delta);
        }
        self
    }
    pub fn new(x: Interval, y: Interval, z: Interval) -> Self {
        let val = Self { x, y, z };
        val.pad_to_minimums()
    }

    pub fn from_points(a: Point, b: Point) -> AxisAlignedBoundingBox {
        let mk_interval = |a, b| {
            if a <= b {
                Interval::new(a, b)
            } else {
                Interval::new(b, a)
            }
        };
        AxisAlignedBoundingBox::new(
            mk_interval(a.0, b.0),
            mk_interval(a.1, b.1),
            mk_interval(a.2, b.2),
        )
    }

    pub fn hit(self, Ray { origin, direction }: Ray, mut ray_t: Interval) -> bool {
        for ((ax, origin_axis), dir_axis) in [self.x, self.y, self.z]
            .into_iter()
            .zip(origin)
            .zip(direction)
        {
            let adinv = 1.0 / dir_axis;
            let t0 = (ax.min - origin_axis) * adinv;
            let t1 = (ax.max - origin_axis) * adinv;
            let [t0, t1] = [t0.min(t1), t0.max(t1)];
            if t0 > ray_t.min {
                ray_t.min = t0
            };
            if t1 < ray_t.max {
                ray_t.max = t1
            };

            if ray_t.max <= ray_t.min {
                return false;
            }
        }
        true
    }

    pub fn merge(self, other: Self) -> Self {
        Self {
            x: self.x.merge(other.x),
            y: self.y.merge(other.y),
            z: self.z.merge(other.z),
        }
    }

    pub fn longest_axis(self) -> usize {
        [self.x, self.y, self.z]
            .into_iter()
            .map(|i| i.size())
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .unwrap()
            .0
    }
}
