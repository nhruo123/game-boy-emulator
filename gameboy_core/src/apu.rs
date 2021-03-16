use crate::apu::noise::Noise;
use crate::apu::tone::Tone;
use crate::apu::wave::Wave;
use crate::hardware::Hardware;
use crate::mmu::IoDevice;
use crate::mmu::{MemRead, MemWrite, Mmu};
use crate::processor::TCycles;
use std::cell::RefCell;
use std::rc::Rc;

mod frame_sequencer;
mod noise;
mod sound_length;
mod tone;
mod volume;
mod wave;

#[derive(Default)]
struct ChanelControlRegister {
    pub left_vin: bool,
    pub right_vin: bool,
    pub left_level: u8,
    pub right_level: u8,
}

impl ChanelControlRegister {
    pub fn write(&mut self, val: u8) {
        self.right_level = val & 0x7;
        self.right_vin = val & 0x4 != 0;

        self.left_level = (val >> 4) & 0x7;
        self.left_vin = val & 0x80 != 0;
    }

    pub fn read(&self) -> u8 {
        self.right_level
            | if self.right_vin { 0x4 } else { 0x0 }
            | (self.left_level << 4)
            | if self.left_vin { 0x80 } else { 0x0 }
    }
}

#[derive(Default)]
struct SoundDirectionRegister {
    pub tone1_right: bool,
    pub tone2_right: bool,
    pub wave_right: bool,
    pub noise_right: bool,

    pub tone1_left: bool,
    pub tone2_left: bool,
    pub wave_left: bool,
    pub noise_left: bool,
}

impl SoundDirectionRegister {
    pub fn write(&mut self, val: u8) {
        self.tone1_right = val & 0x1 != 0;
        self.tone2_right = val & 0x2 != 0;
        self.wave_right = val & 0x4 != 0;
        self.noise_right = val & 0x8 != 0;

        self.tone1_left = val & 0x10 != 0;
        self.tone2_left = val & 0x20 != 0;
        self.wave_left = val & 0x40 != 0;
        self.noise_left = val & 0x80 != 0;
    }

    pub fn read(&self) -> u8 {
        (if self.tone1_right { 0x1 } else { 0x0 })
            | (if self.tone2_right { 0x2 } else { 0x0 })
            | (if self.wave_right { 0x4 } else { 0x0 })
            | (if self.noise_right { 0x8 } else { 0x0 })
            | (if self.tone1_left { 0x10 } else { 0x0 })
            | (if self.tone2_left { 0x20 } else { 0x0 })
            | (if self.wave_left { 0x40 } else { 0x0 })
            | (if self.noise_left { 0x80 } else { 0x0 })
    }
}

pub struct Apu {
    soundDirection: SoundDirectionRegister,
    chanelControl: ChanelControlRegister,
    tone1: Tone,
    tone2: Tone,
    wave: Wave,
    noise: Noise,

    is_sound_enabled: bool,

    emulator_cycle_frequency: u32,
    hardware: Rc<RefCell<Box<dyn Hardware>>>,
    clock: TCycles,
}

impl Apu {
    pub fn new(hardware: Rc<RefCell<Box<dyn Hardware>>>, emulator_cycle_frequency: u32) -> Self {
        Self {
            soundDirection: SoundDirectionRegister::default(),
            chanelControl: ChanelControlRegister::default(),
            tone1: Tone::new(true),
            tone2: Tone::new(false),
            wave: Wave::new(),
            noise: Noise::new(),
            is_sound_enabled: false,
            emulator_cycle_frequency,
            hardware,
            clock: 0,
        }
    }

    pub fn cycle(&mut self, clocks: TCycles) {
        if !self.is_sound_enabled {
            return;
        }

        self.clock += clocks;

        let highest_volume = self
            .chanelControl
            .right_level
            .max(self.chanelControl.left_level) as u16;

        let mut amplitude = 0;

        amplitude += self.tone1.cycle(clocks)
            * highest_volume
            * (self.soundDirection.tone1_left || self.soundDirection.tone1_right) as u16;
        amplitude += self.tone2.cycle(clocks)
            * highest_volume
            * (self.soundDirection.tone2_left || self.soundDirection.tone2_right) as u16;
        amplitude += self.wave.cycle(clocks)
            * highest_volume
            * (self.soundDirection.wave_left || self.soundDirection.wave_right) as u16;
        amplitude += self.noise.cycle(clocks)
            * highest_volume
            * (self.soundDirection.noise_left || self.soundDirection.noise_right) as u16;


        let tcycles_to_output_rate = self.emulator_cycle_frequency / self.hardware.borrow_mut().pcm_sample_rate();

        if self.clock >= tcycles_to_output_rate {
            self.clock -= tcycles_to_output_rate;
            self.hardware.borrow_mut().next_pcm_amplitude((amplitude as u64 / 840) as f32 / 100.0);
        }
    }
}

impl IoDevice for Apu {
    fn read_byte(&mut self, _: &Mmu, addr: u16) -> MemRead {
        match addr {
            0xFF10..=0xFF14 => MemRead::Read(self.tone1.read_byte(0xFF10, addr)),
            0xFF15..=0xFF19 => MemRead::Read(self.tone2.read_byte(0xFF15, addr)),
            0xFF1A..=0xFF1E => MemRead::Read(self.wave.read_byte(0xFF1A, addr)),
            0xFF1F..=0xFF23 => MemRead::Read(self.noise.read_byte(0xFF1F, addr)),
            0xFF24 => MemRead::Read(self.chanelControl.read()),
            0xFF25 => MemRead::Read(self.soundDirection.read()),
            0xFF26 => MemRead::Read(
                (if self.is_sound_enabled { 0x80 } else { 0x0 })
                    | (if self.tone1.channel_enabled { 0x1 } else { 0x0 })
                    | (if self.tone2.channel_enabled { 0x2 } else { 0x0 })
                    | (if self.wave.channel_enabled { 0x4 } else { 0x0 })
                    | (if self.noise.channel_enabled { 0x8 } else { 0x0 }),
            ),
            0xFF30..=0xFF3F => MemRead::Read(self.wave.read_byte(0xFF1A, addr)),
            _ => MemRead::Ignore,
        }
    }
    fn write_byte(&mut self, _: &Mmu, addr: u16, val: u8) -> MemWrite {
        match addr {
            0xFF10..=0xFF14 => {
                self.tone1.write_byte(0xFF10, addr, val);
                MemWrite::Write
            }
            0xFF15..=0xFF19 => {
                self.tone2.write_byte(0xFF15, addr, val);
                MemWrite::Write
            }
            0xFF1A..=0xFF1E => {
                self.wave.write_byte(0xFF1A, addr, val);
                MemWrite::Write
            }
            0xFF1F..=0xFF23 => {
                self.noise.write_byte(0xFF1F, addr, val);
                MemWrite::Write
            }
            0xFF24 => {
                self.chanelControl.write(val);
                MemWrite::Write
            }
            0xFF25 => {
                self.soundDirection.write(val);
                MemWrite::Write
            }
            0xFF26 => {
                self.is_sound_enabled = val & 0x80 != 0;

                MemWrite::Write
            }
            0xFF30..=0xFF3F => {
                self.wave.write_byte(0xFF1A, addr, val);
                MemWrite::Write
            }
            _ => MemWrite::Ignore,
        }
    }
}
