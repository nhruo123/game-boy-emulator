
use std::time::Duration;

pub const DISPLAY_WIDTH: usize = 160;
pub const DISPLAY_HIGHT: usize = 144;

// In nano seconds
pub const PROCESSOR_CLOCK_SPEED: u64  = 238;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Key {
    Right,
    Left,
    Up,
    Down,
    A,
    B,
    Select,
    Start,
}

pub trait Hardware {
    fn draw_line(&mut self, line: usize, buffer: &[u32]);

    fn joypad_pressed(&mut self, key: Key) -> bool;

    fn clock(&mut self) -> Duration;

    // returns the pcm sample rate on hz
    fn pcm_sample_rate(&mut self) -> u32;

    fn next_pcm_amplitude(&mut self, amplitude: f32);

    fn run(&mut self) -> bool;
}