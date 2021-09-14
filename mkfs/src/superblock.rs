#[repr(C)]
#[derive(Clone, Copy)]

pub struct Superblock {
    pub root_inode: u32,
    pub bitmap_block: u32,
}