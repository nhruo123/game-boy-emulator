use crate::mmu::*;

const ZERO_PAGE_SIZE : usize = 0x7F;
const WRAM_BANK_SIZE : usize = 0x1000;
const WRAM_BANK_COUNT : usize = 8;


pub struct Ram {
    selected_wram: usize,
    wram_banks: Vec<Vec<u8>>,
    zero_ram_page: Vec<u8>,
}

impl Ram {
    pub fn new() -> Ram {
        Ram {
            selected_wram: 1,
            wram_banks: (0..WRAM_BANK_SIZE).map(|_| vec![0; WRAM_BANK_COUNT]).collect(),
            zero_ram_page: vec![0; ZERO_PAGE_SIZE],
        }
    }
}



impl IoDevice for Ram {
    fn read_byte(&mut self, _: &Mmu, adder: u16) -> MemRead { 
        match adder {
            0xC000 ..= 0xCFFF => MemRead::Read(self.wram_banks[0][adder as usize - 0xC000]),
            0xD000 ..= 0xDFFF => MemRead::Read(self.wram_banks[self.selected_wram][adder as usize - 0xD000]),
            0xE000 ..= 0xFDFF => MemRead::Read(self.wram_banks[0][adder as usize - 0xE000]),
            0xFF70 => MemRead::Read(self.selected_wram as u8),
            0xFF80 ..= 0xFFFE => MemRead::Read(self.zero_ram_page[adder as usize - 0xFF80]),
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
                self.wram_banks[0][adder as usize - 0xE000] = value;
                MemWrite::Write
            },
            0xFF70 => {
                // self.selected_wram = value.max(1) as usize;
                self.selected_wram = (value & 0x3).max(1) as usize; // TODO: if i remove the & then we crash
                MemWrite::Write
            },
            0xFF80 ..= 0xFFFE => {
                self.zero_ram_page[adder as usize - 0xFF80] = value;
                MemWrite::Write
            },
            _ => MemWrite::Ignore
        }
     }
}