use std::io::{self, Write};

use crate::interval::Interval;
use crate::vec3::{Vec3, Vec3Token};

pub struct ColorToken;

impl Vec3Token for ColorToken {
    type Data = f64;
}

pub type Color = Vec3<ColorToken>;

fn linear_to_gamma(linear_component: f64) -> f64 {
    if linear_component > 0.0 {
        linear_component.sqrt()
    } else {
        0.0
    }
}

impl Color {
    pub fn write_to(self, out: &mut impl Write) -> io::Result<()> {
        let Vec3(r, g, b) = self;
        let intensity = Interval::new(0.000, 0.999);
        let [r, g, b] = [r, g, b]
            .map(linear_to_gamma)
            .map(|x| (256.0 * intensity.clamp(x)) as u64);
        writeln!(out, "{r} {g} {b}")
    }
}
