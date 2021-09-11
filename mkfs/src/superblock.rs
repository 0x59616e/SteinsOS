#[repr(C)]
#[derive(Clone, Copy)]
pub struct Superblock {
    pub inode_count: u32, // the amount of the inodes
    pub inode_bitmap:  u32,
    pub data_block_bitmap: u32,
}
