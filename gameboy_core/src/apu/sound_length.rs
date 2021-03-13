pub struct SoundLength {
    sound_len: u8,
    pub dec_sound_len: bool,

    max_len: u8,
}


impl SoundLength {
    pub fn new(max_len: u8) -> Self {
        Self {
            max_len,
            dec_sound_len: false,
            sound_len: 0,
        }
    }

    pub fn cycle(&mut self, channel_enabled: &mut bool) {
        if !self.dec_sound_len {
            return;
        }

        if self.sound_len == 0 {
            *channel_enabled = false;
        } else {
            self.sound_len -= 1;
        }
    }

    pub fn reset(&mut self) {
        self.sound_len = self.max_len;
    }

    pub fn set_length(&mut self, val: u8) {
        self.sound_len = val % self.max_len;
    }
}