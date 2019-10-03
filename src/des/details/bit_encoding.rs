use crate::math::bit_arithmetics::idx_from_low as low;
use crate::math::bit_arithmetics::idx_from_high as high;

const MAX_INT_SIZE_BITS: u32 = 64;
pub struct EncodingTable<'a> {
    encoding_rules: Vec<u64>,
    input_size: u32,
    output_size: u32,
    row_bits: &'a [u32],
}

impl<'a> EncodingTable<'a> {
    pub fn new(encoding_rules: Vec<u64>, row_bits: &'a [u32],
               input_size: u32, output_size: u32) -> EncodingTable {
        // TODO: return result, check inputs
        // TODO: start bit counting argument (from 1?)
        EncodingTable {
            encoding_rules,
            input_size,
            output_size,
            row_bits,
        }
    }

    pub fn apply(&self, number: u64) -> u64 {
        let total_columns = (self.encoding_rules.len() / (1 << self.row_bits.len())) as u64;
        let (col_idx, _) = high::drop_bits(number, self.row_bits.iter().cloned(), self.input_size);
        let row_idx = self.row_bits.iter().fold(
            0, |acc, &bit| (acc + high::get_bit(number, bit, self.input_size)) << 1
        ) >> 1;
        self.encoding_rules[(row_idx * total_columns + col_idx) as usize]
    }
}

impl<'e> EncodingTable<'e> {
    pub fn input_size(&'e self) -> u32 { self.input_size }
    pub fn output_size(&'e self) -> u32 { self.output_size }
    pub fn encoding_rules(&'e self) -> &Vec<u64> { &self.encoding_rules }
}

#[test]
pub fn test_encoding_table() {
    let table = EncodingTable::new(
        vec![
            14, 4, 13, 1, 2, 15, 11, 8, 3, 10, 6, 12, 5, 9, 0, 7,
            0, 15, 7, 4, 14, 2, 13, 1, 10, 6, 12, 11, 9, 5, 3, 8,
            4, 1, 14, 8, 13, 6, 2, 11, 15, 12, 9, 7, 3, 10, 5, 0,
            15, 12, 8, 2, 4, 9, 1, 7, 5, 11, 3, 14, 10, 0, 6, 13,
        ], &[0, 5], 6, 4,
    );
    assert_eq!(table.apply(0b011000), 0b0101);
    assert_eq!(table.apply(0b101001), 4);
    assert_eq!(table.apply(0b001101), 13);
    let table = EncodingTable::new(
        vec![
            15, 1, 8, 14, 6, 11, 3, 4, 9, 7, 2, 13, 12, 0, 5, 10,
            3, 13, 4, 7, 15, 2, 8, 14, 12, 0, 1, 10, 6, 9, 11, 5,
            0, 14, 7, 11, 10, 4, 13, 1, 5, 8, 12, 6, 9, 3, 2, 15,
            13, 8, 10, 1, 3, 15, 4, 2, 11, 6, 7, 12, 0, 5, 14, 9,
        ], &[0, 5], 6, 4,
    );
    assert_eq!(table.apply(0b010001), 0b1100);
    assert_eq!(table.apply(0b100101), 10);
}
