pub const PAGESIZE:   usize = 4096;
pub const PAGESHIFT:  usize = 12;
pub const KERNELBASE: usize = 0x40000000;
pub const MEMSIZE:    usize = 1 << 30; // 1GB
pub const PHYEND:     usize = KERNELBASE + MEMSIZE;

pub const UARTBASE:   usize = 0x09000000;
pub const UARTSIZE:   usize = 0x00001000;

pub const GICDBASE:   usize = 0x08000000;
pub const GICDSIZE:   usize = 0x00010000;

pub const GICCBASE:   usize = 0x08010000;
pub const GICCSIZE:   usize = 0x00010000;

pub const VIRTMMIOBASE: usize = 0x0a000000;
pub const VIRTMMIOSIZE: usize = 0x00004000;

pub fn round_up_with(v: usize, s: usize) -> usize {
    assert!(s & (s - 1) == 0);
    (v + s - 1) & !(s - 1)
}

pub fn round_down_with(v: usize, s: usize) -> usize {
    assert!(s & (s - 1) == 0);
    v & !(s - 1)
}

pub fn round_up(addr: usize) -> usize {
    round_up_with(addr, PAGESIZE)
}

pub fn round_down(addr: usize) -> usize {
    round_down_with(addr, PAGESIZE)
}
