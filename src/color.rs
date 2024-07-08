use std::io::{self, Write};

use crate::vec3::{Vec3, Vec3Token};

pub struct ColorToken;

impl Vec3Token for ColorToken {
    type Data = f64;
}

pub type Color = Vec3<ColorToken>;

impl Color {
    pub fn write_to(self, out: &mut impl Write) -> io::Result<()> {
        let Vec3(r, g, b) = self;
        let rbyte = (255.9999 * r) as u64;
        let gbyte = (255.9999 * g) as u64;
        let bbyte = (255.9999 * b) as u64;
        writeln!(out, "{rbyte} {gbyte} {bbyte}")
    }
}
