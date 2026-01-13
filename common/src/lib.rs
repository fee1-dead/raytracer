#![cfg_attr(target_arch = "nvptx64", no_std)]
#![allow(internal_features)]
#![feature(core_float_math, core_intrinsics, abi_gpu_kernel)]
#![cfg_attr(target_arch = "nvptx64", feature(stdarch_nvptx))]

pub mod aabb;
pub mod ffi;
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
        libm::cos(self)
    }
    fn sin(self) -> Self {
        libm::sin(self)
    }
    fn tan(self) -> Self {
        libm::tan(self)
    }
    fn sqrt(self) -> Self {
        libm::sqrt(self)
    }
    fn powi(self, n: i32) -> Self {
        libm::pow(self, n as f64)
    }
}

#[cfg(target_arch = "nvptx64")]
#[unsafe(no_mangle)]
pub unsafe extern "gpu-kernel" fn raytrace(
    // camera: &Camera
    camera: *const (),
    // world: &AnyObject
    world: *const (),
    // lights: &AnyObject
    lights: *const (),
    out: *mut u8
) {
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    use crate::object::AnyObject;
    use crate::camera::Camera;

    use core::arch::nvptx::*;

    let i = unsafe { _block_dim_x() * _block_idx_x() + _thread_idx_x() };
    let j = unsafe { _block_dim_y() * _block_idx_y() + _thread_idx_y() };

    let camera = unsafe { &*camera.cast::<Camera>() };
    let world = unsafe { &*world.cast::<AnyObject>() };
    let lights = unsafe { &*lights.cast::<AnyObject>() };

    if i as u64 >= camera.image_width || j as u64 >= camera.image_height {
        return;
    }
    
    let mut rng = SmallRng::seed_from_u64(42);
    let c = camera.render_single(&mut rng, world, lights, i as u64, j as u64);
    let offset = i * 3 + j * 3 * 5000;
    c.write_to_buf(unsafe { core::slice::from_raw_parts_mut(out.offset(offset as isize), 3) });
}

#[cfg(target_arch = "nvptx64")]
#[panic_handler]
fn handle(_pi: &core::panic::PanicInfo) -> ! {
    core::intrinsics::abort()
}

