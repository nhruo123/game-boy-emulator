use crate::processor::T_CYCLE_FREQUENCY;
use crate::processor::TCycles;

const FRAME_FREQUENCY: u32 = 512;
const FRAME_TO_CYCLE_RATIO: u32 = T_CYCLE_FREQUENCY / FRAME_FREQUENCY;

pub struct FrameSequencer {
    clock: TCycles,
    pub current_cycle: u32,
}

impl FrameSequencer {
    pub fn new() -> Self {
        Self {
            clock: 0,
            current_cycle: 0,
        }
    }

    pub fn cycle(&mut self, clocks: TCycles) -> bool {
        self.clock += clocks;

        if self.clock >= FRAME_TO_CYCLE_RATIO {
            self.clock -= FRAME_TO_CYCLE_RATIO;

            self.current_cycle += 1;
            self.current_cycle %= 8;


            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.clock = 0;
        self.current_cycle = 0;
    }
}