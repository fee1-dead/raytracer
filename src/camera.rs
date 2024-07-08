use std::io::{self, stdout};

use crate::color::Color;
use crate::interval::Interval;
use crate::object::{Object, ObjectList};
use crate::ray::Ray;
use crate::vec3::{vec3, Point, Vec3};

pub struct Camera {
    image_width: u64,
    image_height: u64,
    pixel00_loc: Point,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    center: Point,
}

impl Camera {
    pub fn new(aspect_ratio: f64, image_width: u64) -> Camera {
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
        } = self;
        println!("P3\n{image_width} {image_height}\n255");

        for j in 0..image_height {
            eprint!("\rScanlines remianing: {}", image_height - j);
            for i in 0..image_width {
                let pixel_center =
                    pixel00_loc + (pixel_delta_u * i as f64) + (pixel_delta_v * j as f64);
                let ray_direction = pixel_center - center;
                let ray = Ray {
                    origin: center,
                    direction: ray_direction,
                };
                let color = self.ray_color(ray, &world);
                color.write_to(&mut stdout())?;
            }
        }
        eprint!("\rDone.                             \n");
        Ok(())
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
