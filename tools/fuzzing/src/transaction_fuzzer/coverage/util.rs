use core::slice;
use std::{
    collections::HashMap,
    io::Read,
    io::{Cursor, Seek, SeekFrom},
    path::Path,
};

use bitvec::macros::internal::funty;
use object::{Object, ObjectSection};

extern "C" {
    fn __llvm_profile_begin_counters() -> *mut i64;
    fn __llvm_profile_end_counters() -> *mut i64;
    fn __llvm_profile_get_num_counters(begin: *mut i64, end: *mut i64) -> i64;
    fn __llvm_profile_begin_data() -> *const u8;
    fn __llvm_profile_end_data() -> *const u8;
    fn __llvm_profile_begin_names() -> *const u8;
    fn __llvm_profile_end_names() -> *const u8;
}

#[coverage(off)]
pub unsafe fn get_counters() -> &'static mut [i64] {
    let begin = __llvm_profile_begin_counters();
    let end = __llvm_profile_end_counters();
    let num_counters = __llvm_profile_get_num_counters(begin, end);
    slice::from_raw_parts_mut(begin, num_counters as usize)
}

#[coverage(off)]
pub unsafe fn get_data() -> &'static [u8] {
    let begin = __llvm_profile_begin_data();
    let end = __llvm_profile_end_data();
    slice::from_raw_parts(begin, end.offset_from(begin) as usize)
}

#[coverage(off)]
pub unsafe fn get_names() -> &'static [u8] {
    let begin = __llvm_profile_begin_names();
    let end = __llvm_profile_end_names();
    slice::from_raw_parts(begin, end.offset_from(begin) as usize)
}

#[coverage(off)]
pub fn get_module_path() -> String {
    let maps = rsprocmaps::from_path("/proc/self/maps").unwrap();

    for map in maps {
        let map = map.unwrap();

        let rsprocmaps::AddressRange { begin, end } = map.address_range;

        if (begin..end).contains(&(get_module_path as usize as u64)) {
            if let rsprocmaps::Pathname::Path(path) = map.pathname {
                return path;
            } else {
                panic!()
            }
        }
    }

    panic!()
}

#[coverage(off)]
pub fn get_elf_sections(module: String, section_names: &Vec<&str>) -> HashMap<String, Vec<u8>> {
    let mut result = HashMap::new();
    let path = Path::new(&module);

    let bin_data = std::fs::read(path).unwrap();
    let obj_file = object::File::parse(&*bin_data).unwrap();

    for section_name in section_names.iter() {
        let section = obj_file.section_by_name(section_name).unwrap();
        result.insert(section_name.to_string(), section.data().unwrap().to_vec());
    }

    result
}

#[coverage(off)]
pub fn read_int<const N: usize, T, A>(cursor: &mut Cursor<A>) -> T
where
    T: funty::Numeric<Bytes = [u8; N]>,
    A: AsRef<[u8]>,
{
    let mut buf = [0u8; N];
    cursor.read_exact(buf.as_mut_slice()).unwrap();
    T::from_le_bytes(buf)
}

pub trait Leb128 {
    type T;

    fn read_leb128(cursor: &mut Cursor<&Vec<u8>>) -> Self::T;
}

impl Leb128 for i64 {
    type T = i64;

    #[coverage(off)]
    fn read_leb128(cursor: &mut Cursor<&Vec<u8>>) -> Self::T {
        leb128::read::signed(cursor).unwrap()
    }
}

impl Leb128 for u64 {
    type T = u64;

    #[coverage(off)]
    fn read_leb128(cursor: &mut Cursor<&Vec<u8>>) -> Self::T {
        leb128::read::unsigned(cursor).unwrap()
    }
}

impl Leb128 for usize {
    type T = usize;

    #[coverage(off)]
    fn read_leb128(cursor: &mut Cursor<&Vec<u8>>) -> Self::T {
        leb128::read::unsigned(cursor).unwrap() as usize
    }
}

#[coverage(off)]
fn align<T>(pos: u64) -> u64 {
    let alignment = std::mem::size_of::<T>() as u64;
    (pos + (alignment - 1)) & !(alignment - 1)
}

#[coverage(off)]
pub fn cursor_align<T>(cursor: &mut Cursor<&Vec<u8>>) {
    cursor
        .seek(SeekFrom::Start(align::<T>(cursor.position())))
        .unwrap();
}
