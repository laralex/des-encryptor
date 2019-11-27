const BITS_IN_INPUT: u32 = 64;

/// Different implementations of DES use different bit notation for
/// programmers’ intuitive notation. In case if bits are numbered from
/// highest to lowest (instead from lowest to highest), use this macro
macro_rules! index_from_end {
    ($idx: expr) => { index_from_end!($idx, BITS_IN_INPUT) };
    ($idx: expr, $end: expr) => { $end - $idx - 1 };
}

/// Functions, that assume order of numeration is from the least bit
/// So this least bit has number 0, the next has 1 and so on
pub mod idx_from_low {
    use super::*;
    
    /// Creates a binary mask, that is single bit wide
    /// @returns binary mask with the only 1 in position of ‘bit’
    #[inline]
    pub fn bit_mask(bit: u32) -> u64 {
        if bit < BITS_IN_INPUT { 1 << bit }
        else { 0 }
    }
    /// Creates a binary mask, that spans from 0th bit to ‘bit’
    /// argument exclusive
    /// @returns such binary mask
    #[inline]
    pub fn bit_lower_mask(bit: u32) -> u64 {
        if bit < BITS_IN_INPUT { bit_mask(bit) - 1 }
        else { std::u64::MAX }
    }

    /// Creates a binary mask, that spans from 63th bit to ‘bit’
    /// argument exclusive
    /// @returns such binary mask
    #[inline]
    pub fn bit_higher_mask(bit: u32) -> u64 {
        std::u64::MAX - bit_lower_mask(bit + 1)
    }

    /// Creates a binary mask, that spans from ‘begin_bit’ inclusive
    /// to ‘end_bit’ exclusive
    /// @returns Option with such binary mask, None means ‘begin_bit‘
    /// is not lower than ‘end_bit’
    #[inline]
    pub fn bit_range_mask(begin_bit: u32, end_bit: u32) -> Option<u64> {
        if end_bit >= begin_bit {
            Some(bit_lower_mask(end_bit) - bit_lower_mask(begin_bit))
        } else { None }
    }

    /// Discard all bits in ‘number’ but specified bit. Assumes that
    /// number is ‘size_bits’ wide, so requesting to filter a bit
    /// above this size results in 0
    /// @returns a number with all bits discarded but one that is specified 
    #[inline]
    pub fn filter_bit(number: u64, bit: u32, size_bits: u32) -> u64 {
        if bit < size_bits { (number & bit_mask(bit)) }
        else { 0 }
    }

    /// Checks if the ‘bit’ in ‘number’ is set. Assumes that
    /// number is ‘size_bits’ wide, so requesting to filter a bit
    /// above this size results in 0
    /// @returns a number with all bits discarded but one that is specified 
    #[inline]
    pub fn is_bit_set(number: u64, bit: u32, size_bits: u32) -> bool {
        filter_bit(number, bit, size_bits) != 0
    }

    /// Checks if the ‘bit’ in ‘number’ is set, then 1 is returned,
    /// otherwise it’s 0. Assumes that the number is ‘size_bits’ wide,
    /// so requesting to get a bit greater than this size results in 0
    /// @returns 1 if the bit is lower than the ‘size‘’ and set, 0
    /// otherwise
    #[inline]
    pub fn get_bit(number: u64, bit_idx: u32, size_bits: u32) -> u64 {
        filter_bit(number, bit_idx, size_bits) >> bit_idx
    }

    /// Discard all bits in ‘number’ but specified range of
    /// bits. Assumes that number is ‘size_bits’ wide, so requesting
    /// to filter a bit above this size results in 0
    /// @returns Option with a number where all bits are discarded but
    /// the bits in specified range. None denotes that ‘begin_bit‘ is
    /// not lower than ‘end_bit’
    #[inline]
    pub fn filter_bit_range(number: u64, begin_bit: u32, end_bit: u32, size_bits: u32) -> Option<u64> {
        if begin_bit < size_bits {
            Some( number & bit_range_mask(begin_bit, end_bit)? )
        } else { None }
    }

