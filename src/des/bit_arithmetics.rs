const BITS_IN_INPUT: u32 = 64;
macro_rules! index_from_end {
    ($idx: expr) => { index_from_end!($idx, BITS_IN_INPUT) };
    ($idx: expr, $end: expr) => { $end - $idx - 1 };
}

pub mod idx_from_low {
    use super::*;
    use std::ops::Deref;
    
    #[inline]
    pub fn bit_mask(bit: u32) -> u64 {
        if bit < BITS_IN_INPUT { 1 << bit }
        else { 0 }
    }
    #[inline]
    pub fn bit_lower_mask(bit: u32) -> u64 {
        if bit < BITS_IN_INPUT { bit_mask(bit) - 1 }
        else { std::u64::MAX }
    }
    #[inline]
    pub fn bit_higher_mask(bit: u32) -> u64 {
        std::u64::MAX - bit_lower_mask(bit + 1)
    }
    #[inline]
    pub fn bit_range_mask(begin_bit: u32, end_bit: u32) -> Option<u64> {
        if end_bit >= begin_bit {
            Some(bit_lower_mask(end_bit) - bit_lower_mask(begin_bit))
        } else { None }
        //if begin_bit >  || begin_bit == end_bit { return Some(0); }
    }

    #[inline]
    pub fn filter_bit(number: u64, bit: u32, size_bits: u32) -> u64 {
        if bit < size_bits { (number & bit_mask(bit)) }
        else { 0 }
    }

    #[inline]
    pub fn is_bit_set(number: u64, bit: u32, size_bits: u32) -> bool {
        filter_bit(number, bit, size_bits) != 0
    }

    #[inline]
    pub fn get_bit(number: u64, bit_idx: u32, size_bits: u32) -> u64 {
        filter_bit(number, bit_idx, size_bits) >> bit_idx
    }

    #[inline]
    pub fn filter_bit_range(number: u64, begin_bit: u32, end_bit: u32, size_bits: u32) -> Option<u64> {
        if begin_bit < size_bits {
            Some( number & bit_range_mask(begin_bit, end_bit)? )
        } else { None }
    }

    #[inline]
    pub fn extract_bit_range(number: u64, begin_bit: u32, end_bit: u32, size_bits: u32) -> Option<u64> {
        Some( filter_bit_range(number, begin_bit, end_bit, size_bits)? >> begin_bit )
    }

    /// Returns number with dropped bits and valuable lenfth of number
    /// in bits
    // TODO unordered, non unique indices support?
    pub fn drop_bits<'a, T>(number: u64, ordered_drop_indices: T, size_bits: u32) -> (u64, u32)
    where T: Iterator<Item = u32> {
        let mut it = ordered_drop_indices.into_iter()
            .take_while(|&e| e < size_bits)
            .enumerate();
        let first = it.next();
        if let None = first {
            return (number, BITS_IN_INPUT);
        }
        let mut prev_drop_bit = first.unwrap().1;
        let lower_mask = bit_range_mask(0, prev_drop_bit).unwrap();
        let mut middle_of_dropped = 0;
        let mut total_drop = 1;
        let mut it = it.peekable();
        while let Some(&(drop_count, drop_bit)) = it.peek()  {
            if drop_bit >= BITS_IN_INPUT { break; }
            let mask= bit_range_mask(prev_drop_bit + 1, drop_bit).unwrap();
            prev_drop_bit = drop_bit;
            middle_of_dropped += (mask & number) >> drop_count;
            total_drop += 1;
            it.next();
        }
        let higher_mask = bit_higher_mask(prev_drop_bit);
        // println!("{:b} {:b} {:b}", lower_mask, higher_mask, middle_of_dropped);
        let result_number = (lower_mask & number) + middle_of_dropped
            + ((higher_mask & number).overflowing_shr(total_drop).0);
        (result_number, BITS_IN_INPUT - total_drop)
    }

