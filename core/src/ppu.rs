mod color_palette;
mod control_register;
mod status_register;
mod color;
mod dma;
mod sprite;

use crate::ppu::sprite::Attributes;
use crate::ppu::sprite::Sprite;
use crate::ppu::dma::DmaManager;
use crate::mmu::{ MemWrite, MemRead, IoDevice, Mmu };
use crate::ppu::color::Color;
use crate::ppu::color_palette::{ ColorPalette, MonoColorPalette};
use crate::ppu::control_register::ControlRegister;
use crate::ppu::status_register::StatusRegister;
use crate::ic::Irq;

const DISPLAY_WIDTH: usize = 160;
const DISPLAY_HIGHT: usize = 144;


const H_BLINK_CLOCK_CYCLES: u32 = 87;
const V_BLINK_CLOCK_CYCLES: u32 = 456;
const OAM_CLOCK_CYCLES: u32 = 80;
const VRAM_CLOCK_CYCLES: u32 = 172;

const VRAM_BANK_SIZE: usize = 0x2000;
const VRAM_BANK_COUNT: usize = 0x2;

#[derive(PartialEq, Copy, Clone)]
pub enum GameBoyMode {
    Classic,
    Color,
}

#[derive(PartialEq, Copy, Clone)]
enum BackGroundColorPriority {
    ColorZero,
    HighPriority,
    NormalPriority
}

#[derive(Clone, Copy, PartialEq)]
pub enum PpuMode {
    HorizontalBlanking,
    VerticalBlanking,
    OAM,
    VRAM,
}

impl From<PpuMode> for u8 {
    fn from(mode: PpuMode) -> Self {
        match mode {
            PpuMode::HorizontalBlanking => 0,
            PpuMode::VerticalBlanking => 1,
            PpuMode::OAM => 2,
            PpuMode::VRAM => 3,
        }
    }
}

impl From<u8> for PpuMode {

    fn from(mode: u8) -> Self {
        match mode & 0b11 {
            0 => PpuMode::HorizontalBlanking,
            1 => PpuMode::VerticalBlanking,
            2 => PpuMode::OAM,
            3 => PpuMode::VRAM,
            _ => panic!("bad ppu mode"),
        }
    }
}




struct Ppu {
    clock: u32, // CPU clock cycles stored
    irq: Irq,

    game_boy_mode: GameBoyMode,
    dma_manager: DmaManager,

    selected_vram_bank: usize,
    vram: Vec<Vec<u8>>,

    line: u8,
    line_compare: u8,
    
    x_scroll: u8,
    y_scroll: u8,

    window_y_pos: u8,
    window_x_pos: u8,

    pub status_register: StatusRegister,
    control_register: ControlRegister,

    bg_mono_palette: MonoColorPalette,
    object_mono_palette_0: MonoColorPalette,
    object_mono_palette_1: MonoColorPalette,

    bg_color_palette: ColorPalette,
    object_color_palette: ColorPalette,
}



impl Ppu {


    pub fn cycle(&mut self, mmu: &mut Mmu, clock: u32) {        
        let (clock, dma_in_progress) = self.dma_manager.cycle(self.status_register.mode, mmu, clock);

        self.clock += clock;

        if dma_in_progress {
            return;
        }

        match self.status_register.mode {
            PpuMode::HorizontalBlanking => {
                if self.clock >= H_BLINK_CLOCK_CYCLES {
                    self.clock -= H_BLINK_CLOCK_CYCLES;
                    self.line += 1;

                    // we reached bottom of screen switch to vblank 
                    if self.line > 143 {
                        
                        self.irq.v_blank(true);

                        if self.status_register.v_blank_int {
                            self.irq.lcd_stat(true);
                        }

                        self.status_register.mode = PpuMode::VerticalBlanking;
                    } else {

                        if self.status_register.oam_int {
                            self.irq.lcd_stat(true);
                        }

                        self.status_register.mode = PpuMode::OAM;
                    }
                }    
            },
            PpuMode::VerticalBlanking => {
                if self.clock >= V_BLINK_CLOCK_CYCLES {
                    self.clock -= V_BLINK_CLOCK_CYCLES;
                    self.line += 1;

                    if self.line > 153 {
                        self.line = 0;
                        
                        if self.status_register.oam_int {
                            self.irq.lcd_stat(true);
                        }

                        self.status_register.mode = PpuMode::OAM;   
                    }
                }
            },
            PpuMode::OAM => {
                if self.clock >= OAM_CLOCK_CYCLES {
                    self.clock -= OAM_CLOCK_CYCLES;

                    self.status_register.mode = PpuMode::VRAM;
                }
            },
            PpuMode::VRAM => {
                if self.clock >= VRAM_CLOCK_CYCLES {
                    self.clock -= VRAM_CLOCK_CYCLES;

                    // TODO: call draw


                    if self.status_register.h_blank_int {
                        self.irq.lcd_stat(true);
                    }

                    self.status_register.mode = PpuMode::HorizontalBlanking;
                }
            },
        };

        if self.status_register.coincidence_int && self.line == self.line_compare {
            self.irq.lcd_stat(true);
        }
    }


