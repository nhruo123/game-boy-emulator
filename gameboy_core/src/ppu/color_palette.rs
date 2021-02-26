use crate::ppu::color::Color;

pub struct MonoColorPalette {
    colors: (Color, Color, Color, Color)
}

impl MonoColorPalette {


    pub fn new() -> MonoColorPalette {
        MonoColorPalette {
            colors: (Color::White, Color::LightGray, Color::DarkGray, Color::Black)
        }
    }

    pub fn read(&self) -> u8 {
        self.colors.0.monochrome_color_to_u8() | 
        (self.colors.1.monochrome_color_to_u8() << 2) | 
        (self.colors.2.monochrome_color_to_u8() << 4) | 
        (self.colors.3.monochrome_color_to_u8() << 6)    
    }

    pub fn write(&mut self, val: u8) {
        self.colors.0 = Color::u8_to_monochrome(val & 0x3);
        self.colors.1 = Color::u8_to_monochrome((val << 2) & 0x3);
        self.colors.2 = Color::u8_to_monochrome((val << 4) & 0x3);
        self.colors.3 = Color::u8_to_monochrome((val << 6) & 0x3);
    }

    pub fn get_color(&self, index: usize) -> Color {
        match index {
            0 => self.colors.0,
            1 => self.colors.1,
            2 => self.colors.2,
            _ => self.colors.3,
        }
    }
}
pub struct ColorPalette {
    colors: Vec<Vec<Color>>,
    color_index: usize,
    auto_inc: bool,
}

impl ColorPalette {
    pub fn new() -> ColorPalette {
        ColorPalette {
            auto_inc: false,
            color_index: 0,
            colors: vec![vec![Color::new(); 4]; 8]
        }
    }


    pub fn read_index_reg(&self) -> u8 {
        self.color_index as u8 | if self.auto_inc { 0x80 } else { 0 }
    }

    
    pub fn write_index_reg(&mut self, val: u8) {
        self.auto_inc = (val & 0x80) != 0;
        self.color_index = (val & 0b111111) as usize;
    }



    pub fn read_data_reg(&self) -> u8 {
        let palette_index = self.color_index / 8;
        let color_index = self.color_index % 8;

        if (color_index % 2) == 0 {
            self.colors[palette_index][color_index / 2].get_low()
        } else {
            self.colors[palette_index][color_index / 2].get_high()
        }
    }

    pub fn write_data_reg(&mut self, val: u8) {
        let palette_index = self.color_index / 8;
        let color_index = self.color_index % 8;

        if (color_index % 2) == 0 {
            self.colors[palette_index][color_index / 2].set_low(val);
        } else {
            self.colors[palette_index][color_index / 2].set_high(val);
        }

        if self.auto_inc {
            self.color_index = (self.color_index + 1) % 0x40;
        }
    }

    pub fn get_palette(&self, index: usize) -> &Vec<Color> {
        &self.colors[index % 8]
    }

    pub fn get_color(&self, index: usize) -> Color {
        self.colors[self.color_index][index]
    }
}