#[repr(C)]
#[derive(Clone, Copy)]
pub struct Superblock {
    root_inode: u32,
    bitmap_block: u32,
}

impl Superblock {
    pub fn get_root_inode_num(&self) -> u32 {
        self.root_inode
    }

    pub fn get_bitmap_block_num(&self) -> u32 {
        self.bitmap_block
    }
}