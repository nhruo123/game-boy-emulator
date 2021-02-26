mod color_palette;
mod control_register;
mod status_register;
mod color;
mod dma;

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


    pub fn read_from_vram(&mut self, adder: u16) -> u8 {
        self.vram[self.selected_vram_bank][(adder - 0x8000) as usize]
    }

    pub fn write_to_vram(&mut self, adder: u16, value: u8) {
        self.vram[self.selected_vram_bank][(adder - 0x8000) as usize] = value;
    }
}


impl IoDevice for Ppu {
    fn read_byte(&mut self, _mmu: &Mmu, adder: u16) -> MemRead { 
        match adder {
            0x8000 ..= 0x9FFF => MemRead::Read(self.read_from_vram(adder)),

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
            0x8000 ..= 0x9FFF => { (self.write_to_vram(adder, val)); MemWrite::Write },

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