use std::alloc::{GlobalAlloc, Layout};

#[derive(Debug, Default)]
pub struct TracingAllocator<H: 'static, A>(A, H)
where
    A: GlobalAlloc;

impl<H, A> TracingAllocator<H, A>
where
    A: GlobalAlloc,
{
    pub const fn new(hooks: H, allocator: A) -> Self {
        TracingAllocator(allocator, hooks)
    }
}

// pub const fn default_tracing_allocator() -> TracingAllocator<MemoryTracingHooks, System> {
//     TracingAllocator(System, MemoryTracingHooks)
// }

unsafe impl<H, A> GlobalAlloc for TracingAllocator<H, A>
where
    A: GlobalAlloc,
    H: AllocHooks,
{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();
        let pointer = self.0.alloc(layout);
        self.1.on_alloc(pointer, size, align);
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        let size = layout.size();
        let align = layout.align();
        self.0.dealloc(pointer, layout);
        self.1.on_dealloc(pointer, size, align);
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();
        let pointer = self.0.alloc_zeroed(layout);
        self.1.on_alloc_zeroed(pointer, size, align);
        pointer
    }

    unsafe fn realloc(&self, old_pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let old_size = layout.size();
        let align = layout.align();
        let new_pointer = self.0.realloc(old_pointer, layout, new_size);
        self.1
            .on_realloc(old_pointer, new_pointer, old_size, new_size, align);
        new_pointer
    }
}

pub unsafe trait AllocHooks {
    fn on_alloc(&self, pointer: *mut u8, size: usize, align: usize);
    fn on_dealloc(&self, pointer: *mut u8, size: usize, align: usize);
    fn on_alloc_zeroed(&self, pointer: *mut u8, size: usize, align: usize);
    fn on_realloc(
        &self,
        old_pointer: *mut u8,
        new_pointer: *mut u8,
        old_size: usize,
        new_size: usize,
        align: usize,
    );
}
