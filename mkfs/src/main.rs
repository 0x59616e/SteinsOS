use std::env;
use std::fs;

use inode::*;
use superblock::*;

// the first block is superblock
mod inode;
mod superblock;

fn as_u8<T>(src: &T) -> &[u8] {
    let ptr = std::ptr::addr_of!(*src) as *const u8;
    let len = std::mem::size_of::<T>();
    unsafe {
        std::slice::from_raw_parts(ptr, len)
    }
}

fn to_block<T>(src: &T) ->  Vec<u8> {
    let mut v = as_u8(src).to_vec();
    assert!(v.len() <= GATEFS_BLOCK_SIZE);
    v.resize(GATEFS_BLOCK_SIZE, 0_u8);
    v
}

fn inode_range(ino: u32) -> (usize, usize) {
    let start = (inode_blk_no(ino) * GATEFS_BLOCK_SIZE as u32 + inode_blk_shift(ino)) as usize;
    let end = start + core::mem::size_of::<Inode>();
    (start, end)
}

const FREE_BLOCKS: u32 = 50; // 50MiB
fn main() {
    let mut datas: Vec<(Inode, Vec<u8>)> = Vec::new();
    let mut result: Vec<u8> = Vec::new();

    result.resize((FREE_BLOCKS << 20) as usize, 0_u8);

    // prepare root directory
    let progs = env::args().skip(1);

    let mut blk_no = GATEFS_DATA_BLOCK_NR;
    let mut i_no = 0;

    let root_i = Inode {
        i_type: 1, // directory
        i_no: i_no,
        i_parent: 0,
        i_size: 0,
        i_nlink: progs.count() as u32,
        root_addr_block: blk_no,
    };
    blk_no += 1;
    i_no += 1;

    let range = inode_range(0);
    result[range.0..range.1].copy_from_slice(as_u8(&root_i));

    for prog in env::args().skip(1) {
        let prog = fs::read(prog).expect("Read program failed");
        let inode = Inode {
            i_type: 0,
            i_no: i_no,
            i_parent: 0,
            i_size: prog.len() as u32,
            i_nlink: 0,
            root_addr_block: blk_no,
        };
        let range = inode_range(i_no);
        blk_no += 1;
        i_no += 1;
        
        result[range.0..range.1].copy_from_slice(as_u8(&inode));
        datas.push((inode, prog));
    }

    // prepare superblock
    let sb = Superblock {
        magic: GATEFS_MAGIC,
        inode_bitmap_block_nr: GATEFS_INODE_BITMAP_BLOCK_NR,
        data_block_bitmap_nr: GATEFS_DATA_BITMAP_BLOCK_NR,
        root_inode_block_nr: inode_blk_no(0),
        nr_free_blocks: FREE_BLOCKS,
        nr_free_inodes: GATEFS_FREE_INODES,
    };
    result[0..GATEFS_BLOCK_SIZE].copy_from_slice(&to_block(&sb));

    for (inode, mut data) in datas {
        let len = (data.len() + GATEFS_BLOCK_SIZE - 1) & !(GATEFS_BLOCK_SIZE - 1);
        data.resize(len, 0);

        // prepare address block
        let addr = inode.root_addr_block as usize;
        let addr_block = unsafe {
            let (start, end) = (addr * GATEFS_BLOCK_SIZE, (addr + 1) * GATEFS_BLOCK_SIZE);
            let ptr = core::ptr::addr_of_mut!(result[start..end]) as *mut u8 as *mut AddrBlock;
            &mut (*ptr)
        };

        addr_block.header = AddrBlockHeader {
            leaf: 0,
            entries_cnt: 1,
        };

        let blk_cnt = (len / GATEFS_BLOCK_SIZE) as u32;
        addr_block.entries[0] = AddrEntry {
            logical_blk: 0,
            physical_blk: blk_no,
            len: blk_cnt,
        };
        let start = blk_no as usize * GATEFS_BLOCK_SIZE;
        let end = start + len;
        result[start..end].copy_from_slice(&data);

        blk_no += blk_cnt;
    }

    // inode bitmap
    let mut bitmap = [0_u8; 1024 / 8];
    (0..i_no as usize).for_each(|i|{
        bitmap[i / 8] |= 1 << (i % 8);
    });

    let start = GATEFS_INODE_BITMAP_BLOCK_NR as usize * GATEFS_BLOCK_SIZE;
    let end = start + GATEFS_BLOCK_SIZE / 8;
    result[start..end].copy_from_slice(&bitmap);

    // data bitmap
    let mut bitmap = [0_u8; 1024/ 8];
    (0..blk_no as usize).for_each(|i|{
        bitmap[i / 8] |= 1 << (i % 8);
    });
    let start = GATEFS_DATA_BITMAP_BLOCK_NR as usize * GATEFS_BLOCK_SIZE;
    let end = start + GATEFS_BLOCK_SIZE / 8;
    result[start..end].copy_from_slice(&bitmap);

    fs::write("../fs.img", result).expect("Write fs.img failed");
}
