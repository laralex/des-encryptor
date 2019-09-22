const BITS_IN_INPUT: u8 = 64;

#[inline]
pub fn rotate_left(number: u64, mut begin_bit: u8, mut end_bit: u8, mut shift_for: u8) -> Option<u64> {
    if end_bit == 0 || begin_bit >= end_bit {
        return None;
    } else if end_bit > 64 {
        end_bit = 64;
    }
    shift_for %= end_bit - begin_bit;
    let cutoff_offset = end_bit - shift_for;
    let cutoff_mask = bit_mask_between(cutoff_offset, end_bit)?;
    let remain_mask = bit_mask_between(begin_bit, cutoff_offset)?;
    let outer_mask = (bit_mask_between(end_bit, BITS_IN_INPUT),
                      bit_mask_between(0, begin_bit));
    let cutoff_rotated = (number & cutoff_mask).overflowing_shr((cutoff_offset - begin_bit) as u32).0;
    let remain_shifted = (number & remain_mask).overflowing_shl(shift_for as u32).0;
    let outer = match outer_mask {
        (Some(mask_upper), Some(mask_lower)) => number & (mask_upper | mask_lower),
        _ => 0,
    };
    Some( remain_shifted + cutoff_rotated + outer)
} 

#[inline]
pub fn rotate_right(number: u64, mut begin_bit: u8, mut end_bit: u8, mut shift_for: u8) -> Option<u64> {
    if end_bit == 0 || begin_bit >= end_bit {
        return None;
    } else if end_bit > 64 {
        end_bit = 64;
    }
    shift_for %= end_bit - begin_bit;
    let cutoff_offset = begin_bit + shift_for;
    let cutoff_mask = bit_mask_between(begin_bit, cutoff_offset)?;
    let remain_mask = bit_mask_between(cutoff_offset, end_bit)?;
    let outer_mask = (bit_mask_between(end_bit, BITS_IN_INPUT),
                      bit_mask_between(0, begin_bit));
    let cutoff_rotated = (number & cutoff_mask).overflowing_shl((end_bit - cutoff_offset) as u32).0;
    let remain_shifted = (number & remain_mask).overflowing_shr(shift_for as u32).0;
    let outer = match outer_mask {
        (Some(mask_upper), Some(mask_lower)) => number & (mask_upper | mask_lower),
        _ => 0,
    };
    Some( remain_shifted + cutoff_rotated + outer)
}

#[inline]
pub fn split_by_bit(number: u64, split_bit: u8) -> Option<(u64, u64)> {
    Some(
        ((number & bit_mask_between(split_bit, BITS_IN_INPUT)?) >> split_bit,
         number & bit_mask_between(0, split_bit)?)
    )
}

#[inline]
pub fn bit_mask_single(bit: u8) -> u64 {
    1u64.overflowing_shl(bit as u32).0
}

#[inline]
pub fn bit_mask_lower(bit: u8) -> u64{
    if bit >= 64 {
        return std::u64::MAX;
    }
    bit_mask_single(bit) - 1
}

#[inline]
pub fn bit_mask_between(begin_bit: u8, end_bit: u8) -> Option<u64> {
    if end_bit < begin_bit { return None; }
    if begin_bit > 63 || begin_bit == end_bit { return Some(0); }
    Some(bit_mask_lower(end_bit) - bit_mask_lower(begin_bit))
}

#[inline]
pub fn get_bit(number: u64, bit: u8) -> u64 {
    (number & bit_mask_single(bit)) >> bit
}

pub fn drop_bits(number: u64, drop_indices: &Vec<u8>) -> (u64, u8){
    if drop_indices.len() == 0 {
        return (number, BITS_IN_INPUT);
    }
    let total_drop = drop_indices.len();
    if total_drop >= 64 {
        return (0, 0);
    }
    let mut prev_drop_bit = *drop_indices.first().unwrap() + 1;
    let lower_mask = bit_mask_between(0, prev_drop_bit - 1).unwrap();
    let higher_mask = bit_mask_between(*drop_indices.last().unwrap() + 1, BITS_IN_INPUT).unwrap();
    let dropped_intrinsics = drop_indices.into_iter()
        .enumerate()
        .skip(1)
        .fold(0_u64, |acc, (drop_count, drop_bit)| {
            let intermediate_mask = bit_mask_between(prev_drop_bit, *drop_bit);
            if intermediate_mask == None {
                return acc;
            }
            let intermediate_mask = intermediate_mask.unwrap();
            prev_drop_bit = drop_bit + 1;
            acc + ((intermediate_mask & number) >> (drop_count as u32))    
        });
    let shrinked_number = (lower_mask & number) + dropped_intrinsics
        + ((higher_mask & number) >> total_drop);
    (shrinked_number, BITS_IN_INPUT - total_drop as u8)
}

