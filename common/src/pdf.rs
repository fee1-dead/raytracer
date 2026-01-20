use core::f32::consts::{FRAC_1_PI, PI};

use rand::Rng;
use rand::rngs::SmallRng;

use crate::object::Object;
use crate::onb::Onb;
use crate::vec3::{Point, Vec3};

pub trait Pdf {
    fn value(&self, direction: Vec3) -> f32;
    fn generate(&self, r: &mut SmallRng) -> Vec3;
}

pub enum AnyPdf {
    Sphere(SpherePdf),
    Cosine(CosinePdf),
}

impl Pdf for AnyPdf {
    fn value(&self, direction: Vec3) -> f32 {
        match self {
            Self::Cosine(c) => c.value(direction),
            Self::Sphere(s) => s.value(direction)
        }
    }
    fn generate(&self, r: &mut SmallRng) -> Vec3 {
        match self {
            Self::Cosine(c) => c.generate(r),
            Self::Sphere(s) => s.generate(r)
        }
    }
}

pub struct SpherePdf;

impl Pdf for SpherePdf {
    fn value(&self, _: Vec3) -> f32 {
        1. / (4. * PI)
    }
    fn generate(&self, r: &mut SmallRng) -> Vec3 {
        Vec3::random_unit_vector(r)
    }
}

pub struct CosinePdf(Onb);

impl CosinePdf {
    pub fn new(w: Vec3) -> Self {
        Self(Onb::new(w))
    }
}

impl Pdf for CosinePdf {
    fn value(&self, direction: Vec3) -> f32 {
        let cosine_theta = direction.unit_vector().dot(self.0.w());
        0.0f32.max(cosine_theta * FRAC_1_PI)
    }
    fn generate(&self, r: &mut SmallRng) -> Vec3 {
        self.0.transform(Point::random_cosine_direction(r))
    }
}

pub struct ObjectPdf<T> {
    object: T,
    origin: Point,
}

impl<T: Object> ObjectPdf<T> {
    pub fn new(object: T, origin: Point) -> Self {
        Self { object, origin }
    }
}

impl<T: Object> Pdf for ObjectPdf<T> {
    fn value(&self, direction: Vec3) -> f32 {
        self.object.pdf_value(self.origin, direction)
    }
    fn generate(&self, r: &mut SmallRng) -> Vec3 {
        self.object.random(r, self.origin)
    }
}

pub struct MixturePdf<A, B>(A, B);

impl<A: Pdf, B: Pdf> MixturePdf<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self(a, b)
    }
}
 
impl<A: Pdf, B: Pdf> Pdf for MixturePdf<A, B> {
    fn value(&self, direction: Vec3) -> f32 {
        0.5*self.0.value(direction) + 0.5*self.1.value(direction)
    }
    fn generate(&self, r: &mut SmallRng) -> Vec3 {
        if r.random_bool(0.5) {
            self.0.generate(r)
        } else {
            self.1.generate(r)
        }
    }
}



