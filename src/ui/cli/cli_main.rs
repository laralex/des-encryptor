use crate::ui::key_parsing;
use std::path::PathBuf;
use crate::reinterpret_bytes::Endianess;
/*
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
file is located where the source file is
 */

pub static USAGE_MESSAGE: &str  = 
    "USAGE: des {{src_path}} [-k {{key}}] [-o {{dst_path}}] [{-b, -l}] [{-e, -d}] \n\
     where filenames and extensions are arbitrary.";

#[derive(Copy, Clone)]
pub enum Action {
    EncryptFile,
    DecryptFile,
}

impl Default for Action {
    fn default() -> Self { return Action::EncryptFile; }
}

#[derive(Default)]
pub struct Cli {
    pub key: u64,
    pub src_file_path: PathBuf, // TODO: Path?
    pub dst_file_path: PathBuf,
    pub action: Action,
    pub endianess: Endianess,
}

impl Cli {
    pub fn new() -> Self {
        Default::default()
    }
    
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

        // Default state (if some of )
        let mut buf = PathBuf::from(&self.src_file_path);
        buf.set_extension("des");
        self.dst_file_path = buf;

        let mut is_action_specified = false;
        let mut is_endianess_specified = false;
        while let Some(flag) = args.next() {
            let flag = flag.as_ref();
            match flag {
                "-k" | "--key"=> {
                    // TODO: Cow possibility, inplace replace
                    let key_hex_str = &args.next()?;
                    self.key = key_parsing::key_from_str(key_hex_str).ok()?;
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
                    self.dst_file_path = dst_path
                },
                "-h" | "--help" => {
                    return None
                },
                "-d" | "--decrypt" => {
                    if is_action_specified { return None; }
                    is_action_specified = true;
                    self.action = Action::DecryptFile
                },
                "-e" | "--encrypt" => {
                    if is_action_specified { return None; }
                    is_action_specified = true;
                    self.action = Action::EncryptFile
                },
                "-b" | "--big-endian" => {
                    if is_endianess_specified { return None; }
                    is_endianess_specified = true;
                    self.endianess = Endianess::Big;
                }
                "-l" | "--little-endian" => {
                    if is_endianess_specified { return None; }
                    is_endianess_specified = true;
                    self.endianess = Endianess::Little;
                }
                _ => (),
            }
        }
        Some(self)
    }
    
    pub fn default_key<'a, T>(mut self, key: T)
                              -> Result<Self, key_parsing::ParseKeyError>
    where T: AsRef<str> {
        // TODO: no details
        self.key = key_parsing::key_from_str(&key)?;
        Ok(self)
    }

    pub fn default_action(mut self, action: Action) -> Self {
        self.action = action;
        self
    }

    pub fn default_endianess(mut self, endianess: Endianess) -> Self {
        self.endianess = endianess;
        self
    }

    // TODO: accept stream, not just println
    pub fn print_help() {
        println!("{}", USAGE_MESSAGE);
    }

    pub fn announce_begin(&self) {
        use Action::*;
        match self.action {
            EncryptFile => println!("[DES] Encrypting file: {}",
                                    self.src_file_path.display()),
            DecryptFile => println!("[DES] Decrypting file: {}",
                                    self.src_file_path.display()),
        };
        println!("[DES] Output file: {} ",
                 self.dst_file_path.display());
        println!("[DES] Key = {}", self.key)
    }

    pub fn announce_end(&self) {
        println!("[DES] Done");
    }

}

impl Cli {
    pub fn key(&self) -> u64 { self.key }
    pub fn src_file_path(&self) -> &PathBuf { &self.src_file_path }
    pub fn dst_file_path(&self) -> &PathBuf { &self.dst_file_path }
    pub fn action(&self) -> Action { self.action }
    pub fn endianess(&self) -> Endianess { self.endianess }
}
