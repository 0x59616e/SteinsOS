use crate::common::*;
use crate::process;
use crate::gic;
use crate::serial;

extern "C" {
    fn byebye();
}

#[repr(C)]
pub struct UserContext {
    pub elr_el1  : usize,
    pub spsr_el1 : usize,
    pub x        : [usize; 31],
    pub poison   : usize,
}

const POISON_VALUE: usize = 0x7f7f7f7f7f7f7f7f;

#[no_mangle]
extern "C" fn handle_sync(uctx: &mut UserContext, es: usize, fa: usize) {  
    uctx.poison = POISON_VALUE;

    match es >> 26 {
        0b010101 => {
            uctx.x[0] = crate::syscall::SYSCALL_TABLE[es & 0xffff](uctx)
                                .unwrap_or(-1_isize as usize);
        }
        0b100100 => {
            page_fault_handler(es, fa, uctx.elr_el1);
        }
        _ => {
            panic!("pid: {}, ESR: {:x}, ELR:{:x}", process::current().pid, es, uctx.elr_el1);
        }
    }
    
    assert_eq!(uctx.poison, POISON_VALUE);
    back_to_earth();
}

#[no_mangle]
extern "C" fn handle_int(irq: u32) {
    match irq {
        30 => unsafe {
                let x = 1000_000_usize;
                asm!("msr CNTP_TVAL_EL0, {}", in(reg) x);
                ((GICCBASE + 0x10) as *mut u32).write(irq);
                // context switch
                process::yield_cpu();
            },
        33 => {
            serial::SerialPort::new().receive();
        }
        48 => {
            // virtio blk
            unsafe {
                crate::virtio::interrupt_handler();
            }
        }
        _ => panic!("unrecognized irq number {}", irq),
    }

    unsafe {
        gic::irq_eoi(irq);
    }

    back_to_earth();
}

#[no_mangle]
extern "C" fn page_fault_handler(es: usize, fault_addr: usize, elr: usize) {
    assert!((es >> 26) == 0b100100 || (es >> 26) == 0b100101);
    let proc = process::current();
    if fault_addr < proc.heap_end() {
        // heap page fault
        proc.page_tb().create(round_down(fault_addr), PAGESIZE, "rw")
                        .expect("Can't handle page fault");
    } else {
        // FIXME: Segmentation fault ?
        panic!("page fault: from: 0b{:b}\n fault addr: 0x{:x}\n at: 0x{:x}", es >> 26, fault_addr, elr);
    }
}

pub fn back_to_earth() -> ! {
    unsafe {
        byebye();
    }
    panic!("surprise motherfucker");
}