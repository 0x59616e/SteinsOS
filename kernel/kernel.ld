ENTRY(startup)

SECTIONS
{
    . = 0x40000000;
    .text : {
        *(.text*)
    }
    
    .init : {
        *(.init)
    }

    . = ALIGN(0x1000);
    PROVIDE(kernel_text_end = .);

    . += 0x4000;  /* stack */
    PROVIDE(stack_top = .);

    .rodata : {
        *(.srodata .srodata.*) /* do not need to distinguish this from .rodata */
        . = ALIGN(16);
        *(.rodata .rodata.*)
    }

    .data : {
        . = ALIGN(16);
        *(.sdata .sdata.*) /* do not need to distinguish this from .data */
        . = ALIGN(16);
        *(.data .data.*)
    }

    .bss : {
        . = ALIGN(16);
        *(.sbss .sbss.*) /* do not need to distinguish this from .bss */
        . = ALIGN(16);
        *(.bss .bss.*)
    }

    /* kernel tranlation table */
    . = ALIGN(0x1000);
    PROVIDE(kernel_tt = .);
    . += 0x1000;

    . = ALIGN(0x1000);
    PROVIDE(kernel_end = .);
}