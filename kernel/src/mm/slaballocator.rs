use alloc::alloc::{GlobalAlloc, Layout};
use crate::common::*;
use super::buddyallocator::BuddyAllocator;

const SLAB_NODE_COUNT: usize = 8;
const MINIMUM_SLAB_SIZE_SHIFT: usize = 3;
const MINIMUM_SLAB_SIZE: usize = 1 << MINIMUM_SLAB_SIZE_SHIFT;
const MAXIMUM_SLAB_SIZE: usize = 1024;

// 8, 16, 32, 64, 128, 256, 512, 1024
static mut SLAB_LIST: [SlabNode; SLAB_NODE_COUNT]
                            = [SlabNode(core::ptr::null_mut()); SLAB_NODE_COUNT];

#[repr(transparent)]
#[derive(Clone, Copy)]
struct SlabNode(*mut SlabNode);

impl SlabNode {
    unsafe fn alloc_one(&mut self, size: usize) -> *mut u8 {
        if self.0.is_null() {
            let frame = BuddyAllocator.alloc(Layout::from_size_align_unchecked(PAGESIZE, 4));
            self.init(size, frame);
        }

        let res = self.0;
        self.0 = (*self.0).0;
        let res = res as *mut u8;
        core::slice::from_raw_parts_mut(res, size).fill(0);
        res
    }

    unsafe fn dealloc_one(&mut self, ptr: *mut u8) {
        let ptr = ptr as *mut SlabNode;
        (*ptr).0 = self.0;
        self.0 = ptr;
    }

    unsafe fn init(&mut self, size: usize, ptr: *mut u8) {   
        for i in (0..PAGESIZE).step_by(size) {
            (ptr.add(i) as *mut SlabNode).write(SlabNode(ptr.add(i + size) as *mut SlabNode));
        }
        // last one
        (ptr.add(PAGESIZE - size) as *mut SlabNode).write(SlabNode(core::ptr::null_mut()));
        self.0 = ptr as *mut Self;
    }
}

pub struct SlabAllocator;

impl SlabAllocator {
    fn get_size_and_index(size: usize) -> (usize, usize) {
        let size = round_up_with(size.next_power_of_two(), MINIMUM_SLAB_SIZE);
        let idx = (size >> MINIMUM_SLAB_SIZE_SHIFT).trailing_zeros() as usize;
        (size, idx)
    }
}

unsafe impl GlobalAlloc for SlabAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() > MAXIMUM_SLAB_SIZE {
            return BuddyAllocator.alloc(layout);
        }

        let (size, idx) = Self::get_size_and_index(layout.size());

        SLAB_LIST[idx].alloc_one(size)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if layout.size() > MAXIMUM_SLAB_SIZE {
            BuddyAllocator.dealloc(ptr, layout);
            return;
        }

        let (_, idx) = Self::get_size_and_index(layout.size());

        SLAB_LIST[idx].dealloc_one(ptr);

    }
}


