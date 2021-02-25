use crate::ppu::color::Color;

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
}