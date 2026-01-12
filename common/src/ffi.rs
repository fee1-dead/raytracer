use crate::aabb::AxisAlignedBoundingBox;
use crate::interval::Interval;
use crate::object::{HitRecord, Object};
use crate::ray::Ray;

#[repr(C)]
pub struct BvhNode {
    pub left: &'static BvhNode,
    pub right: &'static BvhNode,
    pub bbox: AxisAlignedBoundingBox,
}

impl Object for BvhNode {
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

