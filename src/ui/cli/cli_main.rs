use crate::ui::key_parsing;
use std::path::PathBuf;
use std::str::FromStr;
use crate::reinterpret_bytes;

pub static USAGE_MESSAGE: &str  = 
    // "USAGE: des {{src_path}} [-k {{key}}] [-o {{dst_path}}] [{-b,
    // -l}] [{-e, -d}] \n\
    "\nUSAGE: des {{src_file}} {{dst_file}}\n\n\
     * these paths can't be the same, and src_file should exist\n\
     * different flags are allowed between these tokens\n\
     (add -h/--help to command to find flags list)";

pub static HELP_MESSAGE: &str =
    "\nUSAGE: des {{src_path}} {{dst_path}}\n\n\
     * these paths can't be the same, and src_path should exist\n\
     * different flags are allowed between these tokens\n\n\
     Available flags:\n\
     -h / --help \n\
     to print this message \n\n\
     -e / --encrypt \n\
     program will perform encryption of src_file \n\n\
     -d / --decrypt \n\
     program will perform decryption of src_file \n\n\
     -k / --key \"KEY-HEX-STRING\" \n\n\
     program will perform using specified key \n\
     (which should contain only \n\
     a-f, A-F, 0-9, '-' chars) \n\n\
     -b / --big-endian \n\
     program will generate dst_file using \n\
     big-endian byte order for each 64-bit block \n\n\
     -l / --little-endian \n\
     program will generate dst_file using \n\
     little-endian byte order for each 64-bit block \n\n\
     -v / --verbose \n\
     program will output as much as possible info \n\
     about its execution process in standard output \n\n\
     -s / --silent \n\
     program will output no information \n\
     in standard output \n\n\
     --force \n\
     program will answer 'Yes' on any uncertainty \n\
     it would normally complain about and will \n\
     execute as much as it can unless any fatal \n\
     error occurs
     ";

/// Possible actions user may desire, namely
/// - DES encryption
/// - DES decryption
/// - TripleDES encryption
/// - TripleDES decryption
#[derive(Copy, Clone)]
pub enum Action {
    EncryptFile,
    DecryptFile,
    TripleEncryptFile,
    TripleDecryptFile,
}

impl Default for Action {
    fn default() -> Self { return Action::EncryptFile; }
}

// Delegation of action’s parsing to the action class itself
impl FromStr for Action {
    type Err = ();
    fn from_str(str: &str) -> Result<Self, Self::Err> {
        match str {
            "-e" | "--encrypt" => Ok(Action::EncryptFile),
            "-d" | "--decrypt" => Ok(Action::DecryptFile),
            "-te" | "--triple-encrypt" => Ok(Action::TripleEncryptFile),
            "-td" | "--triple-decrypt" => Ok(Action::TripleDecryptFile),
            _ => Err(()),
        }
    }
} 


/// Level of logging, provided by CLI application
/// TODO: not yet implemented
#[derive(Copy, Clone)]
pub enum MessagingLevel {
    Verbose,
    Normal,
    Silent,
}

impl Default for MessagingLevel {
    fn default() -> Self { MessagingLevel::Normal }
}

// Delegation of messaging level’s parsing to the class itself
impl FromStr for MessagingLevel {
    type Err = ();
    fn from_str(str: &str) -> Result<Self, Self::Err> {
        match str {
            "-v" | "--verbose" => Ok(MessagingLevel::Verbose),
            "-s" | "--silent" => Ok(MessagingLevel::Silent),
            _ => Err(()),
        }
    }
}

/// Endianess of created output file
/// TODO: not yet implemented
pub struct Endianess {
    endianess: reinterpret_bytes::Endianess,
}

impl Endianess {
    fn new(endianess: reinterpret_bytes::Endianess) -> Self {
        Self {endianess}
    }
}
impl Default for Endianess {
    fn default() -> Self {
        Self { endianess:reinterpret_bytes::Endianess::Big }
    }
}
impl FromStr for Endianess {
    type Err = ();
    fn from_str(str: &str) -> Result<Self, Self::Err> {
        match str {
            "-b" | "--big-endian" => Ok( Self {
                endianess: reinterpret_bytes::Endianess::Big } ),
            "-l" | "--little-endian" => Ok( Self {
                endianess: reinterpret_bytes::Endianess::Little } ),
            _ => Err(()),
        }
    }
}

/// Is application allowed to do whatever it takes to create the
/// output file (even through possible confusion or errors)
/// TODO: not yet implemented
#[derive(Default)]
pub struct Force {
    do_force: bool,
}

impl FromStr for Force {
    type Err = ();
    fn from_str(str: &str) -> Result<Self, Self::Err> {
        match str {
            "-f" | "--force" => Ok(Self { do_force: true }),
            _ => Err(()),
        }
    }
}

