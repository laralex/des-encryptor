#[derive(Copy, Clone)]

/// Represents possible Endianess of some computer system
/// Options are standard:
/// Big Endian - bytes of an object (e.g. integer type) are stored in
/// order from the most valuable to the least
/// Little Endian - bytes are stored in order from the least valuable
/// to the most valuable
pub enum Endianess {
    Big,
    Little,
}

impl Default for Endianess {
    fn default() -> Self { return Endianess::Big; }
}

/// Takes a mutable reference to byte array and casts it to an array
/// of other type. Preserves endianess of the target machine
/// @returns a mutable reference to the casted array 
pub fn as_slice_of<DST>(data: &mut [u8]) -> &mut [DST] {
    use std::mem;
    let bytes_in_data = mem::size_of_val(data);
    let bytes_in_dst: usize = mem::size_of::<DST>();
    assert!(bytes_in_data % bytes_in_dst == 0); // can be evenly split
    let dst_len = bytes_in_data / bytes_in_dst;
    unsafe {
        std::slice::from_raw_parts_mut(data as *mut [u8] as *mut DST, dst_len)
    }
}

