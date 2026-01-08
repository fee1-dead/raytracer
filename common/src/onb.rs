use crate::vec3::Vec3;

pub struct Onb {
    axis: [Vec3; 3],
}

impl Onb {
    pub fn new(n: Vec3) -> Self {
        let w = n.unit_vector();
        let a = if n.0.abs() > 0.9 {
            Vec3(0.0,1.0,0.0)
        } else {
            Vec3(1.0,0.0,0.0)
        };
        let v = n.cross(a).unit_vector();
        let u = n.cross(v);

        Self { axis: [u, v, w] }
    }

    pub fn w(&self) -> Vec3 {
        self.axis[2]
    }

    pub fn transform(&self, vec: Vec3) -> Vec3 {
        let [u, v, w] = self.axis;
        (vec.0 * u) + (vec.1 * v) + (vec.2 * w)
    }
}