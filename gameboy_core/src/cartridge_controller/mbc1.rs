use crate::cartridge_controller::MemWrite;
use crate::cartridge_controller::MemRead;
use crate::cartridge_controller::Mmu;
use crate::cartridge_controller::Cartridge;


const MBC1_TYPE: u8 = 0x1;
const MBC1_RAM_TYPE: u8 = 0x1;
const MBC1_RAM_BATTERY_TYPE: u8 = 0x1;

pub struct Mbc1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_bank: usize,
    ram_bank: usize,
    ram_enable: bool,
    ram_select: bool,
}


impl Mbc1 {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            rom,
            ram: vec![0; 0x8000], //TODO ask hardware to give ram
            rom_bank: 1,
            ram_bank: 0,
            ram_enable: false,
            ram_select: false,
        }
    }   

    pub fn probe_cartridge(code: u8) -> bool {
        (code == MBC1_TYPE) | (code == MBC1_RAM_TYPE) | (code == MBC1_RAM_BATTERY_TYPE)
    }
}


impl Cartridge for Mbc1 {
    fn read_byte(&mut self, _mmu: &Mmu, addr: u16) -> MemRead { 
        match addr {
            0x0 ..= 0x3FFF => MemRead::Read(self.rom[addr as usize]),
            0x4000 ..= 0x7FFF => {
                let rom_bank = self.rom_bank.max(1);

                let rom_bank = if rom_bank == 0x20 || rom_bank == 0x40 || rom_bank == 0x60 {
                    rom_bank + 1
                } else {
                    rom_bank
                };

                let base = rom_bank * 0x4000;

                let offset = addr as usize - 0x4000;

                let addr = (base + offset) & (self.rom.len() - 1);
                MemRead::Read(self.rom[addr])
            }
            0xA000 ..= 0xBFFF => {
                if self.ram_enable {
                    let base = self.ram_bank as usize * 0x2000;
                    let offset = addr as usize - 0xa000;
                    let addr = (base + offset) & (self.rom.len() - 1);
                    MemRead::Read(self.ram[addr])
                } else {
                    MemRead::Read(0)
                }
            }

            _ => MemRead::Ignore,
        }
    }

    fn write_byte(&mut self, _mmu: &Mmu, addr: u16, val: u8) -> MemWrite {
        match addr {
            0x0 ..= 0x1FFF => {
                self.ram_enable = val & 0xA == 0xA;
                MemWrite::Write
            }
            0x2000 ..= 0x3FFF => {
                self.rom_bank = ((self.rom_bank & !0x1f) | (val as usize & 0x1f)).max(1);

                MemWrite::Write
            }
            0x4000 ..= 0x5FFF => {
                if self.ram_select {
                    self.ram_bank = val as usize & 0x3;
                } else {
                    self.rom_bank = (self.rom_bank & !0x60) | ((val as usize & 0x3) << 5);
                }

                MemWrite::Write
            },
            0x6000 ..= 0x7FFF => {
                self.ram_select = val & 1 != 0;
                MemWrite::Write
            }, 
            0xa000 ..= 0xbfff => {
                if self.ram_enable {
                    let base = self.ram_bank as usize * 0x2000;
                    let offset = addr as usize - 0xa000;
                    self.ram[base + offset] = val;
                    MemWrite::Write
                } else {
                    // bad ram write
                    MemWrite::Write
                }
            }

            _ => MemWrite::Ignore,
        }
    }
}