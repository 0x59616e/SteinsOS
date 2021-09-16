use core::ptr;
use alloc::boxed::Box;
use alloc::vec;
use core::mem;
use crate::fs::buffer::Buffer;
use crate::process;

macro_rules! reg {
    ($name:ident, $addr:expr) => {
        const $name: usize = $addr;
    };
}

macro_rules! mb {
    () => {
        asm!("dsb sy");
    };
}

reg!(MAGIC_VALUE       , 0x000);
reg!(VERSION           , 0x004);
reg!(DEVICE_ID         , 0x008);
// reg!(VENDOR_ID         , 0x00c);
reg!(DEVICE_FEAT       , 0x010);
// reg!(DEVICE_FEAT_SEL   , 0x014);
reg!(DRIVER_FEAT       , 0x020);
// reg!(DRIVER_FEAT_SEL   , 0x024);
reg!(GUEST_PAGE_SIZE   , 0x028);
reg!(QUEUE_SEL         , 0x030);
reg!(QUEUE_NUM_MAX     , 0x034);
reg!(QUEUE_NUM         , 0x038);
reg!(QUEUE_PFN         , 0x040);
// reg!(QUEUE_READY       , 0x044);
reg!(QUEUE_NOTIFY      , 0x050);
reg!(INTERRUPT_STATUS  , 0x060);
reg!(INTERRUPT_ACK     , 0x064);
reg!(STATUS            , 0x070);
// reg!(QUEUE_DESC_LOW    , 0x080);
// reg!(QUEUE_DESC_HIGH   , 0x084);
// reg!(QUEUE_AVAIL_LOW   , 0x090);
// reg!(QUEUE_AVAIL_HIGH  , 0x094);
// reg!(QUEUE_USED_LOW    , 0x0a0);
// reg!(QUEUE_USED_HIGH   , 0x0a4);
// reg!(CONFIG_GENERATION , 0x0fc);
// reg!(CONFIG            , 0x100);

const VIRTIO_MAGIC:              u32 = 0x74726976;
const VIRTIO_VERSION:            u32 = 0x01;
const VIRTIO_STATUS_ACKNOWLEDGE: u32 = 1;
const VIRTIO_STATUS_DRIVER:      u32 = 2;
const VIRTIO_STATUS_DRIVER_OK:   u32 = 4;
const VIRTIO_STATUS_FEAT_OK:     u32 = 8;
const VIRTIO_DEV_BLK:            u32 = 0x02;

const NUM: u32 = 8;

static mut DISK: Disk = Disk::new();

#[repr(packed)]
struct VirtIO {
    base: usize,
    irq:  usize,
}

impl VirtIO {
    fn new(idx: usize) -> Self {
        Self {
            base: 0x0a000000 + 0x200 * idx,
            irq: 32 + 0x10 + idx,
        }
    }

    // https://brennan.io/2020/03/22/sos-block-device/
    unsafe fn init(&self) {
        if self.read(MAGIC_VALUE) != VIRTIO_MAGIC ||
           self.read(VERSION)     != VIRTIO_VERSION
        {
            panic!("Error: virtio");
        }

        if self.read(DEVICE_ID) == 0 {
            println!("Warn: virtio");
        }

        // reset
        let mut status = 0;
        self.write(STATUS, status);
        mb!();
        status |= VIRTIO_STATUS_ACKNOWLEDGE;
        self.write(STATUS, status);
        mb!();
        status |= VIRTIO_STATUS_DRIVER;
        self.write(STATUS, status);
        mb!();

        if self.read(DEVICE_ID) != VIRTIO_DEV_BLK {
            panic!("Error: virtio");
        }

        let mut features = self.read(DEVICE_FEAT);
        features &= !(1 << 5);
        features &= !(1 << 7);
        features &= !(1 << 11);
        features &= !(1 << 12);
        features &= !(1 << 27);
        features &= !(1 << 28);
        features &= !(1 << 29);
        self.write(DRIVER_FEAT, features);
        status |= VIRTIO_STATUS_FEAT_OK;
        self.write(STATUS, status);
        status |= VIRTIO_STATUS_DRIVER_OK;
        self.write(STATUS, status);

        if self.read(STATUS) & VIRTIO_STATUS_FEAT_OK == 0 {
            panic!("ERROR: virtio");
        }

        self.write(GUEST_PAGE_SIZE, 4096);

        // initialize queue 0
        self.write(QUEUE_SEL, 0);
        let max = self.read(QUEUE_NUM_MAX);
        if max == 0 {
            panic!("virtio disk has no queue 0");
        }

        if max < NUM {
            panic!("virtio disk max queue too short");
        }

        self.write(QUEUE_NUM, NUM);
        let addr = Box::into_raw(vec![0_u8; 8192].into_boxed_slice()) as *mut u8 as usize;
        self.write(QUEUE_PFN, (addr >> 12) as u32);

        DISK.addr = addr;

        crate::gic::irq_enable(self.irq as u32);
    }

