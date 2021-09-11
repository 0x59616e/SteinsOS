TOOLCHAIN=aarch64-none-elf-
CC=$(TOOLCHAIN)gcc
LD=$(TOOLCHAIN)ld

CFLAGS=-Wall -Wextra -pedantic -O0 -g
CFLAGS+=-static -ffreestanding -nostdlib -fno-exceptions -fno-omit-frame-pointer
CFLAGS+=-fno-pie -no-pie
CFLAGS+=-mgeneral-regs-only

USER_CFLAGS=-Wall -c -Wextra -static -ffreestanding -nostdlib -fno-exceptions -fno-omit-frame-pointer

ROOT_DIR=$(shell pwd)
KERNEL_DIR=$(ROOT_DIR)/kernel
SLIBC_DIR=$(ROOT_DIR)/slibc
USER_DIR=$(ROOT_DIR)/user

KERNEL_LINKER=$(KERNEL_DIR)/kernel.ld
ASM=$(wildcard $(KERNEL_DIR)/src/asm/*.S)
LIBS=$(KERNEL_DIR)/target/aarch64-unknown-none/debug
LIB=steinsos

USER_LIB_DIR=$(SLIBC_DIR)/target/aarch64-unknown-none/debug/
USER_SOURCE=$(wildcard $(USER_DIR)/*.c)
USER_BASE=0xffff000000000000
USER_PROG=\
	shell \
	ls    \
	cat

CPUS=1
QEMUOPTS=  -m 1G -smp $(CPUS) -semihosting -machine virt -cpu cortex-a57 -nographic -kernel steinsos.bin
QEMUOPTS+= -machine gic-version=2
QEMUOPTS+= -drive file=fs.img,if=none,format=raw,id=x0
QEMUOPTS+= -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0

.PHONY: slibc all mkfs

crt.o: $(USER_DIR)/crt.S
	$(CC) $(CFLAGS) $<

%:  $(USER_DIR)/crt.o $(USER_DIR)/%.o $(USER_DIR)/libc.o
	cd $(USER_DIR) && \
		$(LD) -z max-page-size=4096 -N --entry __start -Ttext $(USER_BASE) -o $@ $^

mkfs: $(USER_PROG) 
	cd mkfs && \
		cargo run $(patsubst %, $(USER_DIR)/%, $^) $(ROOT_DIR)/README.md && \
		rm $(patsubst %, $(USER_DIR)/%, $^)


steinsos: $(ASM)
	cd $(KERNEL_DIR) && \
		cargo build && \
		$(CC) $(CFLAGS) -T$(KERNEL_LINKER) -L$(LIBS) $^ -l$(LIB) -o steinsos.bin && \
		mv steinsos.bin ../

all: mkfs steinsos

qemu: all
	qemu-system-aarch64 $(QEMUOPTS)
qemu-gdb: all
	qemu-system-aarch64 -s -S $(QEMUOPTS)

objdump: steinsos
	$(TOOLCHAIN)objdump -S steinsos.bin > steinsos.sym; \

clean:
	cd $(KERNEL_DIR) && cargo clean; \
	cd $(SLIBC_DIR) && cargo clean; \
	rm $(ROOT_DIR)/steinsos*; \
