use crate::math::bit_arithmetics;
use super::{PermutationTable};

use crate::lazy_static::*;
const BITS_IN_INPUT: u32 = 64;
const BITS_ORDER_L_TO_R: bool = true;
const BIT_COUNT_FROM: u32 = 1;
const ROTATIONS_TO_WRAPPING: usize = 16;

/// Initial key permutation table. Is applied before any key is generated
lazy_static! {
    static ref INITIAL_PERMUTATION: PermutationTable = PermutationTable::new(vec![
        57, 49, 41, 33, 25, 17, 9, 1, 58, 50, 42, 34, 26, 18,
        10, 2, 59, 51, 43, 35, 27, 19, 11, 3, 60, 52, 44, 36,
        63, 55, 47, 39, 31, 23, 15, 7, 62, 54, 46, 38, 30, 22,
        14, 6, 61, 53, 45, 37, 29, 21, 13, 5, 28, 20, 12, 4
    ], BIT_COUNT_FROM, BITS_IN_INPUT);
}

/// Maps 56-bit long key to a 48-bit long round key
lazy_static! {
    static ref PERMUTING_CHOICE: PermutationTable = PermutationTable::new(vec![
        14, 17, 11, 24, 1, 5, 3, 28, 15, 6, 21, 10, 23, 19, 12, 4,
        26, 8, 16, 7, 27, 20, 13, 2, 41, 52, 31, 37, 47, 55, 30, 40,
        51, 45, 33, 48, 44, 49, 39, 56, 34, 53, 46, 42, 50, 36, 29, 32,
    ], BIT_COUNT_FROM, 56);    
}

/// Encryption / Decryption key -> a binary value of some length
#[derive(Debug, Clone, Copy, Default)]
pub struct Key {
    pub value: u64,
    pub size_bits: u32,
}

/// Scheduler that generates a new key from the previous key
//#[derive(Clone, Eq, PartialEq, Debug)]
pub struct KeyScheduler {
    des_key_size_bits: u32,
    current_key_index: usize,
    cache: Vec<Key>,
    initial_key: Key,
    rotation_stage_split_bit: u32,
    rotation_stage_key_size: u32,
}

/// Holds key scheduling parameters, namely size of input and output
/// keys, location of rotation bit, also holds a cache of previously
/// generated keys 
impl KeyScheduler {
    /// Initialized the key scheduler with a key, and total number of
    /// keys it generates (DES specifies 16)
    pub fn new(initial_key: u64, keys_total: usize) -> KeyScheduler {
        let rotation_stage_key_size = INITIAL_PERMUTATION.output_size();
        let rotation_stage_split_bit = rotation_stage_key_size / 2;
        let inner_key_value = INITIAL_PERMUTATION.apply(initial_key);
        let inner_key_size = INITIAL_PERMUTATION.output_size();
        let cache = vec![Default::default(); keys_total];
        let initial_key = Key {value: inner_key_value, size_bits: inner_key_size};
        KeyScheduler {
            des_key_size_bits: PERMUTING_CHOICE.output_size(),
            current_key_index: 0,
            cache,
            initial_key,
            rotation_stage_split_bit,
            rotation_stage_key_size,
        }
    }

    /// Builds encrypting key scheduler with an initial key
    pub fn new_encrypting(initial_key: u64) -> EncryptingKeyScheduler {
        let mut scheduler = EncryptingKeyScheduler {
            base: KeyScheduler::new(initial_key, ROTATIONS_TO_WRAPPING),
        };
        println!("{:?}", scheduler.base.cache[0]);
        println!("{:?}", scheduler.base.initial_key);
        scheduler.base.cache[0] = KeyScheduler::des_key(scheduler.base.initial_key);
        println!("{:?}", scheduler.base.cache[0]);
        let mut next_inner_key = scheduler.base.initial_key;
        for i in 1..ROTATIONS_TO_WRAPPING {
            next_inner_key = scheduler.base.rotate_key(
                next_inner_key, false).unwrap();
            scheduler.base.cache[i] = KeyScheduler::des_key(next_inner_key);
            println!("{:?}", scheduler.base.cache[i]);
        }
        scheduler
    }
    
