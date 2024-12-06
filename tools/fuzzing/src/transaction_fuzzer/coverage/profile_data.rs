use super::util::{cursor_align, get_data, read_int};
use std::{io::Cursor, mem};

// From LLVM's compiler-rt/include/profile/InstrProfData.inc
#[repr(packed)]
#[derive(Debug)]
pub struct FunControl {
    pub name_hash: u64,
    pub func_hash: u64,
    pub relative_counter_ptr: u64,
    pub relative_bitmap_ptr: u64,
    pub function_ptr: u64,
    pub values_ptr: u64,
    pub num_counters: u32,
    pub num_value_sites: u16,
    pub num_bitmap_bytes: u32,
}

impl FunControl {
    #[coverage(off)]
    pub fn read(cursor: &mut Cursor<&Vec<u8>>) -> Self {
        Self {
            name_hash: read_int(cursor),
            func_hash: read_int(cursor),
            relative_counter_ptr: read_int(cursor),
            relative_bitmap_ptr: read_int(cursor),
            function_ptr: read_int(cursor),
            values_ptr: read_int(cursor),
            num_counters: read_int(cursor),
            num_value_sites: read_int(cursor),
            num_bitmap_bytes: read_int(cursor),
        }
    }

    #[coverage(off)]
    pub fn size() -> usize {
        mem::size_of::<Self>()
    }
}

#[derive(Debug)]
pub struct ProfileData(pub Vec<FunControl>);

impl Default for ProfileData {
    #[coverage(off)]
    fn default() -> Self {
        let data_buf = unsafe { get_data() }.to_vec();
        let mut cursor = Cursor::new(&data_buf);
        let mut output = Vec::new();

        while (cursor.position() as usize + FunControl::size()) < data_buf.len() {
            output.push(FunControl::read(&mut cursor));
            cursor_align::<u64>(&mut cursor);
        }

        Self(output)
    }
}

impl ProfileData {
    #[coverage(off)]
    pub fn new() -> Self {
        Self::default()
    }
}
