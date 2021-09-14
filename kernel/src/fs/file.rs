use alloc::boxed::Box;
use crate::print;
use super::{*, inode::Inode};
use alloc::sync::Arc;
use crate::process;
use crate::common::*;

pub struct File {
    sz: usize,
    pos: usize,
    pub inode: Arc<Inode>,
    flags: usize,
    op: Box<dyn FileOperation>,
}

impl File {
    pub fn new(inode: Arc<Inode>, flags: usize) -> Self {
        Self {
            sz: inode.size as usize,
            pos: 0,
            inode,
            flags,
            op: Box::new(DiskFile),
        }
    }

    pub fn size(&self) -> usize {
        self.sz
    }

    pub fn stdio() -> Self {
        Self {
            sz: 0,
            pos: 0,
            inode: Arc::new(Inode {
                ty: 2, // device
                num: 0,
                parent: 0,
                size: 0,
                addr: [0; 13],
            }),
            flags: FLAGS_O_RDWR,
            op: Box::new(Stdio),
        }
    }

    pub fn write(&mut self, s: &str) -> Result<usize, ()> {
        if self.flags & FLAGS_O_RDONLY != 0 {
            return Err(());
        }
        self.op.write(&self.inode, &mut self.pos, s)
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        if self.flags & FLAGS_O_WRONLY != 0 {
            return Err(());
        }
        self.op.read(&self.inode, &mut self.pos, buf)
    }

    pub fn flags(&self) -> usize {
        self.flags
    }
}

pub struct DiskFile;

pub struct Stdio;

impl FileOperation for DiskFile {
    fn write(&self, _: &Arc<Inode>, _: &mut usize, _: &str) -> Result<usize, ()> {
        unimplemented!()
    }

    fn read(&mut self, inode: &Arc<Inode>, offset: &mut usize, buf: &mut [u8]) -> Result<usize, ()> {
        assert!(*offset as u32 <= inode.size);
        if *offset as u32 == inode.size {
            return Ok(0);
        }

        let start = *offset;
        let end = core::cmp::min(*offset + buf.len(), inode.size as usize);
        let mut curr = start;
        let mut buf_curr = 0;

        for i in (start / BLOCK_SIZE)..(round_up_with(end, BLOCK_SIZE) / BLOCK_SIZE) {
            let data = inode.get_data(i).ok_or(())?;
            let next = core::cmp::min((curr & !1023) + 1024, end);
            let len = next - curr;
            buf[buf_curr..(buf_curr + len)]
                .copy_from_slice(&data[(curr & 1023)..=((next - 1) & 1023)]);
            curr = next;
            buf_curr += len;
        }
        *offset = end;
        Ok(end - start)
    }
}

impl FileOperation for Stdio {
    fn write(&self, _: &Arc<Inode>, _: &mut usize, s: &str) -> Result<usize, ()> {
        print!("{}", s);
        Ok(s.len())
    }

    fn read(&mut self, _: &Arc<Inode>, _: &mut usize, buf: &mut [u8]) -> Result<usize, ()> {
        process::get_user_input(buf)
    }
}

pub trait FileOperation {
    fn write(&self,
            inode: &Arc<Inode>,
            offset: &mut usize,
            _: &str
        ) -> Result<usize, ()>;
    fn read(&mut self,
            inode: &Arc<Inode>,
            offset: &mut usize,
            buf: &mut [u8]
        ) -> Result<usize, ()>;
}
