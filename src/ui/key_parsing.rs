use std::error;
use std::fmt;

use ParseKeyError::*;

/// Possible ERRORS during parsing of DES key from a string, namely:
/// BadLength    - key was of wrong size, contains the required size in digits
/// NotHexFormat - key was not in a hex format (besides ’-’ char) 
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

/// Takes a key (string), consisting of hex digits (0-9, A-F, a-f) and
/// also ’-’, parses it into 64 bit wide DES key
/// @returns a Result with the parsed 64 bit wide key
pub fn key_from_str<S: AsRef<str>>
    (hex_str: &S) -> std::result::Result<u64, ParseKeyError>  {
        let clean = hex_str.as_ref().replace("-","").trim().to_string();
        if clean.len() != 16 { return Err(BadLength(16)); }
        u64::from_str_radix(&clean, 16).or(Err(NotHexFormat))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_key_from_str(){
        let key_hex_1 = "0ACB-38C4-9EA3-1C71";
        assert_eq!(key_from_str(&key_hex_1), Ok(777_777_777_777_777_777),
                   "Failed to parse a simple key of all 7's");
        // test no move
        assert_eq!(key_from_str(&key_hex_1), Ok(777_777_777_777_777_777),
                   "Key parsing function suddenly changed the input it was given");
        assert_eq!(key_from_str(&"ZFFF-FFFF-0000-0000"),
                   Err(NotHexFormat),
                   "Not Hex character in input");
        assert_eq!(key_from_str(&"FFF-FFFF-0000-0000"),
                   Err(BadLength(16)),
                   "Key of wrong length (16 hex digits expected)");
        assert_eq!(key_from_str(&"FFFF-FFFF-FFFF-FFFF"),
                   Ok(0xFFFF_FFFF_FFFF_FFFF),
                   "Corner case key: all F's as hex digits");
        assert_eq!(key_from_str(&"abcd-ffee-0000-1234"),
                   Ok(0xabcd_ffee_0000_1234),
                   "Failed to parse a key with different characters");
}
}
