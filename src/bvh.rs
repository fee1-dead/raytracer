use std::cmp::Ordering;
use std::sync::Arc;

use crate::aabb::AxisAlignedBoundingBox;
use crate::interval::Interval;
use crate::object::{AnyObject, HitRecord, Object};
use crate::ray::Ray;

#[derive(Clone)]
pub struct BvhNode {
    left: Arc<AnyObject>,
    right: Arc<AnyObject>,
    bbox: AxisAlignedBoundingBox,
}

impl BvhNode {
    pub fn from_objects_mut(objects: &mut [AnyObject]) -> BvhNode {
        let mut bbox = AxisAlignedBoundingBox::EMPTY;
        for obj in objects.iter_mut() {
            bbox = bbox.merge(obj.bounding_box());
        }
        let [left, right] = match objects {
            [] => panic!("object list for BVH must be non-empty"),
            [a] => {
                let x = Arc::new(a.clone());
                [x.clone(), x]
            }
            [a, b] => [Arc::new(a.clone()), Arc::new(b.clone())],
            _ => {
                let axis = bbox.longest_axis();
                fn cmp_with_axis(
                    f: impl Fn(AxisAlignedBoundingBox) -> f64,
                ) -> impl Fn(&AnyObject, &AnyObject) -> Ordering {
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
                let (left, right) = objects.split_at_mut(mid);
                [
                    Arc::new(BvhNode::from_objects_mut(left).into()),
                    Arc::new(BvhNode::from_objects_mut(right).into()),
                ]
            }
        };

        BvhNode { left, right, bbox }
    }
}

impl From<Vec<AnyObject>> for BvhNode {
    fn from(mut objects: Vec<AnyObject>) -> Self {
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
