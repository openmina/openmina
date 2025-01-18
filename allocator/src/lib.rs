#![no_std]

use core::{alloc::{GlobalAlloc, Layout}, ptr::NonNull, sync::atomic::{AtomicBool, AtomicPtr, AtomicU64, AtomicUsize, Ordering::{AcqRel, Acquire, Relaxed, Release}}};

use libc_print::{libc_dbg as dbg, libc_eprintln as eprintln};

struct ClassInfo {
    size: usize,
    nelems: usize,
}

static N_OP: AtomicUsize = AtomicUsize::new(0);
static TOTAL_NBYTES: AtomicUsize = AtomicUsize::new(0);

const N_CLASSES: usize = 32;

const CLASSES: [ClassInfo; N_CLASSES] = [
    ClassInfo { size: 16, nelems: 20_000 },
    ClassInfo { size: 24, nelems: 30_000 },
    ClassInfo { size: 32, nelems: 2_000_000 },
    ClassInfo { size: 48, nelems: 50_000 },
    ClassInfo { size: 64, nelems: 1_000_000 },
    ClassInfo { size: 80, nelems: 80_000 },
    ClassInfo { size: 96, nelems: 20_000 },
    ClassInfo { size: 112, nelems: 20_000 },
    ClassInfo { size: 128, nelems: 20_000 },
    ClassInfo { size: 160, nelems: 300_000 },
    ClassInfo { size: 192, nelems: 20_000 },
    ClassInfo { size: 224, nelems: 60_000 },
    ClassInfo { size: 256, nelems: 200_000 },
    ClassInfo { size: 320, nelems: 50_000 },
    ClassInfo { size: 384, nelems: 100_000 },
    ClassInfo { size: 448, nelems: 20_000 },
    ClassInfo { size: 512, nelems: 500_000 },
    ClassInfo { size: 1024, nelems: 50_000 },
    ClassInfo { size: 1280, nelems: 10_000 },
    ClassInfo { size: 1536, nelems: 30_000 },
    ClassInfo { size: 1792, nelems: 10_000 },
    ClassInfo { size: 2048, nelems: 50_000 },
    ClassInfo { size: 3072, nelems: 50_000 },
    ClassInfo { size: 4096, nelems: 50_000 },
    ClassInfo { size: 8192, nelems: 50_000 },
    ClassInfo { size: 16384, nelems: 30_000 },
    ClassInfo { size: 65536, nelems: 20_000 },
    ClassInfo { size: 0x100000, nelems: 200 },
    ClassInfo { size: 0x1000000, nelems: 200 },
    ClassInfo { size: 0x2000000, nelems: 10 },
    ClassInfo { size: 0x3000000, nelems: 10 },
    ClassInfo { size: 0x10000000, nelems: 5 },
];

// const CLASSES: [ClassInfo; N_CLASSES] = [
//     ClassInfo { size: 16, nelems: 7000 },
//     ClassInfo { size: 24, nelems: 12_000 },
//     ClassInfo { size: 32, nelems: 1_300_000 },
//     ClassInfo { size: 48, nelems: 29_000 },
//     ClassInfo { size: 64, nelems: 610_000 },
//     ClassInfo { size: 80, nelems: 75_000 },
//     ClassInfo { size: 96, nelems: 13_500 },
//     ClassInfo { size: 112, nelems: 150 },
//     ClassInfo { size: 128, nelems: 850 },
//     ClassInfo { size: 160, nelems: 8_000 },
//     ClassInfo { size: 192, nelems: 5_000 },
//     ClassInfo { size: 224, nelems: 12_500 },
//     ClassInfo { size: 256, nelems: 2500 },
//     ClassInfo { size: 320, nelems: 38_000 },
//     ClassInfo { size: 384, nelems: 450 },
//     ClassInfo { size: 448, nelems: 11_000 },
//     ClassInfo { size: 512, nelems: 220_000 },
//     ClassInfo { size: 1024, nelems: 31_000 },
//     ClassInfo { size: 1280, nelems: 6_000 },
//     ClassInfo { size: 1536, nelems: 18_000 },
//     ClassInfo { size: 1792, nelems: 5_600 },
//     ClassInfo { size: 2048, nelems: 26_000 },
//     ClassInfo { size: 3072, nelems: 37_500 },
//     ClassInfo { size: 4096, nelems: 12_500 },
//     ClassInfo { size: 8192, nelems: 650 },
//     ClassInfo { size: 16384, nelems: 16_000 },
//     ClassInfo { size: 65536, nelems: 4_000 },
//     ClassInfo { size: 0x100000, nelems: 152 },
//     ClassInfo { size: 0x1000000, nelems: 30 },
//     ClassInfo { size: 0x2000000, nelems: 6 },
//     ClassInfo { size: 0x3000000, nelems: 5 },
//     ClassInfo { size: 0x10000000, nelems: 2 },
// ];

