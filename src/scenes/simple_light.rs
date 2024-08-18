use crate::camera::CameraBuilder;
use crate::color::Color;
use crate::material::{DiffuseLight, Lambertian};
use crate::object::{ObjectList, Quad, Sphere};
use crate::vec3::{Point, Vec3};

use super::Scene;

pub fn simple_light() -> Scene {
    let mut world = ObjectList::default();

    world.add(Sphere::new(
        Point::new(0.0, -1000.0, 0.0),
        1000.0,
        Lambertian::new(Color::new(0.9, 0.9, 0.9)),
    ));

    world.add(Sphere::new(
        Point::new(0.0, 2.0, 0.0),
        2.0,
        Lambertian::new(Color::new(0.9, 0.9, 0.9)),
    ));

    let difflight = DiffuseLight(Color::new(4.0, 4.0, 4.0));

    let light = Quad::new(
        Point::new(3.0, 1.0, -2.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(0.0, 2.0, 0.0),
        difflight,
    );
    world.add(light);

    let camera = CameraBuilder::new()
        .aspect_ratio(16.0 / 9.0)
        .image_width(400)
        .samples_per_pixel(100)
        .max_depth(50)
        .background(Color::new(0.0, 0.0, 0.0))
        .vfov(20.0)
        .look_from(Point::new(26.0, 3.0, 6.0))
        .look_at(Point::new(0.0, 2.0, 0.0))
        .vup(Vec3::new(0.0, 1.0, 0.0))
        .defocus_angle(0.0)
        .build();

    Scene { camera, world, light: Box::new(light) }
}
