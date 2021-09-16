use self::superblock::Superblock;
use self::inode::*;

pub mod file;
pub mod buffer;
pub mod inode;
pub mod superblock;

use buffer::Buffer;
use file::*;
use crate::process;

pub const BLOCK_SIZE: usize = 1024;

pub const FLAGS_O_RDONLY:    usize = 1;
pub const FLAGS_O_WRONLY:    usize = 2;
pub const FLAGS_O_RDWR:      usize = 4;
pub const FLAGS_O_DIRECTORY: usize = 8;

pub fn init() {
    buffer::init();
}

pub unsafe fn get_inode(inode_num: u32) -> &'static mut Inode {
    &mut  *(Buffer::read(inode_num).as_mut_ptr() as *mut Inode)
}

pub unsafe fn get_root_inode() -> &'static mut Inode {
    &mut *(Buffer::read(get_superblock().get_root_inode_num())
                    .as_mut_ptr() as *mut Inode)
}

pub unsafe fn get_superblock() -> &'static Superblock {
    &*(Buffer::read(0).as_ptr() as *const Superblock)
}

unsafe fn get_bitmap() -> &'static mut [u8; 1024] {
    &mut *(Buffer::read(get_superblock().get_bitmap_block_num()))
                    .as_mut_ptr()
}

pub fn path_lookup(path: &str) -> Result<&'static mut Inode, isize>{
    let mut inode = unsafe {
        if path.starts_with('.') || !path.starts_with('/') {
            // current working directory
            process::current().get_cwd()
        } else {
            // root directory
            get_root_inode()
        }
    };

    for name in path.split('/').filter(|name| !name.is_empty()) {
        if inode.is_file() {
            return Err(-1);
        }

        for entry in inode.dirent() {
            if entry.match_name(name) {
                inode = unsafe { get_inode(entry.inode_num()) };
                break;
            }
        }
    }
    Ok(inode)
}

pub fn open(path: &[u8], flags: usize) -> Result<&mut Inode, isize> {
    let path = core::str::from_utf8(path).unwrap();

    let inode = path_lookup(path)?;

    if inode.is_dir() && (flags & FLAGS_O_DIRECTORY) == 0 {
        return Err(-1);
    }

    Ok(inode)
}

pub fn read(file: &mut File, buf: &mut [u8]) -> Result<usize, isize> {
    file.read(buf)
}

pub fn write(file: &mut File, s: &[u8]) -> Result<usize, isize> {
    file.write(s)
}

pub fn mkdir(path: &[u8]) -> Result<usize, isize> {
    let path = core::str::from_utf8(path).unwrap();
    let mut dir = path_lookup(path)?;

    if dir.size() + 16 > 1024 {
        // A dirent is 16 bytes.
        // Directory size is limited under a block, which is 1024 bytes
        return Err(-1);
    }

    // find an empty block
    let inode_block = get_empty_block().ok_or(-1_isize)?;
    let name = path.split('/').last().ok_or(-1_isize)?;
    let dirent = Dirent::new(inode_block, name.as_ref());

    dir.write(&mut (dir.size() as usize), dirent.as_ref())?;
    let mut new_inode = Inode {
        ty: INODE_TYPE_DIR,
        num: inode_block,
        parent: dir.num,
        size: 0,
        addr: [0; 13],
    };

    let dirent = Dirent::new(dir.num, "..".as_bytes());
    (&mut new_inode).write(&mut 0, dirent.as_ref())?;

    unsafe {
        Buffer::read(inode_block).write(0, new_inode.as_ref())
    };
    Ok(0)
}

fn get_empty_block() -> Option<u32> {
    let res = unsafe {
        get_bitmap().iter_mut()
                    .enumerate()
                    .find(|(_, v)| **v != 0xff)?
    };

    let num = Some(res.0 as u32 * 8 + res.1.trailing_ones());
    *res.1 = *res.1 | (*res.1 + 1);
    num
}