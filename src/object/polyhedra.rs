use crate::material::AnyMaterial;
use crate::vec3::Point;

use super::Triangle;

pub fn tetrahedron(a: Point, b: Point, c: Point, d: Point, m: AnyMaterial) -> [Triangle; 4] {
    [
        Triangle::new(a, b, c, m),
        Triangle::new(a, b, d, m),
        Triangle::new(b, c, d, m),
        Triangle::new(a, c, d, m),
    ]
}