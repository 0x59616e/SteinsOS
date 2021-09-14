use crate::vm::*;
use alloc::vec;
use alloc::boxed::Box;
use crate::common::*;
use crate::exception::UserContext;
use crate::fs::file::*;
use core::mem::MaybeUninit;
use alloc::collections::vec_deque::VecDeque;
use alloc::vec::Vec;
use crate::fs::{self, inode::Inode};

mod elf;

static mut PROCESS_LIST: [*mut Process; 20] = [core::ptr::null_mut(); 20];
static mut SCHEDULER_CONTEXT: Context = Context::new();
static mut USER_INPUT: MaybeUninit<(VecDeque<u8>, Vec<*mut Process>)> = MaybeUninit::uninit();

extern "C" {
    fn switch(from: *mut Context, to: *const Context);
}

#[repr(C)]
struct Context {
    sp_el0 : usize,  // 0
    sp_el1 : usize,  // 8
    ttbr1  : usize,  // 16
    x19    : usize,  // 24
    x20    : usize,  // 32
    x21    : usize,  // 40
    x22    : usize,  // 48
    x23    : usize,  // 56
    x24    : usize,  // 64
    x25    : usize,  // 72
    x26    : usize,  // 80
    x27    : usize,  // 88
    x28    : usize,  // 96
    x29    : usize,  // 108
    x30    : usize,  // 116
}

impl Context {
    const fn new() -> Self {
        Self {
            sp_el0 : 0,
            sp_el1 : 0,
            ttbr1  : 0,
            x19    : 0,
            x20    : 0,
            x21    : 0,
            x22    : 0,
            x23    : 0,
            x24    : 0,
            x25    : 0,
            x26    : 0,
            x27    : 0,
            x28    : 0,
            x29    : 0,
            x30    : 0, // link register
        }
    }
}

#[derive(PartialEq, Eq)]
enum ProcessState {
    Blocking,
    Ready,
    Running,
    Dead,
}

pub struct Process {
    pub pid: u8,
    state: ProcessState,
    context: Context,
    stack_size: usize,
    heap_start: usize,
    heap_end: usize,
    sp_el1: Box<[u8]>,
    page_tb: PageTable,
    channel: Option<usize>,
    child: Vec<u8>,
    cwd: Option<u32>,
    // 0 => stdin
    // 1 => stdout
    file: [Option<File>; 10],
}

impl Process {
    const USER_BASE_ADDR: usize = 0xffff_0000_0000_0000;
    const USER_STACK_TOP: usize = 0xffff_ffff_ffff_0000;
    const USER_HEAP_SIZE_LIMIT: usize = 10 * PAGESIZE;

    fn is_ready(&self) -> bool {
        self.state == ProcessState::Ready
    }

    fn default_file_dec() -> [Option<File>; 10] {
        let mut file: [Option<File>; 10] = Default::default();
        file[0] = Some(File::stdio());
        file[1] = Some(File::stdio());
        file
    }

    pub fn get_cwd(&self) -> &Inode {
        fs::get_superblock().get_inode(self.cwd.unwrap())
    }

    pub fn get_file_desc_mut(&mut self, fd: usize) -> Result<&mut File, ()> {
        if fd >= 5 {
            return Err(());
        }
        self.file[fd].as_mut().ok_or(())
    }

    pub fn insert_file_desc(&mut self, file: File) -> Result<usize, ()> {
        let res = self.file.iter_mut()
                            .enumerate()
                            .find(|(_, desc)| desc.is_none()).ok_or(())?;
        *res.1 = Some(file);
        Ok(res.0)
    }

    fn is_waiting_on(&self, channel: usize) -> bool {
        match self.channel {
            Some(ch) if ch == channel => true,
            _ => false,
        }
    }

    pub fn wakeup(&mut self) {
        self.channel = None;
        self.state = ProcessState::Ready;
    }

    pub fn heap_end(&self) -> usize {
        self.heap_end
    }

    pub fn page_tb(&mut self) -> &mut PageTable {
        &mut self.page_tb
    }
}