    fn read_from_vram(&self, bank_index: usize, adder: u16) -> u8 {
        self.vram[bank_index][(adder - 0x8000) as usize]
    }

    fn write_to_vram(&mut self, bank_index: usize, adder: u16, value: u8) {
        self.vram[bank_index][(adder - 0x8000) as usize] = value;
    }


    // returns array of pixels to draw
    fn draw_line(&self, mmu: &Mmu) -> Vec<(u8, u8, u8)> {
        let mut combined_vec: Vec<(u8, u8, u8)> = vec![(0, 0, 0); DISPLAY_WIDTH];

        let bg_vec = self.draw_background_and_window_line();
        let sprite_vec = self.draw_sprites_line(mmu);

        for x in 0..DISPLAY_WIDTH {
            let (bg_color, bg_prio) = bg_vec[x];
            let sprite_option = sprite_vec[x];

            let selected_color = match sprite_option {
                None => bg_color,
                Some((sprit_color, is_blow_bg)) => {
                    if self.control_register.bg_and_win_display {
                        match bg_prio {
                            BackGroundColorPriority::ColorZero => sprit_color,
                            BackGroundColorPriority::NormalPriority => if is_blow_bg { bg_color } else { sprit_color },
                            BackGroundColorPriority::HighPriority => bg_color,
                        }
                    } else {
                        sprit_color
                    }
                }
            };

            combined_vec[x] = selected_color.get_rgb_values();
        }

        combined_vec
    }


