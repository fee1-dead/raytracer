use crate::color::Color;
use crate::object::HitRecord;
use crate::ray::Ray;
use crate::utils::random_double;
use crate::vec3::Vec3;

pub trait Material {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)>;
}

#[derive(Clone, Copy)]
pub enum AnyMaterial {
    Lambertian(Lambertian),
    Metal(Metal),
    Dielectric(Dielectric),
}

macro_rules! impl_from {
    ($x:ident) => {
        impl From<$x> for AnyMaterial {
            fn from(x: $x) -> Self {
                Self::$x(x)
            }
        }
    };
}

impl_from!(Lambertian);
impl_from!(Metal);
impl_from!(Dielectric);

impl Material for AnyMaterial {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        match self {
            Self::Lambertian(l) => l.scatter(r_in, rec),
            Self::Metal(m) => m.scatter(r_in, rec),
            Self::Dielectric(d) => d.scatter(r_in, rec),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Lambertian {
    pub albedo: Color,
}

impl Material for Lambertian {
    fn scatter(&self, _: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let mut scatter_direction = rec.normal + Vec3::random_unit_vector();
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        }
        Some((
            self.albedo,
            Ray {
                origin: rec.point,
                direction: scatter_direction,
            },
        ))
    }
}

#[derive(Clone, Copy)]
pub struct Metal {
    albedo: Color,
    fuzziness: f64,
}

impl Metal {
    pub fn new(albedo: Color, fuzziness: f64) -> Self {
        Metal {
            albedo,
            fuzziness: fuzziness.min(1.0),
        }
    }
}

impl Material for Metal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let reflected = r_in.direction.reflect(rec.normal);
        let reflected = reflected.unit_vector() + self.fuzziness * Vec3::random_unit_vector();

        if reflected.dot(rec.normal) > 0.0 {
            let scattered = Ray {
                origin: rec.point,
                direction: reflected,
            };
            Some((self.albedo, scattered))
        } else {
            None
        }
    }
}

#[derive(Clone, Copy)]
pub struct Dielectric {
    refraction_index: f64,
}

impl Dielectric {
    pub fn new(refraction_index: f64) -> Self {
        Self { refraction_index }
    }

    fn reflectance(refraction_index: f64, cosine: f64) -> f64 {
        let r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
        let r0 = r0 * r0;
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Material for Dielectric {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let ri = if rec.front_face {
            self.refraction_index.recip()
        } else {
            self.refraction_index
        };

        let unit_direction = r_in.direction.unit_vector();
        let cos_theta = (-unit_direction).dot(rec.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refact = ri * sin_theta > 1.0;

        let direction = if cannot_refact || Self::reflectance(ri, cos_theta) > random_double() {
            unit_direction.reflect(rec.normal)
        } else {
            unit_direction.refract(rec.normal, ri)
        };

        let ray = Ray {
            origin: rec.point,
            direction,
        };
        Some((Color::new(1.0, 1.0, 1.0), ray))
    }
}
