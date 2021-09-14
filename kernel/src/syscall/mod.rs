use crate::exception::UserContext;
use crate::fs;
use crate::fs::FLAGS_O_DIRECTORY;
use crate::process;
use alloc::vec::Vec;

pub static SYSCALL_TABLE: &[fn(_: &mut UserContext) -> Result<usize, ()>] = &[
    sys_fork,     // 0x00
    sys_exec,     // 0x01
    sys_open,     // 0x02
    sys_read,     // 0x03
    sys_write,    // 0x04
    sys_close,    // 0x05
    sys_waitpid,  // 0x06
    sys_exit,     // 0x07
    sys_getdents, // 0x08
    sys_sbrk,     // 0x09
    sys_getcwd,   // 0x0A
];

fn string_len(ptr: *const u8) -> usize {
    for i in 0..128 {
        unsafe {
            if ptr.add(i).read() == b'\x00' {
                return i;
            }
        }
    }
    panic!("string too long !");
}

pub fn sys_exec(ctx: &mut UserContext) -> Result<usize, ()> {
    // x0 is the address of the path
    let x0 = ctx.x[0] as *const u8;
    let len = string_len(x0);
    let path = unsafe {
        core::slice::from_raw_parts(x0, len)
    };

    let mut ptr = ctx.x[1] as *const *const u8;
    let mut argv = Vec::<Vec<u8>>::new();
    if ptr as usize != 0 {
        unsafe {
            while ptr.read() as usize != 0 {
                let len = string_len(ptr.read()) + 1;
                let s = core::slice::from_raw_parts(ptr.read(), len);

                argv.push(s.to_vec());
                ptr = ptr.add(1);
            }
        }
    }
    crate::process::exec(path, argv)
}

pub fn sys_fork(_: &mut UserContext) -> Result<usize, ()> {
    crate::process::fork()
}

pub fn sys_open(ctx: &mut UserContext) -> Result<usize, ()> {
    let pathname = unsafe {
        let ptr = ctx.x[0] as *const u8;
        let len = string_len(ptr);
        core::slice::from_raw_parts(ptr, len)
    };

    let flags = ctx.x[1];
    let file = fs::open(pathname, flags)?;

    process::current().insert_file_desc(file)
}

pub fn sys_read(ctx: &mut UserContext) -> Result<usize, ()> {
    let file = process::current().get_file_desc_mut(ctx.x[0])?;
    let buf = ctx.x[1] as *mut u8;
    let count = ctx.x[2];

    unsafe {
        fs::read(file, core::slice::from_raw_parts_mut(buf, count))
    }
}

pub fn sys_write(ctx: &mut UserContext) -> Result<usize, ()> {
    let file = process::current().get_file_desc_mut(ctx.x[0] as usize)?;
    let ptr = ctx.x[1] as *const u8;
    let len = ctx.x[2] as usize;

    unsafe {
        fs::write(file, core::str::from_utf8_unchecked(core::slice::from_raw_parts(ptr, len)))
    }
}

pub fn sys_close(_: &mut UserContext) -> Result<usize, ()> {
    unimplemented!()
}

pub fn sys_waitpid(ctx: &mut UserContext) -> Result<usize, ()> {
    let pid = ctx.x[0];
    process::wait(pid as u8)
}

pub fn sys_exit(_: &mut UserContext) -> Result<usize, ()> {
    process::exit()
}

pub fn sys_getdents(ctx: &mut UserContext) -> Result<usize, ()> {
    let file = process::current().get_file_desc_mut(ctx.x[0])?;
    if file.flags() & FLAGS_O_DIRECTORY == 0 {
        return Err(());
    }

    let buffer = unsafe {
        core::slice::from_raw_parts_mut(ctx.x[1] as *mut u8, ctx.x[2])
    };

    fs::read(file, buffer)
}

pub fn sys_sbrk(ctx: &mut UserContext) -> Result<usize, ()> {
    let inc = ctx.x[0];
    let result = process::sbrk(inc as isize);
    result
}

pub fn sys_getcwd(ctx: &mut UserContext) -> Result<usize, ()> {
    let ptr = ctx.x[0] as *mut u8;
    let len = ctx.x[1];

    let buf = unsafe {
        core::slice::from_raw_parts_mut(ptr, len)
    };

    process::get_cwd(buf)
}