pub fn put_user_input(c: u8) {
    let (buffer, waiting_list) = unsafe {
        USER_INPUT.assume_init_mut()
    };

    buffer.push_back(c);
    for proc in waiting_list {
        unsafe {
            (**proc).wakeup();
        }
    }
}

pub fn get_user_input(buf: &mut [u8]) -> Result<usize, ()> {
    let (buffer, waiting_list) = unsafe {
        USER_INPUT.assume_init_mut()
    };

    let mut len = 0;
    loop {
        while let Some(c) = buffer.pop_front() {
            if len > 0 || (c != 8 && c != 0x7f) {
                print!("{}", c as char);
            }

            match c {
                // time to return
                b'\r' | b'\n' =>  return Ok(len),
                8 | 0x7f => {
                    // backspace
                    if len > 0 {
                        len -= 1;
                        *(buf.get_mut(len).ok_or(())?) = 0;
                    }
                }
                _ => {
                    *(buf.get_mut(len).ok_or(())?) = c;
                    len += 1;
                }
            }
        }
        let proc = current();
        waiting_list.push(proc as *mut Process);
        sleep(0);
    }
}

fn alloc_process() -> u8 {
    unsafe {
        for (i, proc) in PROCESS_LIST.iter_mut().enumerate() {
            if proc.is_null() {
                return i as u8;
            }
        }
    }
    panic!("Running out of process");
}

pub fn init_first(user_entry: usize) {
    let mut page_tb = PageTable::new();
    let source = user_entry as *const u8;

    // stack
    page_tb.create(Process::USER_STACK_TOP - PAGESIZE, PAGESIZE, "rw").unwrap();

    let dest = page_tb.create(Process::USER_BASE_ADDR, PAGESIZE, "rx").unwrap() as *mut u8;

    unsafe {
        core::ptr::copy_nonoverlapping(source, dest, PAGESIZE);
    }

    let mut sp_el1 = vec![0_u8; 4 * PAGESIZE].into_boxed_slice();

    // we place the context of user process on the bottom of kernel stack
    let user_ctx = sp_el1.as_mut_ptr() as *mut UserContext;
    assert!(user_ctx as usize & 0x3fff == 0); // 4 pages align
    unsafe {
        // exception link register
        (*user_ctx).elr_el1 = Process::USER_BASE_ADDR;
        // spsr
        (*user_ctx).spsr_el1 = 0;
        // that's all
    };

    let mut context = Context::new();
    context.sp_el0 = Process::USER_STACK_TOP;
    context.sp_el1 = sp_el1.as_ptr() as usize + 4 * PAGESIZE;
    context.ttbr1  = page_tb.as_ptr() as usize;
    context.x30 = crate::exception::back_to_earth as *const fn() as usize;

    let proc = Process {
        pid: 0,
        state: ProcessState::Ready,

        context,
        stack_size: PAGESIZE,
        heap_start: Process::USER_BASE_ADDR + PAGESIZE,
        heap_end: Process::USER_BASE_ADDR + PAGESIZE,
        sp_el1,
        page_tb,
        child: Vec::new(),
        channel: None,
        cwd: None,
        file: Process::default_file_dec(),
    };

    unsafe {
        PROCESS_LIST[0] = Box::into_raw(Box::new(proc));
        USER_INPUT = MaybeUninit::new((VecDeque::new(), Vec::new()));
    }
}

