mod hardware;

use gameboy_core::emulator;

fn main() {

    let (hw, gui) = hardware::create_app();

    std::thread::spawn(move || {
        let rom = std::fs::read("E:\\projects\\game-boy-emulator\\roms\\Tetris (World) (Rev A).gb").unwrap();
        // let rom = vec![0; 0x3FFF];
        emulator::run(rom , Box::new(hw) , emulator::EmulatorConfig { allow_bad_checksum: true, game_boy_mode: emulator::GameBoyMode::Classic })
    });

    gui.run();
}
