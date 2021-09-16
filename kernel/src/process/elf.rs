#[repr(C)]
#[derive(Debug)]
pub struct FileHeader {
    magic:         [u8; 4], // magic number
    class:         u8,  // 1 indicates 32bits, 2 for 64bits
    data:          u8,  // 1 indicates little endian, 2 for big
    version:       u8,  // set to 1
    osabi:         u8,  // abi
    abiversion:    u8,
    pad:           [u8; 7], // unused
    ty:            u16, // identifies object file type.
    machine:       u16,
    version_again: u32, // ???
    // memory address of the entry point from where the process start executing
    entry:         u64,
    // points to the start of the program header table.
    // it usually follows the file header immediately
    phoff:         u64,
    // points to the start of the section header table.
    shoff:         u64,
    flags:         u32, // interpretation of this field depends on the target architecture.
    size:          u16, // size of this header, 64 bytes for 64bits
    phentsize:     u16, // the size of the program header table entry
    phnum:         u16, // the number of entries in the program header table
    shentsize:     u16, // the size of a section header table entry
    shnum:         u16, // the number of entries in the section header table
    // contains index of the section header table entry that contains the section names.
    shstrndx:      u16,
}

#[repr(C)]
#[derive(Debug)]
pub struct ProgramHeader {
    pub(super) ty:     u32,
    pub(super) flags:  u32,
    pub(super) offset: u64,
    pub(super) vaddr:  u64,
    pub(super) paddr:  u64,
    pub(super) filesz: u64,
    pub(super) memsz:  u64,
    pub(super) align:  u64,
}

impl ProgramHeader {
    pub fn is_loadable(&self) -> bool {
        self.ty ==  0x01
    }
}

pub fn read_fileheader(prog: &[u8]) -> &FileHeader {
    let file_header = unsafe {
        &*(prog.as_ptr() as *const FileHeader)
    };

    assert_eq!(file_header.magic, [0x7f, 0x45, 0x4c, 0x46]);
    file_header
}

pub fn read_program_header_table<'a>(
    prog: &'a [u8],
    file_header: &FileHeader
) -> &'a [ProgramHeader] {
    let ptr = unsafe {
        prog.as_ptr().add(file_header.phoff as usize) as *const ProgramHeader
    };

    unsafe {
        core::slice::from_raw_parts(ptr, file_header.phnum as usize)
    }
}