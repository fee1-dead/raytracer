mod balls;
pub use balls::balls;

mod quads;
use image::ColorType;
pub use quads::quads;

mod simple_light;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use rayon::iter::{IndexedParallelIterator, ParallelIterator};
use rayon::slice::ParallelSliceMut;
pub use simple_light::simple_light;

mod cornell_box;
pub use cornell_box::cornell_box;

mod cornell_box_testing;
pub use cornell_box_testing::cornell_box_testing;

use std::time::{Duration, Instant};

use crate::camera::Camera;
use crate::object::{Object, ObjectList};

pub struct Scene<L: Object> {
    pub camera: Camera,
    pub world: ObjectList,
    pub light: L,
}

impl<L: Object> Scene<L> {
    pub fn render_with_metrics(mut self) -> color_eyre::Result<()> {
        if self.world.len() > 10 {
            self.world.condense();
        }
        let time = Instant::now();
        let pixels = self.camera.num_pixels();
        self.render()?;
        let elapsed = time.elapsed();
        eprintln!(
            "Done! Elapsed: {elapsed:?}, {}",
            time_per(elapsed / pixels as u32, "pixel")
        );
        Ok(())
    }

    pub fn render(mut self) -> color_eyre::Result<()> {
        if self.world.len() > 10 {
            self.world.condense();
        }

        let mut buf = vec![0; self.camera.buffer_len()];
        let width = self.camera.image_width as usize;

        buf.par_chunks_exact_mut(3).enumerate().for_each(|(pixel, chunk)| {
            let mut rng = SmallRng::seed_from_u64(pixel as u64);
            let i = pixel % width;
            let j = pixel / width;
            self.camera.render_single(&mut rng, &self.world, &self.light, i as u64, j as u64).write_to_buf(chunk);
        });

        image::save_buffer(
            "./image.png",
            &buf,
            self.camera.image_width as u32,
            self.camera.image_height as u32,
            ColorType::Rgb8,
        )?;
        Ok(())
    }
}

fn time_per(time: Duration, desc: &str) -> String {
    if time <= Duration::from_secs(1) {
        format!("{desc}s per second: {}", 1.0 / time.as_secs_f32())
    } else {
        format!("time per {desc}: {time:?}")
    }
}
