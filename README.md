# Introduction
SteinsOS is a non-preemptive single-threaded kernel based on armv8.

It is buggy and still in progress. My goal is to make it more stable. Adding more features is not my first priority now.


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
You have to use relative or absolute path in the shell:
```
$ ./ls
.
shell
ls
cat
READMD.md
$ ./cat ./README.md
...
```

# Contribution
Pull requests, bug reports and any kind of suggestions are welcomed.
