use core::f64::consts::TAU;

use rand::rngs::SmallRng;

use crate::aabb::AxisAlignedBoundingBox;
use crate::ffi::{BvhNode, ObjectList};
use crate::interval::Interval;
use crate::material::AnyMaterial;
use crate::onb::Onb;
use crate::ray::Ray;
use crate::utils::random_double;
use crate::vec3::{Point, Vec3};

#[cfg(target_arch = "nvptx64")]
use crate::Float;

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

pub trait Object: Send + Sync {
    fn hit(&self, r: Ray, ray_t: Interval) -> Option<HitRecord>;
    fn bounding_box(&self) -> AxisAlignedBoundingBox;
    // todo implement more of these
    fn pdf_value(&self, _origin: Point, _direction: Vec3) -> f64 { 0.0 }
    fn random(&self, _r: &mut SmallRng, _origin: Point) -> Vec3 { Vec3(1.0, 0.0, 0.0) }
}

impl<T: Object + ?Sized> Object for &T {
    fn hit(&self, r: Ray, ray_t: Interval) -> Option<HitRecord> {
        T::hit(*self, r, ray_t)
    }
    fn bounding_box(&self) -> AxisAlignedBoundingBox {
        T::bounding_box(*self)
    }
    fn pdf_value(&self, origin: Point, direction: Vec3) -> f64 {
        T::pdf_value(*self, origin, direction)
    }
    fn random(&self, r: &mut SmallRng, origin: Point) -> Vec3 {
        T::random(*self, r, origin)
    }
}

