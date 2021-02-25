use crate::mmu::*;
use core::cell::RefCell;
use std::rc::Rc;



pub const V_BLANK_INT: u8 = 0x40;
pub const LCD_STAT_INT: u8 = 0x48;
pub const TIMER_INT: u8 = 0x50;
pub const SERIAL_INT: u8 = 0x48;
pub const JOYPAD_INT: u8 = 0x60;

#[derive(Default)]
struct Interrupts {
    v_blank: bool,
    lcd_stat: bool,
    timer: bool,
    serial: bool,
    joypad: bool,
}

impl Interrupts {

    fn set(&mut self, val: u8) {
        self.v_blank = val & 0x1 != 0;
        self.lcd_stat = val & 0x2 != 0;
        self.timer = val & 0x4 != 0;
        self.serial = val & 0x8 != 0;
        self.joypad = val & 0x10 != 0;
    }

    fn get(& self) -> u8 {
        let mut v = 0;
        v |= if self.v_blank { 0x01 } else { 0x00 };
        v |= if self.lcd_stat { 0x02 } else { 0x00 };
        v |= if self.timer { 0x04 } else { 0x00 };
        v |= if self.serial { 0x08 } else { 0x00 };
        v |= if self.joypad { 0x10 } else { 0x00 };
        v
    }
}


// interrupt request
pub struct Irq {
    request: Rc<RefCell<Interrupts>>
}

impl Irq {
    fn new(request: Rc<RefCell<Interrupts>>) -> Irq {
        Irq {
            request
        }
    }

    pub fn v_blank(&self, v: bool) {
        self.request.borrow_mut().v_blank = v;
    }

    pub fn lcd_stat(&self, v: bool) {
        self.request.borrow_mut().lcd_stat = v;
    }

    pub fn timer(&self, v: bool) {
        self.request.borrow_mut().timer = v;
    }

    pub fn serial(&self, v: bool) {
        self.request.borrow_mut().serial = v;
    }

    pub fn joypad(&self, v: bool) {
        self.request.borrow_mut().joypad = v;
    }
}

// interrupt controller I know its a bad name...
pub struct Ic {
    enabled: Rc<RefCell<Interrupts>>,
    line: Rc<RefCell<Interrupts>>
}


impl Ic {
    pub fn new() -> Ic {
        Ic {
            enabled: Rc::new(RefCell::new(Interrupts::default())),
            line: Rc::new(RefCell::new(Interrupts::default())),
        }
    }

    pub fn get_requester(&self) -> Irq {
        Irq::new(self.line.clone())
    }

    pub fn peek(&self) -> Option<u8> {
        self.read(false)
    }

    pub fn consume(&self) -> Option<u8> {
        self.read(true)
    }


    fn read(&self, consume: bool) -> Option<u8> {
        let enabled = self.enabled.borrow();
        let mut line = self.line.borrow_mut();

        if enabled.v_blank && line.v_blank {
            line.v_blank = !consume;
            Some(V_BLANK_INT)
        } else if enabled.lcd_stat && line.lcd_stat {
            line.lcd_stat = !consume;
            Some(LCD_STAT_INT)
        } else if enabled.timer && line.timer {
            line.timer = !consume;
            Some(TIMER_INT)
        } else if enabled.serial && line.serial {
            line.serial = !consume;
            Some(SERIAL_INT)
        } else if enabled.joypad && line.joypad {
            line.joypad = !consume;
            Some(JOYPAD_INT)
        } else {
            None
        }
    }
}

impl IoDevice for Ic {
    fn read_byte(&mut self, _: &Mmu, adder: u16) -> MemRead {
        if adder == 0xFFF {
            MemRead::Read(self.enabled.borrow().get())
        } else if adder == 0xFF0F {
            MemRead::Read(self.line.borrow().get())
        } else {
            MemRead::Ignore
        }
    }

    fn write_byte(&mut self, _: &Mmu, adder: u16, val: u8) -> MemWrite { 
        if adder == 0xFFF {
            self.enabled.borrow_mut().set(val);
            MemWrite::Write
        } else if adder == 0xFF0F {
            self.line.borrow_mut().set(val);
            MemWrite::Write
        } else {
            MemWrite::Ignore
        }
    }
}