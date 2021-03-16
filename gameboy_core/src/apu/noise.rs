use crate::apu::frame_sequencer::FrameSequencer;
use crate::apu::sound_length::SoundLength;
use crate::apu::volume::Volume;
use crate::processor::TCycles;
use crate::processor::T_CYCLE_FREQUENCY;

const MAX_SOUND_LEN: u8 = 64;

pub struct Noise {
    pub channel_enabled: bool,

    last_lower_bit: u16,

    frequency: u8,
    width_mode: bool,
    frequency_divider: u8,

    shift_register: u16,

    clock: TCycles,

    volume: Volume,
    sound_length: SoundLength,
    frame_sequencer: FrameSequencer,
}

impl Noise {
    pub fn new() -> Self {
        Self {
            channel_enabled: false,
            frequency: 0,
            width_mode: false,
            frequency_divider: 0,
            shift_register: 1,
            clock: 0,
            last_lower_bit: 0,
            volume: Volume::new(),
            sound_length: SoundLength::new(MAX_SOUND_LEN),
            frame_sequencer: FrameSequencer::new(),
        }
    }

    pub fn read_byte(&self, base_addr: u16, addr: u16) -> u8 {
        match addr - base_addr {
            0x00 => 0xFF,
            0x01 => 0xFF,
            0x02 => self.volume.read(),
            0x03 => self.frequency_divider | if self.width_mode { 0x4 } else { 0x0 } | (self.frequency << 4),
            0x04 => 0xFB | if self.sound_length.dec_sound_len { 0x40 } else { 0 },
            _ => unreachable!("Bad addr"),
        }
    }

    pub fn write_byte(&mut self, base_addr: u16, addr: u16, val: u8) {
        match addr - base_addr {
            0x00 => {
                // unmapped
                return;
            },
            0x01 => {
                self.sound_length.set_length(val & 0x1F);
                
            }
            0x02 => self.volume.write(val),
            0x03 => {
                self.frequency_divider = val & 0x3;
                self.width_mode = val & 0x4 != 0;
                self.frequency = val >> 4;
            },
            0x04 => {
                self.sound_length.dec_sound_len = (val & 0x40) != 0;
                if (val & 0x80) != 1 {
                    self.enable_channel();
                }
            }
            _ => unreachable!("Bad addr"),
        };
    }

    fn get_t_cycle_ratio(&self) -> u32 {
        let pow_base: u32 = 2;

        ((T_CYCLE_FREQUENCY / 4) / (self.frequency_divider as u32 + 1))
            / pow_base.pow(self.frequency as u32 + 1)
    }

    fn enable_channel(&mut self) {
        self.frame_sequencer.reset();
        self.sound_length.reset();

        
        self.shift_register = 1; 
        self.clock = 0;
        

        self.channel_enabled = true;
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
            if self.frame_sequencer.current_cycle == 7 {
                self.volume.cycle();
            }
        }

        if self.clock >= self.get_t_cycle_ratio() {
            self.clock -= self.get_t_cycle_ratio();

            self.last_lower_bit = self.shift_register & 0x1;

            let xor_bit = (self.shift_register & 0x0) ^ ((self.shift_register & 0x1) >> 1);

            self.shift_register >>= 1;

            self.shift_register = (self.shift_register & !(1 << 15)) | (xor_bit << 15);

            if self.width_mode {
                self.shift_register = (self.shift_register & !(1 << 7)) | (xor_bit << 7)
            }
        }

        self.last_lower_bit * self.volume.volume as u16
    }
}
