use std::io;

use camera::{Camera, CameraBuilder};
use color::Color;
use material::{Dielectric, Lambertian, Metal};
use object::{ObjectList, Sphere};
use vec3::Point;

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
    world.add(Sphere {
        center: Point::new(0.0, 0.0, -1.0),
        radius: 0.5,
        material: Lambertian { albedo: Color::new(0.1, 0.2, 0.5) }.into()
    });
    world.add(Sphere {
        center: Point::new(0.0, -100.5, -1.0),
        radius: 100.0,
        material: Lambertian { albedo: Color::new(0.8, 0.8, 0.0) }.into(),
    });
    world.add(Sphere {
        center: Point::new(-1.0, 0.0, -1.0),
        radius: 0.5,
        material: Metal::new(Color::new(0.8, 0.8, 0.8), 0.3).into(),
    });
    world.add(Sphere {
        center: Point::new(1.0, 0.0, -1.0),
        radius: 0.5,
        material: Dielectric::new(1.00 / 1.33).into(),
    });

    // camera
    let camera = CameraBuilder::new()
        .aspect_ratio(16.0 / 9.0)
        .image_width(400)
        .samples_per_pixel(100)
        .max_depth(50)
        .build();
    camera.render(world)?;

    Ok(())
}
