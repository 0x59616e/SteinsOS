use super::buffer::Buffer;

pub const INODE_TYPE_DIR:  u8 = 0;
const INODE_TYPE_FILE: u8 = 1;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Inode {
    pub ty:  u8,             // 0 is directory, 1 is file, 2 is device
    pub num: u32,            // inode number
    pub parent: u32,         // parent inode
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
        DirentIter { offset: 0, inode: self }
    }

    pub fn iter(&self) -> InodeDataIter {
        InodeDataIter { pos: 0, block: core::ptr::null(), inode: self }
    }

    pub fn iter_mut(&mut self) -> InodeDataIterMut {
        InodeDataIterMut { pos: 0, block: core::ptr::null_mut(), inode: self}
    }

    pub fn get_data(&self, idx: usize) -> Option<&[u8; 1024]> {
        Some(unsafe {&*Buffer::read(*self.addr.get(idx)?).as_ptr()})
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn resize(&mut self, new: u32) {
        self.size = new;
    }
}

pub struct DirentIter<'a> {
    offset: usize,
    inode: &'a Inode,
}

impl<'a> Iterator for DirentIter<'a> {
    type Item = &'static Dirent;
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
                                        .add(self.offset) as *const Dirent).as_ref().unwrap());
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
    pub fn new(inode_num: u32, name: &[u8]) -> Self {
        let mut res = [0_u8; 12];
        res[..name.len()].copy_from_slice(name);

        Self {
            inode_num,
            name: res,
        }
    }

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

    pub fn name(&self) -> &[u8] {
        &self.name
    }
}

impl AsRef<[u8]> for Dirent {
    fn as_ref(&self) -> &[u8] {
        let ptr = core::ptr::addr_of!(*self) as *const u8;
        unsafe {
            core::slice::from_raw_parts(ptr, core::mem::size_of::<Self>())
        }
    }
}

impl AsRef<[u8]> for Inode {
    fn as_ref(&self) -> &[u8] {
        let ptr = core::ptr::addr_of!(*self) as *const u8;
        unsafe {
            core::slice::from_raw_parts(ptr, core::mem::size_of::<Self>())
        }
    }
}

pub struct InodeDataIter<'a> {
    pos: u32,
    block: *const u8,
    inode: &'a Inode,
}

impl<'a> Iterator for InodeDataIter<'a> {
    type Item = &'a u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos == self.inode.size() {
            None
        } else {
            let idx = self.pos as usize >> 10;
            let offset  = self.pos as usize & 1023;
            self.pos += 1;

            if offset == 0 {
                // time to read a new block
                let blockno = {
                    match idx {
                        0..=11 => self.inode.addr[idx],
                        _ => unsafe {
                            (Buffer::read(self.inode.addr[12]).as_ptr() as *const u32).add(idx - 12).read()
                        }
                    }
                };
                self.block = unsafe {
                    Buffer::read(blockno).as_ptr() as *const u8
                };
            }
            unsafe {
                Some(self.block.add(offset).as_ref().expect("null pointer"))
            }
        }
    }
}

pub struct InodeDataIterMut<'a> {
    pos: u32,
    block: *mut u8,
    inode: &'a mut Inode,
}

impl<'a> Iterator for InodeDataIterMut<'a> {
    type Item = &'a mut u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos == self.inode.size() {
            self.inode.resize(self.pos + 1);
        }

        let idx = self.pos as usize >> 10;
        let offset = self.pos as usize & 1023;
        self.pos += 1;

        if offset == 0 {
            // time to read a new block
            let tmp = if idx < 12 { idx } else { 12 };
            if self.inode.addr[tmp] == 0 {
                self.inode.addr[tmp] = super::get_empty_block().expect("Disk is full");
            }

            let blockno = {
                match idx {
                    0..=11 => self.inode.addr[idx],
                    _ => unsafe {
                        let ptr = (Buffer::read(self.inode.addr[12]).as_mut_ptr() as *mut u32).add(idx - 12);
                        if ptr.read() == 0 {
                            ptr.write(super::get_empty_block().expect("Disk is full"));
                        }
                        ptr.read()
                    }
                }
            };
            self.block = unsafe {
                Buffer::read(blockno).as_mut_ptr() as *mut u8
            };
        }

        unsafe {
            Some(self.block.add(offset).as_mut().expect("null pointer"))
        }
    }
}