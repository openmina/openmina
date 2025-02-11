#![no_std]

use core::{
    alloc::{GlobalAlloc, Layout},
    num::NonZeroU64,
    ptr::NonNull,
    sync::atomic::{
        AtomicBool, AtomicPtr, AtomicU64, AtomicUsize,
        Ordering::{AcqRel, Acquire, Relaxed, Release},
    },
};

use libc_print::{libc_dbg as dbg, libc_eprintln as eprintln};

struct ClassInfo {
    size: usize,
    nelems: usize,
}

static N_OP: AtomicUsize = AtomicUsize::new(0);
static TOTAL_NBYTES: AtomicUsize = AtomicUsize::new(0);

const N_CLASSES: usize = 32;

#[rustfmt::skip]
const CLASSES: [ClassInfo; N_CLASSES] = [
    ClassInfo { size: 16, nelems: 20_00 },
    ClassInfo { size: 24, nelems: 30_00 },
    ClassInfo { size: 32, nelems: 2_000_00 },
    ClassInfo { size: 48, nelems: 50_00 },
    ClassInfo { size: 64, nelems: 1_000_00 },
    ClassInfo { size: 80, nelems: 80_00 },
    ClassInfo { size: 96, nelems: 20_00 },
    ClassInfo { size: 112, nelems: 20_00 },
    ClassInfo { size: 128, nelems: 20_00 },
    ClassInfo { size: 160, nelems: 300_00 },
    ClassInfo { size: 192, nelems: 20_00 },
    ClassInfo { size: 224, nelems: 60_00 },
    ClassInfo { size: 256, nelems: 200_00 },
    ClassInfo { size: 320, nelems: 50_00 },
    ClassInfo { size: 384, nelems: 100_00 },
    ClassInfo { size: 448, nelems: 20_00 },
    ClassInfo { size: 512, nelems: 500_00 },
    ClassInfo { size: 1024, nelems: 50_00 },
    ClassInfo { size: 1280, nelems: 10_00 },
    ClassInfo { size: 1536, nelems: 30_00 },
    ClassInfo { size: 1792, nelems: 10_00 },
    ClassInfo { size: 2048, nelems: 50_00 },
    ClassInfo { size: 3072, nelems: 50_00 },
    ClassInfo { size: 4096, nelems: 50_00 },
    ClassInfo { size: 8192, nelems: 50_00 },
    ClassInfo { size: 16384, nelems: 30_00 },
    ClassInfo { size: 65536, nelems: 20_00 },
    ClassInfo { size: 0x100000, nelems: 20 },
    ClassInfo { size: 0x1000000, nelems: 20 },
    ClassInfo { size: 0x2000000, nelems: 10 },
    ClassInfo { size: 0x3000000, nelems: 10 },
    ClassInfo { size: 0x10000000, nelems: 5 },
];

// [STATS-780] class size:16 max:8701 current:8522 max_since_last:8555
// [STATS-780] class size:24 max:12950 current:12418 max_since_last:12474
// [STATS-780] class size:32 max:1418131 current:1399429 max_since_last:1416208
// [STATS-780] class size:48 max:31395 current:23872 max_since_last:24487
// [STATS-780] class size:64 max:592771 current:127434 max_since_last:127442
// [STATS-780] class size:80 max:73541 current:57574 max_since_last:57851
// [STATS-780] class size:96 max:13138 current:3352 max_since_last:3370
// [STATS-780] class size:112 max:86 current:15 max_since_last:42
// [STATS-780] class size:128 max:940 current:670 max_since_last:704
// [STATS-780] class size:160 max:5807 current:5709 max_since_last:5740
// [STATS-780] class size:192 max:4056 current:587 max_since_last:620
// [STATS-780] class size:224 max:11929 current:11915 max_since_last:11922
// [STATS-780] class size:256 max:2429 current:154 max_since_last:158
// [STATS-780] class size:320 max:39992 current:39685 max_since_last:39961
// [STATS-780] class size:384 max:455 current:378 max_since_last:384
// [STATS-780] class size:448 max:10827 current:10804 max_since_last:10827
// [STATS-780] class size:512 max:229507 current:218130 max_since_last:218136
// [STATS-780] class size:1024 max:31831 current:30060 max_since_last:30069
// [STATS-780] class size:1280 max:5877 current:5852 max_since_last:5856
// [STATS-780] class size:1536 max:17593 current:17564 max_since_last:17591
// [STATS-780] class size:1792 max:5512 current:2570 max_since_last:2590
// [STATS-780] class size:2048 max:27458 current:22688 max_since_last:22898
// [STATS-780] class size:3072 max:43727 current:34855 max_since_last:34863
// [STATS-780] class size:4096 max:12111 current:11755 max_since_last:11757
// [STATS-780] class size:8192 max:742 current:444 max_since_last:450
// [STATS-780] class size:16384 max:16000 current:15444 max_since_last:15454
// [STATS-780] class size:65536 max:3230 current:3126 max_since_last:3220
// [STATS-780] class size:1048576 max:166 current:61 max_since_last:70
// [STATS-780] class size:16777216 max:176 current:10 max_since_last:13
// [STATS-780] class size:33554432 max:5 current:5 max_since_last:0
// [STATS-780] class size:50331648 max:3 current:0 max_since_last:0
// [STATS-780] class size:268435456 max:1 current:0 max_since_last:0
// [STATS-780] TOTAL:835846079

