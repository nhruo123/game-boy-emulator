use crate::cartridge_controller::MemWrite;
use crate::cartridge_controller::Mmu;
use crate::cartridge_controller::MemRead;
use crate::cartridge_controller::Cartridge;

pub struct RomOnly {
    rom: Vec<u8>,
}

impl RomOnly {
    pub fn new(data: Vec<u8>) -> RomOnly {
        RomOnly { rom: data }
    }
}

impl Cartridge for RomOnly {
    fn read_byte(&mut self, _: &Mmu, addr: u16) -> MemRead {
        if addr <= 0x7fff {
            MemRead::Read(self.rom[addr as usize])
        } else {
            MemRead::Ignore
        }
    }
    fn write_byte(&mut self, _: &Mmu, addr: u16, val: u8) -> MemWrite {
        if addr <= 0x7fff {
            MemWrite::Write
        } else if addr >= 0xa000 && addr <= 0xbfff {
            MemWrite::Ignore
        } else {
            unreachable!("Write to ROM: {:02x} {:02x}", addr, val);
        }
    }
}