    #[inline]
    pub fn split_by_bit(number: u64, split_bit: u32, size_bits: u32) -> Option<(u64, u64)> {
        let high_mask = bit_range_mask(split_bit, size_bits)?;
        let low_mask = bit_range_mask(0, split_bit)?;
        // println!("{:#064b} {}", number, size_bits);
        // println!("{:#064b}", high_mask,);
        // println!("{:#064b}", low_mask,);
        // println!("{:#064b}", (number & high_mask) >> split_bit, );
        // println!("{:#064b}",  number & low_mask );
        Some(( (number & high_mask) >> split_bit,
                number & low_mask ))
    }

    pub fn rotate_range_to_high(number: u64, begin_bit: u32,
                                mut end_bit: u32, mut shift_for: u32,
                                size_bits: u32)
                                -> Option<u64> {
        // normalize inputs
        if end_bit == 0 || begin_bit >= end_bit {
            return None;
        } else if end_bit > size_bits {
            end_bit = size_bits;
        }
        shift_for %= end_bit - begin_bit;

        let cutoff_begin = end_bit - shift_for;
        //println!("rotate ok {} {} {}", cutoff_begin, end_bit, cutoff_mask); 
        let cutoff_mask = bit_range_mask(cutoff_begin, end_bit)?;
        let remain_mask = bit_range_mask(begin_bit, cutoff_begin)?;
        let outer_mask = std::u64::MAX - (cutoff_mask | remain_mask);
        let cutoff_rotated = (number & cutoff_mask) >> (cutoff_begin - begin_bit);
        let remain_shifted = (number & remain_mask) << shift_for;
        let outer = number & outer_mask;
        Some( remain_shifted + cutoff_rotated + outer )
    }

    pub fn rotate_range_to_low(number: u64, begin_bit: u32,
                               mut end_bit: u32, mut shift_for: u32,
                               size_bits: u32)
                                -> Option<u64> {
        if end_bit == 0 || begin_bit >= end_bit {
            return None;
        } else if end_bit > size_bits {
            end_bit = size_bits;
        }
        shift_for %= end_bit - begin_bit;
        
        let cutoff_begin = begin_bit + shift_for;
        let cutoff_mask = bit_range_mask(begin_bit, cutoff_begin)?;
        let remain_mask = bit_range_mask(cutoff_begin, end_bit)?;
        let outer_mask = std::u64::MAX - (cutoff_mask | remain_mask);
        let cutoff_rotated = (number & cutoff_mask) << (end_bit - cutoff_begin);
        let remain_shifted = (number & remain_mask) >> shift_for;
        let outer = number & outer_mask;
        Some( remain_shifted + cutoff_rotated + outer )
    }

    #[inline]
    pub fn swap_ranges(number: u64, split_bit: u32, size_bits: u32) -> Option<u64> {
        let high_range = extract_bit_range(number, split_bit, size_bits, size_bits)?;
        let low_range = filter_bit_range(number, 0, split_bit, size_bits)?;
        Some( (low_range << split_bit) + high_range )
    }
}

pub mod idx_from_high {
    use super::*;
    use idx_from_low as low;

    #[inline]
    pub fn bit_mask(high_bit: u32, size_bits: u32) -> u64 {
        low::bit_mask(index_from_end!(high_bit, size_bits))
    }
    #[inline]
    pub fn bit_lower_mask(high_bit: u32, size_bits: u32) -> u64 {
        low::bit_lower_mask(index_from_end!(high_bit, size_bits))
    }
    #[inline]
    pub fn bit_higher_mask(bit: u32, size_bits: u32) -> u64 {
        low::bit_higher_mask(index_from_end!(bit, size_bits))
    }

    #[inline]
    pub fn bit_range_mask(begin_bit: u32, end_bit: u32, size_bits: u32) -> Option<u64> {
        low::bit_range_mask(index_from_end!(end_bit, size_bits), index_from_end!(begin_bit, size_bits))
    }

