use crate::math::bit_arithmetics::idx_from_high as high;
use crate::math::bit_arithmetics::idx_from_low as low;
use crate::des::details::*;

const BITS_IN_INPUT: u32 = 64;
const BITS_IN_LOW_HALF: u32 = 32;
const BIT_COUNT_FROM: u32 = 1;
const GRANULATION_INPUT_SIZE_BITS: u32 = 6;
const GRANULATION_OUTPUT_SIZE_BITS: u32 = 4;
const DATA_INITIAL_SIZE_BITS: u32 = 32;

lazy_static! {
    static ref EXPANSION: PermutationTable = PermutationTable::new(vec![
        32, 1, 2, 3, 4, 5,
        4, 5, 6, 7, 8, 9,
        8, 9, 10, 11, 12, 13,
        12, 13, 14, 15, 16, 17,
        16, 17, 18, 19, 20, 21,
        20, 21, 22, 23, 24, 25,
        24, 25, 26, 27, 28, 29,
        28, 29, 30, 31, 32, 1,
    ], BIT_COUNT_FROM, BITS_IN_LOW_HALF);
}


lazy_static! {
    static ref FINAL_PERMUTATION: PermutationTable = PermutationTable::new(vec![
        16, 7, 20, 21,
        29, 12, 28, 17,
        1, 15, 23, 26,
        5, 18, 31, 10,
        2, 8, 24, 14,
        32, 27, 3, 9,
        19, 13, 30, 6,
        22, 11, 4, 25,
    ], BIT_COUNT_FROM, BITS_IN_LOW_HALF);
}

static GRANULATION_ROW_BITS_INDICES: [u32; 2] = [0, 5];

lazy_static! {
    static ref GRANULATIONS: [EncodingTable<'static>; 8] = [
        EncodingTable::new(
            vec![
                14, 4, 13, 1, 2, 15, 11, 8, 3, 10, 6, 12, 5, 9, 0, 7,
                0, 15, 7, 4, 14, 2, 13, 1, 10, 6, 12, 11, 9, 5, 3, 8,
                4, 1, 14, 8, 13, 6, 2, 11, 15, 12, 9, 7, 3, 10, 5, 0,
                15, 12, 8, 2, 4, 9, 1, 7, 5, 11, 3, 14, 10, 0, 6, 13,
            ],
            &GRANULATION_ROW_BITS_INDICES,
            GRANULATION_INPUT_SIZE_BITS,
            GRANULATION_OUTPUT_SIZE_BITS,
        ),
        EncodingTable::new(
            vec![
                15, 1, 8, 14, 6, 11, 3, 4, 9, 7, 2, 13, 12, 0, 5, 10,
                3, 13, 4, 7, 15, 2, 8, 14, 12, 0, 1, 10, 6, 9, 11, 5,
                0, 14, 7, 11, 10, 4, 13, 1, 5, 8, 12, 6, 9, 3, 2, 15,
                13, 8, 10, 1, 3, 15, 4, 2, 11, 6, 7, 12, 0, 5, 14, 9,
            ],
            &GRANULATION_ROW_BITS_INDICES,
            GRANULATION_INPUT_SIZE_BITS,
            GRANULATION_OUTPUT_SIZE_BITS,
        ),
        EncodingTable::new(
            vec![
                10, 0, 9, 14, 6, 3, 15, 5, 1, 13, 12, 7, 11, 4, 2, 8,
                13, 7, 0, 9, 3, 4, 6, 10, 2, 8, 5, 14, 12, 11, 15, 1,
                13, 6, 4, 9, 8, 15, 3, 0, 11, 1, 2, 12, 5, 10, 14, 7,
                1, 10, 13, 0, 6, 9, 8, 7, 4, 15, 14, 3, 11, 5, 2, 12,
            ],
            &GRANULATION_ROW_BITS_INDICES,
            GRANULATION_INPUT_SIZE_BITS,
            GRANULATION_OUTPUT_SIZE_BITS,
        ),
        EncodingTable::new(
            vec![
                7, 13, 14, 3, 0, 6, 9, 10, 1, 2, 8, 5, 11, 12, 4, 15,
                13, 8, 11, 5, 6, 15, 0, 3, 4, 7, 2, 12, 1, 10, 14, 9,
                10, 6, 9, 0, 12, 11, 7, 13, 15, 1, 3, 14, 5, 2, 8, 4,
                3, 15, 0, 6, 10, 1, 13, 8, 9, 4, 5, 11, 12, 7, 2, 14,
            ],
            &GRANULATION_ROW_BITS_INDICES,
            GRANULATION_INPUT_SIZE_BITS,
            GRANULATION_OUTPUT_SIZE_BITS,
        ),
        EncodingTable::new(
            vec![
                2, 12, 4, 1, 7, 10, 11, 6, 8, 5, 3, 15, 13, 0, 14, 9,
                14, 11, 2, 12, 4, 7, 13, 1, 5, 0, 15, 10, 3, 9, 8, 6,
                4, 2, 1, 11, 10, 13, 7, 8, 15, 9, 12, 5, 6, 3, 0, 14,
                11, 8, 12, 7, 1, 14, 2, 13, 6, 15, 0, 9, 10, 4, 5, 3,
            ],
            &GRANULATION_ROW_BITS_INDICES,
            GRANULATION_INPUT_SIZE_BITS,
            GRANULATION_OUTPUT_SIZE_BITS,
        ),
        EncodingTable::new(
            vec![
                12, 1, 10, 15, 9, 2, 6, 8, 0, 13, 3, 4, 14, 7, 5, 11,
                10, 15, 4, 2, 7, 12, 9, 5, 6, 1, 13, 14, 0, 11, 3, 8,
                9, 14, 15, 5, 2, 8, 12, 3, 7, 0, 4, 10, 1, 13, 11, 6,
                4, 3, 2, 12, 9, 5, 15, 10, 11, 14, 1, 7, 6, 0, 8, 13,
            ],
            &GRANULATION_ROW_BITS_INDICES,
            GRANULATION_INPUT_SIZE_BITS,
            GRANULATION_OUTPUT_SIZE_BITS,
        ),
        EncodingTable::new(
            vec![
                4, 11, 2, 14, 15, 0, 8, 13, 3, 12, 9, 7, 5, 10, 6, 1,
                13, 0, 11, 7, 4, 9, 1, 10, 14, 3, 5, 12, 2, 15, 8, 6,
                1, 4, 11, 13, 12, 3, 7, 14, 10, 15, 6, 8, 0, 5, 9, 2,
                6, 11, 13, 8, 1, 4, 10, 7, 9, 5, 0, 15, 14, 2, 3, 12,
            ],
            &GRANULATION_ROW_BITS_INDICES,
            GRANULATION_INPUT_SIZE_BITS,
            GRANULATION_OUTPUT_SIZE_BITS,
        ),
        EncodingTable::new(
            vec![
                13, 2, 8, 4, 6, 15, 11, 1, 10, 9, 3, 14, 5, 0, 12, 7,
                1, 15, 13, 8, 10, 3, 7, 4, 12, 5, 6, 11, 0, 14, 9, 2,
                7, 11, 4, 1, 9, 12, 14, 2, 0, 6, 10, 13, 15, 3, 5, 8,
                2, 1, 14, 7, 4, 10, 8, 13, 15, 12, 9, 0, 3, 5, 6, 11,
            ],
            &GRANULATION_ROW_BITS_INDICES,
            GRANULATION_INPUT_SIZE_BITS,
            GRANULATION_OUTPUT_SIZE_BITS,
        ),
    ];
}

