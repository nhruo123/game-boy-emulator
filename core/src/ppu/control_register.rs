pub struct ControlRegister {
    pub display_enabled: bool,
    pub window_tile_map_select: bool,
    pub window_display_enabled: bool,
    pub bg_and_win_tile_data_select:  bool,
    pub bg_tile_map_select: bool,
    pub sprite_size: bool,
    pub sprit_display_enabled: bool,
    pub bg_and_win_display: bool,
}

impl ControlRegister {
    pub fn set(&mut self, val: u8) {
        self.bg_and_win_display = (val & 1) != 0;
        self.sprit_display_enabled = (val & (1 << 1)) != 0;
        self.sprite_size = (val & (1 << 2)) != 0;
        self.bg_tile_map_select = (val & (1 << 3)) != 0;
        self.bg_and_win_tile_data_select = (val & (1 << 4)) != 0;
        self.window_display_enabled = (val & (1 << 5)) != 0;
        self.window_tile_map_select = (val & (1 << 6)) != 0;
        self.display_enabled = (val & (1 << 7)) != 0;
    }

    pub fn get(&self) -> u8 {
        let mut val: u8 = 0;
        val |= if self.bg_and_win_display { 0x1 } else { 0x0 };
        val |= if self.sprit_display_enabled { 0x2 } else { 0x0 };
        val |= if self.sprite_size { 0x4 } else { 0x0 };
        val |= if self.bg_tile_map_select { 0x8 } else { 0x0 };
        val |= if self.bg_and_win_tile_data_select { 0x10 } else { 0x0 };
        val |= if self.window_display_enabled { 0x20 } else { 0x0 };
        val |= if self.window_tile_map_select { 0x40 } else { 0x0 };
        val |= if self.display_enabled { 0x80 } else { 0x0 };


        val
    }
}