struct Class {
    elem_size: usize,
    bitfields: NonNull<[AtomicU64]>,
    base: NonNull<u8>,
    /// In bytes
    length: usize,
    nallocated: AtomicUsize,
    max_nallocated: AtomicUsize,
    max_nallocated_since_last: AtomicUsize,
    info: &'static ClassInfo,
    end_ptr: NonNull<u8>,
    // TODO: Make this thread-local
    free_hint: AtomicUsize,
}

struct Classes {
    inner: [Class; N_CLASSES],
    /// end of `Classes` arena
    end_ptr: NonNull<u8>,
}

const HEADER_SIZE: usize = core::mem::size_of::<u64>();

#[derive(Clone)]
struct HeaderPtr {
    ptr: NonNull<u64>,
}

impl HeaderPtr {
    fn new(ptr: *mut u64) -> Option<Self> {
        assert!(ptr.is_aligned());
        Some(Self {
            ptr: NonNull::new(ptr)?,
        })
    }

    fn read(&self) -> Header {
        let atomic = unsafe { AtomicU64::from_ptr(self.ptr.as_ptr()) };
        let header: u64 = atomic.load(Acquire);
        Header::of_u64(header)
    }

    fn write(&self, header: Header) {
        let atomic = unsafe { AtomicU64::from_ptr(self.ptr.as_ptr()) };
        atomic.store(header.to_u64(), Release);
    }

    /// Returns `true` if it was acquired
    fn acquire(&self) -> bool {
        let atomic = unsafe { AtomicU64::from_ptr(self.ptr.as_ptr()) };
        let previous = atomic.fetch_and(!(1 << 63), AcqRel);
        let was_free = (previous >> 63) != 0;
        // eprintln!("acquire was_free:{:?}", was_free);
        was_free
    }

    fn as_ptr(&self) -> *mut u64 {
        self.ptr.as_ptr()
    }

    fn as_nonnull_ptr(&self) -> NonNull<u64> {
        self.ptr
    }
}

#[derive(Clone)]
struct Header {
    /// MSB to LSB:
    /// 1 bit: is_free
    /// 5 bits: class index
    /// 1 bit: is_offset
    /// 9 bits: unused,
    /// 48 bits: next ptr to free Header, or offset
    is_free: bool,
    class_index: usize,
    is_offset: bool,
    next_free: Option<NonZeroU64>, // or offset
}

impl Header {
    fn of_u64(value: u64) -> Self {
        let is_free = (value >> 63) != 0;
        let class_index = (value >> 58) as usize & 0b11111;
        // let class_index = (value >> 58) as usize & 0x11111;
        // eprintln!("header: {:064b} class_index:{:?} (value >> 58):{:?}", value, class_index, value >> 58);
        let is_offset = (value >> 57) & 1 != 0;
        let next_ptr = {
            // 48 bits
            value & 0xFFFFFFFFFFFF
            // let [_, _, a, b, c, d, e, f] = value.to_be_bytes();
            // // TODO: Handle 32 bits pointers
            // let ptr = usize::from_be_bytes([0, 0, a, b, c, d, e, f]);
            // NonNull::new(ptr as *mut Header)
        };
        Self {
            is_free,
            class_index,
            is_offset,
            next_free: NonZeroU64::new(next_ptr),
        }
    }

