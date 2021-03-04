
#[inline]
pub fn get_u16_low(val: u16) -> u8 {
    val as u8
}

#[inline]
pub fn get_u16_high(val: u16) -> u8 {
    (val >> 8) as u8 
}

#[inline]
pub fn build_u16(high: u8, low: u8) -> u16 {
    (low as u16) + ((high as u16) << 8)
}

pub trait UIntExt {
    fn test_add_carry_bit(bit: usize, a: Self, b: Self) -> bool;
}

impl UIntExt for u16 {
    #[inline(always)]
    fn test_add_carry_bit(bit: usize, a: Self, b: Self) -> bool {
        let mask: Self = 1 << bit;
        let mask: Self = mask | mask.wrapping_sub(1);

        (a & mask) + (b & mask) > mask
    }
}

impl UIntExt for u8 {
    #[inline(always)]
    fn test_add_carry_bit(bit: usize, a: Self, b: Self) -> bool {
        let mask: Self = 1 << bit;
        let mask: Self = mask | mask.wrapping_sub(1);

        (a & mask) + (b & mask) > mask
    }
}