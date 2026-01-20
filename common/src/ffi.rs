use rand::rngs::SmallRng;

use crate::aabb::AxisAlignedBoundingBox;
use crate::interval::Interval;
use crate::object::{AnyObject, HitRecord, Object};
use crate::ray::Ray;
use crate::vec3::{Point, Vec3};

#[derive(Clone, Copy)]
pub struct ObjectList {
    pub objects: &'static [AnyObject],
    pub aabb: AxisAlignedBoundingBox,
}

impl Object for ObjectList {
    fn hit(&self, r: Ray, ray_t: Interval) -> Option<HitRecord> {
        let mut closest = ray_t.max;
        let mut hit_record = None;
        for obj in self.objects {
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
    fn pdf_value(&self, origin: Point, direction: Vec3) -> f32 {
        let weight = (self.objects.len() as f32).recip();
        self.objects.iter().map(|o| weight * o.pdf_value(origin, direction)).sum()
    }
    fn random(&self, r: &mut SmallRng, origin: Point) -> Vec3 {
        use rand::prelude::IndexedRandom;
        self.objects.choose(r).unwrap().random(r, origin)
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BvhNode {
    pub left: &'static AnyObject,
    pub right: &'static AnyObject,
    pub bbox: AxisAlignedBoundingBox,
}

impl Object for BvhNode {
    #[inline(never)]
    fn hit(&self, r: Ray, mut ray_t: Interval) -> Option<HitRecord> {
        if !self.bbox.hit(r, ray_t) {
            return None;
        }

        let rec = self.left.hit(r, ray_t);
        if let Some(rec) = rec {
            ray_t.max = rec.t;
        }

        self.right.hit(r, ray_t).or(rec)
    }
    fn bounding_box(&self) -> AxisAlignedBoundingBox {
        self.bbox
    }
}
