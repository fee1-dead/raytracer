use std::cmp::Ordering;
use std::sync::Arc;

use crate::aabb::AxisAlignedBoundingBox;
use crate::interval::Interval;
use crate::object::{HitRecord, Object};
use crate::ray::Ray;

#[derive(Clone)]
pub struct BvhNode {
    left: Arc<dyn Object>,
    right: Arc<dyn Object>,
    bbox: AxisAlignedBoundingBox,
}

impl BvhNode {
    pub fn from_objects_mut(objects: &mut Vec<Box<dyn Object>>) -> BvhNode {
        let mut bbox = AxisAlignedBoundingBox::EMPTY;
        for obj in objects.iter_mut() {
            bbox = bbox.merge(obj.bounding_box());
        }
        let [left, right] = match objects.len() {
            0 => panic!("object list for BVH must be non-empty"),
            1 => {
                let o = objects.pop().unwrap();
                let x: Arc<dyn Object> = Arc::from(o);
                [x.clone(), x]
            }
            2 => {
                let o2 = objects.pop().unwrap();
                let o1 = objects.pop().unwrap();
                [o1.into(), o2.into()]
            }
            _ => {
                let axis = bbox.longest_axis();
                fn cmp_with_axis(
                    f: impl Fn(AxisAlignedBoundingBox) -> f64,
                ) -> impl Fn(&Box<dyn Object>, &Box<dyn Object>) -> Ordering {
                    move |a, b| {
                        let a = f(a.bounding_box());
                        let b = f(b.bounding_box());
                        a.total_cmp(&b)
                    }
                }
                objects.sort_by(cmp_with_axis(if axis == 0 {
                    |o: AxisAlignedBoundingBox| o.x.min
                } else if axis == 1 {
                    |o: AxisAlignedBoundingBox| o.y.min
                } else {
                    |o: AxisAlignedBoundingBox| o.z.min
                }));
                let mid = objects.len() / 2;
                let mut right = objects.split_off(mid);
                let mut left = objects;
                let a1: Arc<dyn Object> = Arc::new(BvhNode::from_objects_mut(&mut left));
                let a2: Arc<dyn Object> = Arc::new(BvhNode::from_objects_mut(&mut right));
                [a1, a2]
            }
        };

        BvhNode { left, right, bbox }
    }
}

impl From<Vec<Box<dyn Object>>> for BvhNode {
    fn from(mut objects: Vec<Box<dyn Object>>) -> Self {
        Self::from_objects_mut(&mut objects)
    }
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
