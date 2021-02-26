pub const DISPLAY_WIDTH: usize = 160;
pub const DISPLAY_HIGHT: usize = 144;

#[derive(Clone, PartialEq, Hash)]
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
    fn draw_line(&mut self, line: usize, buffer: [(u8, u8, u8)]);

    fn joypad_pressed(&mut self, key: Key) -> bool;


    // The return value needs to be epoch time in microseconds.
    fn clock(&mut self) -> u64;
}