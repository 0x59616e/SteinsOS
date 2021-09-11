#![no_std]
#![feature(asm)]
#![feature(alloc_error_handler)]

mod common;
mod fs;
mod gic;
#[macro_use]
mod serial;
mod syscall;
mod mm;
mod process;
mod exception;
mod virtio;
mod vm;

extern crate alloc;

use mm::slaballocator::SlabAllocator;

#[global_allocator]
static ALLOCATOR: SlabAllocator = SlabAllocator;

#[no_mangle]
pub unsafe extern "C" fn start(
    kernel_tt:       usize,
    kernel_text_end: usize,
    user_entry:      usize,
) -> ! {
    // physical memory management initializing
    mm::init();

    // interrupt controller
    gic::init();

    // buffer init
    fs::init();

    // enable timer
    gic::irq_enable(30);

    // uart init
    serial::init();

    // enable uart irq
    gic::irq_enable(33);

    // virtual memory initialization
    vm::init(kernel_tt, kernel_text_end);

    // virtio init
    virtio::init();

    // init first process
    process::init_first(user_entry);

    // time to go
    process::schedule();
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[alloc_error_handler]
fn on_oom(_layout: core::alloc::Layout) -> ! {
    loop {}
}
