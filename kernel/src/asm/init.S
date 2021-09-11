.section .init
.global init_main
.type init_main @function

init_main:
    ldr x0, =path
    svc 0x00 // fork
    cbnz x0, loop
    ldr x0, =1
    ldr x1, =msg
    ldr x2, =18
    svc 0x04 // write
    ldr x0, =path
    ldr x1, =0
    svc 0x01 // exec
    ldr x0, =1
    ldr x1, =err
    ldr x2, =20
loop:
    b .

path: .asciz "/shell"
err:  .asciz "start shell failed\n"
msg:  .asciz "starting shell...\n"
