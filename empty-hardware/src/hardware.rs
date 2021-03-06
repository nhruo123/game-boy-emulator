use core::time::Duration;
use minifb::{Scale, Window, WindowOptions};
use crate::hardware::GameBoyHardware::Key;
use std::collections::HashMap;
use gameboy_core::hardware as GameBoyHardware;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

#[derive(Clone)]
pub struct Hardware {
    screen_buffer: Arc<Mutex<Vec<u32>>>,
    key_state: Arc<Mutex<HashMap<Key, bool>>>,
    exit: Arc<AtomicBool>,
}

struct Gui {
    window: Window,
    screen_buffer: Arc<Mutex<Vec<u32>>>,
    key_state: Arc<Mutex<HashMap<Key, bool>>>,
    exit: Arc<AtomicBool>,
}


impl Gui {
    fn new (screen_buffer: Arc<Mutex<Vec<u32>>>, key_state: Arc<Mutex<HashMap<Key, bool>>>, exit: Arc<AtomicBool>) -> Self {
        let window = Window::new("game boy", GameBoyHardware::DISPLAY_WIDTH, GameBoyHardware::DISPLAY_HIGHT, WindowOptions {
            resize: false,
            scale: Scale::X4,
            ..WindowOptions::default()
        }).unwrap();

        Self {
            window,
            key_state,
            screen_buffer,
            exit,
        }
    }

    fn run(mut self) {
        while !self.exit.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_millis(10));
            self.update_screen();
            self.key_state();
        }
    }

    fn update_screen(&mut self) {
        let vram = self.screen_buffer.lock().unwrap().clone();
        self.window.update_with_buffer(&vram).unwrap();
    }

    fn key_state(&mut self) {
        if !self.window.is_open() {
            self.exit.store(true, Ordering::Relaxed);
        }

        for (_, v) in self.key_state.lock().unwrap().iter_mut() {
            *v = false;
        }

        if let Some(keys) = self.window.get_keys() {
            for k in keys {
                let gbk = match k {
                    minifb::Key::Right => Key::Right,
                    minifb::Key::Left => Key::Left,
                    minifb::Key::Up => Key::Up,
                    minifb::Key::Down => Key::Down,
                    minifb::Key::Z => Key::A,
                    minifb::Key::X => Key::B,
                    minifb::Key::Space => Key::Select,
                    minifb::Key::Enter => Key::Start,
                    minifb::Key::Escape => {
                        self.exit.store(true, Ordering::Relaxed);
                        return;
                    }
                    _ => continue,
                };


                match self.key_state.lock().unwrap().get_mut(&gbk) {
                    Some(v) => *v = true,
                    None => unreachable!(),
                }
            }
        }
    }
}


impl Hardware {
    pub fn new() -> Self {
        let screen_buffer = Arc::new(Mutex::new(vec![0; GameBoyHardware::DISPLAY_WIDTH * GameBoyHardware::DISPLAY_HIGHT]));


        let mut key_state = HashMap::new();
        key_state.insert(Key::Right, false);
        key_state.insert(Key::Left, false);
        key_state.insert(Key::Up, false);
        key_state.insert(Key::Down, false);
        key_state.insert(Key::A, false);
        key_state.insert(Key::B, false);
        key_state.insert(Key::Select, false);
        key_state.insert(Key::Start, false);
        let key_state = Arc::new(Mutex::new(key_state));

        let exit = Arc::new(AtomicBool::new(false));

        Self {
            screen_buffer,
            key_state,
            exit,
        }
    }

    pub fn run(self) {
        let bg = Gui::new(
            self.screen_buffer.clone(),
            self.key_state.clone(),
            self.exit.clone(),
        );
        bg.run();
    }
}

impl GameBoyHardware::Hardware for Hardware {
    fn draw_line(&mut self, line: usize, buffer: &[u32]) {
        let mut screen_buffer = self.screen_buffer.lock().unwrap();

        for i in 0..buffer.len() {
            let base = line * GameBoyHardware::DISPLAY_WIDTH;
            screen_buffer[base + i] = buffer[i];
        }
    }

    fn joypad_pressed(&mut self, key: gameboy_core::hardware::Key) -> bool {
        *self.key_state.lock().unwrap().get_mut(&key).expect("Key err")
    }

    fn clock(&mut self) -> Duration { 
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Couldn't get epoch")
    }

    fn run(&mut self) -> bool {
        !self.exit.load(Ordering::Relaxed)
    }
}