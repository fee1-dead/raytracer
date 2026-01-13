use std::mem::take;

use common::aabb::AxisAlignedBoundingBox;
use common::ffi;
use common::interval::Interval;
use common::material::AnyMaterial;
use common::ray::Ray;
use common::vec3::{Point, Vec3};
pub use common::object::*;
use rand::rngs::SmallRng;

use crate::bvh::BvhNode;

pub mod polyhedra;

#[rustfmt::skip]
pub fn box_3d(a: Point, b: Point, mat: impl Into<AnyMaterial>) -> ObjectList {
    let min = Point::new(a.0.min(b.0), a.1.min(b.1), a.2.min(b.2));
    let max = Point::new(a.0.max(b.0), a.1.max(b.1), a.2.max(b.2));

    let dx = Vec3(max.0 - min.0, 0.0, 0.0);
    let dy = Vec3(0.0, max.1 - min.1, 0.0);
    let dz = Vec3(0.0, 0.0, max.2 - min.2);

    let mat = mat.into();

    let mut list = ObjectList::default();
    let faces = [   
        Quad::new(Point::new(min.0, min.1, max.2),  dx,  dy, mat), // front
        Quad::new(Point::new(max.0, min.1, max.2), -dz,  dy, mat), // right
        Quad::new(Point::new(max.0, min.1, min.2), -dx,  dy, mat), // back
        Quad::new(Point::new(min.0, min.1, min.2),  dz,  dy, mat), // left
        Quad::new(Point::new(min.0, max.1, max.2),  dx, -dz, mat), // top
        Quad::new(Point::new(min.0, min.1, min.2),  dx,  dz, mat), // bottom
    ];
    list.add_all(faces);
    list
}

#[derive(Default)]
pub struct ObjectList {
    objects: Vec<AnyObject>,
    aabb: AxisAlignedBoundingBox,
}

impl ObjectList {
    pub fn add(&mut self, o: impl Into<AnyObject>) {
        let obj = o.into();
        self.aabb = self.aabb.merge(obj.bounding_box());
        self.objects.push(obj)
    }

    pub fn add_all(&mut self, o: impl IntoIterator<Item = impl Into<AnyObject>>) {
        o.into_iter().for_each(|v| self.add(v));
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn condense(&mut self) -> BvhNode {
        let objects = take(&mut self.objects);
        BvhNode::from(objects.into_iter().map(|o| {
            let x: Box<dyn Object> = Box::new(o);
            x
        }).collect::<Vec<_>>())
    }

    pub fn finalize(self) -> ffi::ObjectList {
        // TODO gpu
        ffi::ObjectList { objects: Box::leak(self.objects.into_boxed_slice()), aabb: self.aabb }
    }
}

impl Object for ObjectList {
    fn hit(&self, r: Ray, ray_t: Interval) -> Option<HitRecord> {
        let mut closest = ray_t.max;
        let mut hit_record = None;
        for obj in &self.objects {
            if let Some(record) = obj.hit(r, Interval::new(ray_t.min, closest)) {
                closest = record.t;
                hit_record = Some(record);
            }
        }
        hit_record
    }
    fn bounding_box(&self) -> AxisAlignedBoundingBox {
        self.aabb
    }
    // TODO this is bad
    fn pdf_value(&self, origin: Point, direction: Vec3) -> f64 {
        let weight = (self.objects.len() as f64).recip();
        self.objects.iter().map(|o| weight * o.pdf_value(origin, direction)).sum()
    }
    fn random(&self, r: &mut SmallRng, origin: Point) -> Vec3 {
        use rand::prelude::IndexedRandom;
        self.objects.choose(r).unwrap().random(r, origin)
    }
}
