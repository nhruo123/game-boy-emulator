use core::time::Duration;
use minifb::{Scale, Window, WindowOptions};
use crate::hardware::GameBoyHardware::Key;
use std::collections::HashMap;
use gameboy_core::hardware as GameBoyHardware;
use std::sync::mpsc;
use std::sync::mpsc::{ Sender, Receiver };

pub fn create_app() -> (Hardware, Gui) {
    let (screen_sender, screen_receiver) = mpsc::channel();
    let (keys_sender, keys_receiver) = mpsc::channel();
    let (exit_sender, exit_receiver) = mpsc::channel();
    
    let hardware = Hardware::new(screen_sender, keys_receiver, exit_receiver);
    let gui = Gui::new(screen_receiver, keys_sender, exit_sender);

    (hardware, gui)
}

pub struct Hardware {
    screen_channel: Sender<(Vec<u32>, usize)>,
    keys_channel: Receiver<(Key, bool)>,
    key_state: HashMap<Key, bool>,
    exit_single: Receiver<bool>,
}

pub struct Gui {
    window: Window,
    screen_buffer: Vec<u32>,
    screen_channel: Receiver<(Vec<u32>, usize)>,
    keys_channel: Sender<(Key, bool)>,
    key_state: HashMap<Key, bool>,
    exit_sender: Sender<bool>,
    exit: bool,
}


impl Gui {
    fn new (screen_channel: Receiver<(Vec<u32>, usize)>, keys_channel: Sender<(Key, bool)>, exit_sender: Sender<bool>) -> Self {
        let window = Window::new("game boy", GameBoyHardware::DISPLAY_WIDTH, GameBoyHardware::DISPLAY_HIGHT, WindowOptions {
            resize: false,
            scale: Scale::X4,
            ..WindowOptions::default()
        }).unwrap();

        let mut key_state = HashMap::new();
        key_state.insert(Key::Right, false);
        key_state.insert(Key::Left, false);
        key_state.insert(Key::Up, false);
        key_state.insert(Key::Down, false);
        key_state.insert(Key::A, false);
        key_state.insert(Key::B, false);
        key_state.insert(Key::Select, false);
        key_state.insert(Key::Start, false);

        Self {
            window,
            keys_channel,
            screen_buffer: vec![0; GameBoyHardware::DISPLAY_WIDTH * GameBoyHardware::DISPLAY_HIGHT],
            screen_channel,
            key_state,
            exit_sender,
            exit: false,
        }
    }

    pub fn run(mut self) {
        while !self.exit {
            std::thread::sleep(Duration::from_millis(10));
            self.update_screen();
            self.key_state();
        }
    }

    fn update_screen(&mut self) {
        let (vram, line) = match self.screen_channel.try_recv() {
            Ok(tuple) => tuple,
            Err(err) => { if err == mpsc::TryRecvError::Disconnected { self.close() }; return; }
        };

        let base = GameBoyHardware::DISPLAY_WIDTH * line;
        for i in 0 .. GameBoyHardware::DISPLAY_WIDTH {
            self.screen_buffer[base + i] = vram[i];
        }

        self.window.update_with_buffer(&self.screen_buffer).unwrap();
    }

    fn close(&mut self) {
        self.exit = true;
        self.exit_sender.send(true);
    }

    fn key_state(&mut self) {
        if !self.window.is_open() {
            self.close();
        }

        for (_, v) in self.key_state.iter_mut() {
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
                        self.close();
                        return;
                    }
                    _ => continue,
                };


                match self.key_state.get_mut(&gbk) {
                    Some(v) => *v = true,
                    None => unreachable!(),
                }
            }
        }

        for (key, value) in self.key_state.iter() {
            self.keys_channel.send((Clone::clone(key), Clone::clone(value)));
        }
    }
}


impl Hardware {
    pub fn new(screen_channel: Sender<(Vec<u32>, usize)>, keys_channel: Receiver<(Key, bool)>, exit_single: Receiver<bool>,) -> Self {

        let mut key_state = HashMap::new();
        key_state.insert(Key::Right, false);
        key_state.insert(Key::Left, false);
        key_state.insert(Key::Up, false);
        key_state.insert(Key::Down, false);
        key_state.insert(Key::A, false);
        key_state.insert(Key::B, false);
        key_state.insert(Key::Select, false);
        key_state.insert(Key::Start, false);


        Self {
            screen_channel,
            keys_channel,
            key_state,
            exit_single,
        }
    }
}

impl GameBoyHardware::Hardware for Hardware {
    fn draw_line(&mut self, line: usize, buffer: &[(u8, u8, u8)]) {
        
        let rgb = buffer.iter().map(|(r,g,b)| {
            (*b as u32) | ((*g as u32) << 8) | ((*r as u32) << 16) | (0xFFFF << 24)
        }).collect();


        self.screen_channel.send((rgb, line));
    }

    fn joypad_pressed(&mut self, key: gameboy_core::hardware::Key) -> bool {
        *self.key_state.get_mut(&key).expect("Key err")
    }

    fn clock(&mut self) -> u64 { 
        let epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Couldn't get epoch");

        epoch.as_micros() as u64    
    }

    fn run(&mut self) -> bool {
        match self.exit_single.try_recv() {
            Ok(b) => !b,
            Err(err) => err ==  mpsc::TryRecvError::Empty,
        }
    }
}