use std::error;
use std::fmt;

pub type ParseResult<T> = std::result::Result<T, ParseKeyError>;
use ParseKeyError::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ParseKeyError {
    BadLength(usize),
    NotHexFormat,
}

impl fmt::Display for ParseKeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "<key has either invalid lenght, \n\
                   or not in a hex format>")
    }
}

impl error::Error for ParseKeyError {
    fn description(&self) -> &str { Default::default() }
    fn cause(&self) -> Option<&dyn error::Error> { None }
    fn source(&self) -> Option<&(dyn error::Error + 'static)> { None }
}

// impl Display for ParseKeyError {
    
// }

pub fn key_from_str<S: AsRef<str>>(hex_str: &S) -> ParseResult<u64>  {
    let clean = hex_str.as_ref().replace("-","").trim().to_string();
    if clean.len() != 16 { return Err(BadLength(16)); }
    u64::from_str_radix(&clean, 16).or(Err(NotHexFormat))
}

#[test]
fn test_key_from_str(){
    let key_hex_1 = "0ACB-38C4-9EA3-1C71";
    assert_eq!(key_from_str(&key_hex_1), Ok(777_777_777_777_777_777));
    // test no move
    assert_eq!(key_from_str(&key_hex_1), Ok(777_777_777_777_777_777));
    assert_eq!(key_from_str(&"ZFFF-FFFF-0000-0000"),
               Err(NotHexFormat));
    assert_eq!(key_from_str(&"FFF-FFFF-0000-0000"),
               Err(BadLength(16)));
    assert_eq!(key_from_str(&"FFFF-FFFF-FFFF-FFFF"),
               Ok(0xFFFF_FFFF_FFFF_FFFF));
    assert_eq!(key_from_str(&"abcd-ffee-0000-1234"),
               Ok(0xabcd_ffee_0000_1234));
}