    #[inline]
    pub fn filter_bit(number: u64, bit: u32, size_bits: u32) -> u64 {
        low::filter_bit(number, index_from_end!(bit, size_bits), size_bits)
    }
    #[inline]
    pub fn is_bit_set(number: u64, bit: u32, size_bits: u32) -> bool {
        low::is_bit_set(number, index_from_end!(bit, size_bits), size_bits)
    }

    #[inline]
    pub fn get_bit(number: u64, bit_idx: u32, size_bits: u32) -> u64 {
        println!("{}", index_from_end!(bit_idx, size_bits));
        low::get_bit(number, index_from_end!(bit_idx, size_bits), size_bits) 
    }
    #[inline]
    pub fn filter_bit_range(number: u64, begin_bit: u32, end_bit: u32, size_bits: u32) -> Option<u64> {
        low::filter_bit_range(number, index_from_end!(end_bit, size_bits), index_from_end!(begin_bit, size_bits), size_bits)
    }
    #[inline]
    pub fn extract_bit_range(number: u64, begin_bit: u32, end_bit: u32, size_bits: u32) -> Option<u64> {
        low::extract_bit_range(number, index_from_end!(end_bit, size_bits), index_from_end!(begin_bit, size_bits), size_bits)
    }

    //type IntoIteratorU32 = IntoIterator<
    #[inline]
    pub fn drop_bits<'a, T>(number: u64, ordered_drop_indices: T, size_bits: u32) -> (u64, u32)
    where T: DoubleEndedIterator<Item = u32>{
        let recalculated_indices = ordered_drop_indices
            .rev()
            .filter(|&i| i < BITS_IN_INPUT)
            .map(|i| index_from_end!(i, size_bits));
        // let casted_indices = &(recalculated_indices
        //     as IntoIterator<Item = &'a u32, IntoIter = std::iter::Iterator<Item = &'a u32>>);
        low::drop_bits(number, recalculated_indices, size_bits)
    }

    #[inline]
    pub fn split_by_bit(number: u64, split_bit: u32, size_bits: u32) -> Option<(u64, u64)> {
        low::split_by_bit(number, index_from_end!(split_bit, size_bits), size_bits)
    }

    #[inline]
    pub fn rotate_range_to_high(number: u64,  begin_bit: u32,
                                end_bit: u32,  shift_for: u32, size_bits: u32)
                                -> Option<u64> {
        low::rotate_range_to_high(
            number, index_from_end!(end_bit, size_bits),
            index_from_end!(begin_bit, size_bits), shift_for,
            size_bits
        )
    }
    #[inline]
    pub fn rotate_range_to_low(number: u64, begin_bit: u32,
                               end_bit: u32, shift_for: u32, size_bits: u32)
                                -> Option<u64> {
        low::rotate_range_to_low(
            number, index_from_end!(end_bit, size_bits),
            index_from_end!(begin_bit, size_bits), shift_for,
            size_bits
        )
    }

    #[inline]
    pub fn swap_ranges(number: u64, split_bit: u32, size_bits: u32) -> Option<u64> {
        low::swap_ranges(number, index_from_end!(split_bit, size_bits), size_bits)
    }    
 }



// pub struct BitIteratingBytes<'a> ( &'a [u8] );
// impl BitIteratingBytes {
//     pub fn from<'a>(bytes: &'a [u8]) -> Self {
//         Self(bytes.clone())
//     }
// }