// [STATS-588] class size:16 max:6677
// [STATS-588] class size:24 max:10886
// [STATS-588] class size:32 max:1259172
// [STATS-588] class size:48 max:27647
// [STATS-588] class size:64 max:592569
// [STATS-588] class size:80 max:69234
// [STATS-588] class size:96 max:12851
// [STATS-588] class size:112 max:92
// [STATS-588] class size:128 max:645
// [STATS-588] class size:160 max:5385
// [STATS-588] class size:192 max:3733
// [STATS-588] class size:224 max:11492
// [STATS-588] class size:256 max:2309
// [STATS-588] class size:320 max:36761
// [STATS-588] class size:384 max:355
// [STATS-588] class size:448 max:10379
// [STATS-588] class size:512 max:205067
// [STATS-588] class size:1024 max:29414
// [STATS-588] class size:1280 max:5393
// [STATS-588] class size:1536 max:17034
// [STATS-588] class size:1792 max:5214
// [STATS-588] class size:2048 max:24494
// [STATS-588] class size:3072 max:36940
// [STATS-588] class size:4096 max:12060
// [STATS-588] class size:8192 max:531
// [STATS-588] class size:16384 max:15017
// [STATS-588] class size:65536 max:2407
// [STATS-588] class size:1048576 max:147
// [STATS-588] class size:16777216 max:22
// [STATS-588] class size:33554432 max:4
// [STATS-588] class size:50331648 max:3
// [STATS-588] class size:268435456 max:1
// [STATS-588] TOTAL:742610483

struct Class {
    elem_size: usize,
    bitfields: NonNull<[AtomicU64]>,
    base: NonNull<u8>,
    /// In bytes
    length: usize,
    nallocated: AtomicUsize,
    max_nallocated: AtomicUsize,
    info: &'static ClassInfo,
    end_ptr: NonNull<u8>,
    // TODO: Make this thread-local
    free_hint: AtomicUsize,
}

struct Classes {
    inner: [Class; N_CLASSES],
}

struct MinallocImpl {
    base: NonNull<u8>,
    base_len: usize,
    classes: Classes,
}

static mut MINALLOC_STORAGE: Option<MinallocImpl> = None;
static MINALLOC_PTR: AtomicPtr<MinallocImpl> = AtomicPtr::new(core::ptr::null_mut());

pub struct Minalloc;

unsafe impl GlobalAlloc for Minalloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.get().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.get().dealloc(ptr, layout)
    }
}

impl Minalloc {
    fn get(&self) -> &'static MinallocImpl {
        loop {
            let ptr = MINALLOC_PTR.load(Relaxed);
            if !ptr.is_null() {
                return unsafe { &*ptr };
            }
            static IS_INITIALIZING: AtomicBool = AtomicBool::new(false);
            if IS_INITIALIZING.swap(true, AcqRel) {
                // spin loop
                continue;
            }
            let ptr = unsafe {
                MINALLOC_STORAGE = Some(MinallocImpl::new());
                MINALLOC_STORAGE.as_mut().unwrap() as *mut MinallocImpl
            };
            MINALLOC_PTR.store(ptr, Release);
        }
    }
}

impl MinallocImpl {
    pub fn new() -> Self {
        let (base, base_len) = mmap::initialize();
        let base = NonNull::new(base).expect("Failed to initialize allocator");
        let classes = Classes::new(base, base_len);

        Self {
            base,
            base_len,
            classes,
        }
    }

