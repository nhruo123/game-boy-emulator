use crate::apu::sound_length::SoundLength;
use crate::apu::volume::Volume;
use crate::apu::frame_sequencer::FrameSequencer;
use crate::processor::{TCycles, T_CYCLE_FREQUENCY};
use crate::utils;

const BASE_SWEEP_FREQUENCY: u32 = 128;
pub const MAX_TONE_VOLUME: u32 = 15;

const MAX_SOUND_LEN: u8 = 64;

const WAVE_STATES: usize = 8;
const WAVE_TYPES: usize = 4;

const WAVE_PATTERN: [[u16; WAVE_STATES]; WAVE_TYPES] = [
    [1, 0, 0, 0, 0, 0, 0, 0],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 1, 1, 1],
    [0, 1, 1, 1, 1, 1, 1, 0],
];

pub struct Tone {
    clock: TCycles,
    pub channel_enabled: bool,

    sweep_enabled: bool,

    currant_sweep_cycle: u8,
    sweep_time: u8,
    sweep_increase: bool,
    sweep_shift: u8,

    selected_wave_pattern: u8,
    currant_wave_cycle: usize,


    frequency: u16,

    volume: Volume,
    sound_length: SoundLength,
    frame_sequencer: FrameSequencer,
}

impl Tone {
    pub fn new(sweep_enabled: bool) -> Self {
        Self {
            clock: 0,
            sweep_enabled,
            sweep_time: 0,
            sweep_increase: false,
            sweep_shift: 0,
            currant_sweep_cycle: 0,
            selected_wave_pattern: 0,
            volume: Volume::new(),
            sound_length: SoundLength::new(MAX_SOUND_LEN),
            frequency: 0,
            frame_sequencer: FrameSequencer::new(),
            channel_enabled: false,
            currant_wave_cycle: 0,
        }
    }

    pub fn read_byte(&self, base_addr: u16, addr: u16) -> u8 {
        match addr - base_addr {
            0x00 => match self.sweep_enabled {
                true => {
                    self.sweep_shift
                        | (if self.sweep_increase { 1 } else { 0 } << 3)
                        | (self.sweep_time << 4)
                        | 0x80
                }
                false => 0xFF,
            },
            0x01 => 0x3F | self.selected_wave_pattern << 6,
            0x02 => {
                self.volume.read()
            }
            0x03 => 0xFF, // write only memory
            0x04 => 0xFB | if self.sound_length.dec_sound_len { 0x40 } else { 0 },
            _ => unreachable!("Bad addr"),
        }
    }

    pub fn write_byte(&mut self, base_addr: u16, addr: u16, val: u8) {
        match addr - base_addr {
            0x00 => match self.sweep_enabled {
                true => {
                    self.sweep_shift = val & 0x3;
                    self.sweep_increase = val & 0x4 != 0;
                    self.sweep_time = (val & 0x70) >> 4;
                }
                false => unreachable!("We cant write a sweep register to a tone with no sweep"),
            },
            0x01 => {
                self.sound_length.set_length(val & 0x3F);
                self.selected_wave_pattern = (val & 0xC0) >> 6;
            }
            0x02 => {
                self.volume.write(val);
            }
            0x03 => self.frequency = utils::build_u16(utils::get_u16_high(self.frequency), val),
            0x04 => {
                self.frequency = utils::build_u16(val & 0x3, utils::get_u16_low(self.frequency));
                self.sound_length.dec_sound_len = (val & 0x40) != 0;
                if (val & 0x80) != 0 {
                    self.enable_channel();
                }
            }
            _ => unreachable!("Bad addr"),
        };
    }

    fn cycle_sweep(&mut self) {
        if !self.sweep_enabled || self.sweep_time == 0 {
            return;
        }

        self.currant_sweep_cycle += 1;

        if self.currant_sweep_cycle == self.sweep_time {
            self.currant_sweep_cycle = 0;

            let pow_base: u16 = 2;
            
            let sweep_change = self.frequency / pow_base.pow(self.sweep_shift as u32);

            self.frequency = if self.sweep_increase {
                self.frequency.wrapping_add(sweep_change)
            } else {
                self.frequency.wrapping_sub(sweep_change)
            };

            if self.frequency > 2047 {
                self.channel_enabled = false;
            }
        }
    }

    fn get_t_cycle_ratio(&self) -> u32 {
        T_CYCLE_FREQUENCY / ((2048 - self.frequency as u32) << 5)
    }

    fn enable_channel(&mut self) {
        self.volume.reset();
        self.frame_sequencer.reset();
        
        self.currant_wave_cycle = 0;
        self.clock = 0;
        
        self.channel_enabled = true;
    }

    pub fn cycle(&mut self, clocks: TCycles) -> u16 {
        if !self.channel_enabled {
            return 0;
        }


        self.clock += clocks;

        if self.frame_sequencer.cycle(clocks) {
            // we got a new frame sequencer
            if self.frame_sequencer.current_cycle % 2 == 0 {
                self.sound_length.cycle(&mut self.channel_enabled);
            }
            if self.frame_sequencer.current_cycle == 7 {
                self.volume.cycle();
            }
            if self.frame_sequencer.current_cycle == 2 || self.frame_sequencer.current_cycle == 6 {
                self.cycle_sweep();
            }
        }

        
        // handle wave pattern
        if self.clock >= self.get_t_cycle_ratio() {
            self.clock -= self.get_t_cycle_ratio();
            
            self.currant_wave_cycle += 1;

            self.currant_wave_cycle %= WAVE_STATES;
        }


        WAVE_PATTERN[self.selected_wave_pattern as usize][self.currant_wave_cycle] * self.volume.volume as u16
    }
}
