pub mod bvh;
pub mod object;
pub mod scenes;
pub use common::*;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    scenes::cornell_box_testing().render_with_metrics()?;
    Ok(())
}