    /// Discard all bits in ‘number’ but specified range of bits and
    /// returns a number, where selected bits are shifted by
    /// ‘begin_bit’ bits. Assumes that number is ‘size_bits’ wide, so
    /// requesting to filter a bit above this size results in 0
    /// @returns Option with a number where all bits are discarded but
    /// the bits in specified range, these bits are then shifted by
    /// ‘begin_bit’ bits. None denotes that ‘begin_bit‘ is
    /// not lower than ‘end_bit’
    #[inline]
    pub fn extract_bit_range(number: u64, begin_bit: u32, end_bit: u32, size_bits: u32) -> Option<u64> {
        Some( filter_bit_range(number, begin_bit, end_bit, size_bits)? >> begin_bit )
    }

    /// Removes all bits in a ‘number’ given an ‘ordered_brop_indices’
    /// iterator of ascending indices. Number is squized after drop of
    /// these bits. Dropping indices above of ‘size_bits’ has no effect.
    /// @returns a tuple with a number with dropped bits, and count of
    /// total bits dropped
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
        let result_number = (lower_mask & number) + middle_of_dropped
            + ((higher_mask & number).overflowing_shr(total_drop).0);
        (result_number, BITS_IN_INPUT - total_drop)
    }

    /// Split the ‘number‘ by a ‘split_bit’ specified. Assumes than
    /// the number is ‘size_bits’ long, so data above this size is
    /// discarded in the result of the function
    /// @returns a tuple with two parts of the ‘number’, first if from
    /// end of number (from ‘size_bits’) to the split_bit (inclusive),
    /// the second is from ‘split_bit’ exclusive to 0th bit
    #[inline]
    pub fn split_by_bit(number: u64, split_bit: u32, size_bits: u32) -> Option<(u64, u64)> {
        let high_mask = bit_range_mask(split_bit, size_bits)?;
        let low_mask = bit_range_mask(0, split_bit)?;
        Some(( (number & high_mask) >> split_bit,
                number & low_mask ))
    }

    /// Rotate the specified range of bits in the ‘number’ by
    /// ‘shift_for’ bits towards higher indices. Rotation
    /// affects only this range.
    /// @return Option with the original number, where specified range
    /// is rotated
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
        let cutoff_mask = bit_range_mask(cutoff_begin, end_bit)?;
        let remain_mask = bit_range_mask(begin_bit, cutoff_begin)?;
        let outer_mask = std::u64::MAX - (cutoff_mask | remain_mask);
        let cutoff_rotated = (number & cutoff_mask) >> (cutoff_begin - begin_bit);
        let remain_shifted = (number & remain_mask) << shift_for;
        let outer = number & outer_mask;
        Some( remain_shifted + cutoff_rotated + outer )
    }

    /// Rotate the specified range of bits in the ‘number’ by
    /// ‘shift_for’ bits towards lower indices. Rotation
    /// affects only this range.
    /// @returns Option with the original number, where specified range
    /// is rotated
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

    /// Swap bit ranges in the number over ‘split_bit’ (this bit
    /// belongs to higher range). Assumes that number is ‘size_bits’
    /// long, so bits above are not affected
    /// @returns Option with the same number, but two ranges are
    /// swapped. The first is from 0th bit to ‘split_bit’ exclusive,
    /// the second is from ‘split_bit’ to ‘size_bits’ exclusive
    #[inline]
    pub fn swap_ranges(number: u64, split_bit: u32, size_bits: u32) -> Option<u64> {
        let high_range = extract_bit_range(number, split_bit, size_bits, size_bits)?;
        let low_range = filter_bit_range(number, 0, split_bit, size_bits)?;
        Some( (low_range << split_bit) + high_range )
    }
}

/// Functions, that assume order of numeration is from the highest bit
/// So this highest bit has number 1, the next is 2nd and so on
pub mod idx_from_high {
    use super::*;
    use idx_from_low as low;

    /// Creates a binary mask, that is single bit wide
    /// @returns binary mask with the only 1 in position of ‘bit’
    #[inline]
    pub fn bit_mask(high_bit: u32, size_bits: u32) -> u64 {
        low::bit_mask(index_from_end!(high_bit, size_bits))
    }

    /// Creates a binary mask, that spans from 0th bit to ‘bit’
    /// argument exclusive
    /// @returns such binary mask
    #[inline]
    pub fn bit_lower_mask(high_bit: u32, size_bits: u32) -> u64 {
        low::bit_lower_mask(index_from_end!(high_bit, size_bits))
    }