    fn alloc(&self, layout: Layout) -> *mut u8 {
        // self.show_stats();

        if layout.size() == 0 {
            return NonNull::dangling().as_ptr();
        }

        let size = if layout.align() > 8 {
            layout.size() + layout.align()
        } else {
            layout.size()
        };

        // assert!(layout.align() <= 8, "size: {:?} align: {:?}", layout.size(), layout.align());
        // eprintln!("alloc size: {:?} align: {:?}", layout.size(), layout.align());

        for class in &self.classes.inner {
            if size <= class.elem_size {
                return class.take_next(&layout).as_ptr();
            }
        }

        eprintln!("size too big: size:{:?} align:{:?}", layout.size(), layout.align());

        todo!() // Implement linked list based allocations
    }

    fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // dbg!(&layout);

        if layout.size() == 0 {
            return;
        }

        let size = if layout.align() > 8 {
            layout.size() + layout.align()
        } else {
            layout.size()
        };

        for class in &self.classes.inner {
            if size <= class.elem_size {
                return class.free(ptr, &layout);
            }
        }

        eprintln!("DEALLOC size too big: ptr:{:?} size:{:?} align:{:?}", ptr, layout.size(), layout.align());

        todo!() // Implement linked list based allocations
    }

    fn show_stats(&self) {
        const INTERVAL: usize = 200_000;

        let n_op = N_OP.load(Relaxed);
        if n_op % INTERVAL != 0 {
            return;
        }

        let index = n_op / INTERVAL;

        for class in &self.classes.inner {
            eprintln!(
                "[STATS-{:?}] class size:{:?} max:{:?} current:{:?}",
                index, class.elem_size, class.max_nallocated.load(Relaxed), class.nallocated.load(Relaxed),
            );
        }

        eprintln!("[STATS-{:?}] TOTAL:{:?}", index, TOTAL_NBYTES.load(Relaxed));
    }
}

impl Class {
    fn take_next(&self, layout: &Layout) -> NonNull<u8> {
        let Self { bitfields, base, length, elem_size, nallocated, max_nallocated, info, free_hint, end_ptr } = self;

        TOTAL_NBYTES.fetch_add(layout.size(), Relaxed);
        let n_op = N_OP.fetch_add(1, AcqRel);

        let nallocated = nallocated.fetch_add(1, Relaxed) + 1;

        if nallocated > max_nallocated.load(Relaxed) {
            max_nallocated.store(nallocated, Relaxed);
        }

        if nallocated >= info.nelems {
            eprintln!("\n\nnallocated:{:?} info.nelems:{:?} size:{:?} align:{:?}\n", nallocated, info.nelems, layout.size(), layout.align());
        }

        assert!(nallocated < info.nelems, "\n\nnallocated:{:?} info.nelems:{:?} size:{:?} align:{:?}\n", nallocated, info.nelems, layout.size(), layout.align());

        let bitfields: &[AtomicU64] = unsafe { bitfields.as_ref() };

        // TODO: Make this thread-local
        let start = free_hint.load(Acquire).min(bitfields.len() - 1);
        // let start = 0;

        // 'outer: for (bitfield_index, bitfield_ref) in bitfields.iter().enumerate() {
        'outer: for (bitfield_index, bitfield_ref) in bitfields[start..]
            .iter()
            .enumerate()
            .map(|(index, bits)| (index + start, bits))
            .chain(bitfields[..start].iter().enumerate())
        {
            'inner: loop {
                let bitfield = bitfield_ref.load(Acquire);
                if bitfield == 0 {
                     // full
                    continue 'outer;
                }
                let index_free = bitfield.trailing_zeros() as usize;

                if (bitfield_index * 64) + index_free >= info.nelems {
                    // Going past end of `base`
                    continue 'outer;
                }

                let bit = 1 << index_free;
                let previous_bitfield = bitfield_ref.fetch_and(!bit, AcqRel);
                if previous_bitfield & bit == 0 {
                    // Acquired by another thread
                    continue 'inner;
                }

                free_hint.store(bitfield_index, Release);

                let mut res = self.get(bitfield_index, index_free);

                let ptr_unmodified = res;

                if layout.align() > 8 {
                    let offset = res.align_offset(layout.align());
                    res = unsafe { res.byte_add(offset) };
                }

                assert!(unsafe { res.byte_add(layout.size()) } <= {
                    let (bitfield_index, index_free) = if index_free == 63 {
                        (bitfield_index + 1, 0)
                    } else {
                        assert!(index_free < 63);
                        (bitfield_index, index_free + 1)
                    };
                    self.get(bitfield_index, index_free)
                });
                assert_eq!((bitfield_index, index_free), self.compute_offsets(res.as_ptr(), "alloc"));
                assert_eq!(
                    res.as_ptr() as usize % layout.align(), 0,
                    "ptr: {:?} align: {:?} size: {:?} ptr_unmod: {:?}",
                    res, layout.align(), layout.size(), ptr_unmodified
                );
                // eprintln!("alloc size: {:?} align: {:?}", layout.size(), layout.align());
                // eprintln!(
                //     "[{:?}] alloc: base: {:?} size: {:?} align: {:?} bitfield_index:{:?} index_free:{:?} ptr: {:?} offset: {:?} nallocated:{:?}",
                //     n_op, self.base, layout.size(), layout.align(), bitfield_index, index_free, res, unsafe { res.offset_from(self.base) }, nallocated
                // );
                return res;
            }
        }

