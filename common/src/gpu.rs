use core::alloc::{GlobalAlloc, Layout};

unsafe extern "C" {
    fn malloc(size: usize) -> *mut u8;
    fn free(ptr: *mut u8);
}

struct CudaAllocator;

unsafe impl GlobalAlloc for CudaAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // todo: alignment
        unsafe { malloc(layout.size()) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        unsafe { free(ptr) }
    }
}

#[global_allocator]
static ALLOCATOR: CudaAllocator = CudaAllocator;
