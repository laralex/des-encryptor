// use std::fs::{File, OpenOptions};
// use std::io::prelude::*;
// use std::path::{Path, PathBuf};
// use std::io::{BufReader, BufWriter, Read, Write};

// // TODO: tables are on the heap
// // TODO: table with known size
// struct PermutationTable {
//     forward: Vec<u8>,
//     inverse: Vec<u8>,
// }

// impl PermutationTable {
//     fn new_ip() -> PermutationTable {
//         PermutationTable {
//             forward: [
// 		58, 50, 42, 34, 26, 18, 10, 02,
//                 60, 52, 44, 36, 28, 20, 12, 04,
//                 62, 54, 46, 38, 30, 22, 14, 06,
//                 64, 56, 48, 40, 32, 24, 16, 08,
//                 57, 49, 41, 33, 25, 17, 09, 01,
//                 59, 51, 43, 35, 27, 19, 11, 03,
//                 61, 53, 45, 37, 29, 21, 13, 05,
//                 63, 55, 47, 39, 31, 23, 15, 07,
//             ],
//             inverse: [
//                 40, 08, 48, 16, 56, 24, 64, 32,
// 	        39, 07, 47, 15, 55, 23, 63, 31,
// 	        38, 06, 46, 14, 54, 22, 62, 30,
// 	        37, 05, 45, 13, 53, 21, 61, 29,
// 	        36, 04, 44, 12, 52, 20, 60, 28,
// 	        35, 03, 43 ,11, 51, 19, 59, 27,
// 	        34, 02, 42, 10, 50, 18, 58, 26,
// 	        33, 01, 41, 09, 49, 17, 57, 25,  
//             ],
//         }
//     }
// }

// struct DesEncryptor {
//     ip: PermutationTable,
//     e: ExpansionTable,
//     p: PermutationTable,
//     s: [;8]
// }


// fn des_encrypt<R: Read, W: Write>(input: R, output: W, key: i64) -> std::io::Result<()>{
//     const BUF_SIZE: usize = 4*1024;
//     let mut buf_read = BufReader::with_capacity(BUF_SIZE, input);
//     let mut buf_write = BufWriter::with_capacity(BUF_SIZE, output);
//     // TODO: concurrency
//     while let mut data_batch = buf_read.fill_buf()? {
//         des_encrypt_data(&mut data_batch, key);
//         buf_write.write_all(data_batch)?;
//     }
// }

// #[inline]
// fn des_encrypt_data(data: &mut [u8], key: i64) {
//     const BLOCK_SIZE: usize = 64;
//     assert_eq!(data.len() % BLOCK_SIZE, 0);
//     data.chunks_exact_mut(64).map(
//       |block| des_encrypt_block(block, key)  
//     );
// }

// // TODO: hide in details
// #[inline]
// fn des_encrypt_block(block_64b: &mut [u8], key: i64){
//     //const BLOCK_SIZE: usize = 64;
//     //assert_eq!(block_64b.len(), BLOCK_SIZE);
//     const ROUNDS_NUMBER: usize = 16;
//     let ip = PermutationTable::new_ip();
//     ip.forward.permutate(block_64b);
//     for _ in 0..ROUNDS_NUMBER {
//         let round_key = get_round_key(block_64b, key)
//         des_encrypt_round(block_64b, round_key);
//     }
//     ip.inverse.permutate(block_64b);
// }

// #[inline]
// fn des_encrypt_round(block_64b: &mut [u8], round_key: i64){
//     let mut expansion = des_expand(block_64b[4..]);
//     expansion ^= round_key;
//     let chunks_to_secure = chunks_by_bits(&block_64b, 6);
//     let secured_chunks = chunks_to_secure.iter_mut()
//         .enumerate()
//         .map(|(i, chunk)| secure_chunk(i, chunk))
// }

// fn chunks_by_bits(block: &[u8], bits_in_chunk) -> Vec<u8> {
//     let total_bits = block.len() * 8;
//     assert!(total_bits % bits_in_chunk == 0); // is no-tail division
//     let total_chunks = total_bits / bits_in_chunk;
    
//     for chunk in 0..total_chunks{
        
//     }
//     for bit_i in 0..total_bits {
        
//     }
    
// }

mod des;
mod data_io;

use des::cli;

fn main() {
    let cli = cli::EncryptorCli::new()
        .default_key("FFFF-0000-FFFF-0000")
        .expect("Hard coded key is invalid")
        .parse_args(std::env::args());
    
    if let None = cli {
        cli::EncryptorCli::print_help();
        return;
    }
    let cli = cli.unwrap();
    
    cli.announce_begin();

    let (read, write) = data_io::open_rw_files(
        &cli.src_file_path,
        &cli.dst_file_path
    ).expect("Failed I/O operation.");

    des::api::encrypt(read, write, cli.key);
    
    cli.announce_end();
}
