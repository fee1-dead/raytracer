use std::io;

use camera::{Camera, CameraBuilder};
use object::{ObjectList, Sphere};
use vec3::Point;

mod camera;
mod color;
mod interval;
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
    });
    world.add(Sphere {
        center: Point::new(0.0, -100.5, -1.0),
        radius: 100.0,
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
