use crate::mmu::{ Mmu, MemRead, MemWrite, IoDevice};
use crate::ic::Irq;

const CYCLES_PER_DIVIDER: u32 = 256;


pub struct Timer {
    irq: Irq,
    divider: u8,
    divider_clock: u32,

    timer: u8,
    timer_clock: u32,
    timer_modulo: u8,
    
    timer_enabled: bool,
    timer_speed: u32,
}


impl Timer {
    pub fn new(irq: Irq) -> Self {
        Self {
            irq,
            divider: 0,
            divider_clock: 0,
            timer: 0,
            timer_clock: 0,
            timer_modulo: 0,
            timer_speed: 0,
            timer_enabled: false,
        }
    }

    pub fn cycle(&mut self, clock: u32) {
        self.divider_clock += clock;

        while self.divider_clock >= CYCLES_PER_DIVIDER {
            self.divider_clock -= CYCLES_PER_DIVIDER;
            self.divider = self.divider.wrapping_add(1);
        }

        if !self.timer_enabled {
            return
        }

        self.timer_clock += clock;

        while self.timer_clock >= self.timer_speed {
            self.timer_clock -= self.timer_speed;

            let (time, of) = self.timer.overflowing_add(1);
            self.timer = time;

            if of {
                self.timer = self.timer_modulo;
                self.irq.timer(true);
            }
        }
    }
}

fn speed_to_u8(speed: u32) -> u8 {
    match speed {
        1024 => 0x0,
        16 => 0x1,
        64 => 0x2,
        256 => 0x3,
        _ => unreachable!(),
    }
}

fn u8_to_speed(speed_reg: u8) -> u32 {
    match speed_reg {
        0x0 => 1024,
        0x1 => 16,
        0x2 => 64,
        0x3 => 256,
        _ => unreachable!(),
    }
}

impl IoDevice for Timer {
    fn read_byte(&mut self, _mmu: &Mmu, adder: u16) -> MemRead { 
        match adder {
            0xFF04 => MemRead::Read(self.divider),
            0xFF05 => MemRead::Read(self.timer),
            0xFF06  => MemRead::Read(self.timer_modulo),
            0xFF07 => MemRead::Read(speed_to_u8(self.timer_speed) | if self.timer_enabled { 0x4 } else { 0 }),
            _ => MemRead::Ignore,
        }
    }

    fn write_byte(&mut self, _mmu: &Mmu, adder: u16, value: u8) -> MemWrite {
        match adder {
            0xFF04 => {
                self.divider = 0;
                MemWrite::Write
            },
            0xFF05 => {
                self.timer = value;
                MemWrite::Write
            },
            0xFF06  => {
                self.timer_modulo = value;
                MemWrite::Write
            },
            0xFF07 => {
                self.timer_enabled = (value & 0x4) != 0;
                self.timer_speed = u8_to_speed(value & 0x3);

                MemWrite::Write
            },
            _ => MemWrite::Ignore,
        }
    }
}