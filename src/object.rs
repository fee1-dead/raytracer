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

        if !(0.0..=1.0).contains(&u) {
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

#[derive(Clone, Copy)]
pub struct Quad {
    q: Point,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    mat: AnyMaterial,
    bbox: AxisAlignedBoundingBox,
    normal: Vec3,
    d: f64,
}

impl Quad {
    pub fn new(q: Point, u: Vec3, v: Vec3, mat: impl Into<AnyMaterial>) -> Self {
        let bbox1 = AxisAlignedBoundingBox::from_points(q, q + u + v);
        let bbox2 = AxisAlignedBoundingBox::from_points(q + u, q + v);
        let n = u.cross(v);
        let normal = n.unit_vector();
        let d = normal.dot(q);
        let w = n / n.dot(n);
        Self {
            q,
            u,
            v,
            w,
            mat: mat.into(),
            bbox: bbox1.merge(bbox2),
            normal,
            d,
        }
    }

    pub fn hit_as_interior(&self, a: f64, b: f64, rec: HitRecord) -> Option<HitRecord> {
        let unit_interval = Interval::new(0.0, 1.0);

        if !unit_interval.contains(a) || !unit_interval.contains(b) {
            None
        } else {
            // TODO uv coords should be set here.
            Some(rec)
        }
    }
}

impl Object for Quad {
    fn hit(&self, r: Ray, ray_t: Interval) -> Option<HitRecord> {
        let denom = self.normal.dot(r.direction);

        if denom.abs() < 1e-8 {
            return None;
        }

        let t = (self.d - self.normal.dot(r.origin)) / denom;
        if !ray_t.contains(t) {
            return None;
        }

        let planar_hitpt_vector = r.at(t) - self.q;
        let alpha = self.w.dot(planar_hitpt_vector.cross(self.v));
        let beta = self.w.dot(self.u.cross(planar_hitpt_vector));
        let rec = HitRecord::new(r, t, |_| self.normal, self.mat);
        self.hit_as_interior(alpha, beta, rec)
    }
    fn bounding_box(&self) -> AxisAlignedBoundingBox {
        self.bbox
    }
}

#[rustfmt::skip]
pub fn box_3d(a: Point, b: Point, mat: impl Into<AnyMaterial>) -> [Quad; 6] {
    let min = Point::new(a.0.min(b.0), a.1.min(b.1), a.2.min(b.2));
    let max = Point::new(a.0.max(b.0), a.1.max(b.1), a.2.max(b.2));

    let dx = Vec3(max.0 - min.0, 0.0, 0.0);
    let dy = Vec3(0.0, max.1 - min.1, 0.0);
    let dz = Vec3(0.0, 0.0, max.2 - min.2);

    let mat = mat.into();

    [   
        Quad::new(Point::new(min.0, min.1, max.2),  dx,  dy, mat), // front
        Quad::new(Point::new(max.0, min.1, max.2), -dz,  dy, mat), // right
        Quad::new(Point::new(max.0, min.1, min.2), -dx,  dy, mat), // back
        Quad::new(Point::new(min.0, min.1, min.2),  dz,  dy, mat), // left
        Quad::new(Point::new(min.0, max.1, max.2),  dx, -dz, mat), // top
        Quad::new(Point::new(min.0, min.1, min.2),  dx,  dz, mat), // bottom
    ]
}

macro_rules! generate_any_object {
    ($($variant:ident),*$(,)?) => {
        #[derive(Clone)]
        pub enum AnyObject {
            $($variant($variant),)*
        }
        impl Object for AnyObject {
            fn hit(&self, r: Ray, ray_t: Interval) -> Option<HitRecord> {
                match self {
                    $(AnyObject::$variant(x) => x.hit(r, ray_t),)*
                }
            }
            fn bounding_box(&self) -> AxisAlignedBoundingBox {
                match self {
                    $(AnyObject::$variant(x) => x.bounding_box(),)*
                }
            }
        }
        $(
            impl From<$variant> for AnyObject {
                fn from(value: $variant) -> Self {
                    Self::$variant(value)
                }
            }
        )*
    };
}

generate_any_object!(Sphere, Triangle, BvhNode, Quad);

#[derive(Default)]
pub struct ObjectList {
    objects: Vec<AnyObject>,
    aabb: AxisAlignedBoundingBox,
}

impl ObjectList {
    pub fn add(&mut self, o: impl Into<AnyObject>) {
        let o = o.into();
        self.aabb = self.aabb.merge(o.bounding_box());
        self.objects.push(o)
    }

    pub fn add_all(&mut self, o: impl IntoIterator<Item = impl Into<AnyObject>>) {
        o.into_iter().for_each(|v| self.add(v));
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
