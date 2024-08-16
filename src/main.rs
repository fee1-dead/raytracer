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

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    scenes::balls().render_with_metrics()?;
    Ok(())
}