pub fn exec(path: &[u8], argv: Vec<Vec<u8>>) -> Result<usize, ()> {
    let mut file = crate::fs::open(path, crate::fs::FLAGS_O_RDONLY)?;
    let mut program = Vec::new();
    program.resize(file.size(), 0);
    file.read(&mut program)?;

    let file_header = elf::read_fileheader(&program);

    let prog_header_table = elf::read_program_header_table(&program, file_header);

    let proc = current();

    proc.cwd = Some(file.inode.expect("No inode").parent);

    let mut page_tb = PageTable::new();

    let mut curr = Process::USER_BASE_ADDR;

    for header in prog_header_table {
        if header.is_loadable() {
            let perm = match header.flags {
                0b111 => "rwx",
                0b110 => "rw",
                0b100 => "r",
                _ => unimplemented!()
            };

            let size = header.filesz as usize;
            let src = unsafe {
                program.as_ptr().add(header.offset as usize)
            };

            let dst =  page_tb.create(curr, round_up(size), perm)?;
            
            unsafe {
                core::ptr::copy_nonoverlapping(src, dst as *mut u8, size);
            }

            curr += round_up(size);
        }
    }

    page_tb.create(Process::USER_STACK_TOP - PAGESIZE, PAGESIZE, "rw")?;

    proc.heap_start = curr;
    proc.heap_end = curr;

    let user_ctx = 
    unsafe {
        let user_ctx = proc.sp_el1.as_mut_ptr() as *mut UserContext;

        // reset exception link register
        (*user_ctx).elr_el1 = Process::USER_BASE_ADDR;
        (*user_ctx).spsr_el1 = 0;


        // reset page table
        let x = page_tb.as_ptr() as usize;
        asm!("msr ttbr1_el1, {}",
            "TLBI VMALLE1",
            "dsb sy",
            "isb sy", in(reg) x);
        user_ctx
    };
    core::mem::swap(&mut page_tb, &mut proc.page_tb);
    page_tb.release();

    // put argv onto stack
    let argc = argv.len();
    let mut v = Vec::<*const u8>::new();
    let mut ptr = Process::USER_STACK_TOP as *mut u8;
    let argv = unsafe {
        for arg in argv {
            ptr = ptr.sub(arg.len());
            core::slice::from_raw_parts_mut(ptr, arg.len())
                        .copy_from_slice(&arg);
            v.push(ptr);
        }
        let ptr = (ptr as *mut *const u8).sub(v.len());
        let s = core::slice::from_raw_parts_mut(ptr, v.len());
                    s.copy_from_slice(&v);
        ptr as usize
    };

    unsafe {
        // reset stack
        asm!("msr sp_el0, {}", in(reg) argv);
        (*user_ctx).x[1] = argv;
    }
    return Ok(argc);
}

pub fn fork() -> Result<usize, ()> {
    let proc = current();

    let mut page_tb = PageTable::new();

    // copy text data
    let text = page_tb.create(Process::USER_BASE_ADDR,
                                proc.heap_start - Process::USER_BASE_ADDR,
                                "rx")? as *mut u8;
    unsafe {
        core::ptr::copy_nonoverlapping(Process::USER_BASE_ADDR as *const u8,
                                        text,
                                        proc.heap_start - Process::USER_BASE_ADDR);
    }

    // copy heap data
    let heap = page_tb.create(proc.heap_start, proc.heap_end - proc.heap_start, "rw")? as *mut u8;
    unsafe {
        core::ptr::copy_nonoverlapping(proc.heap_start as *const u8, heap, proc.heap_end - proc.heap_start);
    }

    // copy user stack data
    let stack = page_tb.create(Process::USER_STACK_TOP - proc.stack_size,
                                proc.stack_size, "rw")? as *mut u8;
    unsafe {
        core::ptr::copy_nonoverlapping((Process::USER_STACK_TOP - proc.stack_size) as *const u8, 
                                        stack, proc.stack_size);
    }

    // copy user context
    let mut kernel_stack = vec![0_u8; 4 * PAGESIZE].into_boxed_slice();
    unsafe {
        core::ptr::copy_nonoverlapping(proc.sp_el1.as_ptr(),
                                        kernel_stack.as_mut_ptr(),
                                        PAGESIZE);
        (*(kernel_stack.as_mut_ptr() as *mut UserContext)).x[0] = 0; // return value
    }

    let mut ctx = Context::new();
    let pid = alloc_process();
    ctx.x30 = crate::exception::back_to_earth as *const fn() as usize;
    unsafe {
        asm!("mrs {}, sp_el0", out(reg) ctx.sp_el0);
    }
    ctx.sp_el1 = (kernel_stack.as_ptr() as usize) + 4 * PAGESIZE;
    ctx.ttbr1 = page_tb.as_ptr() as usize | ((pid as usize) << 48);

    proc.child.push(pid);

    let new_proc = Process {
        pid,
        state: ProcessState::Ready,
        context: ctx,
        stack_size: proc.stack_size,
        heap_start: proc.heap_start,
        heap_end:  proc.heap_end,
        sp_el1: kernel_stack,
        page_tb,
        child: Vec::new(),
        channel: None,
        cwd: proc.cwd.clone(),
        // FIXME
        file: Process::default_file_dec(),
    };

    unsafe {
        PROCESS_LIST[pid as usize] = Box::into_raw(Box::new(new_proc));
    }

    Ok(pid as usize)
}

