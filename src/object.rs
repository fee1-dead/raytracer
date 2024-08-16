use std::mem::take;

use crate::aabb::AxisAlignedBoundingBox;
use crate::bvh::BvhNode;
use crate::interval::Interval;
use crate::material::AnyMaterial;
use crate::ray::Ray;
use crate::vec3::{Point, Vec3};

pub mod polyhedra;

#[derive(Clone, Copy)]
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
    fn bounding_box(&self) -> AxisAlignedBoundingBox;
}

#[derive(Clone, Copy)]
pub struct Sphere {
    center: Point,
    radius: f64,
    material: AnyMaterial,
    aabb: AxisAlignedBoundingBox,
}

impl Sphere {
    pub fn new(center: Point, radius: f64, material: impl Into<AnyMaterial>) -> Sphere {
        let rvec = Vec3::splat(radius);
        let aabb = AxisAlignedBoundingBox::from_points(center - rvec, center + rvec);
        Sphere {
            center,
            radius,
            material: material.into(),
            aabb,
        }
    }
}

impl Object for Sphere {
    fn hit(&self, r: Ray, ray_t: Interval) -> Option<HitRecord> {
        let Sphere {
            center,
            radius,
            material,
            aabb: _,
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
    fn bounding_box(&self) -> AxisAlignedBoundingBox {
        self.aabb
    }
}

#[derive(Clone, Copy)]
pub struct Triangle {
    a: Point,
    b: Point,
    c: Point,
    material: AnyMaterial,
    aabb: AxisAlignedBoundingBox,
}

impl Triangle {
    pub fn new(a: Point, b: Point, c: Point, material: impl Into<AnyMaterial>) -> Triangle {
        let aabb1 = AxisAlignedBoundingBox::from_points(a, b);
        let aabb2 = AxisAlignedBoundingBox::from_points(b, c);
        Triangle {
            a,
            b,
            c,
            material: material.into(),
            aabb: aabb1.merge(aabb2),
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
    fn bounding_box(&self) -> AxisAlignedBoundingBox {
        self.aabb
    }
}

#[derive(Clone)]
pub enum AnyObject {
    Sphere(Sphere),
    Triangle(Triangle),
    BvhNode(BvhNode),
}

impl Object for AnyObject {
    fn hit(&self, r: Ray, ray_t: Interval) -> Option<HitRecord> {
        match self {
            AnyObject::Sphere(s) => s.hit(r, ray_t),
            AnyObject::Triangle(t) => t.hit(r, ray_t),
            AnyObject::BvhNode(b) => b.hit(r, ray_t),
        }
    }
    fn bounding_box(&self) -> AxisAlignedBoundingBox {
        match self {
            AnyObject::Sphere(s) => s.bounding_box(),
            AnyObject::Triangle(t) => t.bounding_box(),
            AnyObject::BvhNode(b) => b.bounding_box(),
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

impl From<BvhNode> for AnyObject {
    fn from(value: BvhNode) -> Self {
        Self::BvhNode(value)
    }
}

#[derive(Default)]
pub struct ObjectList {
    objects: Vec<AnyObject>,
    aabb: AxisAlignedBoundingBox,
}

impl ObjectList {
    pub fn add(&mut self, o: impl Into<AnyObject>) {
        let o = o.into();
        self.aabb = self.aabb.merge(o.bounding_box());
        self.objects.push(o.into())
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn condense(&mut self) {
        let objects = take(&mut self.objects);
        self.objects.push(BvhNode::from(objects).into())
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
}
