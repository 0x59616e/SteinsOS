use std::env;
use std::fs;

use inode::*;
use superblock::*;

// the first block is superblock
mod inode;
mod superblock;

const BLOCK_SIZE: usize = 1024;

fn as_u8<T>(src: &T) -> &[u8] {
    let ptr = std::ptr::addr_of!(*src) as *const u8;
    let len = std::mem::size_of::<T>();
    unsafe {
        std::slice::from_raw_parts(ptr, len)
    }
}

fn to_block<T>(src: &T) ->  Vec<u8> {
    let mut v = as_u8(src).to_vec();
    assert!(v.len() <= BLOCK_SIZE);
    v.resize(BLOCK_SIZE, 0_u8);
    v
}

const INODE_COUNT: u32 = 50;
const DATA_BLOCK_COUNT: usize = 1000;
const FIRST_INODE_BLOCK: u32 = 1;
const FIRST_DATA_BLOCK: u32 = 53;

fn main() {
    // block 0: superblock
    // block 1 ~ 50: inode
    // block 51: inode bitmap
    // block 52: data block bitmpa
    // block 53~: data block

    let superblock = Superblock {
        inode_count: INODE_COUNT,
        inode_bitmap:  51,
        data_block_bitmap: 52,
    };

    let mut result = Vec::<u8>::new();
    result.append(&mut to_block(&superblock));

    let mut root_block = Vec::<u8>::new();
    let mut root_inode = Inode {
        ty: 0,
        num: 1,
        size: 0,      // unknown
        addr: [FIRST_DATA_BLOCK, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    };

    // current directory
    root_block.extend_from_slice(as_u8(&FIRST_INODE_BLOCK));
    root_block.extend_from_slice(&{
        let mut name = [0_u8; 12];
        name[0] = b'.';
        name
    });

    let mut data_blocks = Vec::<Vec<u8>>::new();
    let mut inodes = Vec::<Inode>::new();

    let mut inode_curr: u32 = FIRST_INODE_BLOCK + 1;
    let mut block_curr = FIRST_DATA_BLOCK + 1;

    for prog in env::args().skip(1) {
        let mut prog_name = prog.split('/').last().unwrap().as_bytes().to_vec();
        let contents = fs::read(prog).unwrap();

        assert!(prog_name.len() <= 12);

        // struct Dirent {
        //     inode_num: u32,
        //     name: [u8; 12],
        // }

        prog_name.resize(12, 0_u8);
        root_block.extend_from_slice(as_u8(&inode_curr));
        root_block.append(&mut prog_name);

        let mut addr = [0_u32; 13];
        // round up to BLOCK_SIZE
        let block_cnt = ((contents.len() + BLOCK_SIZE - 1) & !(BLOCK_SIZE - 1)) / BLOCK_SIZE;
        (0..block_cnt).for_each(|i| {
            addr[i] = block_curr;
            block_curr += 1;
        });

        let inode = Inode {
            ty: 1,
            num: inode_curr,
            size: contents.len() as u32,
            addr,
        };

        println!("{:?}", inode);
        inodes.push(inode);
        data_blocks.push(contents);
        inode_curr += 1;
    }
    root_inode.size = root_block.len() as u32;
    println!("{:?}", root_inode);
    assert!(root_block.len() <= BLOCK_SIZE);
    inodes.insert(0, root_inode);
    data_blocks.insert(0, root_block);

    let inode_count = inodes.len();
    let data_block_count = data_blocks.iter()
                                        .fold(0_usize, |acc, x| acc + x.len() / 1024 + (x.len() % 1024 != 0) as usize);

    assert!(inodes.len() <= INODE_COUNT as usize);
    inodes.resize(INODE_COUNT as usize, Inode { ty: 0, num: 0, size: 0, addr:[0; 13] });

    assert!(data_blocks.len() <= DATA_BLOCK_COUNT);
    data_blocks.resize(DATA_BLOCK_COUNT, vec![0_u8; BLOCK_SIZE]);

    // write inode
    inodes.into_iter().for_each(|inode| {
        result.append(&mut to_block(&inode));
    });


    println!("data_block_count = {}, inode_count = {}", data_block_count, inode_count);
    // write inode bitmap
    let mut bitmap = vec![0_u8; BLOCK_SIZE];
    (1..=inode_count).for_each(|i| bitmap[i / 8] |= 1 << (i % 8));
    result.append(&mut bitmap);

    // write data block bitmap
    let mut bitmap = vec![0_u8; BLOCK_SIZE];
    (0..data_block_count).for_each(|i| bitmap[i / 8] |= 1 << (i % 8));
    result.append(&mut bitmap);

    // write data block
    data_blocks.into_iter().for_each(|mut data| {
        result.append(&mut data);
        result.resize((result.len() + BLOCK_SIZE - 1) & !(BLOCK_SIZE - 1), 0);
    });

    fs::write("../fs.img", result).unwrap();
}
