# Introduction
SteinsOS is an operating system featuring non-preemptive kernel targeting on single-core armv8 architecture.

SteinOS is derived from following tutorials and software distributions:

[Redox OS](https://gitlab.redox-os.org/redox-os/redox): The state-of-the-art Rust Operating System.

[RISC Vに従うCPUの上で動作するOSをRustで書く（CPU実験余興](https://moraprogramming.hateblo.jp/entry/2019/03/17/165802):
Very useful blog about Rust bare-metal programming.

[Writing an OS in Rust](https://os.phil-opp.com/): An excellent series about kernel dev with Rust.

[xv6](https://github.com/mit-pdos/xv6-riscv): An Unix-like OS written in C.
# Document
Below are SteinsOS docuements, one in Taiwanese, and the other in English.

[Taiwanese](https://hackmd.io/@0x59616e/rJEE2msfY)

[English](https://hackmd.io/@0x59616e/H1kKW4ift)


# Prerequisites
Here's what you need: 
- [Rust compiler](https://www.rust-lang.org/tools/install)
```
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
- [aarch64-none-elf toolchain](https://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-a/downloads)
- qemu-system-aarch64
```
$ sudo apt-get install -y qemu-system-aarch64
```

# Build and run
Just run `make qemu`
```
$ make qemu
```
# Feature
- Preemptive multi-tasking
- Memory management
- Virtual Memory
- File system
- C library

# Shell
```
$ ls
.
shell
ls
cat
READMD.md
$ cat ./README.md
...
```

# Contribution

Pull requests, bug reports and any kind of suggestion are very welcomed.
