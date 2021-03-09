use crate::ic::Ic;
use crate::mmu::Mmu;
use crate::processor::decode::Addr;
use crate::processor::decode::Immediate8;
use crate::processor::decode::In8;
use crate::processor::decode::Out8;
use crate::processor::registers::{ Reg16, Reg8, Registers};
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

mod decode;
mod execute;
mod registers;

const ZERO_PAGE_ADDR: u16 = 0xFF00;

pub type TCycles = u32;
pub const T_CYCLE_FREQUENCY: u32 = 4194304;

const FETCH_T_CYCLES: TCycles = 4;
const MEM_ACCESS_T_CYCLES_8: TCycles = 4;
const MEM_ACCESS_T_CYCLES_16: TCycles = MEM_ACCESS_T_CYCLES_8 * 2;

#[derive(Debug)]
pub struct Processor {
    registers: Registers,

    ime: bool, // Interrupt Master Enable Flag

    halt: bool,
}

impl Processor {
    pub fn new() -> Processor {
        Processor {
            registers: Registers::new(),
            ime: true,
            halt: false,
        }
    }

    pub fn push(&mut self, mmu: &mut Mmu, v: u16) {
        let sp = self.registers.read16(Reg16::SP);

        let sp = sp.wrapping_sub(2);

        self.registers.write16(Reg16::SP, sp);

        mmu.write_word(sp, v);
    }

    pub fn pop(&mut self, mmu: &mut Mmu) -> u16 {
        let sp = self.registers.read16(Reg16::SP);

        let val = mmu.read_word(sp);

        self.registers.write16(Reg16::SP, sp.wrapping_add(2));

        val
    }

