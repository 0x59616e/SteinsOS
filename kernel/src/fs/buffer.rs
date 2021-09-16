use alloc::collections::btree_map::BTreeMap;
use core::mem::MaybeUninit;
use crate::{fs::BLOCK_SIZE, process};
use alloc::boxed::Box;
use crate::virtio;

static mut BUFFERS: MaybeUninit<BTreeMap<u32, Buffer>> = MaybeUninit::uninit();

pub struct Buffer {
    blockno: u32,
    pub busy: bool, // is this buffer in disk r/w operation ?
    data: Box<[u8; 1024]>,
}

pub fn init() {
    unsafe {
        BUFFERS = MaybeUninit::new(BTreeMap::new());
    }
}

impl Buffer {
    pub unsafe fn read(blockno: u32) -> &'static mut Self {
        let buffers = BUFFERS.assume_init_mut();

        match buffers.contains_key(&blockno) {
            true => {
                let buffer = buffers.get_mut(&blockno).unwrap();
                if buffer.busy { // this buffer is in disk r/w operation
                    process::sleep(buffer.as_ptr() as usize);
                }
                buffer
            }
            false => {
                let buffer = {
                    buffers.insert(blockno, Self {
                        blockno,
                        busy: true,
                        data: Box::new([0; 1024]),
                    });
                    buffers.get_mut(&blockno).unwrap()
                };

                virtio::disk_rw(buffer, false);
                buffer
            }
        }
    }

    pub fn write(&mut self, pos: usize, buf: &[u8]) {
        assert!(pos + buf.len() <= BLOCK_SIZE);
        self.data[pos..buf.len()].copy_from_slice(buf);
    }

    pub fn as_ptr(&self) -> *const [u8; 1024] {
        self.data.as_ptr() as *const [u8; 1024]
    }

    pub fn as_mut_ptr(&mut self) -> *mut [u8; 1024] {
        self.data.as_mut_ptr() as *mut [u8; 1024]
    }

    pub fn blockno(&self) -> u32 {
        self.blockno
    }
}
