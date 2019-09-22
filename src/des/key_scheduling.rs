const BITS_IN_INPUT: u8 = 64;
const BITS_ORDER_L_TO_R: bool = true;
const FIRST_BIT_IDX: usize = 1;

use crate::des::bit_arithmetics;
use bit_arithmetics::*;
use crate::des::bit_permutation::*;

pub trait BidirectionalIterator: std::iter::Iterator {
    fn prev(&mut self) -> Option<Self::Item>;
}

//#[derive(Clone, Eq, PartialEq, Debug)]
pub struct KeyScheduler {
    current_key: u64,
    counter: u8,
    bits_in_key: u8,
    split_bit: u8,
    initial_permutation_table: PermutationTable,
    permuting_choice_table: PermutationTable,
}

type Item = (u64, usize);

impl KeyScheduler {
    pub fn new(initial_key: u64) -> KeyScheduler {
        let mut to_drop_bits = vec![7, 15, 23, 31, 39, 47, 55, 63];
        if BITS_ORDER_L_TO_R {
            for v in to_drop_bits.iter_mut() {
                *v = BITS_IN_INPUT - *v + 1;
            }
        }
        //println!("{:#064b}", init_key);
        let initial_permutation_table = PermutationTable::new(vec![
            57, 49, 41, 33, 25, 17, 9, 1, 58, 50, 42, 34, 26, 18,
            10, 2, 59, 51, 43, 35, 27, 19, 11, 3, 60, 52, 44, 36,
            63, 55, 47, 39, 31, 23, 15, 7, 62, 54, 46, 38, 30, 22,
            14, 6, 61, 53, 45, 37, 29, 21, 13, 5, 28, 20, 12, 4
        ], BITS_ORDER_L_TO_R, FIRST_BIT_IDX);
        
        //println!("{:#064b}", initial_key);
        let current_key = initial_permutation_table.apply_with_skip(initial_key, &to_drop_bits);
        println!("{:#064b}", current_key);
        //let (current_key, bits_in_key) = drop_bits(current_key, &to_drop_bits);
        //println!("{:#064b}", current_key);
        let bits_in_key = BITS_IN_INPUT - to_drop_bits.len() as u8;
        let split_bit = bits_in_key / 2;

        let permuting_choice_table = PermutationTable::new(vec![
            14, 17, 11, 24, 1, 5, 3, 28, 15, 6, 21, 10, 23, 19, 12, 4,
            26, 8, 16, 7, 27, 20, 13, 2, 41, 52, 31, 37, 47, 55, 30, 40,
            51, 45, 33, 48, 44, 49, 39, 56, 34, 53, 46, 42, 50, 36, 29, 32,
        ], BITS_ORDER_L_TO_R, FIRST_BIT_IDX);
        
        //println!("{:?}", &permuting_choice_table.ways);
        KeyScheduler {
            counter: 0,
            current_key,
            bits_in_key,
            split_bit,
            initial_permutation_table,
            permuting_choice_table,
        }
    }

    fn rotate_key(&mut self, to_left: bool) -> Option<Item>{
        let shift_for = match self.counter {
            0 | 1 | 8 | 15 => 1,
            _ => 2,
        };
        type Rotate = fn(u64, u8, u8, u8) -> Option<u64>;
        let rotate: Rotate = if to_left { bit_arithmetics::rotate_left }
        else { bit_arithmetics::rotate_right };
        self.current_key = rotate(
            self.current_key,
            0, self.split_bit,
            shift_for
        )?;
        self.current_key = rotate(
            self.current_key,
            self.split_bit, self.bits_in_key,
            shift_for
        )?;

        let shrinked_key = self.permuting_choice_table.apply(
            self.current_key
        );

        println!("{:#064b}", shrinked_key);

        Some((shrinked_key, self.permuting_choice_table.output_size))
    }
}


impl Iterator for KeyScheduler {
    type Item = Item;
    fn next(&mut self) -> Option<Self::Item> {
        if self.counter == self.split_bit {self.counter = 0;} // this shift is same as first key
        else { self.counter += 1; }

        self.rotate_key(true)
    }    
}

impl BidirectionalIterator for KeyScheduler {
    fn prev(&mut self) -> Option<Item> {
        if self.counter == 0 { self.counter = self.split_bit - 1; } // this shift is same as first key
        else { self.counter -= 1; }

        self.rotate_key(false)
    }
}

