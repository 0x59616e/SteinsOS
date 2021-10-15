use super::inode::*;

pub const GATEFS_MAGIC: u32 = 1048596;
pub const GATEFS_BLOCK_SIZE: usize = 1 << 10;
pub const GATEFS_INODE_BITMAP_BLOCK_NR: u32 = 1;
pub const GATEFS_INODE_STORE_BLOCK_NR: u32 = 2;
pub const GATEFS_DATA_BITMAP_BLOCK_NR: u32 = 10;
pub const GATEFS_DATA_BLOCK_NR: u32 = 11;
pub const GATEFS_FREE_INODES: u32 = INODE_PER_BLOCK * (GATEFS_DATA_BITMAP_BLOCK_NR - GATEFS_INODE_STORE_BLOCK_NR);

pub const INODE_PER_BLOCK: u32 = GATEFS_BLOCK_SIZE as u32 / (core::mem::size_of::<Inode>() as u32);

pub fn inode_blk_no(ino: u32) -> u32 {
    ino / INODE_PER_BLOCK + GATEFS_INODE_STORE_BLOCK_NR
}

pub fn inode_blk_shift(ino: u32) -> u32 {
    ino % INODE_PER_BLOCK
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Superblock {
    pub magic: u32,

    pub inode_bitmap_block_nr: u32,
    pub data_block_bitmap_nr: u32,

    pub root_inode_block_nr: u32,

    pub nr_free_blocks: u32,
    pub nr_free_inodes: u32,
}