.global startup
.type startup @function

startup:
    // initialize stack
    msr spsel, 1
    ldr x0, =stack_top
    mov sp, x0

    // initialize exception vector table
    ldr x0, =exception_table
    msr vbar_el1, x0

    // set timer freq
    ldr x0, =1000000 // 1MHz
    msr CNTFRQ_EL0, x0

    // enable timer
    ldr x0, =1
    msr CNTP_CTL_EL0, x0

    ldr x0, =kernel_tt
    ldr x1, =kernel_text_end
    ldr x2, =init_main
    b start
