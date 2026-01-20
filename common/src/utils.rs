use rand::Rng;
use rand::rngs::SmallRng;


/// Returns a random double in [0,1).
pub fn random_double(r: &mut SmallRng) -> f32 {
    r.random_range(0.0 .. 1.0)
}

pub fn random_double_in(r: &mut SmallRng, min: f32, max: f32) -> f32 {
    min + (max - min) * random_double(r)
}