// const GRANUAL_PERMUTATIONS: [EncodingTable; 8] = [
//     PermutationTable::new(vec![
        
//     ], BIT_COUNT_FROM, SMALL_BLOCK_SIZE_BITS),
// ];

#[inline]
pub fn encrypt_round(data: u64, key: Key)  -> u64 {
    let split = low::split_by_bit(
        data, BITS_IN_LOW_HALF, BITS_IN_INPUT).unwrap();
    let (high_half, low_half) = (split.0 as u32, split.1);
    let new_high = low_half << BITS_IN_LOW_HALF;
    let new_low  = high_half ^ feilstel_function(low_half as u32, key);
    new_high + new_low as u64
}

#[inline]
pub fn decrypt_round(data: u64, key: Key) -> u64 {
    println!("{:#066b}", data);
    let swapped_halfs = low::swap_ranges(
        data, BITS_IN_LOW_HALF, BITS_IN_INPUT).unwrap();
    println!("{:#066b}", swapped_halfs);
    let decrypted_data = encrypt_round(swapped_halfs, key);
    println!("{:#066b}", decrypted_data);
    let end = low::swap_ranges(
        decrypted_data, BITS_IN_LOW_HALF, BITS_IN_INPUT).unwrap();
    println!("{:#066b}", end);
    end
}

// TODO: check key size
fn feilstel_function(mut data: u32, key: Key)  -> u32 {
    let data = data as u64;
    
    let expanded_data = EXPANSION.apply(data as u64);    
    // println!("{:#066b}", expanded_data);
    let encrypted_data = expanded_data ^ key.value;
    // println!("{:#066b}", encrypted_data);
    let mut data_size = key.size_bits;
    let mut data_to_split = encrypted_data;
    let mut merged_data = 0;
    for g in 0.. (key.size_bits / GRANULATION_INPUT_SIZE_BITS) {
        let split = high::split_by_bit(
            data_to_split,
            GRANULATION_INPUT_SIZE_BITS - 1,
            data_size
        ).unwrap();
        let block = split.0;
        data_to_split = split.1;
        data_size -= GRANULATION_INPUT_SIZE_BITS;
        let encoded_data = GRANULATIONS[g as usize].apply(block);
        merged_data <<= GRANULATION_OUTPUT_SIZE_BITS;
        merged_data += encoded_data
    }
    FINAL_PERMUTATION.apply(merged_data as u64) as u32
}

#[test]
fn test_feilstel_function() {
    //let val: u32 = 0b1111_0000_1010_1010_1111_0000_1010_1010;
    // println!("{:#066b}", val);
    assert_eq!(
        feilstel_function(
            0b1111_0000_1010_1010_1111_0000_1010_1010,
            Key {value:0b000110_110000_001011_101111_111111_000111_000001_110010, size_bits:48}),
        0b0010_0011_0100_1010_1010_1001_1011_1011
    );
}

#[test]
fn test_encrypt_round() {
    let mut scheduler = KeyScheduler::new_encrypting(0x133457799BBCDFF1);
    assert_eq!(
        encrypt_round(
            0b1100_1100_0000_0000_1100_1100_1111_1111_1111_0000_1010_1010_1111_0000_1010_1010, scheduler.next().unwrap()),
        0b1111_0000_1010_1010_1111_0000_1010_1010__1110_1111_0100_1010_0110_0101_0100_0100,
    )
}
#[test]
fn test_decrypt_round() {
    let mut scheduler = KeyScheduler::new_decrypting(0x133457799BBCDFF1);
    assert_eq!(
        decrypt_round(
            0b1111_0000_1010_1010_1111_0000_1010_1010__1110_1111_0100_1010_0110_0101_0100_0100, scheduler.next().unwrap()),
        0b1100_1100_0000_0000_1100_1100_1111_1111_1111_0000_1010_1010_1111_0000_1010_1010
    )
}
