
pub struct Volume {
    pub initial_volume: u8,
    pub volume: u8,
    vol_envelope_increase: bool,
    envelope_counter: u8,
}

impl Volume {
    pub fn new() -> Self {
        Self {
            envelope_counter: 0,
            initial_volume: 0,
            vol_envelope_increase: false,
            volume: 0,
        }
    }

    pub fn cycle(&mut self) {
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
            self.volume -= 1;
        }

        self.envelope_counter -= 1;
    }

    pub fn read(&self) -> u8 {
        (self.envelope_counter)
        | (if self.vol_envelope_increase { 0x4 } else { 0 })
        | (self.volume << 4)
    }

    pub fn write(&mut self, val: u8) {
        self.envelope_counter = val & 0x3;
        self.vol_envelope_increase = val & 0x4 != 0;
        self.volume = val >> 4;
        self.initial_volume = self.volume;
    }
    
    pub fn reset(&mut self) {
        self.volume = self.initial_volume;
    }
}