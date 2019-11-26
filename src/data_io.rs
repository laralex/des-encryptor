use std::io::{Result};
use std::fs::File;
use std::path::Path;

/// Opens the file at path ’read_filepath’ for read,
/// Opens the file at path ’write_filepath’ for write or creates it,
/// @returns a Result, with a tuple of the read file handle and the
/// write file handle
pub fn open_rw_files<R, W>(to_read: &R, to_write: &W)
                           -> Result<(File, File)>
where R: AsRef<Path>, W: AsRef<Path> {
    // TODO: don’t panic on same files, throw Result
    let read_path = to_read.as_ref();
    let write_path = to_write.as_ref();
    assert!(read_path != write_path);
    let read_file = File::open(read_path)?;
    let write_file = File::create(write_path)?;
    return Ok( (read_file, write_file) );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_open_rw_files() {
        // > test init
        let impossible_name = "HAKSJFHLAJKSFHAKSJLF@@JKJ@J@@LK191jlk2010";
        let impossible_name_2 = "98124yhiohi4o12hkj12h498124yhiohi4o12hkj12h";
        let new_f = File::create(impossible_name);
        // > test init end

        // read path is not a file
        if !open_rw_files(&impossible_name_2, &impossible_name).is_err(){
            std::fs::remove_file(impossible_name);
            panic!("Test fail: opening 'read' path, but file doesn't exist");
        }
        // write path is dir
        let dir = "./";
        if !open_rw_files(&impossible_name, &dir).is_err() {
            std::fs::remove_file(impossible_name);
            panic!("Test fail: opening 'write' path, but it is directory");
        }
        if !open_rw_files(&impossible_name, &impossible_name_2).is_ok(){
            std::fs::remove_file(impossible_name_2);
            std::fs::remove_file(impossible_name);
            panic!("Test fail: should normally open read file and \n\
                    create write file");
        }
        // > test cleanup
        std::fs::remove_file(impossible_name_2);
        std::fs::remove_file(impossible_name);
    }
}
