use std::io;

use camera::Camera;
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
    let camera = Camera::new(16.0 / 9.0, 400, 100);
    camera.render(world)?;

    Ok(())
}