#[test]
pub fn test_bit_mask_operations() {
    use idx_from_low::*;
    assert_eq!(bit_mask(5), 0b_10_0000, "5th bit = 2**5");
    assert_eq!(bit_mask(0), 0b_00_0001, "1st bit = 1");
    assert_eq!(bit_mask(63),0x80_00_00_00_00_00_00_00, "63th bit = 2**63");
    //assert_eq!(bit_mask(65), 0b_10, "overflowing with type rotation");
    
    assert_eq!(bit_lower_mask(5),  0b_01_1111, "mask of 5 first bits");
    assert_eq!(bit_lower_mask(0),  0b_00_0000, "mask of 0 first bits");
    assert_eq!(bit_lower_mask(63), 0x7F_FF_FF_FF_FF_FF_FF_FF, "mask of 63 first bits");
    assert_eq!(bit_lower_mask(64), std::u64::MAX, "mask of all bits");

    assert_eq!(bit_range_mask(3, 5), Some(0b_0001_1000));
    assert_eq!(bit_range_mask(2, 6), Some(0b_0011_1100));
    assert_eq!(bit_range_mask(0, 1), Some(0b_0000_0001));
    assert_eq!(bit_range_mask(0, 0), Some(0b_0000_0000));
    assert_eq!(bit_range_mask(3, 3), Some(0b_0000_0000));
    assert_eq!(bit_range_mask(3, 3), Some(0b_0000_0000));
    assert_eq!(bit_range_mask(64, 64), Some(0));
    assert_eq!(bit_range_mask(63, 65), Some(0x80_00_00_00_00_00_00_00));
    assert_eq!(bit_range_mask(7,3), None);

    assert_eq!(split_by_bit(0b1010_1111, 4, BITS_IN_INPUT), Some((0b1010, 0b1111)));
}

#[test]
pub fn test_bit_rotate(){
    use idx_from_low::*;
    assert_eq!(rotate_range_to_high(1, 0, 23, 3, BITS_IN_INPUT),
               Some(0b1000), "normal shift");
    assert_eq!(rotate_range_to_high(0b10101, 0, 2, 3, BITS_IN_INPUT),
               Some(0b10110),
               "only 2 value bits, others are stationary");
    assert_eq!(rotate_range_to_high(0b00101, 0, 5, 3, BITS_IN_INPUT),
               Some(0b01001), "rotation");
    assert_eq!(rotate_range_to_high(0b00101, 0, 5, 0, BITS_IN_INPUT),
               Some(0b00101), "no shift at all");
    assert_eq!(rotate_range_to_high(0b00101, 0, 5, 5, BITS_IN_INPUT),
               Some(0b00101), "identity shift");
    assert_eq!(rotate_range_to_high(0b00101, 0, 0, 5, BITS_IN_INPUT),
               None, "end bit is zero, then last bit is what?");
    
    let big_val = std::u64::MAX - 7;
    assert_eq!(rotate_range_to_high(big_val, 0, 64, 5, BITS_IN_INPUT),
               Some(big_val.rotate_left(5)),
               "shifts over type boundaries");
    assert_eq!(rotate_range_to_high(big_val, 0, 65, 5, BITS_IN_INPUT),
               Some(big_val.rotate_left(5)),
               "end bit is over type boundaries");
    assert_eq!(rotate_range_to_high(0b00100000, 5, 8, 4, BITS_IN_INPUT),
               Some(0b01000000), "non-zero begin bit");
    assert_eq!(rotate_range_to_high(0b110101, 3, 5, 2, BITS_IN_INPUT),
               Some(0b110101),
               "non-zero begin bit, with outer contents");
}

#[test]
pub fn test_drop_bits() {
    use idx_from_low::*;
    assert_eq!(drop_bits(
        0b_0101_0111_0110,
        [1,2].iter().cloned(),
        BITS_IN_INPUT),
               (0b_0001_0101_1100, 62));
    assert_eq!(drop_bits(
        0b_0101_0111_0110,
        [0].iter().cloned() ,
        BITS_IN_INPUT ),
               (0b_0101_0111_011, 63));
    assert_eq!(drop_bits(
        0b_0101_0111_0110,
        vec![0,1,2,3,5,60,61,62,63,64].iter().cloned(),
        BITS_IN_INPUT),
               (0b_0000_0010_1011, 55));
    assert_eq!(drop_bits(
        std::u64::MAX,
        (0..100).into_iter(),
        BITS_IN_INPUT),
               (0, 0));
    assert_eq!(idx_from_high::drop_bits(
        std::u64::MAX,
        (0..100).into_iter(),
        BITS_IN_INPUT),
               (0, 0));
}

// pub  test_bit_split() {
//     //assert_eq!()
// }