        let nallocated = self.nallocated.load(Acquire);

        eprintln!("\nallocated:{:?} info.nelems:{:?} size:{:?} align:{:?}\n", nallocated, info.nelems, layout.size(), layout.align());

        panic!("limit reached {:?}", elem_size);
    }

    fn get(&self, bitfield_index: usize, bit: usize) -> NonNull<u8> {
        let bitfield_index = bitfield_index * self.elem_size;
        let bit = bit * self.elem_size;
        unsafe { self.base.byte_add((bitfield_index * 64) + bit) }
    }

    fn compute_offsets(&self, ptr: *mut u8, from: &str) -> (usize, usize) {
        let Self { elem_size, base, end_ptr, .. } = self;
        if !(ptr >= base.as_ptr() && ptr < end_ptr.as_ptr()) {
            eprintln!("{} Invalid class PTR ptr:{:?} base:{:?} end_ptr:{:?} size:{:?}", from, ptr, base, end_ptr, elem_size);
            assert!(ptr >= base.as_ptr() && ptr < end_ptr.as_ptr());
        }
        let offset = (ptr as usize).checked_sub(base.as_ptr() as usize).unwrap();
        let offset = offset / *elem_size;
        let bitfield_index = offset / 64;
        let bit_index = offset % 64;
        (bitfield_index, bit_index)
    }

    fn free(&self, ptr: *mut u8, layout: &Layout) {
        let Self { elem_size, bitfields, base, length, nallocated, info, max_nallocated, free_hint, end_ptr } = self;

        TOTAL_NBYTES.fetch_sub(layout.size(), Relaxed);
        let n_op = N_OP.fetch_add(1, AcqRel);

        let (bitfield_index, bit_index) = self.compute_offsets(ptr, "free");

        // let offset = (ptr as usize).checked_sub(base.as_ptr() as usize).unwrap();
        // let offset = offset / *elem_size;

        // // let offset = self.base.as_ptr() as usize - ptr as usize;
        // let bitfield_index = offset / 64;
        // let bit_index = offset % 64;

        let bit = 1 << bit_index;

        let bitfields: &[AtomicU64] = unsafe { bitfields.as_ref() };

        nallocated.fetch_sub(1, Relaxed);

        // eprintln!(
        //     "[{:?}] free ptr: {:?} bitfield_index: {:?} index_free: {:?} bit_index: {:?} size: {:?} offset: {:?}",
        //     n_op, ptr, bitfield_index, offset, bit_index, layout.size(), unsafe { ptr.offset_from(base.as_ptr()) }
        // );
        // eprintln!("free len: {:?} bitfield_index: {:?} size: {:?} ptr: {:?}", bitfields.len(), bitfield_index, layout.size(), ptr);

        // We set our bit to mark the block as free.
        // fetch_add is faster than fetch_or (xadd vs cmpxchg), and
        // we're sure to be the only thread to set this bit.
        let previous = bitfields[bitfield_index].fetch_add(bit, AcqRel);
        assert_eq!(previous & bit, 0); // Double free

        free_hint.store(bitfield_index, Release);
    }
}

