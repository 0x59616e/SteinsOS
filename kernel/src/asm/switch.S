.global switch
.type switch @function

switch:
    mrs x2, sp_el0
    mov x3, sp
    mrs x4, ttbr1_el1
    stp x2, x3,  [x0], #16
    stp x4, x19, [x0], #16
    stp x20, x21, [x0], #16
    stp x22, x23, [x0], #16
    stp x24, x25, [x0], #16
    stp x26, x27, [x0], #16
    stp x28, x29, [x0], #16
    str x30, [x0]

    ldp x2, x3, [x1], #16
    ldp x4, x19, [x1], #16
    ldp x20, x21, [x1], #16
    ldp x22, x23, [x1], #16
    ldp x24, x25, [x1], #16
    ldp x26, x27, [x1], #16
    ldp x28, x29, [x1], #16
    ldr x30, [x1]
    mov sp, x3
    msr sp_el0, x2
    msr ttbr1_el1, x4
    TLBI VMALLE1
    dsb sy
    isb
    ret


