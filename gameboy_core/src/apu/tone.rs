use crate::apu::frame_sequencer::FrameSequencer;
use crate::processor::T_CYCLE_FREQUENCY;
use crate::utils;

const BASE_SWEEP_FREQUENCY: u32 = 128;

const WAVE_PATTERN: [[u16; 8]; 4] = [
    [1, 0, 0, 0, 0, 0, 0, 0],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 1, 1, 1],
    [0, 1, 1, 1, 1, 1, 1, 0],
];

pub struct Tone {
    channel_enabled: bool,

    sweep_enabled: bool,

    currant_sweep_cycle: u8,
    sweep_time: u8,
    sweep_increase: bool,
    sweep_shift: u8,

    selected_wave_pattern: u8,

    initial_volume: u8,
    volume: u8,
    vol_envelope_increase: bool,
    envelope_counter: u8,

    sound_len: u8,
    dec_sound_len: bool,

    frequency: u16,
    frame_sequencer: FrameSequencer,
}

impl Tone {
    pub fn new(sweep_enabled: bool) -> Self {
        Self {
            sweep_enabled,
            sweep_time: 0,
            sweep_increase: false,
            sweep_shift: 0,
            currant_sweep_cycle: 0,
            selected_wave_pattern: 0,
            volume: 0,
            vol_envelope_increase: false,
            envelope_counter: 0,
            sound_len: 0,
            dec_sound_len: false,
            frequency: 0,
            frame_sequencer: FrameSequencer::new(),
            channel_enabled: false,
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
                (self.envelope_counter)
                    | (if self.vol_envelope_increase { 0x4 } else { 0 })
                    | (self.volume << 4)
            }
            0x03 => 0xFF, // write only memory
            0x04 => 0xFB | if self.dec_sound_len { 0x40 } else { 0 },
            _ => unreachable!("Bad addr"),
        }
    }

    // returns if the tone needs to be restarted
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
                self.sound_len = val & 0x3F;
                self.selected_wave_pattern = (val & 0xC0) >> 6;
            }
            0x02 => {
                self.envelope_counter = val & 0x3;
                self.vol_envelope_increase = val & 0x4 != 0;
                self.volume = val >> 4;
                self.initial_volume = self.volume
            }
            0x03 => self.frequency = utils::build_u16(utils::get_u16_high(self.frequency), val),
            0x04 => {
                self.frequency = utils::build_u16(val & 0x3, utils::get_u16_low(self.frequency));
                self.dec_sound_len = (val & 0x40) == 1;
                
                if (val & 0x80) == 1 {
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

            let sweep_change = self.frequency / (self.sweep_shift as u16).pow(2);

            self.frequency = if self.sweep_increase {
                self.frequency.wrapping_add(sweep_change)
            } else {
                self.frequency.wrapping_sub(sweep_change)
            };

            if self.frequency > 2047 {
                self.frequency = 0;
            }
        }
    }

    fn cycle_volume_envelope(&mut self) {
        if self.envelope_counter == 0 {
            return;
        }

        if self.volume == 15 || self.volume == 0 {
            self.envelope_counter = 0;
            return;
        }

        if self.vol_envelope_increase {
            self.volume += 1;
        } else {
            self.volume -=1;
        }

        self.envelope_counter -= 1;
    }

    fn cycle_len_counter(&mut self) {
        if !self.dec_sound_len {
            return;
        }

        if self.sound_len == 0 {
            self.channel_enabled = false;
        } else {
            self.sound_len -= 1;
        }
    }

    fn get_t_cycle_ratio(&self) -> u16 {
        (T_CYCLE_FREQUENCY / (self.frequency as u32)) as u16
    }

    fn enable_channel(&mut self) {
        self.channel_enabled = true;
        self.sound_len = 64;
        self.volume = self.initial_volume;
        self.frame_sequencer.reset();
    }
}
