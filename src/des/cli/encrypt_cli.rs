use std::path::{PathBuf, Path};
use crate::des;
use crate::data_io;
use crate::des::ParseKeyError;
use std::ffi::OsStr;

pub static USAGE_MESSAGE: &str  = 
    "USAGE: des-e {{src_path}} [-k {{key}}] [-o {{dst_path}}] \n\
     where filenames and extensions are arbitrary. \n\
     Options: \n\
     * {{src_path}}: a path to file, that will be encrypted. \n\
     The file will remain untouched \n\
     * {{key}}: a 64 bit long hexadecimal string \n\
     (16 chars, optional '-' delimiter, trailing spaces). \n\
     If not passed from args, key will have \n\
     a default value \n\
     * {{dst_path}}: a path to a destination file: \n\
     </a/b/file.extension> - such file will be created \n\
     (or opened and truncated, if already exists) \n\
     </a/b/file> - name=file, extension='.des' \n\
     </a/b/> - name=source_file, extension='.des' \n\
     <no flag -o> - same name, as with </a/b/> option, \n\
     file is located where the source file is";

// TODO: just CLI ?
#[derive(Default)]
pub struct EncryptorCli {
    pub key: u64,
    pub src_file_path: PathBuf, // TODO: Path?
    pub dst_file_path: PathBuf,
}

impl EncryptorCli {
    pub fn new() -> Self {
        Default::default()
    }
    // fn build(&mut self) {
    
    // }
    
    pub fn parse_args<'a, T, S>(mut self, mut args: T) -> Option<Self>
    where T : Iterator<Item=S>, S: AsRef<str> {
        args.next()?;
        self.src_file_path = PathBuf::from(
            <S as AsRef<str>>::as_ref(&args.next()?)
        );
        
        if !self.src_file_path.is_file() {
            return None;
        }
        // FIXME: if no parent
        // self.dst_file_path = match self.src_file_path.parent() {
            // Some(parent) => {
        let mut buf = PathBuf::from(&self.src_file_path);
        buf.set_extension("des");
        self.dst_file_path = buf;
                // buf
            // },
            // None => PathBuf::new(),
        // };
        
        while let Some(flag) = args.next() {
            let flag = flag.as_ref();
            match flag {
                "-k" | "--key"=> {
                    // TODO: Cow possibility, inplace replace
                    let key_hex_str = &args.next()?;
                    self.key = des::key_from_str(key_hex_str).ok()?;
                },
                "-o" | "--output" => {
                    let dst_path = args.next()?;
                    let mut dst_path = PathBuf::from(dst_path.as_ref());
                    
                    if dst_path.is_dir() {
                        dst_path.set_file_name(
                            self.src_file_path.file_name().unwrap());
                        dst_path.set_extension("des");
                    } else if let Some(_) = dst_path.file_name() {
                        if let None = dst_path.extension() {
                            dst_path.set_extension("des");
                            //println!("{:?}", dst_path);
                        }
                    }
                    self.dst_file_path = dst_path;
                },
                "-h" | "--help" => {
                    return None
                },
                _ => (),
            }
        }
        Some(self)
    }
    
    pub fn default_key<'a, T>(mut self, key: T)
                              -> Result<Self, ParseKeyError>
    where T: AsRef<str> {
        // TODO: no details
        self.key = crate::des::key_from_str(&key)?;
        Ok(self)
    }

    // pub fn encrypt(&self) -> std::io::Result<()> {
    //     assert!(self.src_file_path.is_file());
    //     assert!(self.dst_file_path.is_file());
    //     // TODO: assert key is valid
    //     let (input, output) = data_io::open_rw_files(
    //         &self.src_file_path, &self.dst_file_path
    //     )?;
    //     use crate::des::api;
    //     api::encrypt(input, output, self.key);

    //     Ok( () )
    // }

    pub fn print_help() {
        println!("{}", USAGE_MESSAGE);
    }

    pub fn announce_begin(&self) {
        println!("[DES] Encrypting file: {} ",
                 self.src_file_path.display());
        println!("[DES] Output file: {} ",
                 self.dst_file_path.display());
        println!("[DES] Key = {}", self.key)
    }

    pub fn announce_end(&self) {
        println!("[DES] Done");
    }

}
