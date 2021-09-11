use crate::common::*;

static GICD_CTLR: u32 = 0x000;
static GICD_TYPER: u32 = 0x004;
static GICD_ISENABLER: u32 = 0x100;
static GICD_ICENABLER: u32 = 0x180;
static GICD_IPRIORITY: u32 = 0x400;
static GICD_ITARGETSR: u32 = 0x800;
static GICD_ICFGR: u32 = 0xc00;

static GICC_EOIR: u32 = 0x0010;
static GICC_CTLR: u32 = 0x0000;
static GICC_PMR: u32 = 0x0004;

static mut GIC_DIST_IF: GicDistIf = GicDistIf {
    address: GICDBASE,
    ncpus: 0,
    nirqs: 0,
};

static mut GIC_CPU_IF: GicCpuIf = GicCpuIf {
    address: GICCBASE,
};

pub unsafe fn init() {
    GIC_DIST_IF.init();
    GIC_CPU_IF.init();
}

pub fn irq_enable(irq_num: u32) {
    unsafe { GIC_DIST_IF.irq_enable(irq_num) };
}

// pub fn irq_disable(irq_num: u32) {
//     unsafe { GIC_DIST_IF.irq_disable(irq_num) };
// }

pub unsafe fn irq_eoi(irq_num: u32) {
    GIC_CPU_IF.irq_eoi(irq_num);
}

pub struct GicDistIf {
    pub address: usize,
    pub ncpus: u32,
    pub nirqs: u32,
}

impl GicDistIf {
    unsafe fn init(&mut self) {
        // Disable IRQ Distribution
        self.write(GICD_CTLR, 0);

        let typer = self.read(GICD_TYPER);
        self.ncpus = ((typer & (0x7 << 5)) >> 5) + 1;
        self.nirqs = ((typer & 0x1f) + 1) * 32;

        // Set all SPIs to level triggered
        for irq in (32..self.nirqs).step_by(16) {
            self.write(GICD_ICFGR + ((irq / 16) * 4), 0);
        }

        // Disable all SPIs
        for irq in (32..self.nirqs).step_by(32) {
            self.write(GICD_ICENABLER + ((irq / 32) * 4), 0xffff_ffff);
        }

        // Affine all SPIs to CPU0 and set priorities for all IRQs
        for irq in 0..self.nirqs {
            if irq > 31 {
                let ext_offset = GICD_ITARGETSR + (4 * (irq / 4));
                let int_offset = irq % 4;
                let mut val = self.read(ext_offset);
                val |= 0b0000_0001 << (8 * int_offset);
                self.write(ext_offset, val);
            }

            let ext_offset = GICD_IPRIORITY + (4 * (irq / 4));
            let int_offset = irq % 4;
            let mut val = self.read(ext_offset);
            val |= 0b0000_0000 << (8 * int_offset);
            self.write(ext_offset, val);
        }

        // Enable CPU0's GIC interface
        GIC_CPU_IF.write(GICC_CTLR, 1);

        // Set CPU0's Interrupt Priority Mask
        GIC_CPU_IF.write(GICC_PMR, 0xff);

        // Enable IRQ distribution
        self.write(GICD_CTLR, 0x1);
    }

    unsafe fn irq_enable(&mut self, irq: u32) {
        let offset = GICD_ISENABLER + (4 * (irq / 32));
        // let shift = 1 << (irq % 32);
        // let mut val = self.read(offset);
        // val |= shift;
        self.write(offset, 1 << (irq % 32));
    }

    // unsafe fn irq_disable(&mut self, irq: u32) {
    //     let offset = GICD_ICENABLER + (4 * (irq / 32));
    //     // let shift = 1 << (irq % 32);
    //     // let mut val = self.read(offset);
    //     // val |= shift;
    //     self.write(offset, 1 << (irq % 32));
    // }

    unsafe fn read(&self, reg: u32) -> u32 {
        let val = core::ptr::read_volatile((self.address + reg as usize) as *const u32);
        val
    }

    unsafe fn write(&mut self, reg: u32, value: u32) {
        core::ptr::write_volatile((self.address + reg as usize) as *mut u32, value);
    }
}

pub struct GicCpuIf {
    pub address: usize,
}

impl GicCpuIf {
    unsafe fn init(&mut self) {
    }

    unsafe fn irq_eoi(&mut self, irq: u32) {
        self.write(GICC_EOIR, irq);
    }

    // unsafe fn read(&self, reg: u32) -> u32 {
    //     let val = core::ptr::read_volatile((self.address + reg as usize) as *const u32);
    //     val
    // }

    unsafe fn write(&mut self, reg: u32, value: u32) {
        core::ptr::write_volatile((self.address + reg as usize) as *mut u32, value);
    }
}
