use alloc::boxed::Box;
use crate::print;
use super::{*, inode::Inode};
use crate::process;

pub struct File {
    pos: usize,
    flags: usize,
    op: Box<dyn FileOperation>,
}

impl File {
    pub fn new(inode: &'static mut Inode, flags: usize) -> Self {
        Self {
            pos: 0,
            flags,
            op: Box::new(inode),
        }
    }

    pub fn stdio() -> Self {
        Self {
            pos: 0,
            flags: FLAGS_O_RDWR,
            op: Box::new(Stdio),
        }
    }

    pub fn write(&mut self, s: &[u8]) -> Result<usize, isize> {
        if self.flags & FLAGS_O_RDONLY != 0 {
            return Err(-1);
        }
        self.op.write(&mut self.pos, s)
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, isize> {
        if self.flags & FLAGS_O_WRONLY != 0 {
            return Err(-1);
        }
        self.op.read(&mut self.pos, buf)
    }

    pub fn flags(&self) -> usize {
        self.flags
    }
}

pub struct Stdio;

impl FileOperation for &mut Inode {
    fn write(&mut self, offset: &mut usize, buf: &[u8]) -> Result<usize, isize> {
        self.iter_mut().skip(*offset)
                        .take(buf.len())
                        .enumerate()
                        .for_each(|(i, c)| {
                            *c = buf[i];
                        });
        Ok(buf.len())
    }

    fn read(&mut self, offset: &mut usize, buf: &mut [u8]) -> Result<usize, isize> {
        let mut len = 0;
        self.iter().enumerate()
                    .skip(*offset)
                    .take(buf.len())
                    .for_each(|(i, &c)| {
                        buf[i] = c;
                        len += 1;
                    });
        *offset += len;
        Ok(len)
    }
}

impl FileOperation for Stdio {
    fn write(&mut self, _: &mut usize, s: &[u8]) -> Result<usize, isize> {
        print!("{}", unsafe {core::str::from_utf8_unchecked(s) });
        Ok(s.len())
    }

    fn read(&mut self, _: &mut usize, buf: &mut [u8]) -> Result<usize, isize> {
        process::get_user_input(buf)
    }
}

pub trait FileOperation {
    fn write(&mut self,
            offset: &mut usize,
            buf: &[u8]
        ) -> Result<usize, isize>;
    fn read(&mut self,
            offset: &mut usize,
            buf: &mut [u8]
        ) -> Result<usize, isize>;
}
