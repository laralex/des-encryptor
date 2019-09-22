use crate::des::bit_arithmetics::*;

const FIRST_BIT_IDX: usize = 1;
const BITS_IN_INPUT: usize = 64;

pub struct PermutationTable{
    pub input_size: usize,
    pub output_size: usize,
    pub ways: Vec<usize>,
}

// TODO: test
impl PermutationTable {
    pub fn new(mut ways: Vec<usize>, numbering_left_to_right: bool, numbering_from: usize) -> Self {
        let ways = if numbering_left_to_right  {
            ways.iter().rev()
                .map(|idx| BITS_IN_INPUT - *idx + 1 - numbering_from)
                .collect::<Vec<_>>()
        } else if numbering_from > 0 {
            for idx in ways.iter_mut(){
                *idx -= numbering_from;
            }
            ways
        } else { ways };
        Self {
            input_size: ways.iter().max().unwrap() - numbering_from + 1,
            output_size: ways.len(),
            ways,
        }
    }

    pub fn apply_with_skip(&self, input: u64, skip: &Vec<u8>) -> u64 {
        let mut s = skip.iter();
        let mut next_skip = s.next();
        let mut bit_pos = 0;
        let mut result = 0;
        for (i, &take_bit) in self.ways.iter().enumerate() {
            if next_skip != None && i as u8 == *next_skip.unwrap() {
                bit_pos += 1;
                next_skip = s.next();
            }
            result += get_bit(input, take_bit as u8) << bit_pos;
            bit_pos += 1;
        }
        result // bad rearrange 
    }

    pub fn apply(&self, input: u64) -> u64 {
        self.apply_with_skip(input, &vec![])
    }
}
