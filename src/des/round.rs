use crate::des::bit_arithmetics::idx_from_high as high;
use crate::des::*;
const BITS_IN_INPUT: u32 = 64;
const BITS_IN_LOW_HALF: u32 = 32;
const BIT_COUNT_FROM: u32 = 1;
const SMALL_BLOCK_SIZE_BITS: u32 = 6;
const DATA_INITIAL_SIZE_BITS: u32 = 32;
const EXPANSION: PermutationTable = PermutationTable::new(vec![
    32, 1, 2, 3, 4, 5,
    4, 5, 6, 7, 8, 9,
    8, 9, 10, 11, 12, 13,
    12, 13, 14, 15, 16, 17,
    16, 17, 18, 19, 20, 21,
    20, 21, 22, 23, 24, 25,
    24, 25, 26, 27, 28, 29,
    28, 29, 30, 31, 32, 1,
], BIT_COUNT_FROM, BITS_IN_LOW_HALF);

const GRANUAL_PERMUTATIONS: [PermutationTable; 8] = [
    PermutationTable::new(vec![
        
    ], BIT_COUNT_FROM, SMALL_BLOCK_SIZE_BITS),
];

#[inline]
pub fn encrypt_round(data: u64, (key, size): (u64, u32)) -> u64 {
    let split = high::split_by_bit(data, 32, 64).unwrap();
    let (high_half, low_half) = (split.0 as u32, split.1);
    let new_high = low_half << BITS_IN_LOW_HALF;
    let new_low  = high_half ^ feilstel_function(low_half as u32, (key, size));
    new_high + new_low as u64
}

#[inline]
pub fn decrypt_round(data: u64, (key, size): (u64, u32)) -> u64 {
    let swapped_halfs = high::swap_ranges(data, BITS_IN_LOW_HALF, BITS_IN_INPUT).unwrap();
    let decrypted_data = encrypt_round(swapped_halfs, (key, size));
    high::swap_ranges(decrypted_data, BITS_IN_LOW_HALF, BITS_IN_INPUT).unwrap()
}

// TODO: check key size
fn feilstel_function(mut data: u32, (key, size): (u64, u32)) -> u32 {
    let data = data as u64;
    let expanded_data = EXPANSION.apply(data as u64);
    let encrypted_data = expanded_data ^ key;
    
    let small_blocks = size / SMALL_BLOCK_SIZE_BITS;
    let mut data_size = DATA_INITIAL_SIZE_BITS;
    for block_idx in 0..small_blocks {
        let (block, data) = high::split_by_bit(
            data, SMALL_BLOCK_SIZE_BITS, data_size).unwrap();
        data_size -= SMALL_BLOCK_SIZE_BITS;
        
        
    }
}