    /// Builds a decrypting key scheduler with an initial key
    pub fn new_decrypting(initial_key: u64) -> DecryptingKeyScheduler {
        let mut scheduler = DecryptingKeyScheduler {
            base: KeyScheduler::new(initial_key, ROTATIONS_TO_WRAPPING),
        };
        for i in 1..ROTATIONS_TO_WRAPPING {
            scheduler.base.cache[i] = scheduler.base.rotate_key(
                scheduler.base.cache[i-1], true).unwrap();
        }
        scheduler
    }

    /// Rotate the inner key, according to DES scheduling specification
    /// (some rotations are by 1, some by 2), in direction, determined
    /// by ‘to_high’ boolean
    /// @returns Option with a new inner key, where two ranges are
    /// separately rotated towards high/low bits
    fn rotate_key(&self, mut key: Key, to_high: bool) -> Option<Key>{
        use bit_arithmetics::idx_from_low as low;
        let shift_for = match self.current_key_index {
            1 | 2 | 9 | 16 => 1,
            0 => 0, // FIXME when generating decrypting keys, no
            // rotation is needed, but it doesn’t seem good solution
            _ => 2,
        };
        
        let rotate = if to_high {
            low::rotate_range_to_high
        } else {
            low::rotate_range_to_low
        };

        let key_rotated_half = rotate(
            key.value,
            0, self.rotation_stage_split_bit,
            shift_for, key.size_bits
        )?;
        key.value = rotate(
            key_rotated_half,
            self.rotation_stage_split_bit, self.rotation_stage_key_size,
            shift_for, key.size_bits
        )?;
        
        Some(key)
    }

    /// Extract DES round key from inner scheduled 
    pub fn des_key(key: Key) -> Key {
        let value = PERMUTING_CHOICE.apply(key.value);
        Key { value, ..key }
    }
}

/// Generates another key by left rotation of previous (as specified
/// in DES specification)
pub struct EncryptingKeyScheduler {
    base: KeyScheduler,
}
/// Generates another key by right rotation of previous (as specified
/// in DES specification)
pub struct DecryptingKeyScheduler {
    base: KeyScheduler,
}

/// Fancy Iterator implementation for Encrypt key scheduling, allows
/// to use this object in Iterators chain
impl Iterator for EncryptingKeyScheduler {
    type Item = Key;
    /// Generate the next key for encryption from the current one (if
    /// it’s already cached, then no recalculation occurs)
    /// @returns Option with the key in it
    fn next(&mut self) -> Option<Self::Item> {
        // Wrap index
        if self.base.current_key_index == ROTATIONS_TO_WRAPPING {
            self.base.current_key_index = 1;
        } else {
            self.base.current_key_index += 1;
        }
        Some(self.base.cache[self.base.current_key_index - 1])
    }  
}

