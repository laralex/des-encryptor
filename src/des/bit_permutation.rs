use crate::des::bit_arithmetics::idx_from_high as high;
use crate::des::bit_arithmetics::idx_from_low as low;

pub struct PermutationTable{
    input_size: u32,
    output_size: u32,
    bit_destinations: Vec<u32>,
}

// TODO: test
impl PermutationTable {
    pub fn new(mut bit_destinations: Vec<u32>, numbering_from: u32, input_size_bits: u32) -> Self {
        for idx in bit_destinations.iter_mut(){
            *idx -= numbering_from;
        }
        Self {
            input_size: input_size_bits,
            output_size: bit_destinations.len() as u32,
            bit_destinations,
        }
    }

    pub fn apply(&self, value: u64) -> u64 {
        // self.apply_with_skips(input, &std::iter::empty())
        let mut result = 0;
        // println!("{}", self.input_size);
        for &bit_dest in self.bit_destinations.iter() {
            // FIXME: there is awkward counting from high end, though
            // it’s easier to work with low counting
            let true_bit_index = self.input_size - 1 - bit_dest;
            result += low::is_bit_set(value, true_bit_index) as u64;
            result <<= 1;
        }
        result >> 1
    }
}

impl PermutationTable {
    pub fn input_size(&self) -> u32 { self.input_size }
    pub fn output_size(&self) -> u32 { self.output_size }
    pub fn bit_destinations(&self) -> &Vec<u32> { &self.bit_destinations }
}

#[test]
pub fn test_permutation_table() {
    let pt = PermutationTable::new(vec![
        57, 49, 41, 33, 25, 17, 9, 1, 58, 50, 42, 34, 26, 18,
        10, 2, 59, 51, 43, 35, 27, 19, 11, 3, 60, 52, 44, 36,
        63, 55, 47, 39, 31, 23, 15, 7, 62, 54, 46, 38, 30, 22,
        14, 6, 61, 53, 45, 37, 29, 21, 13, 5, 28, 20, 12, 4
    ], 1, 64);
    assert_eq!(
        pt.apply(0b_00010011_00110100_01010111_01111001_10011011_10111100_11011111_11110001),
        0b_1111000_0110011_0010101_0101111_0101010_1011001_1001111_0001111
    );
}
