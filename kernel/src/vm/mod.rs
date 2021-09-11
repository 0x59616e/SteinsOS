use crate::common::*;
use core::{alloc::{Layout, GlobalAlloc}, ops::{Index, IndexMut}};

pub fn init(kernel_tt: usize, kernel_text_end: usize) {
    // initialize virtual memory translation
    unsafe {
        // Outer-sharable
        // Normal memory, Outer Write-Back Read-Allocate Write-Allocate Cacheable
        // Normal memory, Inner Write-Back Read-Allocate Write-Allocate Cacheable
        // 48 bits address space for TTBR0_EL1
        // 48 bits address space for TTBR1_EL1
        let x = 0x5a5102410_usize;
        asm!("msr TCR_EL1, {}", in(reg) x);

        // Attr1: Normal
        // Attr0: Device_nGnRnE
        let x = 0xff00_usize;
        asm!("msr MAIR_EL1, {}", in(reg) x);
    }

    let mut pgt = PageTable::from(kernel_tt);

    // VIRT MMIO
    pgt.map(VIRTMMIOBASE,
            VIRTMMIOBASE,
            VIRTMMIOSIZE,
            PageTableKind::Kernel, "rw");

    // GIC Distributor interface
    pgt.map(GICDBASE, GICDBASE, GICDSIZE, PageTableKind::Kernel, "rw");
    // GIC CPU interface
    pgt.map(GICCBASE, GICCBASE, GICCSIZE, PageTableKind::Kernel, "rw");
    // UART
    pgt.map(UARTBASE, UARTBASE, UARTSIZE, PageTableKind::Kernel, "rw");
    // kernel code
    pgt.map(KERNELBASE, KERNELBASE, kernel_text_end - KERNELBASE, PageTableKind::Kernel, "rx");
    // kernel data & physical memory
    pgt.map(kernel_text_end,
            kernel_text_end,
            PHYEND - kernel_text_end,
            PageTableKind::Kernel, "rw");

    unsafe {
        let mut x: usize;
        asm!("mrs {}, SCTLR_EL1", out(reg) x);
        x |= 1;
        asm!("msr TTBR1_EL1, {0}",
             "msr TTBR0_EL1, {0}",
             "msr SCTLR_EL1, {1}",
             "DSB SY",
             "ISB", in(reg) kernel_tt, in(reg) x);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PageTableKind {
    User,
    Kernel,
}

#[repr(C)]
#[derive(Debug)]
pub struct PageTableEntry {
    data: usize
}
// APTable [62:61]
const USER_TABLE:      usize = 0;
const KERNEL_TABLE:    usize = 0b01 << 61;
// Upper attribute
const UXN:             usize = 1 << 54;  // Unprivilege execute never
const PXN:             usize = 1 << 53;  // Privilege   execute never
const _CONTIGIOUS:     usize = 1 << 52;  // Contiguous bit

// Lower attribute
const NG:              usize = 1 << 11;  // non-Global
const AF:              usize = 1 << 10;  // Access Flag
// [9:8] => SH: Sharebility bits (10: Outer shareable)
const OUTER_SHAREABLE: usize = 0b10 << 8;
// [7:6] => AP: Data Access Permission bits
const AP_RO:           usize = 1 << 7; // Read Only
const AP_UA:           usize = 1 << 6; // Unprivilege access
// [4:2] => AttrIndex: Memory attributes index
const MEMORY_NORMAL:   usize = 1 << 2;
const MEMORY_DEVICE:   usize = 0 << 2;
// [1]   => 0 indicates block descriptor, 1 indicates table/page descriptor
// [0]   => valid bit
const ENTRY_TABLE:     usize = 1 << 1;
const ENTRY_PAGE:      usize = 1 << 1;
const VALID:           usize = 1 << 0;

impl PageTableEntry {
    const PHYSICAL_ADDRESS_BITS: usize = 0xffff_ffff_f000;

    fn new_table(&mut self, pa: usize, kind: PageTableKind) {
        let data = pa | match kind {
            PageTableKind::User   => USER_TABLE,
            PageTableKind::Kernel => KERNEL_TABLE,
        } | ENTRY_TABLE | VALID;

        self.data = data;
    }

    fn new_block(&mut self, pa: usize, kind: PageTableKind, ro: bool, x: bool, block: usize) {
        assert!(!self.is_valid());
        // In armv8.0, Access Flag set to 0 will trigger trap on the first access
        // I don't want to handle that.
        let mut data = pa | AF | VALID | OUTER_SHAREABLE;
        if ro {
            data |= AP_RO;
        }

        if kind == PageTableKind::User {
            data |= AP_UA;
        }

        data |= match (x, kind) {
            (true, PageTableKind::Kernel) => UXN,
            (true, PageTableKind::User)   => PXN,
            (false, _)                    => UXN | PXN,
        };

        data |= if pa >= KERNELBASE {
            MEMORY_NORMAL
        } else {
            MEMORY_DEVICE
        };

        if kind == PageTableKind::User {
            data |= NG; // non-Global
        }

        if block == BLOCK_4KB {
            data |= ENTRY_PAGE;
        }

        self.data = data;
    }

    fn is_valid(&self) -> bool {
        self.data & VALID != 0
    }

    // fn is_page_or_block(&self, level: u8) -> bool {
    //     self.is_valid() && 
    //     ((level < 3 && self.data & ENTRY_TABLE == 0) || level == 3)
    // }

    fn is_table(&self, level: u8) -> bool {
        self.is_valid() &&
        level < 3 &&
        self.data & ENTRY_TABLE != 0
    }

    fn as_addr(&self) -> Option<usize> {
        match self.is_valid() {
            true  => Some(self.data & Self::PHYSICAL_ADDRESS_BITS),
            false => None
        }
    }

    fn invalidate(&mut self, level: u8) {
        if self.is_valid() {
            let size = match (self.is_table(level), level) {
                (true, _)  => PAGESIZE,
                (false, 1) => BLOCK_1GB,
                (false, 2) => BLOCK_2MB,
                (false, 3) => BLOCK_4KB,
                _ => panic!("??"),
            };

            unsafe {
                crate::ALLOCATOR.dealloc(self.as_addr().unwrap() as *mut u8, Layout::from_size_align_unchecked(size, 4));
            }
            self.data = 0;
        }
    }
}

#[repr(C)]
pub struct PageTable {
    entrys: &'static mut [PageTableEntry],
}

const BLOCK_1GB: usize = 1 << 30;
const BLOCK_2MB: usize = 1 << 21;
const BLOCK_4KB: usize = 1 << 12;

impl PageTable {
    pub fn as_ptr(&self) -> *const u8 {
        self.entrys.as_ptr() as *const u8
    }

    pub fn new() -> Self {
        let addr = unsafe {
            crate::ALLOCATOR.alloc(Layout::from_size_align_unchecked(PAGESIZE, 4))
        } as usize;

        Self::from(addr)
    }

    // pub fn dump(&self) {
    //     self.dump_inner(0, 0);
    // }

    // pub fn translate(&self, vaddr: usize) -> Option<usize> {
    //     self.translate_inner(vaddr, 0)
    // }

    // fn translate_inner(&self, va: usize, level: u8) -> Option<usize> {
    //     assert!(level <= 3);
    //     let entry = &self[(va, level)];

    //     if level == 3 || entry.is_page_or_block(level) {
    //         return Some(entry.as_addr()? | (va & 0xfff));
    //     }

    //     PageTable::from(entry.as_addr()?).translate_inner(va, level + 1)
    // }

    // fn dump_inner(&self, addr: usize, level: u8) {
    //     assert!(level <= 3);
    //     for (i, entry) in self.entrys.iter().enumerate() {
    //         if entry.is_valid() {
    //             let addr = addr | (i << ((3 - level) * 9 + 12));

    //             let phyaddr = entry.as_addr().unwrap();
    //             match entry.is_page_or_block(level) || level == 3 {
    //                 true  => {
    //                     print!("0x{:016X} -> 0x{:016X}, size: {}", addr, phyaddr, match level {
    //                         1 => "1GB",
    //                         2 => "2MB",
    //                         3 => "4KB",
    //                         _ => panic!("???"),
    //                     });

    //                     if entry.data & UXN != 0 {
    //                         print!(" | UXN");
    //                     }
    //                     if entry.data & PXN != 0 {
    //                         print!(" | PXN");
    //                     }
    //                     if entry.data & NG != 0 {
    //                         print!(" | nG");
    //                     }
    //                     if entry.data & AP_RO != 0 {
    //                         print!(" | RO");
    //                     }
    //                     match entry.data & AP_UA != 0 {
    //                         true => print!(" | UA"),
    //                         false => print!(" | UAN"),
    //                     }
    //                     match entry.data & MEMORY_NORMAL != 0 {
    //                         true => print!(" | NORMAL"),
    //                         false => print!(" | DEVICE"),
    //                     }
    //                     println!("");
    //                 }
    //                 false => {
    //                     PageTable::from(phyaddr).dump_inner(addr, level + 1)
    //                 }
    //             }
    //         }
    //     }
    // }

    fn map(&mut self, va: usize, mut pa: usize, mut len: usize, kind: PageTableKind, perm: &str) {
        let mut curr = round_down(va);
        pa = round_down(pa);
        let end = round_down(va + len - 1);
        while curr <= end {
            let block_size = if curr & (BLOCK_1GB - 1) == 0 && len >= BLOCK_1GB {
                BLOCK_1GB
            } else if curr & (BLOCK_2MB - 1) == 0 && len >= BLOCK_2MB {
                BLOCK_2MB
            } else {
                BLOCK_4KB
            };
            // println!("{:X} -> {:X}, {:X}", curr, pa, block_size);
            self.install(curr, pa, kind, perm, 0, block_size);
            curr  += block_size;
            pa    += block_size;
            len   -= block_size;
        }
    }

    fn install(&mut self, va: usize, pa: usize, kind: PageTableKind, perm: &str, level: u8, block: usize) {
        if level == 3 || (level == 2 && block == BLOCK_2MB) ||
                         (level == 1 && block == BLOCK_1GB) 
        {
            let (ro, x) = match perm {
                "rwx" => (false, true),
                "rw" => (false, false),
                "rx" => (true, true),
                "r"  => (true, false),
                _ => panic!("???")
            };
            self[(va, level)].new_block(pa, kind, ro, x, block);
            return;    
        }

        let addr = match self[(va, level)].as_addr() {
            Some(addr) => addr,
            None => {
                let addr = unsafe {
                    crate::ALLOCATOR.alloc(Layout::from_size_align_unchecked(4096, 4))
                } as usize;
                self[(va, level)].new_table(addr, kind);
                addr
            }
        };
        PageTable::from(addr).install(va, pa, kind, perm, level + 1, block);
    }

    pub fn create(&mut self, va: usize, len: usize, perm: &str) -> Result<usize, ()> {
        assert_eq!(va & 0xfff, 0);
        assert_eq!(len & 0xfff, 0);
        let ptr = unsafe {
            crate::ALLOCATOR.alloc(Layout::from_size_align_unchecked(len, 4))
        };

        if ptr.is_null() {
            return Err(());
        }

        self.map(va, ptr as usize, len, PageTableKind::User, perm);
        Ok(ptr as usize)
    }

    pub fn release(&mut self) {
        self.release_inner(0);
        let ptr = self.entrys.as_ptr() as *mut u8;
        unsafe {
            crate::ALLOCATOR.dealloc(ptr, Layout::from_size_align_unchecked(PAGESIZE, 4));
        }
    }

    pub fn release_inner(&mut self, level: u8) {
        for entry in self.entrys.iter_mut() {
            if entry.is_table(level) {
                PageTable::from(entry.as_addr().unwrap()).release_inner(level + 1);
            }

            entry.invalidate(level);
        }
    }
}

impl Index<(usize, u8)> for PageTable {
    type Output = PageTableEntry;
    fn index(&self, (va, level): (usize, u8)) -> &Self::Output {
        &self.entrys[(va >> (12 + (3 - level) * 9)) & 0x1ff]
    }
}

impl IndexMut<(usize, u8)> for PageTable {
    fn index_mut(&mut self, (va, level): (usize, u8)) -> &mut Self::Output {
        &mut self.entrys[(va >> (12 + (3 - level) * 9)) & 0x1ff]
    }
}

impl From<usize> for PageTable {
    fn from(addr: usize) -> Self {
        unsafe {
            PageTable {
                entrys: core::slice::from_raw_parts_mut(addr as *mut PageTableEntry, 512)
            }
        }
    }
}

impl From<*mut u8> for PageTable {
    fn from(ptr: *mut u8) -> Self {
        Self::from(ptr as usize)
    }
}

impl From<*const u8> for PageTable {
    fn from(ptr: *const u8) -> Self {
        Self::from(ptr as usize)
    }
}