// pub struct BitIteratingBytes<'a> ( &'a [u8] );
// impl BitIteratingBytes {
//     pub fn from<'a>(bytes: &'a [u8]) -> Self {
//         Self(bytes.clone())
//     }
// }

#[test]
pub fn test_bit_mask_operations() {
    assert_eq!(bit_mask_single(5), 0b_10_0000, "5th bit = 2**5");
    assert_eq!(bit_mask_single(0), 0b_00_0001, "1st bit = 1");
    assert_eq!(bit_mask_single(63),0x80_00_00_00_00_00_00_00, "63th bit = 2**63");
    assert_eq!(bit_mask_single(65), 0b_10, "overflowing with type rotation");
    
    assert_eq!(bit_mask_lower(5),  0b_01_1111, "mask of 5 first bits");
    assert_eq!(bit_mask_lower(0),  0b_00_0000, "mask of 0 first bits");
    assert_eq!(bit_mask_lower(63), 0x7F_FF_FF_FF_FF_FF_FF_FF, "mask of 63 first bits");
    assert_eq!(bit_mask_lower(64), std::u64::MAX, "mask of all bits");

    assert_eq!(bit_mask_between(3, 5), Some(0b_0001_1000));
    assert_eq!(bit_mask_between(2, 6), Some(0b_0011_1100));
    assert_eq!(bit_mask_between(0, 1), Some(0b_0000_0001));
    assert_eq!(bit_mask_between(0, 0), Some(0b_0000_0000));
    assert_eq!(bit_mask_between(3, 3), Some(0b_0000_0000));
    assert_eq!(bit_mask_between(3, 3), Some(0b_0000_0000));
    assert_eq!(bit_mask_between(64, 64), Some(0));
    assert_eq!(bit_mask_between(63, 65), Some(0x80_00_00_00_00_00_00_00));
    assert_eq!(bit_mask_between(7,3), None);

    assert_eq!(split_by_bit(0b1010_1111, 4), Some((0b1010, 0b1111)));
}

#[test]
pub fn test_bit_rotate(){
    assert_eq!(rotate_left(1, 0, 23, 3), Some(0b1000), "normal shift");
    assert_eq!(rotate_left(0b10101, 0, 2, 3), Some(0b10110),
               "only 2 value bits, others are stationary");
    assert_eq!(rotate_left(0b00101, 0, 5, 3), Some(0b01001), "rotation");
    assert_eq!(rotate_left(0b00101, 0, 5, 0), Some(0b00101), "no shift at all");
    assert_eq!(rotate_left(0b00101, 0, 5, 5), Some(0b00101), "identity shift");
    assert_eq!(rotate_left(0b00101, 0, 0, 5), None, "end bit is zero, then last bit is what?");
    let big_val = std::u64::MAX - 7;
    assert_eq!(rotate_left(big_val, 0, 64, 5), Some(big_val.rotate_left(5)),
               "shifts over type boundaries");
    assert_eq!(rotate_left(big_val, 0, 65, 5), Some(big_val.rotate_left(5)),
               "end bit is over type boundaries");


    assert_eq!(rotate_left(0b00100000, 5, 8, 4), Some(0b01000000), "non-zero begin bit");
    assert_eq!(rotate_left(0b110101, 3, 5, 2), Some(0b110101),
               "non-zero begin bit, with outer contents");
}

#[test]
pub fn test_drop_bits() {
    assert_eq!(drop_bits(0b_0101_0111_0110, &vec![1,2]), (0b_0001_0101_1100, 62));
    assert_eq!(drop_bits(0b_0101_0111_0110, &vec![0,1,2,3,5,60,61,62,63,64]), (0b_0000_0010_1011, 55));
    assert_eq!(drop_bits(std::u64::MAX, &(0..100).rev().collect::<Vec<_>>()), (0, 0));
}

// pub  test_bit_split() {
//     //assert_eq!()
// }
