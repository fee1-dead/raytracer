use crate::interval::Interval;
use crate::material::AnyMaterial;
use crate::ray::Ray;
use crate::vec3::{Point, Vec3};

pub mod polyhedra;

pub struct HitRecord {
    pub point: Point,
    pub normal: Vec3,
    pub t: f64,
    pub front_face: bool,
    pub material: AnyMaterial,
}

impl HitRecord {
    pub fn new(
        r: Ray,
        t: f64,
        outward_normal: impl FnOnce(Point) -> Vec3,
        material: impl Into<AnyMaterial>,
    ) -> Self {
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
            material: material.into(),
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
    pub material: AnyMaterial,
}

impl Object for Sphere {
    fn hit(&self, r: Ray, ray_t: Interval) -> Option<HitRecord> {
        let Sphere {
            center,
            radius,
            material,
        } = *self;
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

        Some(HitRecord::new(
            r,
            root,
            |point| (point - center) / radius,
            material,
        ))
    }
}

pub struct Triangle {
    pub a: Point,
    pub b: Point,
    pub c: Point,
    pub material: AnyMaterial,
}

impl Triangle {
    pub fn new(a: Point, b: Point, c: Point, material: impl Into<AnyMaterial>) -> Triangle {
        Triangle {
            a, b, c, material: material.into()
        }
    }
}

impl Object for Triangle {
    // https://en.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm
    fn hit(&self, r: Ray, ray_t: Interval) -> Option<HitRecord> {
        let e1 = self.b - self.a;
        let e2 = self.c - self.a;

        let r_cross_e2 = r.direction.cross(e2);
        let det = e1.dot(r_cross_e2);

        if det.abs() < 1e-8 {
            return None;
        }

        let inv_det = 1.0 / det;
        let s = r.origin - self.a;
        let u = inv_det * s.dot(r_cross_e2);

        if u < 0.0 || u > 1.0 {
            return None;
        }

        let s_cross_e1 = s.cross(e1);
        let v = inv_det * r.direction.dot(s_cross_e1);

        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = inv_det * e2.dot(s_cross_e1);

        if ray_t.contains(t) {
            Some(HitRecord::new(r, t, |_| e1.cross(e2), self.material))
        } else {
            None
        }
    }
}

pub enum AnyObject {
    Sphere(Sphere),
    Triangle(Triangle),
}

impl Object for AnyObject {
    fn hit(&self, r: Ray, ray_t: Interval) -> Option<HitRecord> {
        match self {
            AnyObject::Sphere(s) => s.hit(r, ray_t),
            AnyObject::Triangle(t) => t.hit(r, ray_t),
        }
    }
}

impl From<Sphere> for AnyObject {
    fn from(value: Sphere) -> Self {
        Self::Sphere(value)
    }
}

impl From<Triangle> for AnyObject {
    fn from(value: Triangle) -> Self {
        Self::Triangle(value)
    }
}

#[derive(Default)]
pub struct ObjectList {
    pub objects: Vec<AnyObject>,
}

impl ObjectList {
    pub fn add(&mut self, o: impl Into<AnyObject>) {
        self.objects.push(o.into())
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
