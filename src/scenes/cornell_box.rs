use crate::camera::CameraBuilder;
use crate::color::Color;
use crate::material::{DiffuseLight, Lambertian};
use crate::object::{box_3d, ObjectList};
use common::object::*;
use crate::vec3::{Point, Vec3};

use super::Scene;

pub fn cornell_box() -> Scene<Quad> {
    let mut world = ObjectList::default();

    let red = Lambertian::new((0.65, 0.05, 0.05));
    let white = Lambertian::new((0.73, 0.73, 0.73));
    let green = Lambertian::new((0.12, 0.45, 0.15));
    let light = DiffuseLight(Color::new(15.0, 15.0, 15.0));

    world.add(Quad::new(
        Point::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        green,
    ));
    world.add(Quad::new(
        Point::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        red,
    ));
    world.add(Quad::new(
        Point::new(0.0, 0.0, 0.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        white,
    ));
    world.add(Quad::new(
        Point::new(555.0, 555.0, 555.0),
        Vec3::new(-555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -555.0),
        white,
    ));
    world.add(Quad::new(
        Point::new(0.0, 0.0, 555.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        white,
    ));
    let box1 = box_3d(
        Point::new(0.0, 0.0, 0.0),
        Point::new(165.0, 330.0, 165.0),
        white,
    ).finalize();
    let box1 = RotateY::new(crate::finalize(box1.into()), 15.0);
    let box1 = Translate::new(crate::finalize(box1.into()), Vec3(265.0, 0.0, 295.0));
    world.add(box1);

    let box2 = box_3d(
        Point::new(0.0, 0.0, 0.0),
        Point::new(165.0, 165.0, 165.0),
        white,
    ).finalize();
    let box2 = RotateY::new(crate::finalize(box2.into()), -18.0);
    let box2 = Translate::new(crate::finalize(box2.into()), Vec3(130.0, 0.0, 65.0));
    world.add(box2);

    let light = Quad::new(
        Point::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        light,
    );

    world.add(light);

    let camera = CameraBuilder::new()
        .aspect_ratio(1.0)
        .image_width(5000)
        .samples_per_pixel(1024)
        .max_depth(50)
        .background(Color::new(0.0, 0.0, 0.0))
        .vfov(40.0)
        .look_from(Point::new(278.0, 278.0, -800.0))
        .look_at(Point::new(278.0, 278.0, 0.0))
        .vup(Vec3(0.0, 1.0, 0.0))
        .defocus_angle(0.0)
        .build();

    Scene { camera, world, light }
}
