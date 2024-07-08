use rand::distributions::Distribution;
use rand::thread_rng;

pub fn random_double() -> f64 {
    let distr = rand::distributions::Uniform::new(0.0, 1.0);
    distr.sample(&mut thread_rng())
}
