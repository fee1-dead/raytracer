pub mod aabb;
pub mod bvh;
pub mod camera;
mod color;
mod interval;
pub mod material;
pub mod object;
mod ray;
pub mod scenes;
mod utils;
pub(crate) mod vec3;
mod onb;
mod pdf;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    scenes::cornell_box().render_with_metrics()?;
    Ok(())
}
