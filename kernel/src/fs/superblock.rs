use super::inode::Inode;
use super::buffer::Buffer;
use alloc::collections::btree_map::BTreeMap;
use core::mem::MaybeUninit;
use alloc::string::String;

static mut PATH_NAME:   MaybeUninit<BTreeMap<u32, String>> = MaybeUninit::uninit();

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Superblock {
    root_inode: u32,
    inode_bitmap:  u32,
    data_block_bitmap: u32,
}

impl Superblock {
    pub fn get_root_inode(&self) -> &Inode {
        self.get_inode(self.root_inode)
    }

    pub fn get_inode(&self, inode_num: u32) -> &Inode {
        unsafe {
            &*(Buffer::read(inode_num).get_data().as_ptr() as *const Inode)
        }
    }
}

pub fn init() {
    unsafe {
        PATH_NAME = MaybeUninit::new(BTreeMap::new());
    }
}