    fn to_u64(&self) -> u64 {
        let Self {
            is_free,
            class_index,
            is_offset,
            next_free,
        } = *self;

        (is_free as u64) << 63
            | ((class_index & 0b11111) as u64) << 58
            | (is_offset as u64) << 57
            | next_free.map(NonZeroU64::get).unwrap_or(0u64)
    }

    fn new(class_index: usize) -> Self {
        assert!(class_index & 0b11111 == class_index);
        Self {
            is_free: false,
            class_index,
            is_offset: false,
            next_free: None,
        }
    }

    fn new_offset(offset: u64) -> Self {
        assert_ne!(offset, 0);

        let [unused1, unused2, ..] = (offset as u64).to_be_bytes();
        // 2 most significant bytes are not set
        assert_eq!([unused1, unused2], [0, 0]);

        Self {
            is_free: false,
            class_index: 0,
            is_offset: true,
            next_free: NonZeroU64::new(offset),
        }
    }

    fn get_offset(&self) -> Option<usize> {
        self.is_offset.then(|| {
            self.next_free
                .map(NonZeroU64::get)
                .unwrap_or(0u64)
                .try_into()
                .unwrap()
        })
    }

    fn set_free(&mut self) {
        self.is_free = true;
    }

    fn set_next_free(&mut self, next: *mut Header) {
        // TODO: 32 bits pointers
        assert_eq!(HEADER_SIZE, 8);

        let [unused1, unused2, ..] = (next as u64).to_be_bytes();
        // 2 most significant bytes are not set
        assert_eq!([unused1, unused2], [0, 0]);

        self.next_free = NonZeroU64::new(next as u64);
    }
}

struct FreeList {
    elem_size: usize,
    next_free: AtomicPtr<Header>,
    nitems: AtomicUsize,
}

struct MinallocImpl {
    base: NonNull<u8>,
    base_len: usize,
    classes: Classes,
    current: AtomicPtr<u8>,
    lists: [FreeList; N_CLASSES],
}

static mut MINALLOC_STORAGE: Option<MinallocImpl> = None;
static MINALLOC_PTR: AtomicPtr<MinallocImpl> = AtomicPtr::new(core::ptr::null_mut());

pub struct Minalloc;