    fn draw_background_and_window_line(&self) -> Vec<(Color, BackGroundColorPriority)> {
        let mut line_vector: Vec<(Color, BackGroundColorPriority)> = vec![(Color::new(), BackGroundColorPriority::NormalPriority); DISPLAY_WIDTH];

        let draw_bg = self.game_boy_mode == GameBoyMode::Color || self.control_register.bg_and_win_display;
        let draw_win = self.control_register.window_display_enabled || (self.game_boy_mode != GameBoyMode::Color && self.control_register.bg_and_win_display);

        if !draw_bg && !draw_win {
            return line_vector;
        }

        let win_y = self.line as i32 - self.window_y_pos as i32;

        let bg_y = self.line.wrapping_add(self.y_scroll);

        for x_index in 0..DISPLAY_WIDTH {
            let win_x = x_index as i32 - (self.window_x_pos as i32 - 7);
            let bg_x = self.x_scroll.wrapping_add(x_index as u8);

            let (tile_index_adder_base, tile_y, tile_x) = if draw_win && win_y >= 0 &&  win_x >= 0 {
                (self.control_register.get_window_tile_index_adder(), win_y as u16, win_x as u16)
            } else if draw_bg {
                (self.control_register.get_bg_tile_index_adder(), bg_y as u16, bg_x as u16)
            } else {
                continue;
            };

            let tile_index = self.read_from_vram(0, tile_index_adder_base + ((tile_y / 8) * 32) + (tile_x / 8));

            let maybe_attributes = if self.game_boy_mode == GameBoyMode::Color {
                let attributes_val = self.read_from_vram(1, tile_index_adder_base + ((tile_y / 8) * 32) + (tile_x / 8));
                Some(Attributes::new(attributes_val, &self.bg_color_palette))
            } else {
                None
            };


            let tile_adder = self.control_register.get_bg_tile_adder() + (tile_index as u16 * 16);

            let tile_line_byte = tile_adder + match maybe_attributes {
                Some(attribute) if attribute.y_flip => 14 - (win_y % 8) * 2 ,
                _ => (win_y % 8) * 2,
                
            } as u16;

            
            let (tile_byte_0, tile_byte_1) = match maybe_attributes {
                Some(attribute) => (self.read_from_vram(attribute.vram_bank, tile_line_byte), self.read_from_vram(attribute.vram_bank, tile_line_byte + 1)),
                None => (self.read_from_vram(0, tile_line_byte), self.read_from_vram(0, tile_line_byte + 1)),
            };

            let color_musk =  1 <<  match maybe_attributes {
                Some(attribute) if attribute.x_flip => (x_index % 8),
                _ => 7 - (x_index % 8),
            };

            let color_index = 
                if color_musk & tile_byte_0 != 0 { 1 } else { 0 } | 
                if color_musk & tile_byte_1 != 0 { 2 } else { 0 };


            let atter_prio = match maybe_attributes {
                Some(attribute) if attribute.priority => true,
                _ => false,
            }; 

            let bg_prio = if color_index == 0 { BackGroundColorPriority::ColorZero } 
                else if atter_prio { BackGroundColorPriority::HighPriority } 
                else { BackGroundColorPriority::NormalPriority };

            if self.game_boy_mode == GameBoyMode::Color {
                let color = self.bg_color_palette.get_color(color_index);

                line_vector[(x_index) as usize] = (color, bg_prio)
            } else {
                let color = self.bg_mono_palette.get_color(color_index);

                line_vector[(x_index) as usize] = (color, bg_prio)
            }
        }

        line_vector
    }

    // color and blow background == true 
    fn draw_sprites_line(&self, mmu: &Mmu) -> Vec<Option<(Color, bool)>> {
        let mut line_vector: Vec<Option<(Color, bool)>> = vec![None; DISPLAY_WIDTH];

        if !self.control_register.sprit_display_enabled { return line_vector }

        let sprite_hight: u16 = if self.control_register.sprite_size { 16 } else { 8 };
        let sprite_width: u16 = 8;

        let line = self.line as i32;

        // let mut sprite_counter = 0;

        for sprite_index in 0..40 {

            // if sprite_counter > 10 {
            //     break;
            // }

            let sprite = Sprite::new(sprite_index, self.control_register.sprite_size, mmu, &self.object_color_palette);

            // sprite out of screen
            if line < sprite.y || line >= sprite.y + (sprite_hight as i32) { continue }
            if sprite.x < -7 || sprite.x >= DISPLAY_WIDTH as i32 + 8 { continue }

            let tile_y: u16 = if sprite.attributes.y_flip {
                sprite_hight - 1 - ((line - sprite.y) as u16)
            } else {
                (line - sprite.y) as u16
            };

            // Every tile is 16 bytes of mem and eve1ry line is 2 bytes aka 8 bytes
            let tile_adder = 0x8000u16 + (sprite.tile_index * 16) + (tile_y * 2);

            let (tile_byte_0, tile_byte_1) = if self.game_boy_mode == GameBoyMode::Color {
                (self.read_from_vram(sprite.attributes.vram_bank, tile_adder),
                 self.read_from_vram(sprite.attributes.vram_bank, tile_adder + 1))
            } else {
                (self.read_from_vram(0, tile_adder), self.read_from_vram(0, tile_adder + 1))
            };


            for x_index in 0..sprite_width {
                if sprite.x + (x_index as i32) < 0 || sprite.x >= (DISPLAY_WIDTH as i32) { continue }

                let color_musk = 1 << (if sprite.attributes.x_flip { x_index } else { 7 - x_index });

                let color_index = 
                    if color_musk & tile_byte_0 != 0 { 1 } else { 0 } | 
                    if color_musk & tile_byte_1 != 0 { 2 } else { 0 };

                // 0 is transparent in sprites
                if color_index == 0 { continue }

                if self.game_boy_mode == GameBoyMode::Color {
                    let color = self.object_color_palette.get_color(color_index);

                    line_vector[(x_index as i32 + sprite.x) as usize] = Some((color, sprite.attributes.priority))
                } else {
                    let color = 
                        if sprite.attributes.palette_number == 0 { self.object_mono_palette_0.get_color(color_index) }
                        else { self.object_mono_palette_1.get_color(color_index) };

                    line_vector[(x_index as i32 + sprite.x) as usize] = Some((color, sprite.attributes.priority))
                }

                // sprite_counter += 1;
            }
        }
        

        line_vector
    }
}


