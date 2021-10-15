use core::mem::size_of;
use super::superblock::*;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Inode {
    pub i_type:  u8,
    pub i_no: u32,
    pub i_parent: u32,
    pub i_size: u32,
    pub i_nlink: u32,
    pub root_addr_block: u32,
}

#[repr(C)]
pub struct AddrBlockHeader {
    pub leaf: u8,
    pub entries_cnt: u32,
}

#[repr(C)]
pub struct AddrEntry {
    pub logical_blk: u32,
    pub physical_blk: u32,
    pub len: u32,
}

#[repr(C)]
pub struct AddrBlock {
    pub header:  AddrBlockHeader,
    pub entries: [AddrEntry; (GATEFS_BLOCK_SIZE - size_of::<AddrBlockHeader>()) / size_of::<AddrEntry>()],
}