unsafe impl GlobalAlloc for Minalloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let n_op = N_OP.load(Relaxed);
        // eprintln!("  alloc[{:?}] size:{:?} [...]", n_op, layout.size());
        let res = self.get().alloc(&layout);
        // eprintln!("  alloc[{:?}] size:{:?} ptr:{:?}", n_op, layout.size(), res);
        res
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let n_op = N_OP.load(Relaxed);
        // eprintln!("dealloc[{:?}] size:{:?} ptr:{:?}", n_op, layout.size(), ptr);
        self.get().dealloc(ptr, &layout);
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
        let lists = Self::make_lists();

        Self {
            base,
            base_len,
            current: AtomicPtr::new(classes.end_ptr.as_ptr()),
            classes,
            lists,
        }
    }

    fn make_lists() -> [FreeList; 32] {
        CLASSES.each_ref().map(|info| {
            FreeList {
                elem_size: info.size,
                next_free: AtomicPtr::default(), // null
                nitems: AtomicUsize::new(0),
            }
        })
    }

    fn alloc(&self, layout: &Layout) -> *mut u8 {
        self.show_stats(false);

        if layout.size() == 0 {
            return NonNull::dangling().as_ptr();
        }

        TOTAL_NBYTES.fetch_add(layout.size(), Relaxed);
        let _n_op = N_OP.fetch_add(1, AcqRel);

        let size = if layout.align() > 8 {
            layout.size() + layout.align()
        } else {
            layout.size()
        };

        // assert!(layout.align() <= 8, "size: {:?} align: {:?}", layout.size(), layout.align());
        // eprintln!("alloc size: {:?} align: {:?}", layout.size(), layout.align());

        // for class in &self.classes.inner {
        //     if size <= class.elem_size {
        //         match class.take_next(layout) {
        //             Some(ptr) => return ptr.as_ptr(),
        //             None => break,
        //         }
        //     }
        // }

        self.alloc_in_list(size, layout).as_ptr()

        // eprintln!("size too big: size:{:?} align:{:?}", layout.size(), layout.align());

        // todo!() // Implement linked list based allocations
    }

    fn dealloc(&self, ptr: *mut u8, layout: &Layout) {
        // dbg!(&layout);

        if layout.size() == 0 {
            return;
        }

        TOTAL_NBYTES.fetch_sub(layout.size(), Relaxed);
        let n_op = N_OP.fetch_add(1, AcqRel);

        let size = if layout.align() > 8 {
            layout.size() + layout.align()
        } else {
            layout.size()
        };

        // if ptr < self.classes.end_ptr.as_ptr() {
        //     for class in &self.classes.inner {
        //         if size <= class.elem_size {
        //             return class.free(ptr, layout);
        //         }
        //     }
        //     todo!()
        // }

        self.dealloc_in_list(ptr);

        // eprintln!("DEALLOC size too big: ptr:{:?} size:{:?} align:{:?}", ptr, layout.size(), layout.align());

        // todo!() // Implement linked list based allocations
    }

    fn find_header(ptr: *mut u8) -> (HeaderPtr, Header) {
        let header_ptr = unsafe { ptr.byte_sub(HEADER_SIZE) }.cast::<u64>();
        let mut header_ptr = HeaderPtr::new(header_ptr).unwrap();
        let mut header = header_ptr.read();

        if let Some(offset) = header.get_offset() {
            let ptr = unsafe { ptr.byte_sub(offset).byte_sub(HEADER_SIZE) }.cast::<u64>();
            header_ptr = HeaderPtr::new(ptr).unwrap();
            header = header_ptr.read();
        }

        (header_ptr, header)
    }

    fn dealloc_in_list(&self, ptr: *mut u8) {
        let (header_ptr, mut header) = Self::find_header(ptr);

        // eprintln!("header: (is_free, class_index, next):{:?}", header.read());

        let Header {
            is_free,
            class_index,
            is_offset,
            next_free,
        } = header;
        // let (is_free, class_index, _) = header.read();
        assert!(!is_free);
        assert!(!is_offset);

        header.set_free();

        // let (is_free, class_index, _) = header.read();

        let list = &self.lists[class_index];

        list.nitems.fetch_sub(1, Relaxed);

        let mut next_free = list.next_free.load(Acquire);

        loop {
            // eprintln!("next_free: {:?} this_header:{:?}", next_free, header as *const Header);
            assert_ne!(header_ptr.as_ptr() as *mut Header, next_free);
            // eprintln!("next_free: {:?} bytes:{:?} be_bytes:{:?}", next_free, (next_free as u64).to_ne_bytes(), (next_free as u64).to_be_bytes());

            header.set_next_free(next_free);
            header_ptr.write(header.clone());

            // let (_, _, ptr) = header_ptr.read().read();
            {
                let Header {
                    next_free: ptr,
                    is_free,
                    class_index: ci,
                    is_offset,
                } = header_ptr.read();
                assert_eq!(
                    ptr.map(|p| p.get() as _).unwrap_or(core::ptr::null_mut()),
                    next_free
                );
                assert!(is_free);
                assert_eq!(ci, class_index);
                assert!(!is_offset);
            }

            match list.next_free.compare_exchange(
                next_free,
                header_ptr.as_ptr() as *mut Header,
                AcqRel,
                Acquire,
            ) {
                Ok(previous) => {
                    assert_eq!(previous, next_free);
                }
                Err(previous) => {
                    next_free = previous;
                    assert_ne!(header_ptr.as_ptr() as *mut Header, next_free);
                    continue;
                }
            }

            return;
        }
    }

    fn alloc_in_list(&self, size: usize, layout: &Layout) -> NonNull<u8> {
        let mut first_matching_size = None;

        // eprintln!("alloc_in_list {:?}", size);

        // skip_while
        // take_while
        // take(2)

        // self.lists
        //     .iter()
        //     .enumerate()
        //     .skip_while(|(index, free_list)| size > free_list.elem_size)
        // .take_while(|(index, free_list)| size * 2 > )

        for (index, free_list) in self.lists.iter().enumerate() {
            if size <= free_list.elem_size {
                if first_matching_size.is_none() {
                    first_matching_size = Some((index, free_list.elem_size));
                }
                // if size * 2 <= free_list.elem_size {
                //     break; // Don't go too far in our lists, allocate new ones instead
                // }
                if let Some(acquired_header) = self.try_take_in_list(free_list) {
                    // eprintln!("AAA");
                    return self.get_from_header(acquired_header, layout, index);
                } else {
                    break;
                }
            }
        }

        let (index_free_list, first_matching_size) = first_matching_size.unwrap();

        // eprintln!("alloc_in_list: grow current {:?}", size);

        let slot_size = first_matching_size + HEADER_SIZE;
        let mut current = self.current.load(Acquire);

        loop {
            let new_current = unsafe { current.byte_add(slot_size) };

            // eprintln!("current:{:?}", current);
            if new_current < self.end_ptr().as_ptr() {
                match self
                    .current
                    .compare_exchange(current, new_current, AcqRel, Acquire)
                {
                    Ok(previous) => {
                        assert_eq!(previous, current);
                    }
                    Err(previous) => {
                        current = previous;
                        continue;
                    }
                }

                let header_ptr = HeaderPtr::new(current.cast()).unwrap();
                header_ptr.write(Header::new(index_free_list));

                self.lists[index_free_list].nitems.fetch_add(1, Relaxed);

                return self.get_from_header(header_ptr, layout, index_free_list);
            } else {
                break;
            }
        }

        let current = self.current.load(Relaxed);
        let offset = unsafe { current.offset_from(self.base.as_ptr()) };

        let n_op = N_OP.load(Relaxed);

        eprintln!(
            "[{:?}] implement grow base:{:?} current:{:?} diff:{:?} size:{:?}",
            n_op, self.base, current, offset, size
        );

        self.show_stats(true);

        todo!("implement grow")
    }

    fn end_ptr(&self) -> NonNull<u8> {
        unsafe { self.base.byte_add(self.base_len) }
    }

    fn get_from_header(
        &self,
        header_ptr: HeaderPtr,
        layout: &Layout,
        class_index: usize,
    ) -> NonNull<u8> {
        let ptr = self.get_from_header_impl(header_ptr.clone(), layout, class_index);

        {
            // Making sure we can retrieve the header
            let (hdr_ptr, header) = Self::find_header(ptr.as_ptr());
            assert_eq!(hdr_ptr.as_ptr(), header_ptr.as_ptr());
            assert_eq!(header.class_index, class_index);

            let end_of_obj = unsafe { ptr.byte_add(layout.size()) };
            let end_of_dedicated = {
                let hdr_ptr = hdr_ptr.as_nonnull_ptr().cast::<u8>();
                unsafe { hdr_ptr.byte_add(HEADER_SIZE + self.lists[class_index].elem_size) }
            };
            assert!(end_of_obj <= end_of_dedicated);
        }

        ptr
    }

    fn get_from_header_impl(
        &self,
        header_ptr: HeaderPtr,
        layout: &Layout,
        class_index: usize,
    ) -> NonNull<u8> {
        let Header {
            is_free,
            class_index: header_class_index,
            is_offset,
            next_free,
        } = header_ptr.read();

        {
            // let (_is_free, header_class_index, _next) = header.read();
            if header_class_index != class_index {
                eprintln!(
                    "header_class_index:{:?} class_index:{:?}",
                    header_class_index, class_index
                );
                assert_eq!(header_class_index, class_index);
            }
            // eprintln!("header: (is_free, class_index, next):{:?}", header.read());
        }

        let mut ptr = unsafe { header_ptr.as_nonnull_ptr().add(1) }.cast::<u8>();

        if layout.align() <= 8 {
            // assert!(ptr )
            return ptr;
        }

        // align > 8
        let offset = ptr.align_offset(layout.align());
        if offset == 0 {
            return ptr;
        }
        assert!(offset >= 8);

        // eprintln!("offset:{:?} layout.size:{:?} layout.align:{:?}", offset, layout.size(), layout.align());
        ptr = unsafe { ptr.byte_add(offset) };

        {
            let header_ptr = unsafe { ptr.byte_sub(HEADER_SIZE) }.cast::<u64>();
            let header_ptr = HeaderPtr::new(header_ptr.as_ptr()).unwrap();
            header_ptr.write(Header::new_offset(offset as u64));
        }

        ptr
    }

    fn try_take_in_list(&self, free_list: &FreeList) -> Option<HeaderPtr> {
        let mut header_ptr: HeaderPtr = {
            let next = free_list.next_free.load(Acquire);
            HeaderPtr::new(next.cast())?
        };

        loop {
            let Header {
                is_free: was_free,
                class_index,
                is_offset,
                next_free,
            } = header_ptr.read();
            // let (was_free, _class_index, next_free_ptr) = header.read();

            let next_free_ptr = next_free
                .map(|ptr| ptr.get() as *mut Header)
                .unwrap_or(core::ptr::null_mut());

            match free_list.next_free.compare_exchange(
                header_ptr.as_ptr() as *mut Header,
                next_free_ptr,
                AcqRel,
                Acquire,
            ) {
                Ok(previous) => {
                    assert_eq!(previous, header_ptr.as_ptr() as *mut _);
                }
                Err(previous) => {
                    header_ptr = HeaderPtr::new(previous.cast())?;
                    continue;
                }
            }

            let Header {
                is_free,
                class_index,
                is_offset,
                next_free,
            } = header_ptr.read();
            // let (is_free, class_index, next_free_ptr) = header.read();

            // let (is_free, _class_index, _next_free_ptr) = next_header.read();

            let is_acquired = header_ptr.acquire();

            if is_free && is_acquired {
                free_list.nitems.fetch_add(1, Relaxed);
                return Some(header_ptr);
            }

            static IS_PANICKING: AtomicBool = AtomicBool::new(false);

            loop {
                if IS_PANICKING.swap(true, AcqRel) {
                    continue; // spin loop
                }
                eprintln!("\npanic here is_free:{:?} is_acquired:{:?} was_free:{:?} class_index:{:?} next_free:{:?}\n", is_free, is_acquired, was_free, class_index, next_free_ptr);
                IS_PANICKING.store(false, Release);
            }

            panic!()
        }
    }

    fn show_stats(&self, force: bool) {
        const INTERVAL: usize = 200_000;

        let n_op = N_OP.load(Relaxed);
        if n_op % INTERVAL != 0 && !force {
            return;
        }

        let index = n_op / INTERVAL;

        // for class in &self.classes.inner {
        //     eprintln!(
        //         "[STATS-{:?}] class size:{:?} max:{:?} current:{:?} max_since_last:{:?}",
        //         index, class.elem_size, class.max_nallocated.load(Relaxed), class.nallocated.load(Relaxed), class.max_nallocated_since_last.load(Acquire),
        //     );
        //     class.max_nallocated_since_last.store(0, Release);
        // }

        for list in &self.lists {
            eprintln!(
                "[STATS-{:?}] class size:{:?} nitems:{:?}",
                index, list.elem_size, list.nitems,
            );
            // class.max_nallocated_since_last.store(0, Release);
        }

        eprintln!("[STATS-{:?}] TOTAL:{:?}", index, TOTAL_NBYTES.load(Relaxed));
    }
}

