use alloc::collections::btree_map::BTreeMap;
use core::mem::MaybeUninit;
use crate::process;
use alloc::sync::Arc;
use crate::virtio;

static mut BUFFERS: MaybeUninit<BTreeMap<u32, Buffer>> = MaybeUninit::uninit();

pub struct Buffer {
    blockno: u32,
    pub disk: bool, // is this buffer in disk r/w operation ?
    data: Arc<[u8; 1024]>,
}

pub fn init() {
    unsafe {
        BUFFERS = MaybeUninit::new(BTreeMap::new());
    }
}

impl Buffer {
    pub fn read(blockno: u32) -> &'static Self {
        let buffers = unsafe {
            BUFFERS.assume_init_mut()
        };

        match buffers.contains_key(&blockno) {
            true => {
                let buffer = buffers.get(&blockno).unwrap();
                if buffer.disk { // this buffer is in disk r/w operation
                    process::sleep(buffer.get_data().as_ptr() as usize);
                }
                buffer
            }
            false => {
                unsafe {
                    let buffer = {
                        buffers.insert(blockno, Self {
                            blockno,
                            disk: true,
                            data: Arc::new([0; 1024]),
                        });
                        buffers.get_mut(&blockno).unwrap()
                    };

                    virtio::disk_rw(buffer, false);
                    buffer
                }
            }
        }
    }

    pub fn get_data(&self) -> Arc<[u8; 1024]> {
        Arc::clone(&self.data)
    }

    pub fn blockno(&self) -> u32 {
        self.blockno
    }
}