/// Fancy Iterator implementation for Decrypt key scheduling, allows
/// to use this object in Iterators chain
impl Iterator for DecryptingKeyScheduler {
    type Item = Key;
    /// Generate the next key for decryption from the current one (if
    /// it’s already cached, then no recalculation occurs)
    /// @returns Option with the key in it
    fn next(&mut self) -> Option<Self::Item> {
        if self.base.current_key_index <= 1 {
            self.base.current_key_index = ROTATIONS_TO_WRAPPING;
        } else {
            self.base.current_key_index -= 1;
        }
        let cache_idx = (ROTATIONS_TO_WRAPPING - self.base.current_key_index) % ROTATIONS_TO_WRAPPING;
        Some(KeyScheduler::des_key(self.base.cache[cache_idx]))
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_encrypting_scheduler(){
        let mut s = KeyScheduler::new_encrypting(
            0b_00010011_00110100_01010111_01111001_10011011_10111100_11011111_11110001
        );
        assert_eq!(s.next().unwrap().value,
                   0b_000110_110000_001011_101111_111111_000111_000001_110010, "k1");
        assert_eq!(s.next().unwrap().value,
                   0b_011110_011010_111011_011001_110110_111100_100111_100101, "k2");
        assert_eq!(s.next().unwrap().value,
                   0b_010101_011111_110010_001010_010000_101100_111110_011001, "k3");
        assert_eq!(s.next().unwrap().value,
                   0b_011100_101010_110111_010110_110110_110011_010100_011101, "k4");
        assert_eq!(s.next().unwrap().value,
                   0b_011111_001110_110000_000111_111010_110101_001110_101000, "k5");
        assert_eq!(s.next().unwrap().value,
                   0b_011000_111010_010100_111110_010100_000111_101100_101111, "k6");
        assert_eq!(s.next().unwrap().value,
                   0b_111011_001000_010010_110111_111101_100001_100010_111100, "k7");
        assert_eq!(s.next().unwrap().value,
                   0b_111101_111000_101000_111010_110000_010011_101111_111011, "k8");
        assert_eq!(s.next().unwrap().value,
                   0b_111000_001101_101111_101011_111011_011110_011110_000001, "k9");
        assert_eq!(s.next().unwrap().value,
                   0b_101100_011111_001101_000111_101110_100100_011001_001111, "k10");
        assert_eq!(s.next().unwrap().value,
                   0b_001000_010101_111111_010011_110111_101101_001110_000110, "k11");
        assert_eq!(s.next().unwrap().value,
                   0b_011101_010111_000111_110101_100101_000110_011111_101001, "k12");
        assert_eq!(s.next().unwrap().value,
                   0b_100101_111100_010111_010001_111110_101011_101001_000001, "k13");
        assert_eq!(s.next().unwrap().value,
                   0b_010111_110100_001110_110111_111100_101110_011100_111010, "k14");
        assert_eq!(s.next().unwrap().value,
                   0b_101111_111001_000110_001101_001111_010011_111100_001010, "k15");
        assert_eq!(s.next().unwrap().value,
                   0b_110010_110011_110110_001011_000011_100001_011111_110101, "k16");
        assert_eq!(s.next().unwrap().value,
                   0b_000110_110000_001011_101111_111111_000111_000001_110010, "k1_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_011110_011010_111011_011001_110110_111100_100111_100101, "k2_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_010101_011111_110010_001010_010000_101100_111110_011001, "k3_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_011100_101010_110111_010110_110110_110011_010100_011101, "k4_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_011111_001110_110000_000111_111010_110101_001110_101000, "k5_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_011000_111010_010100_111110_010100_000111_101100_101111, "k6_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_111011_001000_010010_110111_111101_100001_100010_111100, "k7_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_111101_111000_101000_111010_110000_010011_101111_111011, "k8_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_111000_001101_101111_101011_111011_011110_011110_000001, "k9_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_101100_011111_001101_000111_101110_100100_011001_001111, "k10_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_001000_010101_111111_010011_110111_101101_001110_000110, "k11_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_011101_010111_000111_110101_100101_000110_011111_101001, "k12_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_100101_111100_010111_010001_111110_101011_101001_000001, "k13_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_010111_110100_001110_110111_111100_101110_011100_111010, "k14_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_101111_111001_000110_001101_001111_010011_111100_001010, "k15_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_110010_110011_110110_001011_000011_100001_011111_110101, "k16_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_000110_110000_001011_101111_111111_000111_000001_110010, "k1_move");
        assert_eq!(s.next().unwrap().value,
                   0b_011110_011010_111011_011001_110110_111100_100111_100101, "k2_move");
    }

    #[test]
    fn test_key_decrypting_scheduler(){
        let mut s = KeyScheduler::new_decrypting(
            0b_00010011_00110100_01010111_01111001_10011011_10111100_11011111_11110001
        );
        //println!("{:?}", s.next());
        
        assert_eq!(s.next().unwrap().value,
                   0b_110010_110011_110110_001011_000011_100001_011111_110101, "k16");
        assert_eq!(s.next().unwrap().value,
                   0b_101111_111001_000110_001101_001111_010011_111100_001010, "k15");
        assert_eq!(s.next().unwrap().value,
                   0b_010111_110100_001110_110111_111100_101110_011100_111010, "k14");
        assert_eq!(s.next().unwrap().value,
                   0b_100101_111100_010111_010001_111110_101011_101001_000001, "k13");
        assert_eq!(s.next().unwrap().value,
                   0b_011101_010111_000111_110101_100101_000110_011111_101001, "k12");
        assert_eq!(s.next().unwrap().value,
                   0b_001000_010101_111111_010011_110111_101101_001110_000110, "k11");
        assert_eq!(s.next().unwrap().value,
                   0b_101100_011111_001101_000111_101110_100100_011001_001111, "k10");
        assert_eq!(s.next().unwrap().value,
                   0b_111000_001101_101111_101011_111011_011110_011110_000001, "k9");
        assert_eq!(s.next().unwrap().value,
                   0b_111101_111000_101000_111010_110000_010011_101111_111011, "k8");
        assert_eq!(s.next().unwrap().value,
                   0b_111011_001000_010010_110111_111101_100001_100010_111100, "k7");
        assert_eq!(s.next().unwrap().value,
                   0b_011000_111010_010100_111110_010100_000111_101100_101111, "k6");
        assert_eq!(s.next().unwrap().value,
                   0b_011111_001110_110000_000111_111010_110101_001110_101000, "k5");
        assert_eq!(s.next().unwrap().value,
                   0b_011100_101010_110111_010110_110110_110011_010100_011101, "k4");
        assert_eq!(s.next().unwrap().value,
                   0b_010101_011111_110010_001010_010000_101100_111110_011001, "k3");
        assert_eq!(s.next().unwrap().value,
                   0b_011110_011010_111011_011001_110110_111100_100111_100101, "k2");
        assert_eq!(s.next().unwrap().value,
                   0b_000110_110000_001011_101111_111111_000111_000001_110010, "k1");
        assert_eq!(s.next().unwrap().value,
                   0b_110010_110011_110110_001011_000011_100001_011111_110101, "k16_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_110010_110011_110110_001011_000011_100001_011111_110101, "k16_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_101111_111001_000110_001101_001111_010011_111100_001010, "k15_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_010111_110100_001110_110111_111100_101110_011100_111010, "k14_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_100101_111100_010111_010001_111110_101011_101001_000001, "k13_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_011101_010111_000111_110101_100101_000110_011111_101001, "k12_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_001000_010101_111111_010011_110111_101101_001110_000110, "k11_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_101100_011111_001101_000111_101110_100100_011001_001111, "k10_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_111000_001101_101111_101011_111011_011110_011110_000001, "k9_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_111101_111000_101000_111010_110000_010011_101111_111011, "k8_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_111011_001000_010010_110111_111101_100001_100010_111100, "k7_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_011000_111010_010100_111110_010100_000111_101100_101111, "k6_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_011111_001110_110000_000111_111010_110101_001110_101000, "k5_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_011100_101010_110111_010110_110110_110011_010100_011101, "k4_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_010101_011111_110010_001010_010000_101100_111110_011001, "k3_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_011110_011010_111011_011001_110110_111100_100111_100101, "k2_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_000110_110000_001011_101111_111111_000111_000001_110010, "k1_cache");
        assert_eq!(s.next().unwrap().value,
                   0b_110010_110011_110110_001011_000011_100001_011111_110101, "k16_move");

    }
}
