use std::f64::consts::{FRAC_1_PI, PI};

use rand::random;

use crate::object::Object;
use crate::onb::Onb;
use crate::vec3::{Point, Vec3};

pub trait Pdf {
    fn value(&self, direction: Vec3) -> f64;
    fn generate(&self) -> Vec3;
}

impl<T: Pdf + ?Sized> Pdf for Box<T> {
    fn value(&self, direction: Vec3) -> f64 {
        T::value(&**self, direction)
    }
    fn generate(&self) -> Vec3 {
        T::generate(&**self)
    }
}

pub struct SpherePdf;

impl Pdf for SpherePdf {
    fn value(&self, _: Vec3) -> f64 {
        1. / (4. * PI)
    }
    fn generate(&self) -> Vec3 {
        Vec3::random_unit_vector()
    }
}

pub struct CosinePdf(Onb);

impl CosinePdf {
    pub fn new(w: Vec3) -> Self {
        Self(Onb::new(w))
    }
}

impl Pdf for CosinePdf {
    fn value(&self, direction: Vec3) -> f64 {
        let cosine_theta = direction.unit_vector().dot(self.0.w());
        0.0f64.max(cosine_theta * FRAC_1_PI)
    }
    fn generate(&self) -> Vec3 {
        self.0.transform(Point::random_cosine_direction())
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
    fn value(&self, direction: Vec3) -> f64 {
        self.object.pdf_value(self.origin, direction)
    }
    fn generate(&self) -> Vec3 {
        self.object.random(self.origin)
    }
}

pub struct MixturePdf<A, B>(A, B);

impl<A: Pdf, B: Pdf> MixturePdf<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self(a, b)
    }
}
 
impl<A: Pdf, B: Pdf> Pdf for MixturePdf<A, B> {
    fn value(&self, direction: Vec3) -> f64 {
        0.5*self.0.value(direction) + 0.5*self.1.value(direction)
    }
    fn generate(&self) -> Vec3 {
        if random() {
            self.0.generate()
        } else {
            self.1.generate()
        }
    }
}