    /// Creates a binary mask, that spans from 63th bit to ‘bit’
    /// argument exclusive
    /// @returns such binary mask
    #[inline]
    pub fn bit_higher_mask(bit: u32, size_bits: u32) -> u64 {
        low::bit_higher_mask(index_from_end!(bit, size_bits))
    }

    /// Creates a binary mask, that spans from ‘begin_bit’ inclusive
    /// to ‘end_bit’ exclusive
    /// @returns Option with such binary mask, None means ‘begin_bit‘
    /// is not lower than ‘end_bit’
    #[inline]
    pub fn bit_range_mask(begin_bit: u32, end_bit: u32, size_bits: u32) -> Option<u64> {
        low::bit_range_mask(index_from_end!(end_bit, size_bits), index_from_end!(begin_bit, size_bits))
    }

    /// Discard all bits in ‘number’ but specified bit. Assumes that
    /// number is ‘size_bits’ wide, so requesting to filter a bit
    /// above this size results in 0
    /// @returns a number with all bits discarded but one that is specified 
    #[inline]
    pub fn filter_bit(number: u64, bit: u32, size_bits: u32) -> u64 {
        low::filter_bit(number, index_from_end!(bit, size_bits), size_bits)
    }

    /// Checks if the ‘bit’ in ‘number’ is set. Assumes that
    /// number is ‘size_bits’ wide, so requesting to filter a bit
    /// above this size results in 0
    /// @returns a number with all bits discarded but one that is specified 
    #[inline]
    pub fn is_bit_set(number: u64, bit: u32, size_bits: u32) -> bool {
        low::is_bit_set(number, index_from_end!(bit, size_bits), size_bits)
    }

    /// Checks if the ‘bit’ in ‘number’ is set, then 1 is returned,
    /// otherwise it’s 0. Assumes that the number is ‘size_bits’ wide,
    /// so requesting to get a bit greater than this size results in 0
    /// @returns 1 if the bit is lower than the ‘size‘’ and set, 0
    /// otherwise
    #[inline]
    pub fn get_bit(number: u64, bit_idx: u32, size_bits: u32) -> u64 {
        //println!("{}", index_from_end!(bit_idx, size_bits));
        low::get_bit(number, index_from_end!(bit_idx, size_bits), size_bits) 
    }
    #[inline]

    /// Discard all bits in ‘number’ but specified range of
    /// bits. Assumes that number is ‘size_bits’ wide, so requesting
    /// to filter a bit above this size results in 0
    /// @returns Option with a number where all bits are discarded but
    /// the bits in specified range. None denotes that ‘begin_bit‘ is
    /// not lower than ‘end_bit’
    #[inline]
    pub fn filter_bit_range(number: u64, begin_bit: u32, end_bit: u32, size_bits: u32) -> Option<u64> {
        low::filter_bit_range(number, index_from_end!(end_bit, size_bits), index_from_end!(begin_bit, size_bits), size_bits)
    }

    /// Discard all bits in ‘number’ but specified range of bits and
    /// returns a number, where selected bits are shifted by
    /// ‘begin_bit’ bits. Assumes that number is ‘size_bits’ wide, so
    /// requesting to filter a bit above this size results in 0
    /// @returns Option with a number where all bits are discarded but
    /// the bits in specified range, these bits are then shifted by
    /// ‘begin_bit’ bits. None denotes that ‘begin_bit‘ is
    /// not lower than ‘end_bit’
    #[inline]
    pub fn extract_bit_range(number: u64, begin_bit: u32, end_bit: u32, size_bits: u32) -> Option<u64> {
        low::extract_bit_range(number, index_from_end!(end_bit, size_bits), index_from_end!(begin_bit, size_bits), size_bits)
    }

    /// Removes all bits in a ‘number’ given an ‘ordered_brop_indices’
    /// iterator of ascending indices. Number is squized after drop of
    /// these bits. Dropping indices above of ‘size_bits’ has no effect.
    /// @returns a tuple with a number with dropped bits, and count of
    /// total bits dropped
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

    /// Split the ‘number‘ by a ‘split_bit’ specified. Assumes than
    /// the number is ‘size_bits’ long, so data above this size is
    /// discarded in the result of the function
    /// @returns a tuple with two parts of the ‘number’, first if from
    /// end of number (from ‘size_bits’) to the split_bit (inclusive),
    /// the second is from ‘split_bit’ exclusive to 0th bit
    #[inline]
    pub fn split_by_bit(number: u64, split_bit: u32, size_bits: u32) -> Option<(u64, u64)> {
        low::split_by_bit(number, index_from_end!(split_bit, size_bits), size_bits)
    }

