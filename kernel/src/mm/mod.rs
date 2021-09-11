pub mod buddyallocator;
pub mod slaballocator;
mod buddylist;

use crate::common::*;

extern "C" {
    static kernel_end: u8;
    // static etext: u8;
}

pub unsafe trait FrameAlloc {
    unsafe fn alloc_pages(&self, pg_cnt: usize) -> *mut u8;
    unsafe fn dealloc_pages(&self, ptr: *mut u8, pg_cnt: usize);
}

pub fn init() {
    let start = unsafe { round_up(&kernel_end as *const _ as usize) };
    let end = round_down(PHYEND);

    buddyallocator::BuddyAllocator::free(start, end);
}