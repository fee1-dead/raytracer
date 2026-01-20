#![cfg_attr(target_arch = "nvptx64", no_std)]
#![allow(internal_features)]
#![feature(core_float_math, core_intrinsics, abi_gpu_kernel)]
#![cfg_attr(target_arch = "nvptx64", feature(stdarch_nvptx))]


#[cfg(target_arch = "nvptx64")]
mod gpu;

#[cfg(target_arch = "nvptx64")]
extern crate alloc;

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

impl Float for f32 {
    fn cos(self) -> Self {
        libm::cosf(self)
    }
    fn sin(self) -> Self {
        libm::sinf(self)
    }
    fn tan(self) -> Self {
        libm::tanf(self)
    }
    fn sqrt(self) -> Self {
        libm::sqrtf(self)
    }
    fn powi(self, n: i32) -> Self {
        libm::powf(self, n as f32)
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
    unsafe extern "C" {
        pub fn vprintf(format: *const u8, valist: *const core::ffi::c_void) -> i32;
    }

    macro_rules! cuda_printf {
        ($($tt:tt)*) => {
            unsafe {
                let mut s = alloc::format!($($tt)*);
                s.push('\0');
                vprintf(s.as_ptr(), core::ptr::null())
            }
        };
    }

    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    use crate::object::AnyObject;
    use crate::camera::Camera;

    use core::arch::nvptx::*;

    let i = unsafe { _block_dim_x() * _block_idx_x() + _thread_idx_x() };
    let j = unsafe { _block_dim_y() * _block_idx_y() + _thread_idx_y() };

    // cuda_printf!("kernel launched with i={i}, j={j}!\n");

    let camera = unsafe { &*camera.cast::<Camera>() };
    let world = unsafe { &*world.cast::<AnyObject>() };
    let lights = unsafe { &*lights.cast::<AnyObject>() };
    
    let mut rng = SmallRng::seed_from_u64((i as u64)*1024 + j as u64);
    let c = camera.render_single(&mut rng, world, lights, i as u64, j as u64);
    let offset = i * 3 + j * 3 * 1024;
    // cuda_printf!("offset={offset}!\n");
    c.write_to_buf(unsafe { core::slice::from_raw_parts_mut(out.offset(offset as isize), 3) });
}

#[cfg(target_arch = "nvptx64")]
#[panic_handler]
fn handle(_pi: &core::panic::PanicInfo) -> ! {
    core::intrinsics::abort()
}

