use std::{fs::File, io::Write, sync::Mutex};

use tracking_allocator::{AllocationGroupId, AllocationTracker};

pub struct AllocTracker {
    file: Option<Mutex<File>>,
}

impl AllocTracker {
    pub fn new() -> Self {
        let f = File::create("target/allocations.log")
            .expect("cannot create a file for allocation log");
        AllocTracker {
            file: Some(Mutex::new(f)),
        }
    }

    pub fn void() -> Self {
        AllocTracker { file: None }
    }
}

impl AllocationTracker for AllocTracker {
    fn allocated(
        &self,
        addr: usize,
        object_size: usize,
        wrapped_size: usize,
        group_id: AllocationGroupId,
    ) {
        if let Some(file) = &self.file {
            let mut log = file.lock().expect("poisoned");
            writeln!(
                log,
                "allocation -> addr=0x{:0x} object_size={} wrapped_size={} group_id={:?}",
                addr, object_size, wrapped_size, group_id
            )
            .unwrap_or_default();
        }
    }

    fn deallocated(
        &self,
        addr: usize,
        object_size: usize,
        wrapped_size: usize,
        source_group_id: AllocationGroupId,
        current_group_id: AllocationGroupId,
    ) {
        if let Some(file) = &self.file {
            let mut log = file.lock().expect("poisoned");
            writeln!(
                log,
                "deallocation -> addr=0x{:0x} object_size={} wrapped_size={} source_group_id={:?} current_group_id={:?}",
                addr, object_size, wrapped_size, source_group_id, current_group_id
            )
            .unwrap_or_default();
        }
    }
}
