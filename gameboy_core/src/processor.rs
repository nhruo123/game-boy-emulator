

use crate::ic::Ic;
use std::cell::RefCell;
use std::rc::Rc;
use crate::utils;
use crate::mmu::Mmu;


const ZERO_FLAG_MASK: u8 = 0b1 << 7;
const SUB_FLAG_MASK: u8 = 0b1 << 6;
const HALF_CARRY_FLAG_MASK: u8 = 0b1 << 5;
const CARRY_FLAG_MASK: u8 = 0b1 << 4;

pub struct Processor {
    a: u8,
    f: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    
    sp: u16, // stack ptr
    pc: u16, // program counter

    ime: bool, // Interrupt Master Enable Flag 

    halt: bool,
}

impl Processor {

    pub fn new() -> Processor {
        Processor {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: 0,
            h: 0,
            l: 0,
            pc: 0,
            sp: 0xFFFE,
            ime: true,
            halt: false,
        }
    }

    // registers

    // getters
    pub fn get_a(&self) -> u8 {
        self.a
    }

    pub fn get_b(&self) -> u8 {
        self.b
    }

    pub fn get_c(&self) -> u8 {
        self.c
    }

    pub fn get_d(&self) -> u8 {
        self.d
    }

    pub fn get_e(&self) -> u8 {
        self.e
    }
    
    pub fn get_h(&self) -> u8 {
        self.h
    }
        
    pub fn get_l(&self) -> u8 {
        self.l
    }

    // setters 
    pub fn set_a(&mut self, v: u8) {
        self.a = v;
    }

    pub fn set_b(&mut self, v: u8) {
        self.b = v;
    }

    pub fn set_c(&mut self, v: u8) {
        self.c = v;
    }

    pub fn set_d(&mut self, v: u8) {
        self.d = v;
    }

    pub fn set_e(&mut self, v: u8) {
        self.e = v;
    }
    
    pub fn set_h(&mut self, v: u8) {
        self.h = v;
    }
        
    pub fn set_l(&mut self, v: u8) {
        self.l = v;
    }


    // 16 bit regs
    // getters
    pub fn get_af(&self) -> u16 {
        utils::build_u16(self.a, self.f)
    }

    pub fn get_bc(&self) -> u16 {
        utils::build_u16(self.b, self.c)
    }

    pub fn get_de(&self) -> u16 {
        utils::build_u16(self.d, self.e)
    }

    pub fn get_hl(&self) -> u16 {
        utils::build_u16(self.h, self.l)
    }

    pub fn get_sp(&self) -> u16 {
        self.sp
    }

    pub fn get_pc(&self) -> u16 {
        self.pc
    }

    // setters
    pub fn set_af(&mut self, v: u16) {
        let t = utils::split_u16(v);
        self.a = t.0;
        self.f = t.1 & 0xF0;
    }

    pub fn set_bc(&mut self, v: u16) {
        let t = utils::split_u16(v);
        self.b = t.0;
        self.c = t.1;
    }

    pub fn set_de(&mut self, v: u16) {
        let t = utils::split_u16(v);
        self.d = t.0;
        self.e = t.1;
    }

    pub fn set_hl(&mut self, v: u16) {
        let t = utils::split_u16(v);
        self.h = t.0;
        self.l = t.1;
    }

    pub fn set_sp(&mut self, v:u16) {
        self.sp = v
    }

    pub fn set_pc(&mut self, v:u16) {
        self.pc = v
    }

    // flags

    pub fn get_zero_flag(&self) -> bool {
        self.f & ZERO_FLAG_MASK != 0
    }

    pub fn set_zero_flag(&mut self, v: bool) {
        if v {
            self.f |= ZERO_FLAG_MASK;
        } else {
            self.f &= !ZERO_FLAG_MASK;
        }
    }

    pub fn get_sub_flag(&self) -> bool {
        self.f & SUB_FLAG_MASK != 0
    }

    pub fn set_sub_flag(&mut self, v: bool) {
        if v {
            self.f |= SUB_FLAG_MASK;
        } else {
            self.f &= !SUB_FLAG_MASK;
        }
    }

    pub fn get_half_flag(&self) -> bool {
        self.f & HALF_CARRY_FLAG_MASK != 0
    }

