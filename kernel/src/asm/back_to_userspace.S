.global byebye
.type byebye @function

byebye:
    // clear ESR_EL1
    msr ESR_EL1, xzr
    // the context is store at the bottom of kernel stack
    // load kernel stack
    mov x30, sp

    // go to the bottom of kernel stack
    // since the kernel stack is 4 pages align, the least 14 bits of the bottom must be zero
    bfi x30, xzr, 0, 14

    // reset the kernel stack
    add x29, x30, #16384
    mov sp, x29

    // elr_el1
    ldr x1, [x30], 8
    msr elr_el1, x1

    // spsr_el1
    ldr x1, [x30], 8
    msr spsr_el1, x1

    // general purpose register
    ldp x0, x1, [x30], 16
    ldp x2, x3, [x30], 16
    ldp x4, x5, [x30], 16
    ldp x6, x7, [x30], 16
    ldp x8, x9, [x30], 16
    ldp x10, x11, [x30], 16
    ldp x12, x13, [x30], 16
    ldp x14, x15, [x30], 16
    ldp x16, x17, [x30], 16
    ldp x18, x19, [x30], 16
    ldp x20, x21, [x30], 16
    ldp x22, x23, [x30], 16
    ldp x24, x25, [x30], 16
    ldp x26, x27, [x30], 16
    ldp x28, x29, [x30], 16
    ldr x30, [x30]
    eret