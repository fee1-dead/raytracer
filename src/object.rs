use crate::interval::Interval;
use crate::ray::Ray;
use crate::vec3::{Point, Vec3};

pub struct HitRecord {
    pub point: Point,
    pub normal: Vec3,
    pub t: f64,
    pub front_face: bool,
}

impl HitRecord {
    pub fn new(r: Ray, t: f64, outward_normal: impl FnOnce(Point) -> Vec3) -> Self {
        let point = r.at(t);
        let outward_normal = outward_normal(point);
        let front_face = r.direction.dot(outward_normal) <= 0.0;
        let normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };
        Self {
            point,
            normal,
            t,
            front_face,
        }
    }
}

pub trait Object {
    fn hit(&self, r: Ray, ray_t: Interval) -> Option<HitRecord>;
}

#[derive(Clone, Copy)]
pub struct Sphere {
    pub center: Point,
    pub radius: f64,
}

impl Object for Sphere {
    fn hit(&self, r: Ray, ray_t: Interval) -> Option<HitRecord> {
        let Sphere { center, radius } = *self;
        let oc = center - r.origin;
        let a = r.direction.length_squared();
        let h = r.direction.dot(oc);
        let c = oc.length_squared() - radius * radius;
        let discriminant = h * h - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        // find the nearest root that lies in the acceptable range;
        let mut root = (h - sqrtd) / a;
        if !ray_t.surrounds(root) {
            root = (h + sqrtd) / a;
            if !ray_t.surrounds(root) {
                return None;
            }
        }

        Some(HitRecord::new(r, root, |point| (point - center) / radius))
    }
}

#[derive(Default)]
pub struct ObjectList {
    pub objects: Vec<Box<dyn Object>>,
}

impl ObjectList {
    pub fn add(&mut self, o: impl Object + 'static) {
        self.objects.push(Box::new(o))
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
}
