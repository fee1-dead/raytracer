use std::time::{Duration, Instant};

use camera::CameraBuilder;
use color::Color;
use material::{AnyMaterial, Dielectric, Lambertian, Metal};
use object::{ObjectList, Sphere};
use utils::{random_double, random_double_in};
use vec3::{Point, Vec3};

pub mod aabb;
pub mod bvh;
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

    let ground_material = Lambertian::new((0.5, 0.5, 0.5));
    world.add(Sphere::new(
        Point::new(0.0, -1000.0, 0.0),
        1000.0,
        ground_material,
    ));

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = random_double();
            let center = Point::new(
                a as f64 + 0.9 * random_double(),
                0.2,
                b as f64 + 0.9 * random_double(),
            );

            if (center - Point::new(4.0, 0.2, 0.0)).length() > 0.9 {
                let material = if choose_mat < 0.8 {
                    let albedo = Color::random() * Color::random();
                    AnyMaterial::from(Lambertian::new(albedo))
                } else if choose_mat < 0.95 {
                    let albedo = Color::random_in(0.5, 1.0);
                    let fuzziness = random_double_in(0.0, 0.5);
                    Metal::new(albedo, fuzziness).into()
                } else {
                    Dielectric::new(1.5).into()
                };

                world.add(Sphere::new(center, 0.2, material));
            }
        }
    }

    world.add(Sphere::new(
        Point::new(0.0, 1.0, 0.0),
        1.0,
        Dielectric::new(1.5),
    ));
    world.add(Sphere::new(
        Point::new(-4.0, 1.0, 0.0),
        1.0,
        Lambertian::new(Color::new(0.4, 0.2, 0.1)),
    ));
    world.add(Sphere::new(
        Point::new(4.0, 1.0, 0.0),
        1.0,
        Metal::new(Color::new(0.7, 0.6, 0.5), 0.0),
    ));
    world.condense();

    let samples_per_pixel = 500;
    // camera
    let camera = CameraBuilder::new()
        .aspect_ratio(16.0 / 9.0)
        .image_width(1200)
        .samples_per_pixel(samples_per_pixel)
        .max_depth(50)
        .vup(Vec3::new(0.0, 1.0, 0.0))
        .vfov(20.0)
        .look_from(Point::new(13.0, 2.0, 3.0))
        .look_at(Point::new(0.0, 0.0, 0.0))
        .defocus_angle(0.6)
        .focus_dist(10.0)
        .build();
    let pixels = camera.num_pixels() as u32;
    let time = Instant::now();
    camera.render(world)?;
    let elapsed = time.elapsed();
    eprintln!(
        "Done! Elapsed: {elapsed:?}, {}, {}, {}",
        time_per(elapsed, "frame"),
        time_per(elapsed / pixels, "pixel"),
        time_per(elapsed / pixels / samples_per_pixel as u32, "sample")
    );

    Ok(())
}

fn time_per(time: Duration, desc: &str) -> String {
    if time <= Duration::from_secs(1) {
        format!("{desc}s per second: {}", 1.0 / time.as_secs_f64())
    } else {
        format!("time per {desc}: {time:?}")
    }
}
