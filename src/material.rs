use std::f64::consts::FRAC_1_PI;

use crate::color::Color;
use crate::object::HitRecord;
use crate::pdf::{CosinePdf, Pdf, SpherePdf};
use crate::ray::Ray;
use crate::utils::random_double;
use crate::vec3::{Point, Vec3};

pub struct ScatterRecord {
    pub attenuation: Color,
    /// TODO: i'd like this to be an enum
    pub pdf: Box<dyn Pdf>,
    /// If this is `Some`, skip pdf and use this ray instead.
    pub skip_pdf: Option<Ray>,
}

pub trait Material {
    #[expect(unused_variables)]
    // TODO: no uv because we skipped textures
    fn emitted(&self, r_in: &Ray, rec: &HitRecord, p: Point) -> Color {
        Color::default()
    }
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord>;
    #[expect(unused_variables)]
    fn scattering_pdf(&self, r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        0.0
    }
}

macro_rules! generate_any_material {
    ($($x:ident),*$(,)?) => {
        #[derive(Clone, Copy)]
        pub enum AnyMaterial {
            $($x($x),)*
        }

        impl Material for AnyMaterial {
            fn emitted(&self, r_in: &Ray, rec: &HitRecord, p: Point) -> Color {
                match self {
                    $(Self::$x(v) => v.emitted(r_in, rec, p),)*
                }
            }
            fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
                match self {
                    $(Self::$x(v) => v.scatter(r_in, rec),)*
                }
            }
            fn scattering_pdf(&self, r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
                match self {
                    $(Self::$x(v) => v.scattering_pdf(r_in, rec, scattered),)*
                }
            }
        }

        $(impl From<$x> for AnyMaterial {
            fn from(x: $x) -> Self {
                Self::$x(x)
            }
        })*
    };
}

generate_any_material!(Lambertian, Metal, Dielectric, DiffuseLight, DummyMaterial);

#[derive(Clone, Copy)]
pub struct DummyMaterial;

impl Material for DummyMaterial {
    fn scatter(&self, _: &Ray, _: &HitRecord) -> Option<ScatterRecord> {
        None
    }
}

#[derive(Clone, Copy)]
pub struct Lambertian {
    pub albedo: Color,
}

impl Lambertian {
    pub fn new(albedo: impl Into<Color>) -> Self {
        Lambertian {
            albedo: albedo.into(),
        }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        Some(ScatterRecord {
            attenuation: self.albedo,
            pdf: Box::new(CosinePdf::new(rec.normal)),
            skip_pdf: None,
        })
    }
    fn scattering_pdf(&self, _r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        let cos_theta = rec.normal.dot(scattered.direction.unit_vector());
        cos_theta.max(0.0) * FRAC_1_PI
    }
}

#[derive(Clone, Copy)]
pub struct Metal {
    albedo: Color,
    fuzziness: f64,
}

impl Metal {
    pub fn new(albedo: impl Into<Color>, fuzziness: f64) -> Self {
        Metal {
            albedo: albedo.into(),
            fuzziness: fuzziness.min(1.0),
        }
    }
}

impl Material for Metal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        let reflected = r_in.direction.reflect(rec.normal);
        let reflected = reflected.unit_vector() + self.fuzziness * Vec3::random_unit_vector();

        if reflected.dot(rec.normal) > 0.0 {
            let scattered = Ray {
                origin: rec.point,
                direction: reflected,
            };
            
            Some(ScatterRecord {
                attenuation: self.albedo,
                pdf: Box::new(SpherePdf),
                skip_pdf: Some(scattered),
            })
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
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
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
        Some(ScatterRecord {
            attenuation: Color::splat(1.),
            pdf: Box::new(SpherePdf),
            skip_pdf: Some(ray),
        })
    }
}

#[derive(Clone, Copy)]
pub struct DiffuseLight(pub Color);

impl Material for DiffuseLight {
    fn emitted(&self, _r_in: &Ray, rec: &HitRecord, _: Point) -> Color {
        if !rec.front_face {
            Color::splat(0.)
        } else {
            self.0
        }
    }
    fn scatter(&self, _: &Ray, _: &HitRecord) -> Option<ScatterRecord> {
        None
    }
}
