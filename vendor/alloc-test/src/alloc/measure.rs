use std::{
    mem,
    sync::atomic::{AtomicBool, Ordering},
};

use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Display, Serialize, Deserialize)]
#[display(fmt = r#"Currently allocated (B): {current}
Maximum allocated (B): {peak}
Total amount of claimed memory (B): {total_size}
Total number of allocations: (N): {total_num}
Reallocations (N): {reallocs}
"#)]
pub struct MemoryStats {
    pub current: usize,
    pub peak: usize,
    pub total_size: usize,
    pub total_num: usize,
    pub reallocs: usize,
}

static mut TRACE_ALLOCS: AtomicBool = AtomicBool::new(false);

static mut ALLOC_STATS: MemoryStats = MemoryStats {
    current: 0,
    peak: 0,
    total_size: 0,
    total_num: 0,
    reallocs: 0,
};

/// Traces allocations performed while executing the `f`.
///
/// Beware that allocations made by nother threads will be also recorded.
///
/// ```
/// use tracing_allocator::{TracingAllocator, default_tracing_allocator, trace_allocs};
///
/// #[global_allocator]
/// static ALLOCATOR: TracingAllocator = default_tracing_allocator();
///
/// fn main() {
///     let (_, stats) = trace_allocs(|| {
///         let r: Vec<u8> = vec![1, 2, 3];
///         r
///     });
///     assert_eq!(stats.peak, 3);
///     assert_eq!(stats.current, 3);
/// }
/// ```
pub fn trace_allocs<F: FnOnce() -> O, O>(f: F) -> (O, MemoryStats) {
    unsafe {
        while TRACE_ALLOCS
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire)
            .is_err()
        {}
        let o = f();
        let stats = mem::replace(&mut ALLOC_STATS, Default::default());
        TRACE_ALLOCS.store(false, Ordering::Release);
        (o, stats)
    }
}

pub struct MemoryTracingHooks;

unsafe impl super::allocator::AllocHooks for MemoryTracingHooks {
    fn on_alloc(&self, _pointer: *mut u8, size: usize, _align: usize) {
        unsafe {
            if TRACE_ALLOCS.load(Ordering::Acquire) {
                // println!("allocating {size}");
                ALLOC_STATS.current += size;
                ALLOC_STATS.total_size += size;
                ALLOC_STATS.total_num += 1;
                if ALLOC_STATS.current > ALLOC_STATS.peak {
                    ALLOC_STATS.peak = ALLOC_STATS.current;
                }
            }
        }
    }

    fn on_dealloc(&self, _pointer: *mut u8, size: usize, _align: usize) {
        unsafe {
            if TRACE_ALLOCS.load(Ordering::Acquire) {
                ALLOC_STATS.current = ALLOC_STATS.current.saturating_sub(size);
            }
        }
    }

    fn on_alloc_zeroed(&self, pointer: *mut u8, size: usize, align: usize) {
        self.on_alloc(pointer, size, align);
    }

    fn on_realloc(
        &self,
        old_pointer: *mut u8,
        new_pointer: *mut u8,
        old_size: usize,
        new_size: usize,
        align: usize,
    ) {
        unsafe {
            if TRACE_ALLOCS.load(Ordering::Acquire) {
                // println!("reallocating {old_size} -> {new_size}");
                ALLOC_STATS.reallocs += 1;
            }
        }
        self.on_dealloc(old_pointer, old_size, align);
        self.on_alloc(new_pointer, new_size, align);
    }
}
