use crate::ppu::Ppu;
use crate::ppu::MonoColorPalette;
use crate::ppu::Mmu;
use crate::ppu::ColorPalette;
use crate::ppu::Color;

const OAM_BASE: u16 = 0xFE00;

#[derive(Copy, Clone)]
pub struct Attributes<'a> {
    pub palette: &'a [Color],
    pub vram_bank: usize,
    pub palette_number: usize,
    pub x_flip: bool,
    pub y_flip: bool,
    pub priority: bool,
}

impl<'a> Attributes<'a> {
    pub fn new(val: u8, color_palette: &'a ColorPalette) -> Attributes<'a> {
        Attributes {
            palette: color_palette.get_palette((val & 0x7) as usize),
            vram_bank: ((val >> 3) & 0x1) as usize,
            palette_number: (val & 0x10) as usize,
            x_flip: (val & 0x20) != 0,
            y_flip: (val & 0x40) != 0,
            priority: (val & 0x80) != 0,
        }
    }

    pub fn new_normal_gb(mono_color_palette: &'a MonoColorPalette) ->  Attributes<'a> {
        Attributes {
            palette: mono_color_palette.get_color_array(),
            vram_bank: 0,
            palette_number: 0,
            x_flip: false,
            y_flip: false,
            priority: false,
        }
    }

    pub fn new_new_normal_gb_sprite(val: u8, mono_cp_0: &'a MonoColorPalette, mono_cp_1: &'a MonoColorPalette)  ->  Attributes<'a> {
        let mono_cp = if (val & 0x10) == 0 {
            mono_cp_0
        } else {
            mono_cp_1
        };

        Attributes {
            palette: mono_cp.get_color_array(),
            vram_bank: 0,
            palette_number: (val & 0x10) as usize,
            x_flip: (val & 0x20) != 0,
            y_flip: (val & 0x40) != 0,
            priority: (val & 0x80) != 0,
        }
    }
}


pub struct Sprite<'a> {
    pub y: i32,
    pub x: i32,
    pub tile_index: u16,
    pub attributes: Attributes<'a>
}

impl<'a> Sprite<'a> {
    pub fn new(index:u16, big_sprites: bool, ppu: &Ppu, color_palette: &'a ColorPalette) -> Sprite<'a> {
        Sprite {
            y: ppu.read_oma(OAM_BASE + (index * 4)) as u16 as i32 - 16,
            x: ppu.read_oma(OAM_BASE + 1 + (index * 4)) as u16 as i32 - 8,
            tile_index: (ppu.read_oma(OAM_BASE + 2 + (index * 4)) & if big_sprites { 0xFE } else { 0xFF }) as u16,
            attributes: Attributes::new(ppu.read_oma(OAM_BASE + 3 + (index * 4)), color_palette),
        }
    }

    pub fn new_normal_gb(index:u16, big_sprites: bool, ppu: &Ppu, mono_cp_0: &'a MonoColorPalette, mono_cp_1: &'a MonoColorPalette) -> Sprite<'a> {
        Sprite {
            y: ppu.read_oma(OAM_BASE + (index * 4)) as u16 as i32 - 16,
            x: ppu.read_oma(OAM_BASE + 1 + (index * 4)) as u16 as i32 - 8,
            tile_index: (ppu.read_oma(OAM_BASE + 2 + (index * 4)) & if big_sprites { 0xFE } else { 0xFF }) as u16,
            attributes: Attributes::new_new_normal_gb_sprite(ppu.read_oma(OAM_BASE + 3 + (index * 4)), mono_cp_0, mono_cp_1),
        }
    }
}