use image::ExtendedColorType;

use crate::color::Color;
use crate::interval::Interval;
use crate::material::Material;
use crate::object::{Object, ObjectList};
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
        Camera {
            image_width,
            image_height,
            pixel00_loc,
            pixel_delta_u,
            pixel_delta_v,
            samples_per_pixel,
            pixel_samples_scale: 1.0 / samples_per_pixel as f64,
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
    pixel00_loc: Point,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    defocus_angle: f64,
    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3,
    samples_per_pixel: u64,
    pixel_samples_scale: f64,
    max_depth: u64,
    center: Point,
}

impl Camera {
    pub fn num_pixels(&self) -> u64 {
        self.image_height * self.image_width
    }
    pub fn render(self, world: ObjectList) -> color_eyre::Result<()> {
        let Camera {
            image_width,
            image_height,
            pixel_samples_scale,
            ..
        } = self;

        let mut buffer = vec![0u8; (image_height * image_width * 3) as usize];
        
        for (j, buf) in (0..image_height).zip(buffer.chunks_exact_mut(3 * image_width as usize)) {
            eprint!("\rScanlines remaining: {}   ", image_height - j);
            for (i, buf) in (0..image_width).zip(buf.chunks_exact_mut(3)) {
                let mut pixel_color = Color::new(0.0, 0.0, 0.0);
                for _ in 0..self.samples_per_pixel {
                    let ray = self.get_ray(i, j);
                    pixel_color += self.ray_color(ray, self.max_depth, &world);
                }
                (pixel_samples_scale * pixel_color).write_to_buf(buf);
            }
        }
        image::save_buffer("image.png", &buffer, image_width as u32, image_height as u32, ExtendedColorType::Rgb8)?;
        eprint!("\rDone.                             \n");
        Ok(())
    }
    /// A ray originating from the defocus disk and directed at a random point around
    /// the pixel located at i, j.
    pub fn get_ray(&self, i: u64, j: u64) -> Ray {
        let (offset_x, offset_y) = Self::sample_square();
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
    pub fn defocus_disk_sample(&self) -> Point {
        let p = Point::random_in_unit_disk();
        self.center + (p.0 * self.defocus_disk_u) + (p.1 * self.defocus_disk_v)
    }
    pub fn ray_color(&self, r: Ray, depth: u64, world: &ObjectList) -> Color {
        if depth == 0 {
            return Color::new(0.0, 0.0, 0.0);
        }
        if let Some(record) = world.hit(r, Interval::new(0.001, f64::INFINITY)) {
            return if let Some((attenuation, scattered)) = record.material.scatter(&r, &record) {
                attenuation * self.ray_color(scattered, depth - 1, world)
            } else {
                Color::new(0.0, 0.0, 0.0)
            };
        }
        let unit = r.direction.unit_vector();
        let a = 0.5 * (unit.1 + 1.0);
        (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
    }
}
