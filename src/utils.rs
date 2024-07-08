use rand::distributions::Distribution;
use rand::thread_rng;

/// Returns a random double in [0,1).
pub fn random_double() -> f64 {
    let distr = rand::distributions::Uniform::new(0.0, 1.0);
    distr.sample(&mut thread_rng())
}

pub fn random_double_in(min: f64, max: f64) -> f64 {
    min + (max - min) * random_double()
}
