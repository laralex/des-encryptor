use super::{encrypt_round, encrypt_last_round, KeyScheduler, Key, PermutationTable};

const PERMUTATION_INPUT_SIZE: u32 = 64;
const BIT_COUNT_FROM: u32 = 1;

lazy_static! {
    static ref INITIAL_PERMUTATION: PermutationTable
    = PermutationTable::new(vec![
        58, 50, 42, 34, 26, 18, 10, 2,
        60, 52, 44, 36, 28, 20, 12, 4,
        62, 54, 46, 38, 30, 22, 14, 6,
        64, 56, 48, 40, 32, 24, 16, 8,
        57, 49, 41, 33, 25, 17,  9, 1,
        59, 51, 43, 35, 27, 19, 11, 3,
        61, 53, 45, 37, 29, 21, 13, 5,
        63, 55, 47, 39, 31, 23, 15, 7,
    ], BIT_COUNT_FROM, PERMUTATION_INPUT_SIZE);
}

lazy_static! {
    static ref REVERSE_PERMUTATION: PermutationTable
    = PermutationTable::new(vec![
        40, 8, 48, 16, 56, 24, 64, 32,
        39, 7, 47, 15, 55, 23, 63, 31,
        38, 6, 46, 14, 54, 22, 62, 30,
        37, 5, 45, 13, 53, 21, 61, 29,
        36, 4, 44, 12, 52, 20, 60, 28,
        35, 3, 43, 11, 51, 19, 59, 27,
        34, 2, 42, 10, 50, 18, 58, 26,
        33, 1, 41,  9, 49, 17, 57, 25,
    ], BIT_COUNT_FROM, PERMUTATION_INPUT_SIZE);
}

// TODO: accept arbitrary u8 slice, not strict u64
pub fn encrypt_block(data: u64, key: u64) -> u64 {
    act_on_block(
        data,
        KeyScheduler::new_encrypting(key),
    )
}

pub fn decrypt_block(data: u64, key: u64) -> u64 {
    act_on_block(
        data,
        KeyScheduler::new_decrypting(key),
    )
}

#[inline]
fn act_on_block<I>(mut data: u64, mut key_iterator: I) -> u64
where I: Iterator<Item=Key>{
    const ROUNDS_NUMBER: usize = 16;
    // let action = match do_encrypt {
    //     true => encrypt_round,
    //     false => decrypt_round,
    // };
        // println!("{:#018x}", data);
    data = INITIAL_PERMUTATION.apply(data);
    for (_, round_key) in (0..ROUNDS_NUMBER-1).zip(&mut key_iterator) {
        data = encrypt_round(data, round_key);
        // println!("{:#018x}", data);
    }
    data = encrypt_last_round(data, key_iterator.next().unwrap());
    // println!("{:#018x} done", data);
    REVERSE_PERMUTATION.apply(data)
}

#[test]
fn test_encryption_of_block() {
    assert_eq!(
        encrypt_block(0x0123456789ABCDEF, 0x133457799BBCDFF1),
        0x85E813540F0AB405
    );
    assert_eq!(
        encrypt_block(0x8787878787878787, 0x0e329232ea6d0d73),
        0x0000000000000000
    );

    assert_eq!(
        encrypt_block(0x596f7572206c6970, 0x0e329232ea6d0d73),
        0xc0999fdde378d7ed
    );
    assert_eq!(
        encrypt_block(0x123456ABCD132536, 0xAABB09182736CCDD),
        0xC0B7A8D05F3A829C
    );
}
#[test]
fn test_decryption_of_block() {
    assert_eq!(
        decrypt_block(0x85E813540F0AB405, 0x133457799BBCDFF1),
        0x0123456789ABCDEF
    );
    assert_eq!(
        decrypt_block(0x0000000000000000, 0x0e329232ea6d0d73),
        0x8787878787878787
    );
    assert_eq!(
        decrypt_block(0xc0999fdde378d7ed, 0x0e329232ea6d0d73),
        0x596f7572206c6970
    );
    assert_eq!(
        decrypt_block(0xC0B7A8D05F3A829C, 0xAABB09182736CCDD),
        0x123456ABCD132536
    );
}
