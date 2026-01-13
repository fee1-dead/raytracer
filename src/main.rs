pub mod bvh;
pub mod object;
pub mod scenes;
pub use common::*;

fn finalize<T>(x: T) -> &'static T {
    Box::leak(Box::new(x))
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    scenes::cornell_box_testing().render_with_metrics()?;
    Ok(())
}
