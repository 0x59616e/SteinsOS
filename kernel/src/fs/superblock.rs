use super::inode::Inode;
use super::buffer::Buffer;
use alloc::sync::Arc;
use alloc::collections::btree_map::BTreeMap;
use core::mem::MaybeUninit;

static mut INODES: MaybeUninit<BTreeMap<u32, Arc<Inode>>> = MaybeUninit::uninit();

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Superblock {
    inode_count: u32, // the amount of the inodes
    inode_bitmap:  u32,
    data_block_bitmap: u32,
}

impl Superblock {
    // root inode is at block 1
    pub fn get_inode(&self, inode_num: u32) -> Arc<Inode> {
        assert!(1 <= inode_num && inode_num <= self.inode_count);
        unsafe {
            let inode = (Buffer::read(inode_num).get_data().as_ptr() as *const Inode).read();
            INODES.assume_init_mut().insert(inode_num, Arc::new(inode));
            Arc::clone(INODES.assume_init_ref().get(&inode_num).unwrap())
        }
    }
}

pub fn init() {
    unsafe {
        INODES = MaybeUninit::new(BTreeMap::new());
    }
}