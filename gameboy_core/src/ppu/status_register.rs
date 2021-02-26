use crate::ppu::PpuMode;

pub struct StatusRegister {
    pub mode: PpuMode,
    pub coincidence_flag: bool, // line == selected line
    pub h_blank_int: bool,
    pub v_blank_int: bool,
    pub oam_int: bool,
    pub coincidence_int: bool,
}

impl StatusRegister {

    pub fn new() -> StatusRegister {
        StatusRegister {
            mode: PpuMode::OAM,
            coincidence_flag: false,
            h_blank_int: false,
            v_blank_int: false,
            oam_int: false,
            coincidence_int: false,
        }
    }

    pub fn set(&mut self, val: u8) {
        self.h_blank_int = (val & (1 << 3)) != 0;
        self.v_blank_int = (val & (1 << 4)) != 0;
        self.oam_int = (val & (1 << 5)) != 0;
        self.coincidence_int = (val & (1 << 6)) != 0;
    }

    pub fn get(&self) -> u8 {
        let mut val: u8 = u8::from(self.mode);

        val |= if self.coincidence_flag { 0x4 } else { 0x0 };
        val |= if self.h_blank_int { 0x8 } else { 0x0 };
        val |= if self.v_blank_int { 0x10 } else { 0x0 };
        val |= if self.oam_int { 0x20 } else { 0x0 };
        val |= if self.coincidence_int { 0x40 } else { 0x0 };

        val
    }
}