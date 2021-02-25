use crate::mmu::*;

const ZERO_PAGE_SIZE : usize = 0x7F;
const WRAM_BANK_SIZE : usize = 0x1000;
const WRAM_BANK_COUNT : usize = 8;


pub struct Ram {
    selected_wram: usize,
    wram_banks: Vec<Vec<u8>>,
}

impl Ram {
    pub fn new() -> Ram {
        Ram {
            selected_wram: 1,
            wram_banks: (0..WRAM_BANK_SIZE).map(|_| vec![0; WRAM_BANK_COUNT]).collect(),
        }
    }
}



impl IoDevice for Ram {
    fn read_byte(&mut self, _: &Mmu, adder: u16) -> MemRead { 
        match adder {
            0xC000 ..= 0xCFFF => MemRead::Read(self.wram_banks[0][adder as usize - 0xC000]),
            0xD000 ..= 0xDFFF => MemRead::Read(self.wram_banks[self.selected_wram][adder as usize - 0xD000]),
            0xE000 ..= 0xFDFF => MemRead::Read(self.wram_banks[0][adder as usize - 0xC000]),
            0xFF70 => MemRead::Read(self.selected_wram as u8),
            _ => MemRead::Ignore
        }
    }

    fn write_byte(&mut self, _: &Mmu, adder: u16, value: u8) -> MemWrite { 
        match adder {
            0xC000 ..= 0xCFFF => {
                self.wram_banks[0][adder as usize - 0xC000] = value;
                MemWrite::Write
            },
            0xD000 ..= 0xDFFF => {
                self.wram_banks[self.selected_wram][adder as usize - 0xD000] = value;
                MemWrite::Write
            },
            0xE000 ..= 0xFDFF => {
                self.wram_banks[0][adder as usize - 0xC000] = value;
                MemWrite::Write
            },
            0xFF70 => {
                self.selected_wram = value.max(1) as usize;
                MemWrite::Write
            }
            _ => MemWrite::Ignore
        }
     }
}