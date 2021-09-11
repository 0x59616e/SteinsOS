# Introduction
SteinsOS is a non-preemptive single-threaded kernel based on armv8. \
It is buggy and still in progress. My goal is to make it more stable. Adding more features is not my first priority now. \
Any kind of suggestions are welcomed. \
# Prerequisites
Here's what you need: 
- [Rust compiler](https://www.rust-lang.org/tools/install)
- aarch64-none-elf toolchain
- qemu-system-aarch64

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
