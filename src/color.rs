use std::io::{self, Write};

use crate::interval::Interval;
use crate::vec3::{Vec3, Vec3Token};

pub struct ColorToken;

impl Vec3Token for ColorToken {
    type Data = f64;
}

pub type Color = Vec3<ColorToken>;

impl Color {
    pub fn write_to(self, out: &mut impl Write) -> io::Result<()> {
        let Vec3(r, g, b) = self;
        let intensity = Interval::new(0.000, 0.999);
        let rbyte = (256.0 * intensity.clamp(r)) as u64;
        let gbyte = (256.0 * intensity.clamp(g)) as u64;
        let bbyte = (256.0 * intensity.clamp(b)) as u64;
        writeln!(out, "{rbyte} {gbyte} {bbyte}")
    }
}
