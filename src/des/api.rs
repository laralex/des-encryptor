use std::io::{Read, Write, BufRead, BufReader, BufWriter};
use std::io;
use std::convert::AsMut;
use crate::des::details;
use crate::reinterpret_bytes;
use crate::reinterpret_bytes::Endianess;
use details::key_scheduling::*;

pub const BITS_IN_BLOCK: usize = 64;
pub const BYTES_IN_BLOCK: usize =
    (BITS_IN_BLOCK / 8) + (BITS_IN_BLOCK % 8 == 0) as usize;
pub const IO_BUF_SIZE: usize = BYTES_IN_BLOCK * 1024 * 4;

/// Main and easy to use function for standard DES encryption. Takes
/// data from the Read object (buffered), encrypts it and puts it
/// buffer-wise in the Write object
/// @returns I/O Error if one occured
pub fn encrypt<R, W>
    (src: R, dst: W, key: u64, endianess: Endianess) -> io::Result<()>
where R: Read, W: Write {
    transform_data(src, dst, &details::encrypt_block, KeyScheduler::new_encrypting(key), endianess)
}

/// Main and easy to use function for standard DES decrytion. Takes
/// data from the Read object (buffered), decrypts it and puts it
/// buffer-wise in the Write object
/// @returns I/O Error if one occured
pub fn decrypt<R, W>
    (src: R, dst: W, key: u64, endianess: Endianess) -> io::Result<()>
where R: Read, W: Write {
    transform_data(src, dst, &details::decrypt_block, KeyScheduler::new_decrypting(key),  endianess)
}

/// Performs 3DES encryption algorithm (i.e. encrypt the same data thrice) with
/// all keys different. Takes data from the Read object (buffered),
/// encrypts it and puts it buffer-wise in the Write object
/// @returns I/O Error if one occured
pub fn triple_encrypt<R, W>
    (src: R, dst: W, (key1, key2, key3): (u64, u64, u64), endianess: Endianess) -> io::Result<()>
where R: Read, W: Write {
    Ok( () )
}

/// Performs 3DES decryption algorithm (i.e. encrypt the same data thrice) with
/// all keys different. Takes data from the Read object (buffered),
/// decrypts it and puts it buffer-wise in the Write object
/// @returns I/O Error if one occured
pub fn triple_decrypt<R, W>
    (src: R, dst: W, (key1, key2, key3): (u64, u64, u64), endianess: Endianess) -> io::Result<()>
where R: Read, W: Write {
    Ok( () )
}

/// Supportive function to DES, since both encryption and decryption
/// goes essentially the same way (but with a different keys generation),
/// this function just applies the same steps on Read and Write
/// objects and encrypts/decrypts them.
/// @returns I/O Error if one occured
fn transform_data<R, W, I>
    (mut src: R, dst: W,  block_affector: &Fn(u64, &mut I) -> u64, mut key_iterator: I, endianess: Endianess) -> io::Result<()>
where R: Read, W: Write, I: Iterator<Item=Key> {
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
                for padded_mem in read_buf[len..len + padding].iter_mut() {
                    *padded_mem = 0;
                }
                slice_to_write = &mut read_buf[..len + padding];
            },
            _ => slice_to_write = &mut read_buf,
        }
        transform_padded_slice(slice_to_write, block_affector, &mut key_iterator);
        write.write_all(slice_to_write)?;
    }
    Ok( () )
} 

/// Encrypts / decrypts (considering function pointer action) a 64 bit
/// block of data inplace. Respects target machine’s endianess
// TODO: asmut
fn transform_padded_slice<I>(bytes: &mut [u8], block_affector: &Fn(u64, &mut I) -> u64, key_iterator: &mut I)
where I: Iterator<Item=Key>{
    let blocks = reinterpret_bytes::as_slice_of::<u64>(bytes);
    for block in blocks.iter_mut() {
        if cfg!(target_endian = "little") {
            *block = block_affector(block.to_be(), key_iterator).to_be();
        } else {
            *block = block_affector(*block, key_iterator);
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
                .all(|(&a, &b)| a == b), "Basic DES encryption of a stream is wrong");
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
                .all(|(&a, &b)| a == b), "Basic DES decryption of a stream is wrong");
    }
}

