use std::f64::consts::PI;
use std::io;

use camera::CameraBuilder;
use color::Color;
use material::{Dielectric, Lambertian, Metal};
use object::{ObjectList, Sphere, Triangle};
use vec3::{Point, Vec3};

mod camera;
mod color;
mod interval;
mod material;
mod object;
mod ray;
mod utils;
pub(crate) mod vec3;

fn main() -> io::Result<()> {
    // world
    let mut world = ObjectList::default();

    // ground
    world.add(Sphere {
        center: Point::new(0.0, -100.5, -1.0),
        radius: 100.0,
        material: Lambertian {
            albedo: Color::new(0.8, 0.8, 0.0),
        }
        .into(),
    });

    world.add(Triangle {
        a: Point::new(0.0, 0.0, -1.0),
        b: Point::new(1.0, 2.0, -1.0),
        c: Point::new(2.0, 2.0, -1.0),
        material: Lambertian {
            albedo: Color::new(0.0, 1.0, 0.0),
        }.into(),
    });

    // camera
    let camera = CameraBuilder::new()
        .aspect_ratio(16.0 / 9.0)
        .image_width(400)
        .samples_per_pixel(100)
        .max_depth(50)
        .vup(Vec3::new(0.0, 1.0, 0.0))
        .vfov(100.0)
        .build();
    camera.render(world)?;

    Ok(())
}