impl Class {
    fn take_next(&self, layout: &Layout) -> Option<NonNull<u8>> {
        let Self {
            bitfields,
            base,
            length,
            elem_size,
            nallocated,
            max_nallocated,
            info,
            free_hint,
            end_ptr,
            max_nallocated_since_last,
        } = self;

        let nallocated = nallocated.fetch_add(1, Relaxed) + 1;
        if nallocated > max_nallocated.load(Relaxed) {
            max_nallocated.store(nallocated, Relaxed);
        }
        if nallocated > max_nallocated_since_last.load(Relaxed) {
            max_nallocated_since_last.store(nallocated, Relaxed);
        }

        // if nallocated >= info.nelems {
        //     eprintln!("\n\nnallocated:{:?} info.nelems:{:?} size:{:?} align:{:?}\n", nallocated, info.nelems, layout.size(), layout.align());
        // }

        // assert!(nallocated < info.nelems, "\n\nnallocated:{:?} info.nelems:{:?} size:{:?} align:{:?}\n", nallocated, info.nelems, layout.size(), layout.align());

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

                assert!(
                    unsafe { res.byte_add(layout.size()) } <= {
                        let (bitfield_index, index_free) = if index_free == 63 {
                            (bitfield_index + 1, 0)
                        } else {
                            assert!(index_free < 63);
                            (bitfield_index, index_free + 1)
                        };
                        self.get(bitfield_index, index_free)
                    }
                );
                assert_eq!(
                    (bitfield_index, index_free),
                    self.compute_offsets(res.as_ptr(), "alloc")
                );
                assert_eq!(
                    res.as_ptr() as usize % layout.align(),
                    0,
                    "ptr: {:?} align: {:?} size: {:?} ptr_unmod: {:?}",
                    res,
                    layout.align(),
                    layout.size(),
                    ptr_unmodified
                );
                // eprintln!("alloc size: {:?} align: {:?}", layout.size(), layout.align());
                // eprintln!(
                //     "[{:?}] alloc: base: {:?} size: {:?} align: {:?} bitfield_index:{:?} index_free:{:?} ptr: {:?} offset: {:?} nallocated:{:?}",
                //     n_op, self.base, layout.size(), layout.align(), bitfield_index, index_free, res, unsafe { res.offset_from(self.base) }, nallocated
                // );
                return Some(res);
            }
        }

        None

        // let nallocated = self.nallocated.load(Acquire);

        // eprintln!("\nallocated:{:?} info.nelems:{:?} size:{:?} align:{:?}\n", nallocated, info.nelems, layout.size(), layout.align());

        // panic!("limit reached {:?}", elem_size);
    }

    fn get(&self, bitfield_index: usize, bit: usize) -> NonNull<u8> {
        let bitfield_index = bitfield_index * self.elem_size;
        let bit = bit * self.elem_size;
        unsafe { self.base.byte_add((bitfield_index * 64) + bit) }
    }

    fn compute_offsets(&self, ptr: *mut u8, from: &str) -> (usize, usize) {
        let Self {
            elem_size,
            base,
            end_ptr,
            ..
        } = self;
        if !(ptr >= base.as_ptr() && ptr < end_ptr.as_ptr()) {
            eprintln!(
                "{} Invalid class PTR ptr:{:?} base:{:?} end_ptr:{:?} size:{:?}",
                from, ptr, base, end_ptr, elem_size
            );
            assert!(ptr >= base.as_ptr() && ptr < end_ptr.as_ptr());
        }
        let offset = (ptr as usize).checked_sub(base.as_ptr() as usize).unwrap();
        let offset = offset / *elem_size;
        let bitfield_index = offset / 64;
        let bit_index = offset % 64;
        (bitfield_index, bit_index)
    }

    fn free(&self, ptr: *mut u8, layout: &Layout) {
        let Self {
            elem_size,
            bitfields,
            base,
            length,
            nallocated,
            info,
            max_nallocated,
            free_hint,
            end_ptr,
            max_nallocated_since_last,
        } = self;

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
            let mut bitfields: NonNull<[AtomicU64]> =
                NonNull::slice_from_raw_parts(bitfields.cast::<AtomicU64>(), bitfields_length);

            // dbg!(bitfields_length);

            let bitfields_nbytes = {
                let bitfields: &mut [AtomicU64] = unsafe { bitfields.as_mut() };
                let bitfields: &mut [u64] = unsafe { core::mem::transmute(bitfields) };
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

            eprintln!(
                "size: {:?} bitfields: {:?} bitfields_len: {:?} base: {:?} bitfields_nbytes: {:?}",
                size, bitfields, bitfields_length, base, bitfields_nbytes
            );
            // eprintln!("size: {:?} bitfields: {:?} base: {:?} bitfields_nbytes: {:?} nbytes:{:?}", size, bitfields, base, bitfields_nbytes, nbytes);

            Class {
                bitfields,
                base,
                length,
                elem_size: *size,
                nallocated: AtomicUsize::new(0),
                max_nallocated: AtomicUsize::new(0),
                max_nallocated_since_last: AtomicUsize::new(0),
                info,
                end_ptr: current,
                free_hint: AtomicUsize::new(0),
            }
        });

        eprintln!("total_bitfields_nbytes: {:?}", total_bitfields_nbytes);
        eprintln!("remaining spaces: {:?}", unsafe {
            end_ptr.offset_from(current)
        });
        eprintln!("end_ptr: {:?}", current);

        Self {
            inner: classes,
            end_ptr: current,
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
        /// 16 GB
        const LENGTH: usize = 4294967296 * 4;
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
