
// high then low
// pub fn split_u16(val: u16) -> (u8, u8) {
//     ((val >> 8) as u8 , (val & 0xFF) as u8)
// }

pub fn get_u16_low(val: u16) -> u8 {
    val as u8
}

pub fn get_u16_high(val: u16) -> u8 {
    (val >> 8) as u8 
}

pub fn build_u16(high: u8, low: u8) -> u16 {
    (low as u16) + ((high as u16) << 8)
}

