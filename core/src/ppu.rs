mod color_palette;
mod control_register;
mod status_register;
mod color;

use crate::mmu::{ MemWrite, MemRead, IoDevice, Mmu };
use crate::ppu::color::Color;
use crate::ppu::color_palette::ColorPalette;
use crate::ppu::control_register::ControlRegister;
use crate::ppu::status_register::StatusRegister;
use crate::ic::Irq;

const DISPLAY_WIDTH: usize = 160;
const DISPLAY_HIGHT: usize = 144;


const H_BLINK_CLOCK_CYCLES: u32 = 87;
const V_BLINK_CLOCK_CYCLES: u32 = 456;
const OAM_CLOCK_CYCLES: u32 = 80;
const VRAM_CLOCK_CYCLES: u32 = 172;


#[derive(Clone, Copy)]
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

fn mono_palette_to_u8(p: Vec<Color>) -> u8 {
    // TODO: make sure palette is 4 colors

    p[0].monochrome_color_to_u8() | 
    (p[1].monochrome_color_to_u8() << 2) | 
    (p[2].monochrome_color_to_u8() << 4) | 
    (p[3].monochrome_color_to_u8() << 6)
}


struct Ppu {
    clock: u32, // CPU clock cycles stored
    irq: Irq,

    line: u8,
    line_compare: u8,
    
    x_scroll: u8,
    y_scroll: u8,

    window_y_pos: u8,
    window_x_pos: u8,

    status_register: StatusRegister,
    control_register: ControlRegister,

    bg_mono_palette: Vec<Color>,
    object_mono_palette_0: Vec<Color>,
    object_mono_palette_1: Vec<Color>,

    bg_color_palette: ColorPalette,
    object_color_palette: ColorPalette,
}



impl Ppu {


    pub fn cycle(&mut self, mmu: &mut Mmu, clock: u32) {
        self.clock += clock;

        // TODO: deal with dma requests

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
}


impl IoDevice for Ppu {
    fn read_byte(&mut self, _mmu: &Mmu, adder: u16) -> MemRead { 
        match adder {
            0xFF40 => MemRead::Read(self.control_register.get()),
            0xFF41 => MemRead::Read(self.status_register.get()),
            0xFF42 => MemRead::Read(self.y_scroll),
            0xFF43 => MemRead::Read(self.x_scroll),
            0xFF44 => MemRead::Read(self.line),
            0xFF45 => MemRead::Read(self.line_compare),

            0xFF47 => MemRead::Read(mono_palette_to_u8(self.bg_mono_palette.clone())),
            0xFF48 => MemRead::Read(mono_palette_to_u8(self.object_mono_palette_0.clone())),
            0xFF49 => MemRead::Read(mono_palette_to_u8(self.object_mono_palette_1.clone())),

            0xFF4A => MemRead::Read(self.window_y_pos),
            0xFF4B => MemRead::Read(self.window_x_pos),
            
            _ => MemRead::Ignore,
        }
    }
    fn write_byte(&mut self, _: &Mmu, _: u16, _: u8) -> MemWrite { todo!() }
}