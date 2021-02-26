#[derive(Clone, Copy)]
pub enum Color {
    White,
    LightGray,
    DarkGray,
    Black,
    Rgb(u8,u8,u8)
}

impl Color {


    pub fn u8_to_monochrome(value: u8) -> Color {
        match value & 0x3 {
            0 => Color::White,
            1 => Color::LightGray,
            2 => Color::DarkGray,
            3 => Color::Black,
            _ => unreachable!(),
        }
    }

    pub fn monochrome_color_to_u8(&self) -> u8 {
        match *self {
            Color::White => 0,
            Color::LightGray => 1,
            Color::DarkGray => 2,
            Color::Black => 3,
            _ => unreachable!("We can only call this function on a monochrome color"),
        }
    }

    pub fn new() -> Color {
        Color::Rgb(0xFF, 0xFF, 0xFF)
    }

    pub fn set_low(&mut self, low: u8) {
        match *self {
            Color::Rgb(_, g, b) => {
                let red = low & 0x1F;
                let half_green = (low >> 5) | (g & !0b111);
                *self = Color::Rgb(red, half_green, b);
            },
            _ => unreachable!("We will never set a gray scale color"),
        }
    }

    pub fn set_high(&mut self, high: u8) {
        match *self {
            Color::Rgb(r, g, _) => {
                
                let half_green = (g & !0x18) | ((high & 0b11) << 3);

                let blue = (high >> 2) & 0x1F;

                *self = Color::Rgb(r, half_green, blue);
            },
            _ => unreachable!("We will never set a gray scale color"),
        }
    }


    pub fn get_low(&self) -> u8 {
        match *self {
            Color::Rgb(r, g, _) => (r & 0x1f) | (g & 0x7) << 5,
            _ => unreachable!(),
        }
    }

    pub fn get_high(&self) -> u8 {
        match *self {
            Color::Rgb(_, g, b) => ((g >> 3) & 0x3) | (b & 0x1f) << 2,
            _ => unreachable!(),
        }
    }


    pub fn get_rgb_values(&self) -> (u8, u8, u8) {
        match *self {
            Color::Rgb(r,g,b) => (r,g,b),
            Color::Black => (0,0,0),
            Color::LightGray => (0xD3, 0xD3, 0xD3),
            Color::DarkGray => (0xA9, 0xA9, 0xA9),
            Color::White => (0xFF, 0xFF, 0xFF),
        }
    }
}