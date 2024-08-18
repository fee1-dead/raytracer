use image::ExtendedColorType;
use rayon::iter::{IndexedParallelIterator, ParallelIterator};
use rayon::slice::ParallelSliceMut;

use crate::color::Color;
use crate::interval::Interval;
use crate::material::Material;
use crate::object::{Object, ObjectList};
use crate::pdf::{MixturePdf, ObjectPdf, Pdf};
use crate::ray::Ray;
use crate::utils::random_double;
use crate::vec3::{Point, Vec3};

pub struct CameraBuilder {
    aspect_ratio: f64,
    image_width: u64,
    samples_per_pixel: u64,
    max_depth: u64,
    /// Vertical view angle
    vfov: f64,
    look_from: Point,
    look_at: Point,
    vup: Vec3,
    defocus_angle: f64,
    focus_dist: f64,
    background: Color,
}

macro_rules! builder_methods {
    ($($name:ident: $ty:ty),*$(,)?) => {
        $(
            pub fn $name(&mut self, value: $ty) -> &mut Self {
                self.$name = value;
                self
            }
        )*
    };
}

impl CameraBuilder {
    pub const fn new() -> Self {
        Self {
            aspect_ratio: 1.0,
            image_width: 100,
            samples_per_pixel: 10,
            max_depth: 10,
            vfov: 90.0,
            look_from: Point::new(0.0, 0.0, 0.0),
            look_at: Point::new(0.0, 0.0, -1.0),
            vup: Vec3::new(0.0, 1.0, 0.0),
            defocus_angle: 0.0,
            focus_dist: 10.0,
            background: Color::new(0.0, 0.0, 0.0),
        }
    }
    builder_methods!(
        aspect_ratio: f64,
        image_width: u64,
        samples_per_pixel: u64,
        max_depth: u64,
        vfov: f64,
        look_from: Point,
        look_at: Point,
        vup: Vec3,
        defocus_angle: f64,
        focus_dist: f64,
        background: Color,
    );
    pub fn build(&self) -> Camera {
        let Self {
            image_width,
            aspect_ratio,
            samples_per_pixel,
            max_depth,
            vfov,
            look_from,
            look_at,
            vup,
            defocus_angle,
            focus_dist,
            background,
        } = *self;
        let image_height = (image_width as f64 / aspect_ratio) as u64;
        let image_height = image_height.max(1);

        let theta = vfov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * focus_dist;
        let viewport_width = viewport_height * (image_width as f64 / image_height as f64);
        let center = look_from;

        // Calculate the u,v,w unit basis vectors for the camera coordinate frame.
        let w = (look_from - look_at).unit_vector();
        let u = vup.cross(w).unit_vector();
        let v = w.cross(u);

        // vector across the horizontal and vertical edges of the viewport
        let viewport_u = viewport_width * u;
        let viewport_v = viewport_height * -v;

        // horizontal and vertical delta vectors per pixel
        let pixel_delta_u = viewport_u / image_width as f64;
        let pixel_delta_v = viewport_v / image_height as f64;

        // location of the upper left pixel
        let viewport_upper_left = center - (focus_dist * w) - viewport_u / 2.0 - viewport_v / 2.0;
        let pixel00_loc = viewport_upper_left + (pixel_delta_u + pixel_delta_v) * 0.5;

        let defocus_radius = focus_dist * (defocus_angle / 2.0).to_radians().tan();
        let sqrt_spp = (samples_per_pixel as f64).sqrt() as u64;
        let pixel_samples_scale = 1.0 / (sqrt_spp as f64 * sqrt_spp as f64);
        let recip_sqrt_spp = 1.0 / sqrt_spp as f64;
        Camera {
            image_width,
            image_height,
            background,
            pixel00_loc,
            pixel_delta_u,
            pixel_delta_v,
            sqrt_spp,
            recip_sqrt_spp,
            pixel_samples_scale,
            max_depth,
            center,
            defocus_angle,
            defocus_disk_u: defocus_radius * u,
            defocus_disk_v: defocus_radius * v,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Camera {
    image_width: u64,
    image_height: u64,
    background: Color,
    pixel00_loc: Point,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    defocus_angle: f64,
    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3,
    pixel_samples_scale: f64,
    /// Square root of # of samples per pixel
    sqrt_spp: u64,
    /// 1 / sqrt_spp
    recip_sqrt_spp: f64,
    max_depth: u64,
    center: Point,
}

impl Camera {
    pub fn num_pixels(&self) -> u64 {
        self.image_height * self.image_width
    }
    pub fn render(self, world: ObjectList, lights: &dyn Object) -> color_eyre::Result<()> {
        let Camera {
            image_width,
            image_height,
            pixel_samples_scale,
            ..
        } = self;

        let mut buffer = vec![0u8; (image_height * image_width * 3) as usize];

        buffer
            .par_chunks_exact_mut(3 * image_width as usize)
            .enumerate()
            .for_each(|(j, buf)| {
                buf.par_chunks_exact_mut(3)
                    .enumerate()
                    .for_each(|(i, buf)| {
                        let mut pixel_color = Color::new(0.0, 0.0, 0.0);
                        for s_i in 0..self.sqrt_spp {
                            for s_j in 0..self.sqrt_spp {
                                let ray = self.get_ray(i as u64, j as u64, s_i, s_j);
                                pixel_color += self.ray_color(ray, self.max_depth, &world, &lights);
                                pixel_color.assert_finite();
                            }
                        }
                        (pixel_samples_scale * pixel_color).write_to_buf(buf);
                    })
            });
        image::save_buffer(
            "image.png",
            &buffer,
            image_width as u32,
            image_height as u32,
            ExtendedColorType::Rgb8,
        )?;
        Ok(())
    }
    /// A ray originating from the defocus disk and directed at a random point around
    /// the pixel located at i, j for stratified sample square s_i, s_j.
    pub fn get_ray(&self, i: u64, j: u64, s_i: u64, s_j: u64) -> Ray {
        let (offset_x, offset_y) = self.sample_square_stratified(s_i, s_j);

        let pixel_sample = self.pixel00_loc
            + ((i as f64 + offset_x) * self.pixel_delta_u)
            + ((j as f64 + offset_y) * self.pixel_delta_v);
        let origin = if self.defocus_angle <= 0.0 {
            self.center
        } else {
            self.defocus_disk_sample()
        };
        let direction = pixel_sample - origin;
        Ray { origin, direction }
    }
    /// vector to a random point in the square from (-0.5, -0.5) to (0.5, 0.5)
    pub fn sample_square() -> (f64, f64) {
        (random_double() - 0.5, random_double() - 0.5)
    }
    /// Returns the vector to a random point in the square sub-pixel specified by grid
    /// indices s_i and s_j for an idealized unit square pixel [-.5,-.5] to [+.5,+.5].
    pub fn sample_square_stratified(&self, s_i: u64, s_j: u64) -> (f64, f64) {
        let px = ((s_i as f64 + random_double()) * self.recip_sqrt_spp as f64) - 0.5;
        let py = ((s_j as f64 + random_double()) * self.recip_sqrt_spp as f64) - 0.5;
        (px, py)
    }
    pub fn defocus_disk_sample(&self) -> Point {
        let p = Point::random_in_unit_disk();
        self.center + (p.0 * self.defocus_disk_u) + (p.1 * self.defocus_disk_v)
    }
    // todo condense params
    pub fn ray_color(&self, r: Ray, depth: u64, world: &ObjectList, lights: &dyn Object) -> Color {
        if depth == 0 {
            return Color::new(0.0, 0.0, 0.0);
        }
        if let Some(record) = world.hit(r, Interval::new(0.001, f64::INFINITY)) {
            let color_from_emission = record.material.emitted(&r, &record, record.point);
            color_from_emission.assert_finite();
            let Some(srec) = record.material.scatter(&r, &record) else {
                return color_from_emission;
            };

            if let Some(ray) = srec.skip_pdf {
                return srec.attenuation * self.ray_color(ray, depth-1, world, lights)
            }

            let light_pdf = ObjectPdf::new(lights, record.point);
            let mixed = MixturePdf::new(light_pdf, srec.pdf);
        
            let scattered = Ray { origin: record.point, direction: mixed.generate() };
            let pdf_value = mixed.value(scattered.direction);


            /*let surface_pdf = CosinePdf::new(record.normal);
            let scattered = Ray {
                origin: record.point,
                direction: surface_pdf.generate(),
            };
            let pdf_value = surface_pdf.value(scattered.direction);*/
            let scattering_pdf = record.material.scattering_pdf(&r, &record, &scattered);
            // let pdf_value = scattering_pdf;

            let sample_color = self.ray_color(scattered, depth-1, world, lights);

            let color_from_scatter =
                srec.attenuation * scattering_pdf * sample_color
                    / pdf_value;
            color_from_scatter.assert_finite();

            color_from_emission + color_from_scatter
        } else {
            self.background
        }
    }
}
