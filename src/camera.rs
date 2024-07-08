use std::io::{self, stdout};

use crate::color::Color;
use crate::interval::Interval;
use crate::object::{Object, ObjectList};
use crate::ray::Ray;
use crate::utils::random_double;
use crate::vec3::{vec3, Point, Vec3};

pub struct Camera {
    image_width: u64,
    image_height: u64,
    pixel00_loc: Point,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    samples_per_pixel: u64,
    pixel_samples_scale: f64,
    center: Point,
}

impl Camera {
    pub fn new(aspect_ratio: f64, image_width: u64, samples_per_pixel: u64) -> Camera {
        let image_height = (image_width as f64 / aspect_ratio) as u64;
        let image_height = image_height.max(1);
        let focal_length = 1.0;
        let viewport_height = 2.0;
        let viewport_width = viewport_height * (image_width as f64 / image_height as f64);
        let center = Point::new(0.0, 0.0, 0.0);

        // vector across the horizontal and vertical edges of the viewport
        let viewport_u = vec3(viewport_width, 0.0, 0.0);
        let viewport_v = vec3(0.0, -viewport_height, 0.0);

        // horizontal and vertical delta vectors per pixel
        let pixel_delta_u = viewport_u / image_width as f64;
        let pixel_delta_v = viewport_v / image_height as f64;

        // location of the upper left pixel
        let viewport_upper_left =
            center - vec3(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
        let pixel00_loc = viewport_upper_left + (pixel_delta_u + pixel_delta_v) * 0.5;
        Camera {
            image_width,
            image_height,
            pixel00_loc,
            pixel_delta_u,
            pixel_delta_v,
            samples_per_pixel,
            pixel_samples_scale: 1.0 / samples_per_pixel as f64,
            center,
        }
    }
    pub fn render(self, world: ObjectList) -> io::Result<()> {
        let Camera {
            image_width,
            image_height,
            pixel00_loc,
            center,
            pixel_delta_u,
            pixel_delta_v,
            samples_per_pixel,
            pixel_samples_scale,
        } = self;
        println!("P3\n{image_width} {image_height}\n255");

        for j in 0..image_height {
            eprint!("\rScanlines remianing: {}", image_height - j);
            for i in 0..image_width {
                let mut pixel_color = Color::new(0.0, 0.0, 0.0);
                for _ in 0..self.samples_per_pixel {
                    let ray = self.get_ray(i, j);
                    pixel_color += self.ray_color(ray, &world);
                }
                (pixel_samples_scale * pixel_color).write_to(&mut stdout())?;
            }
        }
        eprint!("\rDone.                             \n");
        Ok(())
    }
    /// A ray originating from the origin and directed at a random point around
    /// the pixel located at i, j.
    pub fn get_ray(&self, i: u64, j: u64) -> Ray {
        let (offset_x, offset_y) = Self::sample_square();
        let pixel_sample = self.pixel00_loc
            + ((i as f64 + offset_x) * self.pixel_delta_u)
            + ((j as f64 + offset_y) * self.pixel_delta_v);
        let origin = self.center;
        let direction = pixel_sample - origin;
        Ray {
            origin,
            direction,
        }
    }
    /// vector to a random point in the square from (-0.5, -0.5) to (0.5, 0.5)
    pub fn sample_square() -> (f64, f64) {
        (random_double() - 0.5, random_double() - 0.5)
    }
    pub fn ray_color(&self, r: Ray, world: &ObjectList) -> Color {
        if let Some(record) = world.hit(r, Interval::new(0.0, f64::INFINITY)) {
            return 0.5
                * Color::new(
                    record.normal.0 + 1.0,
                    record.normal.1 + 1.0,
                    record.normal.2 + 1.0,
                );
        }
        let unit = r.direction.unit_vector();
        let a = 0.5 * (unit.1 + 1.0);
        (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
    }
}