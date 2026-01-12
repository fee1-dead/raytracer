#![no_std]
#![allow(internal_features)]
#![feature(core_float_math, core_intrinsics)]

pub mod aabb;
pub mod vec3;
pub mod ray;
pub mod interval;
pub mod utils;
pub mod camera;
pub mod color;
pub mod object;
pub mod onb;
pub mod material;
pub mod pdf;

pub trait Float: Copy + Sized {
    fn cos(self) -> Self;
    fn sin(self) -> Self;
    fn tan(self) -> Self;
    fn sqrt(self) -> Self;
    fn sin_cos(self) -> (Self, Self) {
        (self.sin(), self.cos())
    }
    fn powi(self, n: i32) -> Self;
}

impl Float for f64 {
    fn cos(self) -> Self {
        core::intrinsics::cosf64(self)
    }
    fn sin(self) -> Self {
        core::intrinsics::sinf64(self)
    }
    fn tan(self) -> Self {
        self.sin() / self.cos()
    }
    fn sqrt(self) -> Self {
        core::f64::math::sqrt(self)
    }
    fn powi(self, n: i32) -> Self {
        core::f64::math::powi(self, n)
    }
}

/*
#[unsafe(no_mangle)]
pub unsafe extern "gpu-kernel" fn raytrace(
    // camera: &Camera
    camera: *const (),
    // world: &&dyn Object
    world: *const (),
    // lights: &&dyn Object
    lights: *const (),
    out: *mut u64
) {
    let camera = unsafe { &*camera.cast::<Camera>() };
    camera.render_single(world, lights, buf);
    out.write(camera.buffer_len() as u64)
}
*/

#[cfg(not(test))]
#[panic_handler]
fn handle(_pi: &core::panic::PanicInfo) -> ! {
    core::intrinsics::abort()
}