    /// Rotate the specified range of bits in the ‘number’ by
    /// ‘shift_for’ bits towards higher indices. Rotation
    /// affects only this range.
    /// @return Option with the original number, where specified range
    /// is rotated
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

    /// Rotate the specified range of bits in the ‘number’ by
    /// ‘shift_for’ bits towards lower indices. Rotation
    /// affects only this range.
    /// @returns Option with the original number, where specified range
    /// is rotated
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

    /// Swap bit ranges in the number over ‘split_bit’ (this bit
    /// belongs to higher range). Assumes that number is ‘size_bits’
    /// long, so bits above are not affected
    /// @returns Option with the same number, but two ranges are
    /// swapped. The first is from 0th bit to ‘split_bit’ exclusive,
    /// the second is from ‘split_bit’ to ‘size_bits’ exclusive
    #[inline]
    pub fn swap_ranges(number: u64, split_bit: u32, size_bits: u32) -> Option<u64> {
        low::swap_ranges(number, index_from_end!(split_bit, size_bits), size_bits)
    }    
 }

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    pub fn test_bit_mask_operations() {
        use idx_from_low::*;
        assert_eq!(bit_mask(5), 0b_10_0000, "5th bit = 2**5");
        assert_eq!(bit_mask(0), 0b_00_0001, "1st bit = 1");
        assert_eq!(bit_mask(63),0x80_00_00_00_00_00_00_00, "63th bit = 2**63");
        
        assert_eq!(bit_lower_mask(5),  0b_01_1111, "mask of 5 first bits");
        assert_eq!(bit_lower_mask(0),  0b_00_0000, "mask of 0 first bits");
        assert_eq!(bit_lower_mask(63), 0x7F_FF_FF_FF_FF_FF_FF_FF, "mask of 63 first bits");
        assert_eq!(bit_lower_mask(64), std::u64::MAX, "mask of all bits");

        assert_eq!(bit_range_mask(3, 5), Some(0b_0001_1000), "simple mask 2 bits wide");
        assert_eq!(bit_range_mask(2, 6), Some(0b_0011_1100), "simple mask 4 bits wide");
        assert_eq!(bit_range_mask(0, 1), Some(0b_0000_0001), "lowest bit mask");
        assert_eq!(bit_range_mask(0, 0), Some(0b_0000_0000), "empty mask in the beginning of the number");
        assert_eq!(bit_range_mask(3, 3), Some(0b_0000_0000), "empty mask in the middle of the number");
        assert_eq!(bit_range_mask(64, 64), Some(0), "empty mask in the end of number");
        assert_eq!(bit_range_mask(63, 65), Some(0x80_00_00_00_00_00_00_00), "empty mask exceeding type's size");
        assert_eq!(bit_range_mask(7,3), None, "begin_bit is not lower than end_bit");

        assert_eq!(split_by_bit(0b1010_1111, 4, BITS_IN_INPUT), Some((0b1010, 0b1111)), "normal split");
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
        assert_eq!(
            drop_bits(0b_0101_0111_0110, [1,2].iter().cloned(), BITS_IN_INPUT),
            (0b_0001_0101_1100, 62),
            "Non tricky usage"
        );
        assert_eq!(
            drop_bits(0b_0101_0111_0110, [0].iter().cloned() , BITS_IN_INPUT ),
            (0b_0101_0111_011, 63),
            "Dropping one lowest bit"
        );
        assert_eq!(
            drop_bits(0b_0101_0111_0110, vec![0,1,2,3,5,60,61,62,63,64].iter().cloned(), BITS_IN_INPUT),
            (0b_0000_0010_1011, 55),
            "Dropping many bits from beginning and end"
        );
        assert_eq!(
            drop_bits(std::u64::MAX, (0..100).into_iter(), BITS_IN_INPUT),
            (0, 0),
            "Drop all bits"
        );
        assert_eq!(
            idx_from_high::drop_bits(std::u64::MAX, (0..100).into_iter(), BITS_IN_INPUT),
            (0, 0),
            "Dropp all bits in a high-bit-numeration notation"
        );
    }
}
