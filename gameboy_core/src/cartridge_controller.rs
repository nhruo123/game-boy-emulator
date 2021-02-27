mod rom_only;
mod mbc1;

use crate::cartridge_controller::mbc1::Mbc1;
use crate::mmu::MemWrite;
use crate::mmu::MemRead;
use crate::mmu::Mmu;
use crate::cartridge_controller::rom_only::RomOnly;
use crate::emulator::GameBoyMode;
use crate::mmu::IoDevice;

const CARTRIDGE_TYPE_ADDER: usize = 0x0147;



const COLOR_BOOT_ROM: &[u8] = {
    include_bytes!("boot_roms\\cgb.bin")
};

const CLASSIC_BOOT_ROM: &[u8] = {
    include_bytes!("boot_roms\\dmg.bin")
};



pub trait Cartridge {
    fn read_byte(&mut self, mmu: &Mmu, adder: u16) -> MemRead;

    fn write_byte(&mut self, mmu: &Mmu, adder: u16, value: u8) -> MemWrite;
}

pub struct CartridgeController {
    cartridge: Box<dyn Cartridge>,
    use_boot_rom: bool,
    game_boy_mode: GameBoyMode,
}

fn calc_checksum(rom: &[u8]) -> u8 {
    let mut sum = 0u8;

    for i in 0x134..0x14D {
        sum = sum.wrapping_sub(rom[i]).wrapping_sub(1);
    }

    sum
}

impl CartridgeController {
    pub fn new(rom: Vec<u8>, game_boy_mode: GameBoyMode, allow_bad_checksum: bool) -> CartridgeController {


        if !calc_checksum(&rom) == rom[0x14D] && !allow_bad_checksum {
            panic!("bad checksum");
        }

        let cartridge_type = rom[CARTRIDGE_TYPE_ADDER];

        let cartridge: Box<dyn Cartridge> = if RomOnly::probe_cartridge(cartridge_type) {
            Box::new(RomOnly::new(rom.clone()))
        } else if Mbc1::probe_cartridge(cartridge_type) {
            Box::new(Mbc1::new(rom.clone()))
        } else {
            unimplemented!("unimplemented cartridge type")
        };

        CartridgeController {
            cartridge: cartridge,
            use_boot_rom: true,
            game_boy_mode: game_boy_mode,
        }
    }

    fn in_boot_rom(&self, addr: u16) -> bool {
        match self.game_boy_mode {
            GameBoyMode::Color => {
                assert_eq!(0x900, COLOR_BOOT_ROM.len());

                addr < 0x100 || (addr >= 0x200 && addr < 0x900)
            },
            GameBoyMode::Classic => {
                assert_eq!(0x100, CLASSIC_BOOT_ROM.len());

                addr < 0x100
            }
        }
    }


}

impl IoDevice for CartridgeController {
    fn read_byte(&mut self, mmu: &Mmu, addr: u16) -> MemRead { 
        if self.use_boot_rom && self.in_boot_rom(addr) {
            match self.game_boy_mode {
                GameBoyMode::Classic => MemRead::Read(CLASSIC_BOOT_ROM[addr as usize]),
                GameBoyMode::Color => MemRead::Read(COLOR_BOOT_ROM[addr as usize]),
            }
        } else {
            self.cartridge.read_byte(mmu, addr)
        }
    }
    fn write_byte(&mut self, mmu: &Mmu, addr: u16, val: u8) -> MemWrite {
        if self.use_boot_rom && addr < 0x100 {
            unreachable!("Writing to boot ROM")
        } else if addr == 0xff50 {
            self.use_boot_rom = false;
            MemWrite::Write
        } else {
            self.cartridge.write_byte(mmu, addr, val)
        }
    }
}