/// Gets a u64 where the lowest `bit_count` bits are ones and the rest are zeroes.
pub const fn low_bit_mask_u64(bit_count: u32) -> u64 {
    let (n, overflowed) = 1u64.overflowing_shl(bit_count);
    n.wrapping_sub(1).wrapping_sub(overflowed as u64)
}
