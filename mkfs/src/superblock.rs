#[repr(C)]
#[derive(Clone, Copy)]

pub struct Superblock {
    pub root_inode: u32,
    pub inode_bitmap:  u32,
    pub data_block_bitmap: u32,
}