    // read next byte inc program counter
    pub fn fetch_byte(&mut self, mmu: &mut Mmu) -> u8 {
        let b = mmu.read_byte(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(1);

        b
    }

    // read next word inc program counter
    pub fn fetch_word(&mut self, mmu: &mut Mmu) -> u16 {
        let w = mmu.read_word(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(2);

        w
    }

    pub fn check_interrupt(&mut self, mmu: &mut Mmu, ic: &Rc<RefCell<Ic>>) -> TCycles {
        if !self.ime {
            if self.halt {
                if let Some(_value) = ic.borrow_mut().peek() {
                    self.halt = false;
                }
            }

            0
        } else {
            let value = match ic.borrow_mut().consume() {
                Some(value) => value,
                None => return 0,
            };

            self.interrupted(mmu, value);

            self.halt = false;

            16
        }
    }

    fn interrupted(&mut self, mmu: &mut Mmu, value: u8) {
        self.ime = false;

        self.push(mmu, self.registers.pc);
        self.registers.pc = value as u16;
    }

    fn ctrl_call(&mut self, mmu: &mut Mmu, condition: bool) -> TCycles {
        let addr = self.fetch_word(mmu);

        if condition {
            self.push(mmu, self.registers.pc);
            self.registers.pc = addr;

            (FETCH_T_CYCLES * 4) + MEM_ACCESS_T_CYCLES_16
        } else {
            FETCH_T_CYCLES + MEM_ACCESS_T_CYCLES_16
        }
    }

    fn ctrl_ret(&mut self, mmu: &mut Mmu, condition: bool) -> TCycles {
        if condition {
            let addr = self.pop(mmu);
            self.registers.pc = addr;

            (FETCH_T_CYCLES * 3) + MEM_ACCESS_T_CYCLES_16
        } else {
            FETCH_T_CYCLES * 2
        }
    }

    // alu operations

    fn alu_rlc(&mut self, val: u8) -> u8 {
        let old_msb = (val & 0x80) != 0;

        let mut val = val << 1;

        if old_msb {
            val |= 0x1;
        }

        self.registers.set_zf(val == 0);
        self.registers.set_nf(false);
        self.registers.set_hf(false);
        self.registers.set_cf(old_msb);

        val
    }

    fn alu_rl(&mut self, val: u8) -> u8 {
        let old_msb = (val & 0x80) != 0;

        let val = val << 1;

        let val = val | self.registers.cf() as u8;

        self.registers.set_zf(val == 0);
        self.registers.set_nf(false);
        self.registers.set_hf(false);
        self.registers.set_cf(old_msb);

        val
    }

    fn alu_rrc(&mut self, val: u8) -> u8 {
        let old_lsb = (val & 0x1) != 0;

        let val = val >> 1 | if old_lsb { 0x80 } else { 0 };
        
        self.registers.set_zf(val == 0);
        self.registers.set_nf(false);
        self.registers.set_hf(false);
        self.registers.set_cf(old_lsb);

        val
    }

    fn alu_rr(&mut self, val: u8) -> u8 {
        let old_lsb = (val & 0x1) != 0;

        let val = val >> 1 | if self.registers.cf() { 0x80 } else { 0 };
        
        self.registers.set_zf(val == 0);
        self.registers.set_nf(false);
        self.registers.set_hf(false);
        self.registers.set_cf(old_lsb);

        val
    }

    fn alu_sla(&mut self, val: u8) -> u8 {
        let old_msb = (val & 0x80) != 0;

        let val = val << 1;

        self.registers.set_zf(val == 0);
        self.registers.set_nf(false);
        self.registers.set_hf(false);
        self.registers.set_cf(old_msb);

        val
    }

    fn alu_sra(&mut self, val: u8) -> u8 {
        let msb = val & 0x80;
        let old_lsb = (val & 0x1) != 0;

        let val = (val >> 1) | msb;

        self.registers.set_zf(val == 0);
        self.registers.set_nf(false);
        self.registers.set_hf(false);
        self.registers.set_cf(old_lsb);

        val
    }

    fn alu_srl(&mut self, val: u8) -> u8 {
        let old_lsb = (val & 0x1) != 0;

        let val = val >> 1;

        self.registers.set_zf(val == 0);
        self.registers.set_nf(false);
        self.registers.set_hf(false);
        self.registers.set_cf(old_lsb);

        val
    }

    fn alu_add(&mut self, val: u8, carry: bool) -> u8 {
        let carry = carry as u8;

        let a = self.registers.a;

        let res = a.wrapping_add(val).wrapping_add(carry);

        self.registers.set_zf(res == 0);
        self.registers.set_nf(false);
        self.registers.set_hf((a & 0xF) + (val & 0xF) + carry > 0xF);
        self.registers
            .set_cf((a as u16) + (val as u16) + (carry as u16) > 0xFF);

        res
    }

    fn alu_sub(&mut self, val: u8, carry: bool) -> u8 {
        let carry = carry as u8;

        let a = self.registers.a;

        let res = a.wrapping_sub(carry).wrapping_sub(val);
        self.registers.set_zf(res == 0);
        self.registers.set_nf(true);
        self.registers
            .set_hf((a & 0xF) < (val & 0xF) + (carry & 0xF));
        self.registers
            .set_cf((a as u16) < (val as u16) + (carry as u16));

        res
    }

    fn get_addr(&mut self, addr: Addr, mmu: &mut Mmu) -> (u16, TCycles) {

        match addr {
            Addr::BC => (self.registers.read16(Reg16::BC), 0),
            Addr::DE => (self.registers.read16(Reg16::DE), 0),
            Addr::HL => (self.registers.read16(Reg16::HL), 0),
            Addr::HLD => {
                let addr = self.registers.read16(Reg16::HL);
                self.registers.write16(Reg16::HL, addr.wrapping_sub(1));
                (addr, 0)
            }
            Addr::HLI => {
                let addr = self.registers.read16(Reg16::HL);
                self.registers.write16(Reg16::HL, addr.wrapping_add(1));
                (addr, 0)
            }
            Addr::Immediate16 => (self.fetch_word(mmu), MEM_ACCESS_T_CYCLES_16),
            Addr::ZeroPage => (
                ZERO_PAGE_ADDR | self.fetch_byte(mmu) as u16,
                MEM_ACCESS_T_CYCLES_8,
            ),
            Addr::ZeroPageAndC => (ZERO_PAGE_ADDR | self.registers.c as u16, 0),
        }
    }
}

impl In8<Reg8> for Processor {
    fn read(&mut self, reg: Reg8, _: &mut Mmu) -> (u8, TCycles) {
        (
            match reg {
                Reg8::A => self.registers.a,
                Reg8::B => self.registers.b,
                Reg8::C => self.registers.c,
                Reg8::D => self.registers.d,
                Reg8::E => self.registers.e,
                Reg8::H => self.registers.h,
                Reg8::L => self.registers.l,
            },
            0,
        )
    }
}

impl Out8<Reg8> for Processor {
    fn write(&mut self, reg: Reg8, _: &mut Mmu, val: u8) -> TCycles {
        match reg {
            Reg8::A => self.registers.a = val,
            Reg8::B => self.registers.b = val,
            Reg8::C => self.registers.c = val,
            Reg8::D => self.registers.d = val,
            Reg8::E => self.registers.e = val,
            Reg8::H => self.registers.h = val,
            Reg8::L => self.registers.l = val,
        };

        0
    }
}

impl In8<Immediate8> for Processor {
    fn read(&mut self, _: Immediate8, mmu: &mut Mmu) -> (u8, TCycles) {
        (self.fetch_byte(mmu), MEM_ACCESS_T_CYCLES_8)
    }
}

impl In8<Addr> for Processor {
    fn read(&mut self, src: Addr, mmu: &mut Mmu) -> (u8, TCycles) {
        let (addr, cycles) = self.get_addr(src, mmu);

        (mmu.read_byte(addr), cycles + MEM_ACCESS_T_CYCLES_8)
    }
}

impl Out8<Addr> for Processor {
    fn write(&mut self, src: Addr, mmu: &mut Mmu, val: u8) -> TCycles {
        let (addr, cycles) = self.get_addr(src, mmu);

        mmu.write_byte(addr, val);

        cycles + MEM_ACCESS_T_CYCLES_8
    }
}


impl std::fmt::Display for Processor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.registers)
    }
}