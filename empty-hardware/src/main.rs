mod hardware;

use gameboy_core::emulator;
use gameboy_core::hardware as gameboy_hw;
use hardware::Hardware;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("You must supply the rom name!");
        process::exit(1);
    }

    start_gameboy(args[1].clone());
}

fn start_gameboy(rom: String) {
    let hardware = Hardware::new();
    let hardware_clone = hardware.clone();

    std::thread::spawn(move || {
        let rom = std::fs::read(rom).unwrap();
        let conf = emulator::EmulatorConfig {
            allow_bad_checksum: true,
            game_boy_mode: emulator::GameBoyMode::Classic,
            native_speed: cfg!(debug_assertions), // run on native speed on debug mode
            cpu_speed: gameboy_hw::PROCESSOR_CLOCK_SPEED,
        };

        emulator::run(rom, Box::new(hardware_clone), conf)
    });

    hardware.run();
}