pub fn wait(pid: u8) -> Result<usize, ()> {
    // must be child process
    let proc = current();
    if !proc.child.iter().any(|child| *child == pid) {
        return Err(())
    }

    let mut child = unsafe {
        Box::from_raw(PROCESS_LIST[pid as usize])
    };

    while child.state != ProcessState::Dead {
        sleep(pid as usize);
    }

    child.page_tb.release();
    unsafe {
        PROCESS_LIST[pid as usize] = core::ptr::null_mut();
    }
    Ok(0)
}

pub fn exit() -> ! {
    let proc = current();
    proc.state = ProcessState::Dead;
    wakeup(proc.pid as usize);
    switch_to_scheduler();
    panic!("error: exit");
}

pub fn yield_cpu() {
    let mut proc = current();
    proc.state = ProcessState::Ready;
    switch_to_scheduler();
}

pub fn sleep(channel: usize) {
    let proc = current();
    proc.state = ProcessState::Blocking;
    proc.channel = Some(channel);
    switch_to_scheduler();
}

pub fn wakeup(channel: usize) {
    let list = unsafe {
        &mut PROCESS_LIST
    };

    for proc in list.into_iter().filter(|ptr| !ptr.is_null()).map(|ptr| unsafe{&mut **ptr}) {
        if proc.is_waiting_on(channel) {
            proc.wakeup();
        }
    }
}

pub fn sbrk(inc: isize) -> Result<usize, ()> {
    let proc = current();
    if inc < 0 || proc.heap_end + inc as usize - proc.heap_start > Process::USER_HEAP_SIZE_LIMIT {
        return Ok(0);
    } else {
        let ret = Ok(proc.heap_end);
        proc.heap_end += inc as usize;
        ret
    }
}

pub fn schedule() -> ! {
    loop {
        let list = unsafe {
            &mut PROCESS_LIST
        };

        for i in 0..20 {
            let ptr = list[i];
            clear_current();

            if ptr.is_null() {
                continue;
            }

            let proc = unsafe {
                &mut *ptr
            };

            if proc.is_ready() {
                proc.state = ProcessState::Running;
                let from = unsafe {
                    core::ptr::addr_of_mut!(SCHEDULER_CONTEXT)
                };
                let to = core::ptr::addr_of!(proc.context);

                write_current(ptr);

                unsafe {
                    switch(from, to);
                }
            }
        }
    }
}

pub fn switch_to_scheduler() {
    let proc = current();
    let curr_ctx = core::ptr::addr_of_mut!(proc.context);
    unsafe {
        let sched_ctx = core::ptr::addr_of_mut!(SCHEDULER_CONTEXT);
        switch(curr_ctx, sched_ctx);
    }
}

fn write_current(ptr: *mut Process) {
    let addr = ptr as usize;
    unsafe {
        asm!("msr CONTEXTIDR_EL1, {}", in(reg) addr);
    }
}

fn clear_current() {
    let addr = 0_usize;
    unsafe {
        asm!("msr CONTEXTIDR_EL1, {}", in(reg) addr);
    }
}

pub fn current() -> &'static mut Process {
    let addr: usize;
    unsafe {
        asm!("mrs {}, CONTEXTIDR_EL1", out(reg) addr);
    }

    if addr == 0 {
        panic!("No running process")
    }

    unsafe {
        return &mut *(addr as *mut Process);
    }
}
