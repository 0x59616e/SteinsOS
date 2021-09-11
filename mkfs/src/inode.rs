#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Inode {
    pub ty:  u8,             // 0 is directory, 1 is file
    pub num: u32,            // inode number
    pub size: u32,           // file size
    pub addr: [u32; 12 + 1], // block number
}
