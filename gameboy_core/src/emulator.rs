
use crate::ram::Ram;
use crate::joypad::Joypad;
use crate::cartridge_controller::CartridgeController;
use crate::timer::Timer;
use crate::ppu::Ppu;
use crate::ic::Ic;
use std::rc::Rc;
use std::cell::RefCell;
use crate::hardware::Hardware;
use crate::processor::Processor;
use crate::mmu::Mmu;

#[derive(PartialEq, Copy, Clone)]
pub enum GameBoyMode {
    Classic,
    Color,
}

#[derive(Clone)]
pub struct EmulatorConfig {
    pub game_boy_mode: GameBoyMode,
    pub allow_bad_checksum: bool,
}

pub struct Emulator {
    hw: Rc<RefCell<Box<dyn Hardware>>>,
    processor: Processor,
    mmu: Mmu,
    ic: Rc<RefCell<Ic>>,
    ppu: Rc<RefCell<Ppu>>,
    timer: Rc<RefCell<Timer>>,
    cartridge_controller: Rc<RefCell<CartridgeController>>,
    joypad: Rc<RefCell<Joypad>>,
}



impl Emulator {
    pub fn new(rom: Vec<u8>, hw: Box<dyn Hardware>, emulator_config: EmulatorConfig) -> Emulator {

        let hw = Rc::new(RefCell::new(hw));
        let ic = Rc::new(RefCell::new(Ic::new()));
        let irq = ic.borrow().get_requester();
        let cartridge_controller = Rc::new(RefCell::new(CartridgeController::new(rom, emulator_config.game_boy_mode, emulator_config.allow_bad_checksum)));
        let joypad = Rc::new(RefCell::new(Joypad::new(Rc::clone(&hw), irq.clone())));
        let ram = Rc::new(RefCell::new(Ram::new()));

        let processor = Processor::new();
        let mut mmu = Mmu::new();
        let ppu = Rc::new(RefCell::new(Ppu::new(Rc::clone(&hw), irq.clone(), emulator_config.game_boy_mode)));
        let timer = Rc::new(RefCell::new(Timer::new(irq.clone())));


        mmu.register_device((0x0000, 0x7fff), Rc::clone(&cartridge_controller));
        mmu.register_device((0xff50, 0xff50), Rc::clone(&cartridge_controller));
        mmu.register_device((0xa000, 0xbfff), Rc::clone(&cartridge_controller));

        mmu.register_device((0x8000, 0x9fff), Rc::clone(&ppu));
        mmu.register_device((0xff40, 0xff55), Rc::clone(&ppu));
        mmu.register_device((0xff68, 0xff6b), Rc::clone(&ppu));

        mmu.register_device((0xff0f, 0xff0f), Rc::clone(&ic));
        mmu.register_device((0xffff, 0xffff), Rc::clone(&ic));
        mmu.register_device((0xff00, 0xff00), Rc::clone(&joypad));
        mmu.register_device((0xff04, 0xff07), Rc::clone(&timer));

        mmu.register_device((0xC000, 0xCFFF), Rc::clone(&ram));
        mmu.register_device((0xD000, 0xDFFF), Rc::clone(&ram));
        mmu.register_device((0xE000, 0xFDFF), Rc::clone(&ram));
        mmu.register_device((0xFF70, 0xFF70), Rc::clone(&ram));
        mmu.register_device((0xFF80, 0xFFFE), Rc::clone(&ram));

        Emulator {
            hw,
            ic,
            processor,
            mmu,
            ppu,
            timer,
            cartridge_controller,
            joypad,
        }
    }

    fn cycle(&mut self) {

        let mut clock = self.processor.cycle(&mut self.mmu);

        clock += self.processor.check_interrupt(&mut self.mmu, &self.ic);

        self.ppu.borrow_mut().cycle(&mut self.mmu, clock);
        self.timer.borrow_mut().cycle(clock);
        self.joypad.borrow_mut().poll();

        // TODO add sleep for the clock cycle the cpu did.

    }

    // cycle as long as hw allows
    pub fn poll(&mut self) -> bool {
        if !self.hw.borrow_mut().run() {
            return false;
        }

        self.cycle();

        true
    }
}



pub fn run(rom: Vec<u8>, hw: Box<dyn Hardware>, emulator_config: EmulatorConfig) {
    let mut emulator = Emulator::new(rom, hw, emulator_config);
    while emulator.poll() {}
}