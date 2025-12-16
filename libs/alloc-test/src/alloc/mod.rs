use std::alloc::System;

use self::{allocator::TracingAllocator, measure::MemoryTracingHooks};

pub mod allocator;
pub mod benchmark;
pub mod compare;
pub mod measure;

pub const fn default_tracing_allocator() -> TracingAllocator<MemoryTracingHooks, System> {
    TracingAllocator::new(MemoryTracingHooks, System)
}
