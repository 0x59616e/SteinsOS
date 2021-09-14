use self::superblock::Superblock;

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
    superblock::init();
}

pub fn get_superblock() -> &'static Superblock {
    // FIXME: We can't modify superblock and inode if
    //        immutable reference is used.
    unsafe {
        &*(Buffer::read(0).get_data().as_ptr() as *const Superblock)
    }
}

pub fn open(path: &[u8], flags: usize) -> Result<File, ()> {
    let superblock = get_superblock();

    let path = core::str::from_utf8(path).unwrap();

    let mut inode = if path.starts_with('.') || !path.starts_with('/') {
        // current working directory
        process::current().get_cwd()
    } else {
        // root directory
        superblock.get_root_inode()
    };

    for name in path.split('/').filter(|name| name.len() > 0) {
        if inode.is_file() {
            return Err(());
        }

        for entry in inode.dirent() {
            if entry.match_name(name) {
                inode = superblock.get_inode(entry.inode_num());
                break;
            }
        }
    }

    if inode.is_dir() && (flags & FLAGS_O_DIRECTORY) == 0 {
        return Err(());
    }

    Ok(File::new(inode, flags))
}

pub fn read(file: &mut File, buf: &mut [u8]) -> Result<usize, ()> {
    file.read(buf)
}

pub fn write(file: &mut File, s: &str) -> Result<usize, ()> {
   file.write(s)
}