impl Classes {
    fn new(base: NonNull<u8>, length: usize) -> Self {
        let end_ptr = unsafe { base.byte_add(length) };

        let offset_to_aligned = base.align_offset(8); // TODO: 4 in wasm32
        let mut current = unsafe { base.byte_add(offset_to_aligned) };

        let mut total_bitfields_nbytes = 0;

        let classes = CLASSES.each_ref().map(|info| {
            // dbg!(size);

            let ClassInfo { size, nelems } = info;

            assert!(current.cast::<AtomicU64>().is_aligned());

            let bitfields = current;

            // let bitfields_length = nelems / core::mem::size_of::<AtomicU64>();
            // let remaining = nelems % core::mem::size_of::<AtomicU64>();
            // assert_eq!(remaining, 0);
            // let bitfields_length = nelems.div_ceil(core::mem::size_of::<AtomicU64>());
            let bitfields_length = nelems.div_ceil(64);
            let mut bitfields: NonNull<[AtomicU64]> = NonNull::slice_from_raw_parts(bitfields.cast::<AtomicU64>(), bitfields_length);

            // dbg!(bitfields_length);

            let bitfields_nbytes = {
                let bitfields: &mut [AtomicU64] = unsafe { bitfields.as_mut() };
                let bitfields: &mut [u64] = unsafe {
                     core::mem::transmute(bitfields)
                };
                bitfields.fill(u64::MAX);
                bitfields.len() * core::mem::size_of::<u64>()
            };

            // let before = current;
            total_bitfields_nbytes += bitfields_nbytes;
            // let bitfields_nbytes = nelems * std::mem::size_of::<u64>();
            current = unsafe { current.byte_add(bitfields_nbytes) };
            // dbg!(before, current, nbytes);
            assert!(current.cast::<u64>().is_aligned()); // TODO: u32 in wasm32

            let base = current;
            let length = size * nelems;

            // let end_ptr = unsafe { base.byte_add(bitfields_nbytes + length) };

            current = unsafe { current.byte_add(length) };
            assert!(current.cast::<u64>().is_aligned()); // TODO: u32 in wasm32

            assert!(current < end_ptr);

            eprintln!("size: {:?} bitfields: {:?} bitfields_len: {:?} base: {:?} bitfields_nbytes: {:?}", size, bitfields, bitfields_length, base, bitfields_nbytes);
            // eprintln!("size: {:?} bitfields: {:?} base: {:?} bitfields_nbytes: {:?} nbytes:{:?}", size, bitfields, base, bitfields_nbytes, nbytes);

            Class {
                bitfields,
                base,
                length,
                elem_size: *size,
                nallocated: AtomicUsize::new(0),
                max_nallocated: AtomicUsize::new(0),
                info,
                end_ptr: current,
                free_hint: AtomicUsize::new(0),
            }
        });

        eprintln!("total_bitfields_nbytes: {:?}", total_bitfields_nbytes);
        eprintln!("remaining spaces: {:?}", unsafe { end_ptr.offset_from(current) });

        Self {
            inner: classes
        }
    }
}

#[cfg(target_family = "wasm")]
mod mmap {
    pub fn initialize() -> (*mut u8, usize) {
        todo!()
    }

    pub fn grow() {
        // no-op
    }
}

#[cfg(not(target_family = "wasm"))]
mod mmap {
    pub fn initialize() -> (*mut u8, usize) {
        /// 12 GB
        const LENGTH: usize = 4294967296 * 3;
        // 4 GB
        // const LENGTH: usize = 4294967296;

        let ptr = unsafe {
            libc::mmap(
                core::ptr::null_mut(),
                LENGTH,
                libc::PROT_WRITE | libc::PROT_READ,
                libc::MAP_ANON | libc::MAP_PRIVATE,
                -1,
                0,
            )
        };

        if ptr == libc::MAP_FAILED {
            (core::ptr::null_mut(), 0)
        } else {
            (ptr as _, LENGTH)
        }
    }

    pub fn grow() {
        // no-op
    }

    // #[thread_local]
    // pub static FOO: [&str; 1] = [ "Hello" ];

    // fn thread_id() {
    //     use core::sync::atomic::{AtomicU64, Ordering};

    //     static COUNTER: AtomicU64 = AtomicU64::new(0);

    //     thread_local! {
    //         static ID: usize = COUNTER.fetch_add(1, Ordering::Relaxed);
    //     }

    //     ID.with(|id| *id)
    // }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_alloc() {
//         let a = MinallocImpl::new();
//         dbg!("ok");
//     }
// }
