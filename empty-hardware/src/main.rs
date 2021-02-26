extern crate gameboy_core;

use gameboy_core::emulator;
use gameboy_core::hardware;

struct Empty;


impl hardware::Hardware for Empty {
    fn draw_line(&mut self, _: usize, _line: &[(u8, u8, u8)]) { () }
    fn joypad_pressed(&mut self, _: gameboy_core::hardware::Key) -> bool { false }
    fn clock(&mut self) -> u64 { 
        let epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Couldn't get epoch");

        epoch.as_micros() as u64    
    }
    fn run(&mut self) -> bool { true }
}

fn main() {
    emulator::run(vec![0; 0x2000] , Box::new(Empty{}), emulator::EmulatorConfig { allow_bad_checksum: true, game_boy_mode: emulator::GameBoyMode::Classic })
}
