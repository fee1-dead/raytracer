mod balls;
pub use balls::balls;

mod quads;
pub use quads::quads;

mod simple_light;
pub use simple_light::simple_light;

mod cornell_box;
pub use cornell_box::cornell_box;

mod cornell_box_testing;
pub use cornell_box_testing::cornell_box_testing;

use std::time::{Duration, Instant};

use crate::camera::Camera;
use crate::object::{Object, ObjectList};

pub struct Scene {
    camera: Camera,
    world: ObjectList,
    light: Box<dyn Object>,
}

impl Scene {
    pub fn render_with_metrics(self) -> color_eyre::Result<()> {
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

        self.camera.render(self.world, &*self.light)
    }
}

fn time_per(time: Duration, desc: &str) -> String {
    if time <= Duration::from_secs(1) {
        format!("{desc}s per second: {}", 1.0 / time.as_secs_f64())
    } else {
        format!("time per {desc}: {time:?}")
    }
}
