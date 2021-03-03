mod hardware;

use gameboy_core::emulator;

use hardware::Hardware;

fn main() {
    let hardware = Hardware::new();
    let hardware_clone = hardware.clone();

    std::thread::spawn(move || {
        let rom = std::fs::read("E:\\projects\\game-boy-emulator\\roms\\08-misc instrs.gb").unwrap();
        // let rom = vec![0; 0x3FFF];
        emulator::run(rom , Box::new(hardware_clone) , emulator::EmulatorConfig { allow_bad_checksum: true, game_boy_mode: emulator::GameBoyMode::Classic })
    });

    hardware.run();
}
