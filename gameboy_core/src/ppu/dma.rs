
use crate::ppu::Mmu;
use crate::ppu::{ Ppu, PpuMode };
use crate::utils::{ get_u16_low, get_u16_high, build_u16};

const OAM_DMA_TIME: u32 = 640;
const G_DMA_TIME: u32 = 640;


#[derive(Clone, Copy, PartialEq)]
pub enum DmaType {
    Oma,
    Hdma,
    Gdma,
    None,
}


pub struct DmaManager {
    dma_type: DmaType,
    oma_base_adder: u8,

    vram_dma_source: u16,
    vram_dma_target: u16,
    vram_dma_len: u8,
}


impl DmaManager {
    pub fn new() -> DmaManager {
        DmaManager {
            dma_type: DmaType::None,
            oma_base_adder: 0,
            vram_dma_len: 0,
            vram_dma_source: 0,
            vram_dma_target: 0, 
        }
    }

    // returns remaining clocks and if operation needs more time
    pub fn cycle(&mut self, ppu_mode: PpuMode, mmu: &mut Mmu, clock: u32) -> (u32, bool) {
        match self.dma_type {
            DmaType::None => (clock, false),  
            DmaType::Hdma => {
                if ppu_mode == PpuMode::HorizontalBlanking {
                    if clock >= 8 {
                        self.transfer_row(mmu);
                        
                        if self.vram_dma_len == 0x7F { self.dma_type = DmaType::None }

                        (clock - 8, false)
                    } else {
                        (clock, true)
                    }
                } else {
                    (clock, false)
                }
            },
            DmaType::Oma => {
                if clock >= OAM_DMA_TIME {
                    
                    let base = (self.oma_base_adder as u16) << 8;

                    for index in 0x00..0xA0 {
                        let byte = mmu.read_byte(base + index);
                        mmu.write_byte(0xFE00 + index, byte);
                    }

                    self.dma_type = DmaType::None;
                    (clock - OAM_DMA_TIME, false)
                } else {
                    (clock, true)
                }
            },
            DmaType::Gdma => {
                let rows = self.vram_dma_len + 1;
                if clock > (rows as u32 * 8 ) {
                    for _ in 0..rows {
                        self.transfer_row(mmu);
                    }
    
                    self.dma_type = DmaType::None;
    
                    (clock - (rows as u32 * 8), false)
                } else {
                    (clock, true)
                }
            }
        }
    }


    fn transfer_row(&mut self, mmu: &mut Mmu) {
        for index in 0 .. 0x10 {
            let b = mmu.read_byte(self.vram_dma_source + index);
            mmu.write_byte(self.vram_dma_target + index, b);
        }

        self.vram_dma_source += 0x10;
        self.vram_dma_target += 0x10;

        if self.vram_dma_len == 0 {
            self.vram_dma_len = 0x7F;
        } else {
            self.vram_dma_len -= 1;
        }
    }

    pub fn read_oam(&self) -> u8 {
        0
    }

    pub fn write_oam(&mut self, val: u8) {
        self.oma_base_adder = val;
        self.dma_type = DmaType::Oma;
    }

    pub fn read_vram_dma(&self, adder: u16) -> u8 {
        match adder {
            0xFF51 => get_u16_high(self.vram_dma_target),
            0xFF52 => get_u16_low(self.vram_dma_target),
            0xFF53 => get_u16_high(self.vram_dma_source),
            0xFF54 => get_u16_low(self.vram_dma_source),
            0xFF55 => self.vram_dma_len | if self.dma_type == DmaType::None { 0x80 } else { 0 },
            _ => panic!("Dma manager cannot handle adder at {}", adder),
        }
    }

    pub fn write_vram_dma(&mut self, adder: u16, val: u8) {
        match adder {
            0xFF51 => { self.vram_dma_source = build_u16(val, get_u16_low(self.vram_dma_source)) },
            0xFF52 => { self.vram_dma_source = build_u16(get_u16_high(self.vram_dma_source), val & 0xF0) },
            0xFF53 => { self.vram_dma_target = build_u16(val & 0x1F, get_u16_low(self.vram_dma_source)) },
            0xFF54 => { self.vram_dma_target = build_u16(get_u16_high(self.vram_dma_source), val & 0xF0) },
            0xFF55 => { 
                self.vram_dma_len = val & 0x7F; 
                if (val & 0x80) == 0 { self.dma_type == DmaType::Gdma } 
                else { self.dma_type == DmaType::Hdma }; },
            _ => panic!("Dma manager cannot handle adder at {}", adder),
        }
    }
}