.global exception_table

.balign 0x800
//sync -> irq -> fiq -> serror
exception_table:
// current exception level with sp_el0
b .
.balign 0x80
b . 
.balign 0x80
b . 
.balign 0x80
b .

// current exception level with sp_el1
.balign 0x80
b kernel_trap_handler
.balign 0x80
b .
.balign 0x80
b .
.balign 0x80
b .

// lower exception level with aarch64
.balign 0x80
b user_trap_handler
.balign 0x80
b user_trap_handler
.balign 0x80
b .
.balign 0x80
b .


// lower exception level with aarch32
.balign 0x80
b .
.balign 0x80
b . 
.balign 0x80
b . 
.balign 0x80
b .

user_trap_handler:
    stp x0, x1, [sp, -16]!

    // calculate the bottom of kernel stack
    mov x0, sp
    bfi x0, xzr, 0, 14

    // preserve exception link register
    mrs x1, elr_el1
    str x1, [x0], 8

    // preserve PSTATE
    mrs x1, spsr_el1
    str x1, [x0], 8

    // x0
    ldr x1, [sp], 8
    str x1, [x0], 8

    // x1
    ldr x1, [sp], 8
    str x1, [x0], 8

    stp x2, x3, [x0], 16
    stp x4, x5, [x0], 16
    stp x6, x7, [x0], 16
    stp x8, x9, [x0], 16
    stp x10, x11, [x0], 16
    stp x12, x13, [x0], 16
    stp x14, x15, [x0], 16
    stp x16, x17, [x0], 16
    stp x18, x19, [x0], 16
    stp x20, x21, [x0], 16
    stp x22, x23, [x0], 16
    stp x24, x25, [x0], 16
    stp x26, x27, [x0], 16
    stp x28, x29, [x0], 16
    str x30, [x0]

    bfi x0, xzr, 0, 14
    mrs x1, esr_el1
    mrs x2, far_el1
    cbz x1, int
    b handle_sync
int:
    ldr x19, =0x08010000
    ldr x20, [x19, 0x0C]
    mov w0, w20
    b handle_int

kernel_trap_handler:
    stp  x0,  x1, [sp, -16]!
    stp  x2,  x3, [sp, -16]!
    stp  x4,  x5, [sp, -16]!
    stp  x6,  x7, [sp, -16]!
    stp  x8,  x9, [sp, -16]!
    stp x10, x11, [sp, -16]!
    stp x12, x13, [sp, -16]!
    stp x14, x15, [sp, -16]!
    stp x16, x17, [sp, -16]!
    stp x18, x19, [sp, -16]!
    stp x20, x21, [sp, -16]!
    stp x22, x23, [sp, -16]!
    stp x24, x25, [sp, -16]!
    stp x26, x27, [sp, -16]!
    stp x28, x29, [sp, -16]!
    str x30, [sp, -8]!

    mrs x0, spsr_el1
    str x0, [sp, -8]!

    mrs x0, esr_el1
    mrs x1, far_el1
    bl page_fault_handler
    
    ldr x0, [sp], 8
    msr spsr_el1, x0

    ldr x30, [sp], 8
    ldp x28, x29, [sp], 16
    ldp x26, x27, [sp], 16
    ldp x24, x25, [sp], 16
    ldp x22, x23, [sp], 16
    ldp x20, x21, [sp], 16
    ldp x18, x19, [sp], 16
    ldp x16, x17, [sp], 16
    ldp x14, x15, [sp], 16
    ldp x12, x13, [sp], 16
    ldp x10, x11, [sp], 16
    ldp  x8,  x9, [sp], 16
    ldp  x6,  x7, [sp], 16
    ldp  x4,  x5, [sp], 16
    ldp  x2,  x3, [sp], 16
    ldp  x0,  x1, [sp], 16
    eret