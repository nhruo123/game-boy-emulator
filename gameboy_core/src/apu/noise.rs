use crate::apu::frame_sequencer::FrameSequencer;
use crate::apu::sound_length::SoundLength;
use crate::apu::volume::Volume;
use crate::processor::TCycles;
use crate::processor::T_CYCLE_FREQUENCY;

const MAX_SOUND_LEN: u8 = 64;

pub struct Noise {
    channel_enabled: bool,

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

    fn get_t_cycle_ratio(&self) -> u32 {
        let pow_base: u32 = 2;

        ((T_CYCLE_FREQUENCY / 4) / (self.frequency_divider as u32 + 1))
            / pow_base.pow(self.frequency as u32 + 1)
    }

    pub fn step(&mut self, clocks: TCycles) -> u16 {
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
