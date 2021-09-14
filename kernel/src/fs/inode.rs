use super::buffer::Buffer;
use alloc::sync::Arc;

const INODE_TYPE_DIR:  u8 = 0;
const INODE_TYPE_FILE: u8 = 1;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Inode {
    pub ty:  u8,             // 0 is directory, 1 is file, 2 is device
    pub num: u32,            // inode number
    pub parent: u32,             // parent inode
    pub size: u32,           // file size
    pub addr: [u32; 12 + 1], // block number
}

impl Inode {
    pub fn is_file(&self) -> bool {
        self.ty == INODE_TYPE_FILE
    }

    pub fn is_dir(&self) -> bool {
        self.ty == INODE_TYPE_DIR
    }

    pub fn dirent(&self) -> DirentIter {
        DirentIter { offset: 0, inode: &self }
    }

    pub fn get_data(&self, idx: usize) -> Option<Arc<[u8; 1024]>> {
        Some(Buffer::read(*self.addr.get(idx)?).get_data())
    }
}

pub struct DirentIter<'a> {
    offset: usize,
    inode: &'a Inode,
}

impl<'a> Iterator for DirentIter<'a> {
    type Item = Dirent;
    fn next(&mut self) -> Option<Self::Item> {
        if self.offset > self.inode.size as usize {
            panic!("dirent offset = {}, inode_size = {}", self.offset, self.inode.size);
        }

        if self.offset == self.inode.size as usize {
            return None;
        }

        if self.offset + core::mem::size_of::<Dirent>() > 1024 {
            unimplemented!();
        }

        return unsafe {
            let result = Some((self.inode.get_data(0)?
                                        .as_ptr()
                                        .add(self.offset) as *const Dirent).read());
            self.offset += core::mem::size_of::<Dirent>();
            result
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Dirent {
    inode_num: u32,
    name: [u8; 12],
}

impl Dirent {
    pub fn match_name(&self, name: &str) -> bool {
        let len = self.name.iter().position(|&c| c == 0).unwrap();

        if len != name.len() {
            return false;
        }

        self.name.iter().zip(name.as_bytes()).all(|(c1, c2)| c1 == c2)
    }

    pub fn inode_num(&self) -> u32 {
        self.inode_num
    }
}
