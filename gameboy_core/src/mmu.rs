use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use crate::utils;

// io devices that can be mapped to memory
pub trait IoDevice {
    fn read_byte(&mut self, mmu: &Mmu, addr: u16) -> MemRead;

    fn write_byte(&mut self, mmu: &Mmu, addr: u16, value: u8) -> MemWrite;
}


// every Device that is io mapped will be asked for a MemRead on read and memWrite on write for its mapped range of addresses
pub enum MemRead {
    // The device handled the read and gave a value to return.
    Read(u8),

    // The device isn't sure how to handle the read request.
    Ignore,
}

pub enum MemWrite {
    // The device dealt with the write request
    Write,

    // The device isn't sure how to handle the write request.
    Ignore,
}


pub struct Mmu {
    memory_mapped_devices: HashMap<u16, Vec<Rc<RefCell<dyn IoDevice>>>>
}


impl Mmu {

    pub fn new() -> Mmu {
        Mmu {
            memory_mapped_devices: HashMap::new(),
        }
    }

    pub fn register_device<T>(&mut self, range: (u16, u16), io_device: Rc<RefCell<T>>) where T: IoDevice + 'static {
        for i in range.0 ..= range.1 {
            if self.memory_mapped_devices.contains_key(&i) {
                match self.memory_mapped_devices.get_mut(&i) {
                    Some(v) => v.push(io_device.clone()),
                    None => unreachable!(),
                }
            } else {
                self.memory_mapped_devices.insert(i, vec![io_device.clone()]);
            }
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {



        match self.memory_mapped_devices.get(&addr) {
            Some(devices) => {
                for device in devices {
                    match device.borrow_mut().read_byte(self, addr) {
                        MemRead::Read(data) => return data,
                        MemRead::Ignore => ()
                    }
                }
            },
            None => ()
        }

        // TODO move this to a a new device 
        // this is double speed controller that isn't implement but makes test fail.
        if addr == 0xFF4D {
            // tell the game is unsupported
            return 0xFF
        }

        // no device knows how to deal with read return 0
        return 0;
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        let maybe_devices = self.memory_mapped_devices.get(&addr);
        match maybe_devices {
            Some(devices) => {
                for device in devices {
                    device.borrow_mut().write_byte(self, addr, value);
                    
                }
            },
            None => ()
        }


        // no device knows how deal with write do nothing.
        // dbg!("no device knows how deal with write do nothing");
        return;
    }

    pub fn read_word(&self, addr: u16) -> u16 {
        utils::build_u16(self.read_byte(addr + 1), self.read_byte(addr))
    }

    pub fn write_word(&mut self, addr: u16, value: u16) {
        self.write_byte(addr, utils::get_u16_low(value));
        self.write_byte(addr + 1, utils::get_u16_high(value));
    }
}