    pub fn set_half_flag(&mut self, v: bool) {
        if v {
            self.f |= HALF_CARRY_FLAG_MASK;
        } else {
            self.f &= !HALF_CARRY_FLAG_MASK;
        }
    }

    pub fn get_carry_flag(&self) -> bool {
        self.f & CARRY_FLAG_MASK != 0
    }

    pub fn set_carry_flag(&mut self, v: bool) {
        if v {
            self.f |= CARRY_FLAG_MASK;
        } else {
            self.f &= !CARRY_FLAG_MASK;
        }
    }

    pub fn disable_interrupt(&mut self) {
        self.ime = false;
    }

    pub fn enable_interrupt(&mut self) {
        self.ime = true;
    }


    pub fn push(&mut self, mmu: &mut Mmu, v: u16) {
        self.set_pc(self.get_sp().wrapping_sub(2));

        mmu.write_word(self.get_sp(), v);
    }

    pub fn pop(&mut self, mmu: &mut Mmu) -> u16 {
        let val = mmu.read_word(self.get_sp());

        self.set_pc(self.get_sp().wrapping_add(2));

        val
    }

    
    // read next byte inc program counter
    pub fn fetch_byte(&mut self, mmu: &mut Mmu) -> u8 {
        let b = mmu.read_byte(self.get_pc());
        self.set_pc(self.get_pc().wrapping_add(1));

        b
    }

    // read next word inc program counter
    pub fn fetch_word(&mut self, mmu: &mut Mmu) -> u16 {
        let w = mmu.read_word(self.get_pc());
        self.set_pc(self.get_pc().wrapping_add(2));

        w
    }

    pub fn check_interrupt(&mut self, mmu: &mut Mmu, ic: &Rc<RefCell<Ic>>) -> u32 {
        if !self.ime {
            if self.halt {
                if let Some(value) = ic.borrow_mut().peek() {
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
        self.disable_interrupt();

        self.push(mmu, self.get_pc());
        self.set_pc(value as u16);
    }


    // return machine cycles spend on execution + fetch
    pub fn cycle (&mut self, mmu: &mut Mmu) -> u32 {
        let opcode = self.fetch_byte(mmu);


        // TODO: fill 500 instructions...
        match opcode {
            0x00 => 4, // NOP
            0x01 => { let v = self.fetch_word(mmu); self.set_bc(v); 12 }, // LD BC, U16
            0x02 => { mmu.write_byte(self.get_bc(), self.get_a()); 8 }, // LD (BC),A
            0x03 => { self.set_bc(self.get_bc().wrapping_add(1)); 8 }, // INC BC
            0x04 => { let v = self.aul_inc(self.b); self.set_b(v); 4 }, // INC B
            0x05 => { let v = self.aul_dec(self.b); self.set_b(v); 4 }, // DEC B
            0x06 => { let v = self.fetch_byte(mmu); self.set_b(v); 8 }, // LD B,u8
            0x07 => { let v = self.aul_rlc(self.get_a()); self.set_a(v); 8 }, // RLCA
            0x08 => { let v = self.fetch_word(mmu); mmu.write_word(v, self.get_sp()); 8 }, // LD (u16),SP

            _other => todo!("Unimplemented opcode {:#x}!", opcode),

        }
    }


    // aul operations

    fn aul_inc(&mut self, val: u8) -> u8 {
        self.set_half_flag((val & 0x0F) + 1 > 0x0F);
        let val = val.wrapping_add(1);

        self.set_zero_flag(val == 0);
        self.set_sub_flag(false);

        val
    }

    fn aul_dec(&mut self, val: u8) -> u8 {
        self.set_half_flag((val & 0x0F) == 0);
        let val = val.wrapping_sub(1);

        self.set_zero_flag(val == 0);
        self.set_sub_flag(true);

        val
    }

    fn aul_rlc(&mut self, val: u8) -> u8 {
        let last_bit_was_on = (val & 0x80) != 0;

        self.set_sub_flag(false);
        self.set_half_flag(false);
        self.set_zero_flag(false);

        self.set_carry_flag(last_bit_was_on);

        let mut val = val << 1;

        if last_bit_was_on {
            val |= 0x1;
        }


        val
    }


}