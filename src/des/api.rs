use std::io::{Read, Write, BufRead, BufReader, BufWriter};
use std::io;
use std::convert::AsMut;
use crate::des::details;
use crate::reinterpret_bytes;
use crate::reinterpret_bytes::Endianess;
//use super::details::*;

pub const BITS_IN_BLOCK: usize = 64;
pub const BYTES_IN_BLOCK: usize =
    (BITS_IN_BLOCK / 8) + (BITS_IN_BLOCK % 8 == 0) as usize;
pub const IO_BUF_SIZE: usize = BYTES_IN_BLOCK * 1024 * 4;

pub fn encrypt<R, W>(src: R, dst: W, key: u64, endianess: Endianess)
                     -> io::Result<()>
where R: Read, W: Write {
    transform_data(src, dst, key, &details::encrypt_block, endianess)
}

pub fn decrypt<R, W>(src: R, dst: W, key: u64, endianess: Endianess)
                     -> io::Result<()>
where R: Read, W: Write {
    transform_data(src, dst, key, &details::decrypt_block, endianess)
}

fn transform_data<R, W>(mut src: R, dst: W, key: u64, block_affector: &Fn(u64, u64) -> u64, endianess: Endianess)
                        -> io::Result<()>
where R: Read, W: Write {
    let mut read_buf  = [0u8; IO_BUF_SIZE];
    let mut write = BufWriter::with_capacity(IO_BUF_SIZE, dst);
    // TODO: concurrency
    // FIXME: somehow mutate inside of BufReader
    // TODO: endianess
    loop {
        let slice_to_write;
        let processed_size = src.read(&mut read_buf)?;
        match processed_size {
            0 => break,
            len if len < IO_BUF_SIZE => {
                const TYPE_SIZE: usize = std::mem::size_of::<u64>();
                let tail_size = len % TYPE_SIZE;
                let padding = if tail_size == 0 { 0 } else { TYPE_SIZE - tail_size };
                if tail_size > 0 {
                    //read_buf[padding..len+padding].copy_from_slice(&read_buf[..len]);
                    // unsafe{ std::ptr::copy(
                    //     read_buf[len - tail_size..len].as_ptr(),
                    //     read_buf[len - tail_size + padding..len+padding].as_mut_ptr(),
                    //     len
                    // );}
                }
                // println!("{:x?}", &read_buf[..]);
                // for padded_mem in read_buf[len - tail_size..len -
                // tail_size + padding].iter_mut() {
                for padded_mem in read_buf[len..len + padding].iter_mut() {
                    *padded_mem = 0;
                }
                // println!("{:x?}", &read_buf[..]);
                slice_to_write = &mut read_buf[..len + padding];
            },
            _ => slice_to_write = &mut read_buf,
        }
        transform_padded_slice(slice_to_write, key, block_affector);
        write.write_all(slice_to_write)?;
    }
    Ok( () )
} 
// TODO: asmut
fn transform_padded_slice(bytes: &mut [u8], key: u64, block_affector: &Fn(u64, u64) -> u64) {
    let blocks = reinterpret_bytes::as_slice_of::<u64>(bytes);
    for block in blocks.iter_mut() {
        if cfg!(target_endian = "little") {
            *block = block_affector(block.to_be(), key).to_be();
        } else {
            *block = block_affector(*block, key);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt(){
    
        let data = vec![
            0x59, 0x6F, 0x75, 0x72, 0x20, 0x6C, 0x69, 0x70,
            0x73, 0x20, 0x61, 0x72, 0x65, 0x20, 0x73, 0x6D,
            0x6F, 0x6F, 0x74, 0x68, 0x65, 0x72, 0x20, 0x74,
            0x68, 0x61, 0x6E, 0x20, 0x76, 0x61, 0x73, 0x65,
            0x6C, 0x69, 0x6E, 0x65, 0x0D, 0x0A,   
        ];

        let output_test = vec![
            0xC0, 0x99, 0x9F, 0xDD, 0xE3, 0x78, 0xD7, 0xED,
            0x72, 0x7D, 0xA0, 0x0B, 0xCA, 0x5A, 0x84, 0xEE,
            0x47, 0xF2, 0x69, 0xA4, 0xD6, 0x43, 0x81, 0x90,
            0xD9, 0xD5, 0x2F, 0x78, 0xF5, 0x35, 0x84, 0x99,
            0x82, 0x8A, 0xC9, 0xB4, 0x53, 0xE0, 0xE6, 0x53,   
        ];
        let mut output = vec![];
        encrypt(&data[..], &mut output, 0x0E329232EA6D0D73, Endianess::Big)
            .expect("Encryption internal error");
        assert!(output.iter()
                .zip(output_test.iter())
                .all(|(&a, &b)| a == b));
    }

    #[test]
    fn test_decrypt() {

        let output_test = vec![
            0x59, 0x6F, 0x75, 0x72, 0x20, 0x6C, 0x69, 0x70,
            0x73, 0x20, 0x61, 0x72, 0x65, 0x20, 0x73, 0x6D,
            0x6F, 0x6F, 0x74, 0x68, 0x65, 0x72, 0x20, 0x74,
            0x68, 0x61, 0x6E, 0x20, 0x76, 0x61, 0x73, 0x65,
            0x6C, 0x69, 0x6E, 0x65, 0x0D, 0x0A,   
        ];

        let data = vec![
            0xC0, 0x99, 0x9F, 0xDD, 0xE3, 0x78, 0xD7, 0xED,
            0x72, 0x7D, 0xA0, 0x0B, 0xCA, 0x5A, 0x84, 0xEE,
            0x47, 0xF2, 0x69, 0xA4, 0xD6, 0x43, 0x81, 0x90,
            0xD9, 0xD5, 0x2F, 0x78, 0xF5, 0x35, 0x84, 0x99,
            0x82, 0x8A, 0xC9, 0xB4, 0x53, 0xE0, 0xE6, 0x53,   
        ];
        let mut output = vec![];
        decrypt(&data[..], &mut output, 0x0E329232EA6D0D73, Endianess::Big)
            .expect("Encryption internal error");
        assert!(output.iter()
                .zip(output_test.iter())
                .all(|(&a, &b)| a == b));
    }
}

    // let output_test: Vec<u8> = vec![
    //     0x70, 0x69, 0x6c, 0x20, 0x72, 0x75, 0x6f, 0x59,
    //     0x6d, 0x73, 0x20, 0x65, 0x72, 0x61, 0x20, 0x73,
    //     0x74, 0x20, 0x72, 0x65, 0x68, 0x74, 0x6f, 0x6f,
    //     0x65, 0x73, 0x61, 0x76, 0x20, 0x6e, 0x61, 0x68,
    //     0x0a, 0x0d, 0x65, 0x6e, 0x69, 0x6c, ];
    // let data = vec![
    //     0xed, 0xd7, 0x78, 0xe3, 0xdd, 0x9f, 0x99, 0xc0,
    //     0xee, 0x84, 0x5a, 0xca, 0x0b, 0xa0, 0x7d, 0x72,
    //     0x90, 0x81, 0x43, 0xd6, 0xa4, 0x69, 0xf2, 0x47,
    //     0x99, 0x84, 0x35, 0xf5, 0x78, 0x2f, 0xd5, 0xd9, // 9d
    //     0x53, 0xe6, 0xe0, 0x53, 0xb4, 0xc9, 0x8a, 0x82,];
    //use std::convert::in

    // let output_test = vec![
    //     0xed, 0xd7, 0x78, 0xe3, 0xdd, 0x9f, 0x99, 0xc0,
    //     0xee, 0x84, 0x5a, 0xca, 0x0b, 0xa0, 0x7d, 0x72,
    //     0x90, 0x81, 0x43, 0xd6, 0xa4, 0x69, 0xf2, 0x47,
    //     0x99, 0x84, 0x35, 0xf5, 0x78, 0x2f, 0xd5, 0xd9, // 9d
    //     0x53, 0xe6, 0xe0, 0x53, 0xb4, 0xc9, 0x8a, 0x82,];
//use std::convert::in
// data: Vec<u8> = vec![
    //     0x70, 0x69, 0x6c, 0x20, 0x72, 0x75, 0x6f, 0x59,
    //     0x6d, 0x73, 0x20, 0x65, 0x72, 0x61, 0x20, 0x73,
    //     0x74, 0x20, 0x72, 0x65, 0x68, 0x74, 0x6f, 0x6f,
    //     0x65, 0x73, 0x61, 0x76, 0x20, 0x6e, 0x61, 0x68,
    //     0x0a, 0x0d, 0x65, 0x6e, 0x69, 0x6c, ];