#[test]
fn test_key_scheduler(){
    let mut s = KeyScheduler::new(
        0b_00010011_00110100_01010111_01111001_10011011_10111100_11011111_11110001
    );

    assert_eq!(s.current_key, 0b1111000011001100101010101111_0101010101100110011110001111, "init_key");
    // assert_eq!(s.next().unwrap().0, 0b_1110000110011001010101011111_1010101011001100111100011110, "k1");
    // assert_eq!(s.next().unwrap().0, 0b_1100001100110010101010111111_0101010110011001111000111101, "k2");
    // assert_eq!(s.next().unwrap().0, 0b_0000110011001010101011111111_0101011001100111100011110101);
    // assert_eq!(s.next().unwrap().0, 0b_0011001100101010101111111100_0101100110011110001111010101);
    // assert_eq!(s.next().unwrap().0, 0b_1100110010101010111111110000_0110011001111000111101010101);
    // assert_eq!(s.next().unwrap().0, 0b_0011001010101011111111000011_1001100111100011110101010101);
    // assert_eq!(s.next().unwrap().0, 0b_1100101010101111111100001100_0110011110001111010101010110);
    // assert_eq!(s.next().unwrap().0, 0b_0010101010111111110000110011_1001111000111101010101011001, "k8");
    // assert_eq!(s.next().unwrap().0, 0b_0101010101111111100001100110_0011110001111010101010110011);
    // assert_eq!(s.next().unwrap().0, 0b_0101010111111110000110011001_1111000111101010101011001100);
    // assert_eq!(s.next().unwrap().0, 0b_0101011111111000011001100101_1100011110101010101100110011);
    // assert_eq!(s.next().unwrap().0, 0b_0101111111100001100110010101_0001111010101010110011001111);
    // assert_eq!(s.next().unwrap().0, 0b_0111111110000110011001010101_0111101010101011001100111100);
    // assert_eq!(s.next().unwrap().0, 0b_1111111000011001100101010101_1110101010101100110011110001);
    // assert_eq!(s.next().unwrap().0, 0b_1111100001100110010101010111_1010101010110011001111000111, "k15");
    // assert_eq!(s.next().unwrap().0, 0b_1111000011001100101010101111_0101010101100110011110001111, "k16");

    assert_eq!(s.next().unwrap().0, 0b_000110_110000_001011_101111_111111_000111_000001_110010, "k");
    assert_eq!(s.next().unwrap().0, 0b_011110_011010_111011_011001_110110_111100_100111_100101, "k");
    assert_eq!(s.next().unwrap().0, 0b_010101_011111_110010_001010_010000_101100_111110_011001, "k");
    assert_eq!(s.next().unwrap().0, 0b_011100_101010_110111_010110_110110_110011_010100_011101, "k");
    assert_eq!(s.next().unwrap().0, 0b_011111_001110_110000_000111_111010_110101_001110_101000, "k");
    assert_eq!(s.next().unwrap().0, 0b_011000_111010_010100_111110_010100_000111_101100_101111, "k");
    assert_eq!(s.next().unwrap().0, 0b_111011_001000_010010_110111_111101_100001_100010_111100, "k");
    assert_eq!(s.next().unwrap().0, 0b_111101_111000_101000_111010_110000_010011_101111_111011, "k");
    assert_eq!(s.next().unwrap().0, 0b_111000_001101_101111_101011_111011_011110_011110_000001, "k");
    assert_eq!(s.next().unwrap().0, 0b_101100_011111_001101_000111_101110_100100_011001_001111, "k");
    assert_eq!(s.next().unwrap().0, 0b_001000_010101_111111_010011_110111_101101_001110_000110, "k");
    assert_eq!(s.next().unwrap().0, 0b_011101_010111_000111_110101_100101_000110_011111_101001, "k");
    assert_eq!(s.next().unwrap().0, 0b_100101_111100_010111_010001_111110_101011_101001_000001, "k");
    assert_eq!(s.next().unwrap().0, 0b_010111_110100_001110_110111_111100_101110_011100_111010, "k");
    assert_eq!(s.next().unwrap().0, 0b_101111_111001_000110_001101_001111_010011_111100_001010, "k");
    assert_eq!(s.next().unwrap().0, 0b_110010_110011_110110_001011_000011_100001_011111_110101, "k");
}
