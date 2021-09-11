.global __start
.type __start @function

__start:
    bl main
    svc 0x07