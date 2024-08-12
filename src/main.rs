use std::io;
use std::time::{Duration, Instant};

use camera::CameraBuilder;
use color::Color;
use material::Lambertian;
use object::{ObjectList, Sphere, Triangle};
use vec3::{Point, Vec3};

pub mod camera;
mod color;
mod interval;
pub mod material;
pub mod object;
mod ray;
mod utils;
pub(crate) mod vec3;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    // world
    let mut world = ObjectList::default();

    // ground
    world.add(Sphere {
        center: Point::new(0.0, -100.5, -1.0),
        radius: 100.0,
        material: Lambertian::new(Color::new(0.8, 0.8, 0.0)).into(),
    });

    world.add(Triangle {
        a: Point::new(0.0, 0.0, -1.0),
        b: Point::new(0.5, 0.5, -1.0),
        c: Point::new(-0.5, 0.5, -1.0),
        material: Lambertian::new(Color::new(0.0, 1.0, 0.0)).into(),
    });

    let samples_per_pixel = 100;
    // camera
    let camera = CameraBuilder::new()
        .aspect_ratio(16.0 / 9.0)
        .image_width(400)
        .samples_per_pixel(samples_per_pixel)
        .max_depth(50)
        .vup(Vec3::new(0.0, 1.0, 0.0))
        .vfov(100.0)
        .build();
    let pixels = camera.num_pixels() as u32;
    let time = Instant::now();
    camera.render(world)?;
    let elapsed = time.elapsed();
    eprintln!("Elapsed: {elapsed:?}, {}, {}, {}", time_per(elapsed, "frame"), time_per(elapsed / pixels, "pixel"), time_per(elapsed / pixels / samples_per_pixel as u32, "sample"));

    Ok(())
}

fn time_per(time: Duration, desc: &str) -> String {
    if time <= Duration::from_secs(1) {
        format!("{desc}s per second: {}", 1.0/time.as_secs_f64())
    } else {
        format!("time per {desc}: {time:?}")
    }
}
