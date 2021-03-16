
use crate::processor::T_CYCLE_FREQUENCY;
use crate::apu::frame_sequencer::FrameSequencer;
use crate::apu::sound_length::SoundLength;
use crate::processor::TCycles;
use crate::utils;

const WAVE_PATTERN_RAM_SIZE: usize = 32;
const MAX_WAVE_LEN: u8 = 255;

pub struct Wave {
    pub channel_enabled: bool,

    frequency: u16,

    volume: u8,

    wave_index: usize,



    clock: TCycles,

    sound_length: SoundLength,
    frame_sequencer: FrameSequencer,

    wave_pattern_ram: [u8; WAVE_PATTERN_RAM_SIZE / 2],
}

impl Wave {
    pub fn new() -> Self {
        Self {
            channel_enabled: false,
            frequency: 0,
            clock: 0,
            frame_sequencer: FrameSequencer::new(),
            wave_index: 0,
            sound_length: SoundLength::new(MAX_WAVE_LEN),
            volume: 0,
            wave_pattern_ram: [0; WAVE_PATTERN_RAM_SIZE / 2],
        }
    }

    pub fn read_byte(&self, base_addr: u16, addr: u16) -> u8 {
        match addr - base_addr {
            0x00 => (if self.channel_enabled { 0x80 } else { 0 }) | 0x7F,
            0x01 => 0xFF,
            0x02 => self.volume << 5 | 0x9F,
            0x03 => 0xFF,
            0x04 => 0xFB | if self.sound_length.dec_sound_len { 0x40 } else { 0 },
            0x16 ..= 0x26 => self.wave_pattern_ram[(addr - base_addr - 0x16) as usize],
            _ => unreachable!("Bad addr"),
        }
    }

    pub fn write_byte(&mut self, base_addr: u16, addr: u16, val: u8) {
        match addr - base_addr {
            0x00 => {
                self.channel_enabled = val & 0x80 != 0;
            },
            0x01 => {
                self.sound_length.set_length(val);
            }
            0x02 => {
                self.volume = val & 0x3;
            }
            0x03 => self.frequency = utils::build_u16(utils::get_u16_high(self.frequency), val),
            0x04 => {
                self.frequency = utils::build_u16(val & 0x3, utils::get_u16_low(self.frequency));
                self.sound_length.dec_sound_len = (val & 0x40) != 0;
                if (val & 0x80) != 1 {
                    self.enable_channel();
                }
            },
            0x16 ..= 0x25 => self.wave_pattern_ram[(addr - base_addr - 0x16) as usize] = val,
            _ => unreachable!("Bad addr"),
        };
    }

    fn enable_channel(&mut self) {
        self.frame_sequencer.reset();
        self.sound_length.reset();
        
        self.wave_index = 0;
        self.clock = 0;
        

        self.channel_enabled = true;
    }

    fn get_t_cycle_ratio(&self) -> u32 {
        T_CYCLE_FREQUENCY / ((2048 - self.frequency as u32) * 64)
    }

    pub fn cycle(&mut self, clocks: TCycles) -> u16 {
        if !self.channel_enabled {
            return 0;
        }

        self.clock += clocks;

        if self.frame_sequencer.cycle(clocks) {
            if self.frame_sequencer.current_cycle % 2 == 0 {
                self.sound_length.cycle(&mut self.channel_enabled);
            }
        }

        if self.clock >= self.get_t_cycle_ratio() {
            self.clock -= self.get_t_cycle_ratio();

            self.wave_index += 1;

            self.wave_index %= WAVE_PATTERN_RAM_SIZE;
        }

        if self.volume != 0 {
            let current_amplitude_byte = self.wave_pattern_ram[self.wave_index / 2];

            let current_amplitude = if self.wave_index % 2 == 0 {
                current_amplitude_byte & 0xF
            } else {
                (current_amplitude_byte & 0xF0) >> 4
            };

            current_amplitude as u16 >> self.volume
        } else {
            0
        }
        
    }
}