/// Should application display help for it’s usage
#[derive(Default)]
pub struct Help {
    need_help: bool,
}

impl FromStr for Help {
    type Err = ();
    fn from_str(str: &str) -> Result<Self, Self::Err> {
        match str {
            "-h" | "--help" => Ok(Self { need_help: true }),
            _ => Err(()),
        }
    }
}

trait DataFlag: FromStr {
    type DataType;
    fn parse_data(&self, arg_str: &str) -> Option<Self::DataType>;
}

pub struct Key {
    pub parsed_key: u64,
}

impl Key {
    fn new(parsed_key: u64) -> Self { Self {parsed_key} }
}
impl FromStr for Key {
    type Err = ();
    fn from_str(str: &str) -> Result<Self, Self::Err> {
        match str {
            "-k" | "--key" => Ok( Self::new(0) ),
            _ => Err(()),
        }
    }
}
impl DataFlag for Key {
    type DataType = u64;
    fn parse_data(&self, arg_str: &str) -> Option<Self::DataType> {
        key_parsing::key_from_str(&arg_str).ok()
    }
}

#[derive(Default)]
pub struct Cli {
    pub key: u64,
    pub src_file_path: PathBuf, // TODO: Path?
    pub dst_file_path: PathBuf,
    pub action: Action,
    pub endianess: Endianess,
    pub messaging_level: MessagingLevel,
    pub force: bool,
    pub help_requested: bool,
}

impl Cli {
    pub fn new() -> Self {
        Default::default()
    }
    
    pub fn parse_args<'a, T, S>(mut self, mut args: T) -> Option<Self>
    where T : Iterator<Item=S>, S: AsRef<str> {
        args.next()?;

        let mut dest_path_buf;
        let mut is_action_specified = false;
        let mut is_endianess_specified = false;
        let mut free_arg_cnt = 0;
        while let Some(flag) = args.next() {
            let flag = flag.as_ref();
            match flag {
                "-k" | "--key"=> {
                    // TODO: Cow possibility, inplace replace
                    let key_hex_str = &args.next()?;
                    self.key = key_parsing::key_from_str(key_hex_str).ok()?;
                },
                "-h" | "--help" => {
                    self.help_requested = true;
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
                // "-b" | "--big-endian" => {
                //     if is_endianess_specified { return None; }
                //     is_endianess_specified = true;
                //     self.endianess = Endianess::Big;
                // }
                // "-l" | "--little-endian" => {
                //     if is_endianess_specified { return None; }
                //     is_endianess_specified = true;
                //     self.endianess = Endianess::Little;
                // }
                free_arg => {
                    free_arg_cnt += 1;
                    match free_arg_cnt {
                        1 => { // input file
                            self.src_file_path = PathBuf::from(free_arg);
                            
                            if !self.src_file_path.is_file() {return None;}
                            // FIXME: if no parent
                            // self.dst_file_path = match self.src_file_path.parent() {
                            // Some(parent) => {

                            // Default state (if some of )
                            dest_path_buf = PathBuf::from(&self.src_file_path);
                            dest_path_buf.set_extension("des");
                            self.dst_file_path = dest_path_buf;
                        },
                        2 => { // output file
                            let mut dst_path = PathBuf::from(free_arg);
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
                        _ => (),
                    }
                    
                },
            }
        }
        if self.help_requested || free_arg_cnt >= 1 { Some(self) }
        else { None }
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

    pub fn default_endianess(mut self, endianess: reinterpret_bytes::Endianess) -> Self {
        self.endianess = Endianess::new(endianess);
        self
    }

    // TODO: accept stream, not just println
    pub fn print_help() {
        println!("{}", HELP_MESSAGE);
    }
    
    pub fn print_usage() {
        println!("{}", USAGE_MESSAGE);
    }

    pub fn announce_begin(&self) {
        use Action::*;
        let tag = match self.action {
            EncryptFile => "[ ENCRYPT ]",
            DecryptFile => "[ DECRYPT ]",
        };
        println!("{} Input  file: {}", tag, self.src_file_path.display());
        println!("{} Output file: {}", tag, self.dst_file_path.display());
        println!("{} Key = {:#018x}", tag, self.key)
    }

    pub fn announce_end(&self) {
        use Action::*;
        let tag = match self.action {
            EncryptFile => "[ ENCRYPT ]",
            DecryptFile => "[ DECRYPT ]",
        };
        println!("{} Done", tag);
    }

}

impl Cli {
    pub fn key(&self) -> u64 { self.key }
    pub fn src_file_path(&self) -> &PathBuf { &self.src_file_path }
    pub fn dst_file_path(&self) -> &PathBuf { &self.dst_file_path }
    pub fn action(&self) -> Action { self.action }
    pub fn endianess(&self) -> reinterpret_bytes::Endianess { self.endianess.endianess }
}
