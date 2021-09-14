use super::inode::Inode;
use super::buffer::Buffer;
use alloc::sync::Arc;
use alloc::collections::btree_map::BTreeMap;
use core::mem::MaybeUninit;
use alloc::string::String;

static mut INODES: MaybeUninit<BTreeMap<u32, Arc<Inode>>> = MaybeUninit::uninit();
static mut PATH:   MaybeUninit<BTreeMap<u32, String>> = MaybeUninit::uninit();

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Superblock {
    root_inode: u32,
    inode_bitmap:  u32,
    data_block_bitmap: u32,
}

impl Superblock {
    pub fn get_root_inode(&self) -> Arc<Inode> {
        self.get_inode(self.root_inode)
    }

    pub fn get_inode(&self, inode_num: u32) -> Arc<Inode> {
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
        PATH   = MaybeUninit::new(BTreeMap::new());
    }
}