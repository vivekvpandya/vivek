
#![no_main]
#![no_std]

extern crate alloc;
use alloc::vec::Vec;

fn min_max() -> Vec<u8> {
    let min = core::cmp::min(100_u32, 1000_u32);
    let max = core::cmp::max(100_u32, 1000_u32);
    assert!(min < max);
    assert!(min > 100_u32);
    max.to_be_bytes().to_vec()
}

pub fn main() {
    let result = min_max();
    write(&result);
}

entry!(main);

#[macro_export]
macro_rules! entry {
    ($path:path) => {
        // Type check the given path
        const MOZAK_ENTRY: fn() = $path;

        mod mozak_generated_main {
            #[no_mangle]
            fn main() { super::MOZAK_ENTRY() }
        }
    };
}

#[no_mangle]
unsafe extern "C" fn __start() {
    init();
    {
        extern "C" {
            fn main();
        }
        main()
    }
    finalize();
}

#[no_mangle]
unsafe extern "C" fn _start() {
    __start();
}


mod handlers {
    use core::panic::PanicInfo;

#[cfg(not(feature = "std"))]
    #[panic_handler]
    #[no_mangle]
    fn panic_fault(panic_info: &PanicInfo) -> ! {
        // This line works as some how it avoids memcpy and hence also avoids `movaps` instruction.
        // let msg = alloc::fmt::format(format_args!("{}", panic_info));
        
        // Using following line to get msg causes SEGFAULT due to `movaps` having unaligned memory
        // address as operand.
        let msg = alloc::format!("{}", panic_info);
        unsafe {
        libc::write(1, msg.as_ptr() as *const libc::c_void, msg.len()); 
        libc::exit(255);
        }
    }
}



static mut OUTPUT_BYTES: Option<Vec<u8>> = None;

pub fn init() {
    unsafe {
        OUTPUT_BYTES = Some(Vec::new());
    }
}

pub fn finalize() {
    unsafe {
        let output_bytes_vec = OUTPUT_BYTES.as_ref().unwrap_unchecked();
        let output_0 = output_bytes_vec.first().unwrap_unchecked();
        libc::exit((*output_0).into());
    }
}

pub fn write(output_data: &[u8]) {
    let output_bytes_vec = unsafe { OUTPUT_BYTES.as_mut().unwrap_unchecked() };
    output_bytes_vec.extend_from_slice(output_data);
}

#[no_mangle]
pub extern "C" fn alloc_aligned(bytes: usize, align: usize) -> *mut u8 {
    extern "C" {
        // This symbol is defined by the loader and marks the end
        // of all elf sections, so this is where we start our
        // heap.
        //
        // This is generated automatically by the linker; see
        // https://lld.llvm.org/ELF/linker_script.html#sections-command
        static _end: u8;
    }

    // Pointer to next heap address to use, or 0 if the heap has not yet been
    // initialized.
    static mut HEAP_POS: usize = 0;

    // SAFETY: Single threaded, so nothing else can touch this while we're working.
    let mut heap_pos = unsafe { HEAP_POS };

    if heap_pos == 0 {
        heap_pos = unsafe { (&_end) as *const u8 as usize };
    }

    let offset = heap_pos & (align - 1);
    if offset != 0 {
        heap_pos += align - offset;
    }

    let ptr = heap_pos as *mut u8;
    heap_pos += bytes;

    unsafe { HEAP_POS = heap_pos };
    ptr
}

use core::alloc::{GlobalAlloc, Layout};

struct BumpPointerAlloc;

unsafe impl GlobalAlloc for BumpPointerAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        alloc_aligned(layout.size(), layout.align())
    }

    unsafe fn dealloc(&self, _: *mut u8, _: Layout) {
        // BumpPointerAlloc never deallocates memory
    }
}

#[global_allocator]
static HEAP: BumpPointerAlloc = BumpPointerAlloc;

