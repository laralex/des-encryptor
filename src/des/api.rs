use std::io::{Read, Write, Result, BufRead, BufReader, BufWriter};
use std::convert::AsMut;
//use super::details::*;

pub const BITS_IN_BLOCK: usize = 64;
pub const BYTES_IN_BLOCK: usize =
    (BITS_IN_BLOCK / 8) + (BITS_IN_BLOCK % 8 == 0) as usize;
pub const IO_BUF_SIZE: usize = BYTES_IN_BLOCK * 1024;
pub const BITS_IN_KEY: usize = 64;

pub fn encrypt<R, W>(mut src: R, mut dst: W, key: u64) -> Result<()>
where R: Read, W: Write {
    let mut read_buf  = [0u8; IO_BUF_SIZE];
    let mut write = BufWriter::with_capacity(IO_BUF_SIZE, dst);
    // TODO: concurrency
    // FIXME: somehow mutate inside of BufReader
    loop {
        let processed_size = src.read(&mut read_buf)?;
        match processed_size {
            0 => break,
            len if len < IO_BUF_SIZE => break,
            len => {
                encrypt_inplace(&mut read_buf, key);
                write.write_all(&read_buf)?;
                // discard read data from buffer
                // read.consume(processed_size);
            }
        }
        
    }
    Ok( () )
}

// pub fn decrypt {
    
// }

// TODO: asmut
pub fn encrypt_inplace(bytes: &mut [u8] , key: u64) {

}

// pub fn decrypt_inplace() {
    
// }

#[test]
fn test_encrypt_inplace(){
    
}
