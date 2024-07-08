use crate::vec3::{Point, Vec3};

#[derive(Clone, Copy)]
pub struct Ray {
    pub origin: Point,
    pub direction: Vec3,
}

impl Ray {
    pub fn at(self, t: f64) -> Point {
        self.origin + self.direction * t
    }
}
