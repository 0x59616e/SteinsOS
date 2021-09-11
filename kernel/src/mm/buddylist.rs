pub static mut BUDDY_LIST: [FreeArea; 11] = [FreeArea::new(); 11];

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FreeArea {
    next: *mut FreeArea,
}

impl FreeArea {
    pub fn next(&mut self) -> &mut *mut FreeArea {
        &mut self.next
    }

    pub const fn new() -> Self {
        FreeArea {
            next: core::ptr::null_mut(),
        }
    }

    pub fn insert(&mut self, addr: usize) {
        let ptr = addr as *mut FreeArea;
        unsafe {
            ptr.write(FreeArea::new());
        }

        let mut next = &mut self.next;
        loop {
            if !(*next).is_null() && addr > (*next) as usize {
                unsafe {
                    next = &mut (**next).next;
                }
            } else {
                unsafe {
                    (*ptr).next = *next;
                    *next = ptr;
                }
                break;
            }
        }
    }

    pub fn remove(&mut self) -> usize {
        if self.next.is_null() {
            panic!("allocator should ensure the list is not empty");
        } else {
            let ptr = self.next;
            unsafe {
                self.next = (*self.next).next;
            }
            ptr as usize
        }
    }

    pub fn is_empty(&self) -> bool {
        self.next.is_null()
    }

    #[cfg(feature = "debug")]
    pub fn count_size(&self, mut sum: usize, sz: usize) -> usize {
        let mut runner = self.next;
        while !runner.is_null() {
            sum += sz;
            unsafe {
                runner = (*runner).next;
            }
        }
        sum
    }
}
