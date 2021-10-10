use std::env;
use std::fs;
use std::cmp::min;

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

const DATA_BLOCK_COUNT: usize = 1000;
const BITMAP_BLOCK: u32 = 1;
const ROOT_INODE_BLOCK: u32 = 2;

fn main() {
    // block 0: superblock
    // block 1: bitmap
    // block 2: root inode
    // block 3~: data block

    let superblock = Superblock {
        root_inode: ROOT_INODE_BLOCK,
        bitmap_block: BITMAP_BLOCK,
    };

    let inode_count = env::args().count() as u32;

    let mut result = Vec::<u8>::new();
    result.append(&mut to_block(&superblock));

    let mut root_block = Vec::<u8>::new();
    let mut root_inode = Inode {
        ty: 0,
        num: ROOT_INODE_BLOCK,
        parent: ROOT_INODE_BLOCK,
        size: 0,      // unknown
        addr: [inode_count + 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    };

    // current directory
    root_block.extend_from_slice(as_u8(&ROOT_INODE_BLOCK));
    root_block.extend_from_slice(&{
        let mut name = [0_u8; 12];
        name[0] = b'.';
        name
    });

    let mut data_blocks = Vec::<Vec<u8>>::new();
    let mut inodes = Vec::<Inode>::new();

    let mut inode_curr: u32 = ROOT_INODE_BLOCK + 1;
    let mut block_curr = inode_count + 3;

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

        (0..min(block_cnt, 13)).for_each(|i| {
            addr[i] = block_curr;
            block_curr += 1;
        });

        if block_cnt >= 13 {
            unimplemented!()
        }

        let inode = Inode {
            ty: 1,
            num: inode_curr,
            parent: ROOT_INODE_BLOCK,
            size: contents.len() as u32,
            addr,
        };

        // println!("{:?}", inode);
        inodes.push(inode);
        data_blocks.push(contents);
        inode_curr += 1;
    }
    root_inode.size = root_block.len() as u32;
    // println!("{:?}", root_inode);
    assert!(root_block.len() <= BLOCK_SIZE);
    inodes.insert(0, root_inode);
    data_blocks.insert(0, root_block);

    let data_block_count = data_blocks.iter()
                                        .fold(0_usize, |acc, x| acc + x.len() / 1024 + (x.len() % 1024 != 0) as usize);

    assert!(data_blocks.len() <= DATA_BLOCK_COUNT);
    data_blocks.resize(DATA_BLOCK_COUNT, vec![0_u8; BLOCK_SIZE]);

    // write data block bitmap
    let mut bitmap = vec![0_u8; BLOCK_SIZE];
    (0..(inode_count as usize + data_block_count + 2)).for_each(|i| bitmap[i / 8] |= 1 << (i % 8));
    // println!("{:?}", bitmap);
    result.append(&mut bitmap);

    // write inode
    inodes.into_iter().for_each(|inode| {
        result.append(&mut to_block(&inode));
    });


    // println!("data_block_count = {}, inode_count = {}", data_block_count, inode_count);


    // write data block
    data_blocks.into_iter().for_each(|mut data| {
        result.append(&mut data);
        result.resize((result.len() + BLOCK_SIZE - 1) & !(BLOCK_SIZE - 1), 0);
    });

    fs::write("../fs.img", result).unwrap();
}
