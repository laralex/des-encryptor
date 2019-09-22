pub fn encrypt_block(block_64: &mut [u8], key: u64) {
    //const BLOCK_SIZE: usize = 64;
    //assert_eq!(block_64b.len(), BLOCK_SIZE);
    const ROUNDS_NUMBER: usize = 16;
    let ip = PermutationTable::new_ip();
    ip.forward.permutate(block_64b);
    for _ in 0..ROUNDS_NUMBER {
        let round_key = get_round_key(block_64b, key)
        des_encrypt_round(block_64b, round_key);
    }
    ip.inverse.permutate(block_64b);
}
