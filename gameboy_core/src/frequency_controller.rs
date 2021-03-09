use crate::processor::TCycles;
use std::time::Duration;
use crate::hardware::Hardware;
use std::cell::RefCell;
use std::rc::Rc;


pub struct FrequencyController {
    hardware: Rc<RefCell<Box<dyn Hardware>>>,
    
    target_freq: u64, // nano sec per cycle

    native_speed: bool,
}

impl FrequencyController {
    pub fn new(hardware: Rc<RefCell<Box<dyn Hardware>>>, target_freq: u64, native_speed: bool) -> Self {
        Self {
            hardware,
            target_freq,
            native_speed,
        }
    }

    pub fn add_delay(&mut self, cycle_start: Duration, cpu_cycles: TCycles) {
        if self.native_speed {
            return;
        }

        let target_time = Duration::from_nanos(self.target_freq * cpu_cycles as u64);
        let mut cycle_end = self.hardware.borrow_mut().clock();

        while (cycle_end - cycle_start) < target_time {
            cycle_end = self.hardware.borrow_mut().clock();
        }
    }
}