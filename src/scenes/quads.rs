use crate::camera::CameraBuilder;
use crate::color::Color;
use crate::material::Lambertian;
use crate::object::{ObjectList, Quad};
use crate::vec3::{Point, Vec3};

use super::Scene;

pub fn quads() -> Scene {
    let mut world = ObjectList::default();
    let left_red = Lambertian::new((1.0, 0.2, 0.2));
    let back_green = Lambertian::new((0.2, 1.0, 0.2));
    let right_blue = Lambertian::new((0.2, 0.2, 1.0));
    let upper_orange = Lambertian::new((1.0, 0.5, 0.0));
    let lower_teal = Lambertian::new((0.2, 0.8, 0.8));

    world.add(Quad::new(
        Point::new(-3.0, -2.0, 5.0),
        Vec3::new(0.0, 0.0, -4.0),
        Vec3::new(0.0, 4.0, 0.0),
        left_red,
    ));

    world.add(Quad::new(
        Point::new(-2.0, -2.0, 0.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 4.0, 0.0),
        back_green,
    ));
    world.add(Quad::new(
        Point::new(3.0, -2.0, 1.0),
        Vec3::new(0.0, 0.0, 4.0),
        Vec3::new(0.0, 4.0, 0.0),
        right_blue,
    ));
    world.add(Quad::new(
        Point::new(-2.0, 3.0, 1.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 4.0),
        upper_orange,
    ));
    world.add(Quad::new(
        Point::new(-2.0, -3.0, 5.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -4.0),
        lower_teal,
    ));

    let camera = CameraBuilder::new()
        .aspect_ratio(1.0)
        .image_width(400)
        .samples_per_pixel(100)
        .max_depth(50)
        .vfov(80.0)
        .look_from(Point::new(0.0, 0.0, 9.0))
        .look_at(Point::new(0.0, 0.0, 0.0))
        .vup(Vec3::new(0.0, 1.0, 0.0))
        .defocus_angle(0.0)
        .background(Color::new(0.7, 0.8, 1.0))
        .build();

    Scene { camera, world }
}
