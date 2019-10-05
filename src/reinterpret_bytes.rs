#[derive(Copy, Clone)]
pub enum Endianess {
    Big,
    Little,
}

impl Default for Endianess {
    fn default() -> Self { return Endianess::Big; }
}

pub fn as_slice_of<'a, DST>(data: &'a mut [u8]) -> &'a mut [DST] {
    use std::mem;
    let bytes_in_data = mem::size_of_val(data);
    let bytes_in_dst: usize = mem::size_of::<DST>();
    assert!(bytes_in_data % bytes_in_dst == 0); // can be evenly split
    let dst_len = bytes_in_data / bytes_in_dst;
    unsafe {
        std::slice::from_raw_parts_mut(data as *mut [u8] as *mut DST, dst_len)
        //mem::transmute::<&'a mut [u8], &'a mut [DST]>(data)
    }
}