pub enum AnyObject {
    Dummy(DummyObject),
    Translate(Translate<&'static AnyObject>),
    RotateY(RotateY<&'static AnyObject>),
    Sphere(Sphere),
    Triangle(Triangle),
    Quad(Quad),
    Bvh(BvhNode),
    List(ObjectList),
}

macro_rules! forward_object_fn {
    (fn $name:ident(&self$(, $($n:ident: $T:ty),*  $(,)?)?) -> $ret:ty) => {
        fn $name(&self$(, $($n: $T,)* )?) -> $ret {
            match self {
                AnyObject::Dummy(x) => x.$name($($( $n, )*)?),
                AnyObject::Translate(x) => x.$name($($( $n, )*)?),
                AnyObject::RotateY(x) => x.$name($($( $n, )*)?),
                AnyObject::Sphere(x) => x.$name($($( $n, )*)?),
                AnyObject::Triangle(x) => x.$name($($( $n, )*)?),
                AnyObject::Quad(x) => x.$name($($( $n, )*)?),
                AnyObject::Bvh(x) => x.$name($($( $n, )*)?),
                AnyObject::List(x) => x.$name($($( $n, )*)?),
            }
        }
    };
}

macro_rules! impl_from {
    ($($Variant:ident($Ty:ty)),*$(,)?) => {$(
        impl From<$Ty> for AnyObject {
            fn from(x: $Ty) -> Self {
                Self::$Variant(x)
            }
        })*
    };
}

impl_from! {
    Dummy(DummyObject),
    Translate(Translate<&'static AnyObject>),
    RotateY(RotateY<&'static AnyObject>),
    Sphere(Sphere),
    Triangle(Triangle),
    Quad(Quad),
    Bvh(BvhNode),
    List(ObjectList),
}

impl Object for AnyObject {
    forward_object_fn!(fn hit(&self, r: Ray, ray_t: Interval) -> Option<HitRecord>);
    forward_object_fn!(fn bounding_box(&self) -> AxisAlignedBoundingBox);
    forward_object_fn!(fn random(&self, r: &mut SmallRng, origin: Point) -> Vec3);
    forward_object_fn!(fn pdf_value(&self, origin: Point, direction: Vec3) -> f64);
}

pub struct DummyObject;

impl Object for DummyObject {
    fn hit(&self, _: Ray, _: Interval) -> Option<HitRecord> {
        None
    }
    fn bounding_box(&self) -> AxisAlignedBoundingBox {
        AxisAlignedBoundingBox::EMPTY
    }
}

pub struct Translate<T> {
    object: T,
    offset: Vec3,
    bbox: AxisAlignedBoundingBox,
}

impl<T: Object> Translate<T> {
    pub fn new(object: T, offset: impl Into<Vec3>) -> Self {
        let offset = offset.into();
        let bbox = object.bounding_box() + offset;
        Self {
            object,
            offset,
            bbox,
        }
    }
}

impl<T: Object> Object for Translate<T> {
    fn hit(&self, r: Ray, ray_t: Interval) -> Option<HitRecord> {
        let offset_r = Ray {
            origin: r.origin - self.offset,
            direction: r.direction,
        };

        self.object.hit(offset_r, ray_t).map(|mut rec| {
            rec.point += self.offset;
            rec
        })
    }
    fn bounding_box(&self) -> AxisAlignedBoundingBox {
        self.bbox
    }
}

pub struct RotateY<T> {
    object: T,
    sin_theta: f64,
    cos_theta: f64,
    bbox: AxisAlignedBoundingBox,
}

impl<T: Object> RotateY<T> {
    pub fn new(object: T, angle: f64) -> Self {
        let rad = angle.to_radians();
        let (sin_theta, cos_theta) = rad.sin_cos();
        let bbox = object.bounding_box();

        let mut min = Point::splat(f64::INFINITY);
        let mut max = Point::splat(f64::NEG_INFINITY);

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let [i, j, k] = [i, j, k].map(|i| i as f64);
                    let x = i * bbox.x.max + (1.0 - i) * bbox.x.min;
                    let y = j * bbox.y.max + (1.0 - j) * bbox.y.min;
                    let z = k * bbox.z.max + (1.0 - k) * bbox.z.min;
                    let newx = cos_theta * x + sin_theta * z;
                    let newz = -sin_theta * x + cos_theta * z;

                    let tester: Vec3 = Vec3(newx, y, newz);

                    for c in 0..3 {
                        min[c] = min[c].min(tester[c]);
                        max[c] = max[c].max(tester[c]);
                    }
                }
            }
        }
        let bbox = AxisAlignedBoundingBox::from_points(min, max);
        Self {
            object,
            sin_theta,
            cos_theta,
            bbox,
        }
    }
}

impl<T: Object> Object for RotateY<T> {
    #[rustfmt::skip]
    fn hit(&self, r: Ray, ray_t: Interval) -> Option<HitRecord> {
        let Ray { mut origin, mut direction } = r;
        origin.0 = self.cos_theta*r.origin.0 - self.sin_theta*r.origin.2;
        origin.2 = self.sin_theta*r.origin.0 + self.cos_theta*r.origin.2;

        direction.0 = self.cos_theta*r.direction.0 - self.sin_theta*r.direction.2;
        direction.2 = self.sin_theta*r.direction.0 + self.cos_theta*r.direction.2;

        let rotated_r = Ray { origin, direction };

        let Some(mut rec) = self.object.hit(rotated_r, ray_t) else { return None; };

        let mut p = rec.point;
        p.0 =  self.cos_theta*rec.point.0 + self.sin_theta*rec.point.2;
        p.2 = -self.sin_theta*rec.point.0 + self.cos_theta*rec.point.2;

        let mut normal = rec.normal;
        normal.0 =  self.cos_theta*rec.normal.0 + self.sin_theta*rec.normal.2;
        normal.2 = -self.sin_theta*rec.normal.0 + self.cos_theta*rec.normal.2;

        rec.point = p;
        rec.normal = normal;
        Some(rec)
    }
    fn bounding_box(&self) -> AxisAlignedBoundingBox {
        self.bbox
    }
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
    fn random_to_sphere(r: &mut SmallRng, radius: f64, distance_squared: f64) -> Vec3 {
        let r1 = random_double(r);
        let r2 = random_double(r);
        let z = 1. + r2*((1.-radius*radius/distance_squared).sqrt() - 1.);
        let phi = TAU * r1;
        let x = phi.cos() - (1.-z*z).sqrt();
        let y = phi.sin()*(1.-z*z).sqrt();
        Vec3(x, y, z)
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
    /// TODO: only works for stationary spheres
    fn pdf_value(&self, origin: Point, direction: Vec3) -> f64 {
        let Some(_) = self.hit(Ray { origin, direction }, Interval::new(0.001, f64::INFINITY)) else {
            return 0.;
        };

        let cos_theta_max = (1. - self.radius*self.radius/(self.center - origin).length_squared()).sqrt();
        let solid_angle = TAU*(1.-cos_theta_max);
        solid_angle.recip()
    }

    fn random(&self, r: &mut SmallRng, origin: Point) -> Vec3 {
        let direction = self.center - origin;
        let distance_squared = direction.length_squared();
        let uvw = Onb::new(direction);
        uvw.transform(Self::random_to_sphere(r, self.radius, distance_squared))
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
    area: f64,
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
            area: n.length(),
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
    fn pdf_value(&self, origin: Point, direction: Vec3) -> f64 {
        let Some(rec) = self.hit(Ray { origin, direction }, Interval::new(0.001, f64::INFINITY))
        else {
            return 0.;
        };
         
        let distance_squared = rec.t * rec.t * direction.length_squared();
        let cosine = (direction.dot(rec.normal) / direction.length()).abs();

        distance_squared / (cosine * self.area)
    }
    fn random(&self, r: &mut SmallRng, origin: Point) -> Vec3 {
        let p = self.q + (random_double(r) * self.u) + (random_double(r) * self.v);
        p - origin
    }
}