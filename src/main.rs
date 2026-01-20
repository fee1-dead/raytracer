pub mod bvh;
pub mod object;
pub mod scenes;
pub use common::*;

#[cfg(not(feature = "gpu"))]
fn finalize<T>(x: T) -> &'static T {
    Box::leak(Box::new(x))
}

#[cfg(feature = "gpu")]
fn finalize<T: Copy>(x: T) -> &'static T {
    use cust::memory::*;
    #[repr(transparent)]
    #[derive(Clone, Copy)]
    struct UnsoundButWhatever<T>(T);
    unsafe impl<T: Copy> DeviceCopy for UnsoundButWhatever<T> {}
    unsafe {
        &(&*UnifiedBox::into_unified(UnifiedBox::new(UnsoundButWhatever(x)).unwrap()).as_raw()).0
    }
}

#[cfg(not(feature = "gpu"))]
fn finalize_vec<T>(x: Vec<T>) -> &'static [T] {
    Box::leak(x.into_boxed_slice())
}

#[cfg(feature = "gpu")]
fn finalize_vec<T: Copy>(x: Vec<T>) -> &'static [T] {
    // gosh i hope this doesn't crash
    use std::mem::{transmute, ManuallyDrop};

    use cust::memory::*;
    #[repr(transparent)]
    #[derive(Clone, Copy)]
    struct UnsoundButWhatever<T>(T);
    unsafe impl<T: Copy> DeviceCopy for UnsoundButWhatever<T> {}
    let s: &[T] = &x;
    let b: UnifiedBuffer<UnsoundButWhatever<T>> =
        UnifiedBuffer::from_slice(unsafe { transmute(s) }).unwrap();
    let f = ManuallyDrop::new(b);
    unsafe { transmute(f.as_slice()) }
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    #[cfg(feature = "gpu")]
    gpu_main()?;
    #[cfg(not(feature = "gpu"))]
    scenes::cornell_box_testing().render_with_metrics()?;
    Ok(())
}

#[cfg(feature = "gpu")]
fn gpu_main() -> color_eyre::Result<()> {
    use std::ffi::CString;
    use std::fs;

    use common::object::AnyObject;
    use cust::context::{CurrentContext, ResourceLimit};
    use cust::prelude::Context;
    use cust::{CudaFlags, launch};
    use cust::memory::{CopyDestination, DeviceBuffer, UnifiedPointer};
    use cust::module::Module;
    use cust::stream::{Stream, StreamFlags};

    let x = unsafe { cust::sys::cuInit(0) };
    println!("{x:?}");
    cust::init(CudaFlags::empty()).unwrap();


    println!("1");

    let device = cust::device::Device::get_device(0)?;
    let _ctx = Context::new(device).unwrap();
    println!("2");
    let ss = CurrentContext::get_resource_limit(ResourceLimit::StackSize).unwrap();
    println!("Device Name: {}; default stack size: {ss}", device.name()?);

    // 32 KiB of stack
    CurrentContext::set_resource_limit(ResourceLimit::StackSize, 1024*32);

    let scene = scenes::cornell_box_testing();
    let cam = finalize(scene.camera);
    let cam = unsafe { UnifiedPointer::wrap(cam as *const _ as *mut ()) };
    let world = finalize(AnyObject::from(scene.world.finalize()));
    let world = unsafe { UnifiedPointer::wrap(world as *const _ as *mut ()) };
    let light = finalize(AnyObject::from(scene.light));
    let light = unsafe { UnifiedPointer::wrap(light as *const _ as *mut ()) };
    let ptx = CString::new(fs::read_to_string(
        "target/nvptx64-nvidia-cuda/release/common.ptx",
    )?)?;
    let buf: DeviceBuffer<u8> = unsafe { DeviceBuffer::uninitialized(1024*1024 * 3) }?;
    let buf_ptr = buf.as_device_ptr();
    println!("{buf_ptr:p}");
    let module = Module::from_ptx_cstr(&ptx, &[])?;
    let stream = Stream::new(StreamFlags::NON_BLOCKING, None)?;
    unsafe {
        launch! {
            module.raytrace<<<  (64,64,1), (16,16,1), 0, stream >>>(cam, world, light, buf_ptr)
            // module.raytrace<<<  (1,1,1), (1,1,1), 0, stream >>>(cam, world, light, buf_ptr)
        }
    }?;
    stream.synchronize()?;
    let mut outbuf = vec![0; 1024*1024*3];
    buf.copy_to(&mut outbuf)?;
    drop(buf);
    image::save_buffer(
            "./image_gpu.png",
            &outbuf,
            1024,
            1024,
            image::ColorType::Rgb8,
    )?;
    Ok(())
}