    fn read(&self, reg: usize) -> u32 {
        unsafe {
            ptr::read_volatile((self.base + reg) as *const u32)
        }
    }

    fn write(&self, reg: usize, val: u32) {
        unsafe {
            ptr::write_volatile((self.base + reg) as *mut u32, val);
        }
    }
}

pub fn init() {
    unsafe {
        VirtIO::new(0).init();
    }
}

unsafe fn alloc3_desc() -> Result<[usize; 3], ()> {
    let mut res = [0; 3];
    let mut idx = 0;
    for (i, v) in DISK.free.iter_mut().enumerate() {
        if *v {
            *v = false;
            res[idx] = i;
            idx += 1;
        }
        if idx == 3 {
            break;
        }
    }

    if idx < 3 {
        free_desc(res, idx);
        return Err(());
    }

    Ok(res)
}

unsafe fn free_desc(idx: [usize; 3], cnt: usize) {
    for i in (0..cnt).map(|i| idx[i]) {
        if DISK.free[i] {
            panic!("free desc");
        }
        let desc = DISK.desc();
        desc[i].addr = 0;
        desc[i].len = 0;
        desc[i].flags = 0;
        desc[i].next = 0;
        DISK.free[i] = true;
        process::wakeup(core::ptr::addr_of!(DISK.free) as usize);
    }
}

pub unsafe fn disk_rw(buffer: &mut Buffer, write: bool) {
    let sector = buffer.blockno() as usize * (crate::fs::BLOCK_SIZE / 512);

    // allocate descriptor
    let idx = loop {
        match alloc3_desc() {
            Ok(idx) => break idx,
            Err(()) => process::sleep(core::ptr::addr_of!(DISK.free) as usize),
        };
    };

    let blk_req = &mut DISK.ops[idx[0]];
    blk_req.ty = match write {
        true  => VIRTIO_BLK_T_OUT,
        false => VIRTIO_BLK_T_IN,
    };
    blk_req.reserved = 0;
    blk_req.sector = sector as u64;

    let desc = DISK.desc();

    desc[idx[0]].addr = core::ptr::addr_of!(*blk_req) as u64;
    desc[idx[0]].len = mem::size_of::<VirtioBlkReq>() as u32;
    desc[idx[0]].flags = VRING_DESC_F_NEXT;
    desc[idx[0]].next = idx[1] as u16;

    let ptr = buffer.as_ptr() as u64;
    desc[idx[1]].addr = ptr;
    desc[idx[1]].len = crate::fs::BLOCK_SIZE as u32;
    desc[idx[1]].flags = match write {
        true  => 0,
        false => VRING_DESC_F_WRITE
    } | VRING_DESC_F_NEXT;
    desc[idx[1]].next = idx[2] as u16;

    DISK.info[idx[0]].status = 0xff;
    desc[idx[2]].addr = core::ptr::addr_of!(DISK.info[idx[0]].status) as u64;
    desc[idx[2]].len = 1;
    desc[idx[2]].flags = VRING_DESC_F_WRITE;
    desc[idx[2]].next = 0;

    DISK.info[idx[0]].buf = core::ptr::addr_of_mut!(*buffer);
    (*buffer).busy = true;

    DISK.avail().ring[DISK.avail().idx as usize % NUM as usize] = idx[0] as u16;
    mb!();
    DISK.avail().idx += 1;
    mb!();
    VirtIO::new(0).write(QUEUE_NOTIFY, 0);
    mb!();

    while (*DISK.info[idx[0]].buf).busy {
        process::sleep((*DISK.info[idx[0]].buf).as_ptr() as usize);
    }

    DISK.info[idx[0]].buf = core::ptr::null_mut();
    free_desc(idx, 3);
}

