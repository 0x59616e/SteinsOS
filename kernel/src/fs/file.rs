use alloc::boxed::Box;
use crate::print;
use super::{*, inode::Inode};
use alloc::sync::Arc;
use crate::process;
use crate::common::*;

pub struct File {
    sz: usize,
    flags: usize,
    op: Box<dyn FileOperation>,
}

impl File {
    pub fn new(inode: Arc<Inode>, flags: usize) -> Self {
        Self {
            sz: inode.size as usize,
            flags,
            op: Box::new(DiskFile {
                pos: 0,
                inode
            }),
        }
    }

    pub fn size(&self) -> usize {
        self.sz
    }

    pub fn stdio() -> Self {
        Self {
            sz: 0,
            flags: FLAGS_O_RDWR,
            op: Box::new(Stdio),
        }
    }

    pub fn write(&self, s: &str) -> Result<usize, ()> {
        if self.flags & FLAGS_O_RDONLY != 0 {
            return Err(());
        }
        self.op.write(s)
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        if self.flags & FLAGS_O_WRONLY != 0 {
            return Err(());
        }
        self.op.read(buf)
    }

    pub fn flags(&self) -> usize {
        self.flags
    }
}

pub struct DiskFile {
    pos: usize,
    inode: Arc<Inode>
}

pub struct Stdio;

impl FileOperation for DiskFile {
    fn write(&self, _: &str) -> Result<usize, ()> {
        unimplemented!()
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        assert!(self.pos as u32 <= self.inode.size);
        if self.pos as u32 == self.inode.size {
            return Ok(0);
        }

        let start = self.pos;
        let end = core::cmp::min(self.pos + buf.len(), self.inode.size as usize);
        let mut curr = start;
        let mut buf_curr = 0;

        for i in (start / BLOCK_SIZE)..(round_up_with(end, BLOCK_SIZE) / BLOCK_SIZE) {
            let data = self.inode.get_data(i).ok_or(())?;
            let next = core::cmp::min((curr & !1023) + 1024, end);
            let len = next - curr;
            buf[buf_curr..(buf_curr + len)]
                .copy_from_slice(&data[(curr & 1023)..=((next - 1) & 1023)]);
            curr = next;
            buf_curr += len;
        }
        self.pos = end;
        Ok(end - start)
    }
}

impl FileOperation for Stdio {
    fn write(&self, s: &str) -> Result<usize, ()> {
        print!("{}", s);
        Ok(s.len())
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        process::get_user_input(buf)
    }
}

pub trait FileOperation {
    fn write(&self, _: &str) -> Result<usize, ()>;
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()>;
}
