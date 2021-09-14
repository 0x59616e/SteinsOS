use super::*;
use crate::mm::buddylist::{BUDDY_LIST, FreeArea};
use alloc::alloc::{GlobalAlloc, Layout};

#[derive(Clone, Copy, Default)]
pub struct BuddyAllocator;

impl BuddyAllocator {
    pub fn request(order: usize, list: &mut [FreeArea; 11]) -> Result<usize, ()> {
        if order > 10 {
            return Err(());
        }

        if list[order].is_empty() {
            let ptr = Self::request(order + 1, list)?;
            list[order].insert(ptr + (1 << (PAGESHIFT + order)));
            return Ok(ptr);
        }

        Ok(list[order].remove())
    }

    pub fn free(mut start: usize, end: usize) {
        while start < end {
            // determine the order, maximum is 10
            let mut order = (start >> PAGESHIFT).trailing_zeros() as usize;
            while order > 10 || start + (1 << (order + PAGESHIFT)) > end {
                order -= 1;
            }
            unsafe {
                BUDDY_LIST[order].insert(start);
            }
            start +=  1 << (PAGESHIFT + order);
        }
        assert_eq!(start, end);
    }

    pub fn merge(lists: &mut [FreeArea; 11]) {
        for order in 0..10 {
            let mut ptr = lists[order].next() as *mut *mut FreeArea;

            unsafe {
                loop {
                    if (*ptr).is_null() || (**ptr).next().is_null() {
                        break;
                    }

                    let ptr1 = *ptr as usize;
                    let ptr2 = *(**ptr).next() as usize;

                    if ((ptr1 ^ ptr2) >> PAGESHIFT) ^ (1 << order) == 0 {
                        // They're buddy
                        *ptr = *(*(ptr2 as *mut FreeArea)).next();
                        lists[order + 1].insert(ptr1);
                    } else {
                        ptr = (**ptr).next();
                    }
                }
            }
        }
    }
}
unsafe impl FrameAlloc for BuddyAllocator {
    unsafe fn alloc_pages(&self, pg_cnt: usize) -> *mut u8 {

        let order = pg_cnt.next_power_of_two().trailing_zeros() as usize;

        let res = {
            Self::request(order, &mut BUDDY_LIST)
        };

        if let Ok(ptr) = res {
            Self::free(ptr + pg_cnt * PAGESIZE, ptr + (1 << (order + PAGESHIFT)));
            // initialized
            let ptr = ptr as *mut u8;
            core::slice::from_raw_parts_mut(ptr, pg_cnt * PAGESIZE).fill(0);
            ptr
        } else {
            core::ptr::null_mut()
        }
    }

    unsafe fn dealloc_pages(&self, ptr: *mut u8, pg_cnt: usize) {
        let addr = ptr as usize;
        let sz = pg_cnt * PAGESIZE;

        assert!(addr & (PAGESIZE - 1) == 0);

        BuddyAllocator::free(addr, addr + sz);

        BuddyAllocator::merge(&mut BUDDY_LIST);

    }
}

unsafe impl GlobalAlloc for BuddyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        assert!(layout.size() > 0);

        let sz = round_up(layout.size());

        let ptr = self.alloc_pages(sz >> PAGESHIFT);
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let sz = round_up(layout.size());

        self.dealloc_pages(ptr, sz >> PAGESHIFT);
    }

    unsafe fn realloc(
        &self,
        ptr: *mut u8,
        layout: Layout,
        new_size: usize
    ) -> *mut u8
    {
        let original_page_cnt = round_up(layout.size());
        let new_page_cnt      = round_up(new_size);

        if original_page_cnt == new_page_cnt {
            ptr
        } else {
            let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());
            
            let new_ptr = self.alloc(new_layout);

            if !new_ptr.is_null() {
                // SAFETY: the previously allocated block cannot overlap the newly allocated block.
                // The safety contract for `dealloc` must be upheld by the caller.
                core::ptr::copy_nonoverlapping(ptr, new_ptr, core::cmp::min(layout.size(), new_size));
                self.dealloc(ptr, layout);
            }
            new_ptr
        }
    }
}