pub unsafe fn interrupt_handler() {
    let virtio = VirtIO::new(0);
    virtio.write(INTERRUPT_ACK, virtio.read(INTERRUPT_STATUS) & 0x03);
    mb!();

    while DISK.used_idx != DISK.used().idx {
        mb!();
        let id = DISK.used().ring[DISK.used_idx as usize % NUM as usize].id as usize;

        if DISK.info[id].status != 0 {
            panic!("virtio disk intr status {}", DISK.info[id].status);
        }

        (*DISK.info[id].buf).busy = false;
        process::wakeup((*DISK.info[id].buf).as_ptr() as usize);

        DISK.used_idx += 1;
    }
}

struct Disk {
    addr: usize,
    free: [bool; NUM as usize],
    used_idx: u16,
    ops: [VirtioBlkReq; NUM as usize],
    info: [Info; NUM as usize]
}

#[derive(Clone, Copy)]
struct Info {
    status: u8,
    buf: *mut Buffer,
}

impl Info {
    const fn new() -> Self {
        Self {
            status: 0,
            buf: core::ptr::null_mut(),
        }
    }
}

impl Disk {
    const fn new() -> Self {
        Self {
            addr: 0,
            free: [true; NUM as usize],
            used_idx: 0,
            ops: [VirtioBlkReq::new(); NUM as usize],
            info: [Info::new(); NUM as usize]
        }
    }

    unsafe fn desc(&mut self) -> &mut [VirtqDesc] {
        &mut *(self.addr as *mut VirtqDesc as *mut [VirtqDesc; NUM as usize])
    }

    unsafe fn avail(&mut self) -> &mut VirtqAvail {
        &mut *((self.addr + NUM as usize * mem::size_of::<VirtqDesc>()) as *mut VirtqAvail)
    }

    unsafe fn used(&mut self) -> &mut VirtqUsed {
        &mut *((self. addr + 0x1000) as *mut VirtqUsed)
    }
}

#[repr(C)]
struct VirtqDesc {
    addr:  u64,
    len:   u32, 
    flags: u16,
    next:  u16,
}
const VRING_DESC_F_NEXT:  u16 = 1;
const VRING_DESC_F_WRITE: u16 = 2;

#[repr(C)]
struct VirtqAvail {
    flags: u16,
    idx:   u16,
    ring:  [u16; NUM as usize],
    unused: u16,
}

#[repr(C)]
struct VirtqUsedElem {
    id: u32,
    len: u32,
}

#[repr(C)]
struct VirtqUsed {
    flags: u16,
    idx:   u16,
    ring: [VirtqUsedElem; NUM as usize],
}

const VIRTIO_BLK_T_IN: u32 = 0;
const VIRTIO_BLK_T_OUT: u32 = 1;

#[repr(C)]
#[derive(Clone, Copy)]
struct VirtioBlkReq {
    ty: u32,
    reserved: u32,
    sector: u64,
}

impl VirtioBlkReq {
    const fn new() -> Self {
        Self {
            ty: 0,
            reserved: 0,
            sector: 0,
        }
    }
}