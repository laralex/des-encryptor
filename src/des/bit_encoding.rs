use crate::des::bit_arithmetics::idx_from_low as low;

const MAX_INT_SIZE_BITS: u32 = 64;
pub struct EncodingTable<'a> {
    encoding_rules: Vec<u64>;
    input_size: u32;
    output_size: u32;
    row_bits: &'a [u32];
}

impl EncodingTable {
    pub fn new(encoding_rules: Vec<u64>, row_bits: &[u32], output_size: u32) -> EncodingTable {
        // ExponentialIntSequence::new(2, 1)
        EncodingTable {
            encoding_rules,
            input_size: (0..MAX_INT_SIZE_BITS).map(|&e| 1 << e).find(
                |e| e > encoding_rules.size()
            ).unwrap(),
            row_bits,
        }
    }

    pub fn apply(&self, number: u64, size_bits: u32){
        low::filter_bit_range(number, 0, self.input_size, size_bits);
    }
}

impl EncodingTable {
    pub fn input_size(&self) -> u32 { self.input_size }
    pub fn output_size(&self) -> u32 { self.output_size }
    pub fn encoding_rules(&self) -> &Vec<u64> { &self.encoding_rules }
}