impl IoDevice for Ppu {
    fn read_byte(&mut self, _mmu: &Mmu, adder: u16) -> MemRead { 
        match adder {
            0x8000 ..= 0x9FFF => MemRead::Read(self.read_from_vram(self.selected_vram_bank, adder)),

            0xFF40 => MemRead::Read(self.control_register.get()),
            0xFF41 => MemRead::Read(self.status_register.get()),
            0xFF42 => MemRead::Read(self.y_scroll),
            0xFF43 => MemRead::Read(self.x_scroll),
            0xFF44 => MemRead::Read(self.line),
            0xFF45 => MemRead::Read(self.line_compare),
            0xFF46 => MemRead::Read(self.dma_manager.read_oam()),
            0xFF47 => MemRead::Read(self.bg_mono_palette.read()),
            0xFF48 => MemRead::Read(self.object_mono_palette_0.read()),
            0xFF49 => MemRead::Read(self.object_mono_palette_1.read()),

            0xFF4A => MemRead::Read(self.window_y_pos),
            0xFF4B => MemRead::Read(self.window_x_pos),

            0xFF4F => MemRead::Read(self.selected_vram_bank as u8),

            0xFF51 ..= 0xFF55 => MemRead::Read(self.dma_manager.read_vram_dma(adder)),

            0xFF68 => MemRead::Read(self.bg_color_palette.read_index_reg()),
            0xFF69 => MemRead::Read(self.bg_color_palette.read_data_reg()),

            0xFF6A => MemRead::Read(self.object_color_palette.read_index_reg()),
            0xFF6B => MemRead::Read(self.object_color_palette.read_data_reg()),
            
            _ => MemRead::Ignore,
        }
    }

    fn write_byte(&mut self, _mmu: &Mmu, adder: u16, val: u8) -> MemWrite { 
        match adder {
            0x8000 ..= 0x9FFF => { (self.write_to_vram(self.selected_vram_bank, adder, val)); MemWrite::Write },

            0xFF40 => { self.control_register.set(val); MemWrite::Write },
            0xFF41 => { self.status_register.set(val); MemWrite::Write },
            0xFF42 => { self.y_scroll = val; MemWrite::Write },
            0xFF43 => { self.x_scroll = val; MemWrite::Write },
            0xFF44 => MemWrite::Write,
            0xFF45 => { self.line_compare = val; MemWrite::Write },

            0xFF46 => { self.dma_manager.write_oam(val); MemWrite::Write },
            0xFF47 => { self.bg_mono_palette.write(val); MemWrite::Write },
            0xFF48 => { self.object_mono_palette_0.write(val); MemWrite::Write },
            0xFF49 => { self.object_mono_palette_1.write(val); MemWrite::Write },

            0xFF4A => { self.window_y_pos = val; MemWrite::Write },
            0xFF4B => { self.window_x_pos = val; MemWrite::Write },

            0xFF4F => { self.selected_vram_bank = (val & 0x1) as usize; MemWrite::Write },

            0xFF51 ..= 0xFF55 => { self.dma_manager.write_vram_dma(adder, val); MemWrite::Write },

            0xFF68 => { self.bg_color_palette.write_index_reg(val); MemWrite::Write },
            0xFF69 => { self.bg_color_palette.write_data_reg(val); MemWrite::Write },

            0xFF6A => { self.object_color_palette.write_index_reg(val); MemWrite::Write },
            0xFF6B => { self.object_color_palette.write_data_reg(val); MemWrite::Write },
            
            _ => MemWrite::Ignore,
        }
    }
}