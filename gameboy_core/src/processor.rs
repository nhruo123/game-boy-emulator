

use crate::ic::Ic;
use std::cell::RefCell;
use std::rc::Rc;
use crate::utils;
use crate::mmu::Mmu;


const ZERO_FLAG_MASK: u8 = 0b1 << 7;
const NEG_FLAG_MASK: u8 = 0b1 << 6;
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
        self.a = utils::get_u16_high(v);
        self.f = utils::get_u16_low(v) & 0xF0;
    }

    pub fn set_bc(&mut self, v: u16) {
        self.b = utils::get_u16_high(v);
        self.c = utils::get_u16_low(v);
    }

    pub fn set_de(&mut self, v: u16) {
        self.d = utils::get_u16_high(v);
        self.e = utils::get_u16_low(v);
    }

    pub fn set_hl(&mut self, v: u16) {
        self.h = utils::get_u16_high(v);
        self.l = utils::get_u16_low(v);
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

    pub fn get_neg_flag(&self) -> bool {
        self.f & NEG_FLAG_MASK != 0
    }

    pub fn set_neg_flag(&mut self, v: bool) {
        if v {
            self.f |= NEG_FLAG_MASK;
        } else {
            self.f &= !NEG_FLAG_MASK;
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
        self.set_sp(self.get_sp().wrapping_sub(2));

        mmu.write_word(self.get_sp(), v);
    }

    pub fn pop(&mut self, mmu: &mut Mmu) -> u16 {
        let val = mmu.read_word(self.get_sp());

        self.set_sp(self.get_sp().wrapping_add(2));

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
        self.disable_interrupt();

        self.push(mmu, self.get_pc());
        self.set_pc(value as u16);
    }

    fn stop(&mut self) {
        unimplemented!("stop is unimplemented");
    }

    fn halt(&mut self) {
        self.halt = true;
    }


    // return machine cycles spend on execution + fetch
    pub fn cycle (&mut self, mmu: &mut Mmu) -> u32 {

        if self.halt {
            return 4;
        }

        let opcode = self.fetch_byte(mmu);


        // TODO: fill 500 instructions...
        match opcode {
            0x00 => 4, // NOP
            0x01 => { let v = self.fetch_word(mmu); self.set_bc(v); 12 }, // LD BC, U16
            0x02 => { mmu.write_byte(self.get_bc(), self.get_a()); 8 }, // LD (BC),A
            0x03 => { self.set_bc(self.get_bc().wrapping_add(1)); 8 }, // INC BC
            0x04 => { let v = self.alu_inc(self.get_b()); self.set_b(v); 4 }, // INC B
            0x05 => { let v = self.alu_dec(self.get_b()); self.set_b(v); 4 }, // DEC B
            0x06 => { let v = self.fetch_byte(mmu); self.set_b(v); 8 }, // LD B,u8
            0x07 => { let v = self.alu_rlc(self.get_a()); self.set_a(v); self.set_zero_flag(false); 4 }, // RLCA
            0x08 => { let v = self.fetch_word(mmu); mmu.write_word(v, self.get_sp()); 8 }, // LD (u16),SP
            0x09 => { let v = self.alu_add_16b(self.get_hl(), self.get_bc()); self.set_hl(v) ; 8 }, // ADD HL,BC
            0x0A => { let v = mmu.read_byte(self.get_bc()); self.set_a(v) ; 8 }, // LD A,(BC)
            0x0B => { self.set_bc(self.get_bc().wrapping_add(1)) ; 8 }, // DEC BC
            0x0C => { let v = self.alu_inc(self.get_c()); self.set_c(v); 4 }, // INC C
            0x0D => { let v = self.alu_dec(self.get_c()); self.set_c(v); 4 }, // DEC C
            0x0E => { let v = self.fetch_byte(mmu); self.set_c(v); 8 }, // LD C,u8
            0x0F => { let v = self.alu_rrc(self.get_a()); self.set_a(v); self.set_zero_flag(false); 4 }, // RRCA
            0x10 => { self.stop(); 4 }, // STOP
            0x11 => { let v = self.fetch_word(mmu); self.set_de(v); 12 }, // LD DE,u16
            0x12 => { mmu.write_byte(self.get_de(), self.get_a()); 8 }, // LD (DE),A
            0x13 => { self.set_de(self.get_de().wrapping_add(1)); 8 }, // INC DE
            0x14 => { let v = self.alu_inc(self.get_d()); self.set_d(v); 4 }, // INC D
            0x15 => { let v = self.alu_dec(self.get_d()); self.set_d(v); 4 }, // DEC D
            0x16 => { let v = self.fetch_byte(mmu); self.set_d(v); 4 }, // LD D,u8
            0x17 => { let v = self.alu_rl(self.get_a()); self.set_a(v); self.set_zero_flag(false); 4 }, // RLA
            0x18 => { self.jump_relative(mmu); 12 }, // JR i8
            0x19 => { let v = self.alu_add_16b(self.get_hl(), self.get_de()); self.set_hl(v); 8 }, // ADD HL,DE
            0x1A => { self.set_a(mmu.read_byte(self.get_de())); 8 }, // LD A,(DE)
            0x1B => { self.set_de(self.get_de().wrapping_add(1)) ; 8 }, // DEC DE
            0x1C => { let v = self.alu_inc(self.get_e()); self.set_e(v); 4 }, // INC E
            0x1D => { let v = self.alu_dec(self.get_e()); self.set_e(v); 4 }, // DEC E
            0x1E => { let v = self.fetch_byte(mmu); self.set_e(v); 8 }, // LD E,u8
            0x1F => { let v = self.alu_rr(self.get_a()); self.set_a(v); self.set_zero_flag(false); 4 }, // RRA
            0x20 => if !self.get_zero_flag() {self.jump_relative(mmu); 12} else { self.set_pc(self.get_pc() + 1); 8 }, // JR NZ,i8
            0x21 => { let v = self.fetch_word(mmu); self.set_hl(v); 12 }, // LD HL,u16
            0x22 => { let v = self.get_hl(); self.set_hl(v.wrapping_add(1)); mmu.write_byte(v, self.get_a()); 8 }, // LD (HL+),A
            0x23 => { self.set_hl(self.get_hl().wrapping_add(1)); 8 }, // INC HL
            0x24 => { let v = self.alu_inc(self.get_h()); self.set_h(v); 4 }, // INC H
            0x25 => { let v = self.alu_dec(self.get_h()); self.set_h(v); 4 }, // DEC H
            0x26 => { let v = self.fetch_byte(mmu); self.set_h(v); 8 }, // LD H,u8
            0x27 => { self.alu_daa(); 4 }, // DAA
            0x28 => if self.get_zero_flag() { self.jump_relative(mmu); 12} else { self.set_pc(self.get_pc() + 1); 8 }, // JR Z,i8
            0x29 => { let v = self.alu_add_16b(self.get_hl(), self.get_hl()); self.set_hl(v); 8 }, // ADD HL,HL
            0x2A => { let v = self.get_hl(); self.set_hl(v.wrapping_add(1)); self.set_a(mmu.read_byte(v)); 8 }, // LD A,(HL+)
            0x2B => { self.set_hl(self.get_hl().wrapping_add(1)) ; 8 }, // DEC HL
            0x2C => { let v = self.alu_inc(self.get_l()); self.set_l(v); 4 }, // INC L
            0x2D => { let v = self.alu_dec(self.get_l()); self.set_l(v); 4 }, // DEC L
            0x2E => { let v = self.fetch_byte(mmu); self.set_l(v); 8 }, // LD L,u8
            0x2F => { self.set_a(!self.get_a()); self.set_half_flag(true); self.set_neg_flag(true); 4 }, // CPL
            0x30 => if !self.get_carry_flag() { self.jump_relative(mmu); 12 } else { self.set_pc(self.get_pc() + 1); 8 }, // JR NC,i8
            0x31 => { let v = self.fetch_word(mmu); self.set_sp(v); 12 }, // LD SP,u16
            0x32 => { let v = self.get_hl(); self.set_hl(v.wrapping_sub(1)); mmu.write_byte(v, self.get_a()); 8 }, // LD (HL-),A
            0x33 => { self.set_sp(self.get_sp().wrapping_add(1)); 8 }, // INC SP
            0x34 => { let v = mmu.read_byte(self.get_hl()); mmu.write_byte(self.get_hl(), self.alu_inc(v)); 12 }, // INC (HL)
            0x35 => { let v = mmu.read_byte(self.get_hl()); mmu.write_byte(self.get_hl(), self.alu_dec(v)); 12 }, // DEC (HL)
            0x36 => { let v = self.fetch_byte(mmu); mmu.write_byte(self.get_hl(), v); 12 }, // LD (HL),u8
            0x37 => { self.set_carry_flag(true); self.set_neg_flag(false); self.set_half_flag(false); 4 }, // SCF
            0x38 => if self.get_carry_flag() { self.jump_relative(mmu); 12 } else { self.set_pc(self.get_pc() + 1); 8 }, // JR C,i8
            0x39 => { let v = self.alu_add_16b(self.get_hl(), self.get_hl()); self.set_sp(v); 8 }, // ADD HL,SP
            0x3A => { let v = self.get_hl(); self.set_hl(v.wrapping_sub(1)); self.set_a(mmu.read_byte(v)); 8 }, // LD A,(HL-)
            0x3B => { self.set_sp(self.get_sp().wrapping_sub(1)); 8 }, // DEC SP
            0x3C => { let v = self.alu_inc(self.get_a()); self.set_a(v); 4 }, // INC A
            0x3D => { let v = self.alu_dec(self.get_a()); self.set_a(v); 4 }, // DEC A
            0x3E => { let v = self.fetch_byte(mmu); self.set_a(v); 8 }, // LD A,u8
            0x3F => { self.set_carry_flag(!self.get_carry_flag()); self.set_neg_flag(false); self.set_half_flag(false); 4 }, // CCF
            0x40 => 4, // LD B, B
            0x41 => { self.set_b(self.get_c()); 4 }, // LD B, C
            0x42 => { self.set_b(self.get_d()); 4 }, // LD B, D
            0x43 => { self.set_b(self.get_e()); 4 }, // LD B, E
            0x44 => { self.set_b(self.get_h()); 4 }, // LD B, H
            0x45 => { self.set_b(self.get_l()); 4 }, // LD B, L
            0x46 => { self.set_b(mmu.read_byte(self.get_hl())); 8 }, // LD B, (HL)
            0x47 => { self.set_b(self.get_a()); 4 }, // LD B, A
            0x48 => { self.set_c(self.get_b()); 4 }, // LD C, B
            0x49 => 4, // LD C, C
            0x4A => { self.set_c(self.get_d()); 4 }, // LD C, D
            0x4B => { self.set_c(self.get_e()); 4 }, // LD C, E
            0x4C => { self.set_c(self.get_h()); 4 }, // LD C, H
            0x4D => { self.set_c(self.get_l()); 4 }, // LD C, L
            0x4E => { self.set_c(mmu.read_byte(self.get_hl())); 8 }, // LD C, (HL)
            0x4F => { self.set_c(self.get_a()); 4 }, // LD C, A
            0x50 => { self.set_d(self.get_b()); 4 }, // LD D, B
            0x51 => { self.set_d(self.get_c()); 4 }, // LD D, C
            0x52 => { 4 }, // LD D, D
            0x53 => { self.set_d(self.get_e()); 4 }, // LD D, E
            0x54 => { self.set_d(self.get_h()); 4 }, // LD D, H
            0x55 => { self.set_d(self.get_l()); 4 }, // LD D, L
            0x56 => { self.set_d(mmu.read_byte(self.get_hl())); 8 }, // LD D, (HL)
            0x57 => { self.set_d(self.get_a()); 4 }, // LD D, A
            0x58 => { self.set_e(self.get_b()); 4 }, // LD E, B
            0x59 => { self.set_e(self.get_c()); 4 }, // LD E, C
            0x5A => { self.set_e(self.get_d()); 4 }, // LD E, D
            0x5B => { 4 }, // LD E, E
            0x5C => { self.set_e(self.get_h()); 4 }, // LD E, H
            0x5D => { self.set_e(self.get_l()); 4 }, // LD E, L
            0x5E => { self.set_e(mmu.read_byte(self.get_hl())); 8 }, // LD E, (HL)
            0x5F => { self.set_e(self.get_a()); 4 }, // LD E, A
            0x60 => { self.set_h(self.get_b()); 4 }, // LD H, B
            0x61 => { self.set_h(self.get_c()); 4 }, // LD H, C
            0x62 => { self.set_h(self.get_d()); 4 }, // LD H, D
            0x63 => { self.set_h(self.get_e()); 4 }, // LD H, E
            0x64 => { 4 }, // LD H, H
            0x65 => { self.set_h(self.get_l()); 4 }, // LD H, L
            0x66 => { self.set_h(mmu.read_byte(self.get_hl())); 8 }, // LD H, (HL)
            0x67 => { self.set_h(self.get_a()); 4 }, // LD H, A
            0x68 => { self.set_l(self.get_b()); 4 }, // LD L, B
            0x69 => { self.set_l(self.get_c()); 4 }, // LD L, C
            0x6A => { self.set_l(self.get_d()); 4 }, // LD L, D
            0x6B => { self.set_l(self.get_e()); 4 }, // LD L, E
            0x6C => { self.set_l(self.get_h()); 4 }, // LD L, H
            0x6D => { 4 }, // LD L, L
            0x6E => { self.set_l(mmu.read_byte(self.get_hl())); 8 }, // LD L, (HL)
            0x6F => { self.set_l(self.get_a()); 4 }, // LD L, A
            0x70 => { mmu.write_byte(self.get_hl(), self.get_b()); 8 }, // LD (HL), B
            0x71 => { mmu.write_byte(self.get_hl(), self.get_c()); 8 }, // LD (HL), C
            0x72 => { mmu.write_byte(self.get_hl(), self.get_d()); 8 }, // LD (HL), D
            0x73 => { mmu.write_byte(self.get_hl(), self.get_h()); 8 }, // LD (HL), E
            0x74 => { mmu.write_byte(self.get_hl(), self.get_h()); 8 }, // LD (HL), H
            0x75 => { mmu.write_byte(self.get_hl(), self.get_l()); 8 }, // LD (HL), L
            0x76 => { self.halt(); 4 } // HALT
            0x77 => { mmu.write_byte(self.get_hl(), self.get_a()); 8 }, // LD (HL), A
            0x78 => { self.set_a(self.get_b()); 4 }, // LD A, B
            0x79 => { self.set_a(self.get_c()); 4 }, // LD A, C
            0x7A => { self.set_a(self.get_d()); 4 }, // LD A, D
            0x7B => { self.set_a(self.get_e()); 4 }, // LD A, E
            0x7C => { self.set_a(self.get_h()); 4 }, // LD A, H
            0x7D => { self.set_a(self.get_l()); 4 }, // LD A, L
            0x7E => { self.set_a(mmu.read_byte(self.get_hl())); 8 }, // LD A, (HL)
            0x7F => { 4 }, // LD A, A
            0x80 => { self.alu_add_a(self.get_b(), false); 4 }, // ADD A, B
            0x81 => { self.alu_add_a(self.get_c(), false); 4 }, // ADD A, C
            0x82 => { self.alu_add_a(self.get_d(), false); 4 }, // ADD A, D
            0x83 => { self.alu_add_a(self.get_e(), false); 4 }, // ADD A, E
            0x84 => { self.alu_add_a(self.get_h(), false); 4 }, // ADD A, H
            0x85 => { self.alu_add_a(self.get_l(), false); 4 }, // ADD A, L
            0x86 => { self.alu_add_a(mmu.read_byte(self.get_hl()), false); 8 }, // ADD A, (HL)
            0x87 => { self.alu_add_a(self.get_a(), false); 4 }, // ADD A, A
            0x88 => { self.alu_add_a(self.get_b(), true); 4 }, // ADC A, B
            0x89 => { self.alu_add_a(self.get_c(), true); 4 }, // ADC A, C
            0x8A => { self.alu_add_a(self.get_d(), true); 4 }, // ADC A, D
            0x8B => { self.alu_add_a(self.get_e(), true); 4 }, // ADC A, E
            0x8C => { self.alu_add_a(self.get_h(), true); 4 }, // ADC A, H
            0x8D => { self.alu_add_a(self.get_l(), true); 4 }, // ADC A, L
            0x8E => { self.alu_add_a(mmu.read_byte(self.get_hl()), true); 8 }, // ADC A, (HL)
            0x8F => { self.alu_add_a(self.get_a(), true); 4 }, // ADC A, A
            0x90 => { self.alu_sub_a(self.get_b(), false); 4 }, // SUB A, B
            0x91 => { self.alu_sub_a(self.get_c(), false); 4 }, // SUB A, C
            0x92 => { self.alu_sub_a(self.get_d(), false); 4 }, // SUB A, D
            0x93 => { self.alu_sub_a(self.get_e(), false); 4 }, // SUB A, E
            0x94 => { self.alu_sub_a(self.get_h(), false); 4 }, // SUB A, H
            0x95 => { self.alu_sub_a(self.get_l(), false); 4 }, // SUB A, L
            0x96 => { self.alu_sub_a(mmu.read_byte(self.get_hl()), false); 8 }, // SUB A, (HL)
            0x97 => { self.alu_sub_a(self.get_a(), false); 4 }, // SUB A, A
            0x98 => { self.alu_sub_a(self.get_b(), true); 4 }, // SBC A, B
            0x99 => { self.alu_sub_a(self.get_c(), true); 4 }, // SBC A, C
            0x9A => { self.alu_sub_a(self.get_d(), true); 4 }, // SBC A, D
            0x9B => { self.alu_sub_a(self.get_e(), true); 4 }, // SBC A, E
            0x9C => { self.alu_sub_a(self.get_h(), true); 4 }, // SBC A, H
            0x9D => { self.alu_sub_a(self.get_l(), true); 4 }, // SBC A, L
            0x9E => { self.alu_sub_a(mmu.read_byte(self.get_hl()), true); 8 }, // SBC A, (HL)
            0x9F => { self.alu_sub_a(self.get_a(), true); 4 }, // SBC A, A
            0xA0 => { self.alu_and_a(self.get_b()); 4 }, // AND A, B
            0xA1 => { self.alu_and_a(self.get_c()); 4 }, // AND A, C
            0xA2 => { self.alu_and_a(self.get_d()); 4 }, // AND A, D
            0xA3 => { self.alu_and_a(self.get_e()); 4 }, // AND A, E
            0xA4 => { self.alu_and_a(self.get_h()); 4 }, // AND A, H
            0xA5 => { self.alu_and_a(self.get_l()); 4 }, // AND A, L
            0xA6 => { self.alu_and_a(mmu.read_byte(self.get_hl())); 8 }, // AND A, (HL)
            0xA7 => { self.alu_and_a(self.get_a()); 4 }, // AND A, A
            0xA8 => { self.alu_xor_a(self.get_b()); 4 }, // XOR A, B
            0xA9 => { self.alu_xor_a(self.get_c()); 4 }, // XOR A, C
            0xAA => { self.alu_xor_a(self.get_d()); 4 }, // XOR A, D
            0xAB => { self.alu_xor_a(self.get_e()); 4 }, // XOR A, E
            0xAC => { self.alu_xor_a(self.get_h()); 4 }, // XOR A, H
            0xAD => { self.alu_xor_a(self.get_l()); 4 }, // XOR A, L
            0xAE => { self.alu_xor_a(mmu.read_byte(self.get_hl())); 8 }, // XOR A, (HL)
            0xAF => { self.alu_xor_a(self.get_a()); 4 }, // XOR A, A
            0xB0 => { self.alu_or_a(self.get_b()); 4 }, // OR A, B
            0xB1 => { self.alu_or_a(self.get_c()); 4 }, // OR A, C
            0xB2 => { self.alu_or_a(self.get_d()); 4 }, // OR A, D
            0xB3 => { self.alu_or_a(self.get_e()); 4 }, // OR A, E
            0xB4 => { self.alu_or_a(self.get_h()); 4 }, // OR A, H
            0xB5 => { self.alu_or_a(self.get_l()); 4 }, // OR A, L
            0xB6 => { self.alu_or_a(mmu.read_byte(self.get_hl())); 8 }, // OR A, (HL)
            0xB7 => { self.alu_or_a(self.get_a()); 4 }, // OR A, A
            0xB8 => { self.alu_cp_a(self.get_b()); 4 }, // CP A, B
            0xB9 => { self.alu_cp_a(self.get_c()); 4 }, // CP A, C
            0xBA => { self.alu_cp_a(self.get_d()); 4 }, // CP A, D
            0xBB => { self.alu_cp_a(self.get_e()); 4 }, // CP A, E
            0xBC => { self.alu_cp_a(self.get_h()); 4 }, // CP A, H
            0xBD => { self.alu_cp_a(self.get_l()); 4 }, // CP A, L
            0xBE => { self.alu_cp_a(mmu.read_byte(self.get_hl())); 8 }, // CP A, (HL)
            0xBF => { self.alu_cp_a(self.get_a()); 4 }, // CP A, A
            0xC0 => if !self.get_zero_flag() { let v = self.pop(mmu) ; self.set_pc(v); 20 } else { 8 }, // RET NZ
            0xC1 => { let v = self.pop(mmu); self.set_bc(v) ; 12}, // POP BC
            0xC2 => if !self.get_zero_flag() { let v = self.fetch_word(mmu); self.set_pc(v); 16 } else { self.set_pc(self.get_pc().wrapping_add(2)); 12 }, // JP NZ,u16
            0xC3 => { let v = self.fetch_word(mmu); self.set_pc(v); 16 } // JP u16
            0xC4 => if !self.get_zero_flag() { let v = self.fetch_word(mmu); self.push(mmu, self.pc); self.set_pc(v); 24 } 
                    else { self.set_pc(self.get_pc().wrapping_add(2)); 12 } // CALL NZ,u16
            0xC5 => { self.push(mmu, self.get_bc()); 16 } // PUSH BC
            0xC6 => { let v = self.fetch_byte(mmu); self.alu_add_a(v, false); 8 } // ADD A,u8
            0xC7 => { self.push(mmu, self.get_pc()); self.set_pc(0); 16 } // RST 00h
            0xC8 => if self.get_zero_flag() { let v = self.pop(mmu); self.set_pc(v); 20 } else { 8 }  // RET Z
            0xC9 => { let v = self.pop(mmu); self.set_pc(v); 16 } // RET
            0xCA => if self.get_zero_flag() { let v = self.fetch_word(mmu); self.set_pc(v); 16 } else { self.set_pc(self.get_pc().wrapping_add(2)); 12 }, // JP Z,u16
            0xCB => self.decode_and_execute_bc_opcodes(mmu), // PREFIX CB
            0xCC => if self.get_zero_flag() { let v = self.fetch_word(mmu); self.push(mmu, self.pc); self.set_pc(v); 24 } 
                else { self.set_pc(self.get_pc().wrapping_add(2)); 12 } // CALL Z,u16
            0xCD => { let v = self.fetch_word(mmu); self.push(mmu, self.pc); self.set_pc(v); 24 } // CALL u16
            0xCE => { let v = self.fetch_byte(mmu); self.alu_add_a(v, true); 8 } // ADC A,u8
            0xCF => { self.push(mmu, self.get_pc()); self.set_pc(0x8); 16 } // RST 08h
            0xD0 => if !self.get_carry_flag() { let v = self.pop(mmu) ; self.set_pc(v); 20 } else { 8 }, // RET NC
            0xD1 => { let v = self.pop(mmu); self.set_de(v) ; 12}, // POP DE
            0xD2 => if !self.get_carry_flag() { let v = self.fetch_word(mmu); self.set_pc(v); 16 } else { self.set_pc(self.get_pc().wrapping_add(2)); 12 }, // JP NC,u16
            0xD4 => if !self.get_carry_flag() { let v = self.fetch_word(mmu); self.push(mmu, self.pc); self.set_pc(v); 24 } 
                    else { self.set_pc(self.get_pc().wrapping_add(2)); 12 } // CALL NC,u16
            0xD5 => { self.push(mmu, self.get_de()); 16 } // PUSH DE
            0xD6 => { let v = self.fetch_byte(mmu); self.alu_sub_a(v, false); 8 } // SUB A,u8
            0xD7 => { self.push(mmu, self.get_pc()); self.set_pc(0x10); 16 } // RST 10h
            0xD8 => if self.get_carry_flag() { let v = self.pop(mmu); self.set_pc(v); 20 } else { 8 }  // RET C
            0xD9 => { let v = self.pop(mmu); self.set_pc(v); self.enable_interrupt(); 16 } // RETI
            0xDA => if self.get_carry_flag() { let v = self.fetch_word(mmu); self.set_pc(v); 16 } else { self.set_pc(self.get_pc().wrapping_add(2)); 12 }, // JP C,u16
            0xDC => if self.get_carry_flag() { let v = self.fetch_word(mmu); self.push(mmu, self.pc); self.set_pc(v); 24 } 
                else { self.set_pc(self.get_pc().wrapping_add(2)); 12 } // CALL C,u16
            0xDE => { let v = self.fetch_byte(mmu); self.alu_sub_a(v, true); 8 } // SBC A,u8
            0xDF => { self.push(mmu, self.get_pc()); self.set_pc(0x18); 16 } // RST 18h
            0xE0 => { let v = self.fetch_byte(mmu); mmu.write_byte(0xFF00 | v as u16, self.get_a()); 12 }, // LD (FF00+u8),A
            0xE1 => { let v = self.pop(mmu); self.set_hl(v) ; 12}, // POP HL
            0xE2 => { mmu.write_byte(0xFF00 | (self.get_c() as u16), self.get_a()); 8 }, // LD (FF00+c),A
            0xE5 => { self.push(mmu, self.get_hl()); 16 } // PUSH HL
            0xE6 => { let v = self.fetch_byte(mmu); self.alu_and_a(v); 8 } // AND A,u8
            0xE7 => { self.push(mmu, self.get_pc()); self.set_pc(0x20); 16 } // RST 20h
            0xE8 => { let nb = self.fetch_byte(mmu); let v = self.alu_add_16b(self.get_sp(), nb as u16); self.set_sp(v); self.set_zero_flag(false); 16 }, // ADD SP,i8
            0xE9 => { self.set_pc(self.get_hl()) ; 4 }, // JP HL
            0xEA => { let v = self.fetch_word(mmu); mmu.write_byte(v, self.get_a()); 8 }, // LD (u16),A 
            0xEE => { let v = self.fetch_byte(mmu); self.alu_xor_a(v); 8 } // XOR A,u8
            0xEF => { self.push(mmu, self.get_pc()); self.set_pc(0x28); 16 } // RST 28h
            0xF0 => { let v = self.fetch_byte(mmu); self.set_a(mmu.read_byte(0xFF00 | v as u16)); 12 }, // LD A,(FF00+u8)
            0xF1 => { let v = self.pop(mmu); self.set_af(v);  12 } // POP AF
            0xF2 => { self.set_a(mmu.read_byte(0xFF00 | self.get_c() as u16)); 12 }, // LD A,(FF00+C)
            0xF3 => { self.disable_interrupt(); 4 } // DI
            0xF5 => { self.push(mmu, self.get_af()); 16 } // PUSH AF
            0xF6 => { let v = self.fetch_byte(mmu); self.alu_or_a(v); 8 } // OR A,u8
            0xF7 => { self.push(mmu, self.get_pc()); self.set_pc(0x30); 16 } // RST 30h
            0xF8 => { let nb = self.fetch_byte(mmu); let v = self.alu_add_16b(self.get_sp(), nb as u16); self.set_hl(v); self.set_zero_flag(false); 12 }, // LD HL,SP+i8
            0xF9 => { self.set_sp(self.get_hl()); 8 } // LD SP,HL
            0xFA => { let v = self.fetch_word(mmu); self.set_a(mmu.read_byte(v)) ;16 } // LD A,(u16)
            0xFB => { self.enable_interrupt(); 4 } // EI
            0xFE => { let v = self.fetch_byte(mmu); self.alu_cp_a(v); 8 } // CP A,u8
            0xFF => { self.push(mmu, self.get_pc()); self.set_pc(0x38); 16 } // RST 38h

            opcode => panic!("bad opcode {:#x}!", opcode),
        }
    }

    pub fn decode_and_execute_bc_opcodes (&mut self, mmu: &mut Mmu) -> u32 {
        let opcode = self.fetch_byte(mmu);

        match opcode {
            0x00 => { let v = self.alu_rlc(self.get_b()); self.set_b(v); 8 }, // RLC B
            0x01 => { let v = self.alu_rlc(self.get_c()); self.set_c(v); 8 }, // RLC C
            0x02 => { let v = self.alu_rlc(self.get_d()); self.set_d(v); 8 }, // RLC D
            0x03 => { let v = self.alu_rlc(self.get_e()); self.set_e(v); 8 }, // RLC E
            0x04 => { let v = self.alu_rlc(self.get_h()); self.set_h(v); 8 }, // RLC H
            0x05 => { let v = self.alu_rlc(self.get_l()); self.set_l(v); 8 }, // RLC L
            0x06 => { let v = self.alu_rlc(mmu.read_byte(self.get_hl())); mmu.write_byte(self.get_hl(), v); 16 }, // RLC (HL)
            0x07 => { let v = self.alu_rlc(self.get_a()); self.set_a(v); 8 }, // RLC A

            0x08 => { let v = self.alu_rrc(self.get_b()); self.set_b(v); 8 }, // RRC B
            0x09 => { let v = self.alu_rrc(self.get_c()); self.set_c(v); 8 }, // RRC C
            0x0A => { let v = self.alu_rrc(self.get_d()); self.set_d(v); 8 }, // RRC D
            0x0B => { let v = self.alu_rrc(self.get_e()); self.set_e(v); 8 }, // RRC E
            0x0C => { let v = self.alu_rrc(self.get_h()); self.set_h(v); 8 }, // RRC H
            0x0D => { let v = self.alu_rrc(self.get_l()); self.set_l(v); 8 }, // RRC L
            0x0E => { let v = self.alu_rrc(mmu.read_byte(self.get_hl())); mmu.write_byte(self.get_hl(), v); 16 }, // RRC (HL)
            0x0F => { let v = self.alu_rrc(self.get_a()); self.set_a(v); 8 }, // RRC A

            0x10 => { let v = self.alu_rl(self.get_b()); self.set_b(v); 8 }, // RL B
            0x11 => { let v = self.alu_rl(self.get_c()); self.set_c(v); 8 }, // RL C
            0x12 => { let v = self.alu_rl(self.get_d()); self.set_d(v); 8 }, // RL D
            0x13 => { let v = self.alu_rl(self.get_e()); self.set_e(v); 8 }, // RL E
            0x14 => { let v = self.alu_rl(self.get_h()); self.set_h(v); 8 }, // RL H
            0x15 => { let v = self.alu_rl(self.get_l()); self.set_l(v); 8 }, // RL L
            0x16 => { let v = self.alu_rl(mmu.read_byte(self.get_hl())); mmu.write_byte(self.get_hl(), v); 16 }, // RL (HL)
            0x17 => { let v = self.alu_rl(self.get_a()); self.set_a(v); 8 }, // RL A

            0x18 => { let v = self.alu_rr(self.get_b()); self.set_b(v); 8 }, // RR B
            0x19 => { let v = self.alu_rr(self.get_c()); self.set_c(v); 8 }, // RR C
            0x1A => { let v = self.alu_rr(self.get_d()); self.set_d(v); 8 }, // RR D
            0x1B => { let v = self.alu_rr(self.get_e()); self.set_e(v); 8 }, // RR E
            0x1C => { let v = self.alu_rr(self.get_h()); self.set_h(v); 8 }, // RR H
            0x1D => { let v = self.alu_rr(self.get_l()); self.set_l(v); 8 }, // RR L
            0x1E => { let v = self.alu_rr(mmu.read_byte(self.get_hl())); mmu.write_byte(self.get_hl(), v); 16 }, // RR (HL)
            0x1F => { let v = self.alu_rr(self.get_a()); self.set_a(v); 8 }, // RR A

            0x20 => { let v = self.alu_sla(self.get_b()); self.set_b(v); 8 }, // SLA B
            0x21 => { let v = self.alu_sla(self.get_c()); self.set_c(v); 8 }, // SLA C
            0x22 => { let v = self.alu_sla(self.get_d()); self.set_d(v); 8 }, // SLA D
            0x23 => { let v = self.alu_sla(self.get_e()); self.set_e(v); 8 }, // SLA E
            0x24 => { let v = self.alu_sla(self.get_h()); self.set_h(v); 8 }, // SLA H
            0x25 => { let v = self.alu_sla(self.get_l()); self.set_l(v); 8 }, // SLA L
            0x26 => { let v = self.alu_sla(mmu.read_byte(self.get_hl())); mmu.write_byte(self.get_hl(), v); 16 }, // SLA (HL)
            0x27 => { let v = self.alu_sla(self.get_a()); self.set_a(v); 8 }, // SLA A

            0x28 => { let v = self.alu_sra(self.get_b()); self.set_b(v); 8 }, // SRA B
            0x29 => { let v = self.alu_sra(self.get_c()); self.set_c(v); 8 }, // SRA C
            0x2A => { let v = self.alu_sra(self.get_d()); self.set_d(v); 8 }, // SRA D
            0x2B => { let v = self.alu_sra(self.get_e()); self.set_e(v); 8 }, // SRA E
            0x2C => { let v = self.alu_sra(self.get_h()); self.set_h(v); 8 }, // SRA H
            0x2D => { let v = self.alu_sra(self.get_l()); self.set_l(v); 8 }, // SRA L
            0x2E => { let v = self.alu_sra(mmu.read_byte(self.get_hl())); mmu.write_byte(self.get_hl(), v); 16 }, // SRA (HL)
            0x2F => { let v = self.alu_sra(self.get_a()); self.set_a(v); 8 }, // SRA A

            0x30 => { let v = self.alu_swap(self.get_b()); self.set_b(v); 8 }, // SWAP B
            0x31 => { let v = self.alu_swap(self.get_c()); self.set_c(v); 8 }, // SWAP C
            0x32 => { let v = self.alu_swap(self.get_d()); self.set_d(v); 8 }, // SWAP D
            0x33 => { let v = self.alu_swap(self.get_e()); self.set_e(v); 8 }, // SWAP E
            0x34 => { let v = self.alu_swap(self.get_h()); self.set_h(v); 8 }, // SWAP H
            0x35 => { let v = self.alu_swap(self.get_l()); self.set_l(v); 8 }, // SWAP L
            0x36 => { let v = self.alu_swap(mmu.read_byte(self.get_hl())); mmu.write_byte(self.get_hl(), v); 16 }, // SWAP (HL)
            0x37 => { let v = self.alu_swap(self.get_a()); self.set_a(v); 8 }, // SWAP A

            0x38 => { let v = self.alu_srl(self.get_b()); self.set_b(v); 8 }, // SRL B
            0x39 => { let v = self.alu_srl(self.get_c()); self.set_c(v); 8 }, // SRL C
            0x3A => { let v = self.alu_srl(self.get_d()); self.set_d(v); 8 }, // SRL D
            0x3B => { let v = self.alu_srl(self.get_e()); self.set_e(v); 8 }, // SRL E
            0x3C => { let v = self.alu_srl(self.get_h()); self.set_h(v); 8 }, // SRL H
            0x3D => { let v = self.alu_srl(self.get_l()); self.set_l(v); 8 }, // SRL L
            0x3E => { let v = self.alu_srl(mmu.read_byte(self.get_hl())); mmu.write_byte(self.get_hl(), v); 16 }, // SRL (HL)
            0x3F => { let v = self.alu_srl(self.get_a()); self.set_a(v); 8 }, // SRL A

            0x40 => { self.alu_bit(self.get_b(), 0); 8 }, // BIT 0 B
            0x41 => { self.alu_bit(self.get_c(), 0); 8 }, // BIT 0 C
            0x42 => { self.alu_bit(self.get_d(), 0); 8 }, // BIT 0 D
            0x43 => { self.alu_bit(self.get_e(), 0); 8 }, // BIT 0 E
            0x44 => { self.alu_bit(self.get_h(), 0); 8 }, // BIT 0 H
            0x45 => { self.alu_bit(self.get_l(), 0); 8 }, // BIT 0 L
            0x46 => { self.alu_bit(mmu.read_byte(self.get_hl()), 0); 16 }, // BIT 0 (HL)
            0x47 => { self.alu_bit(self.get_a(), 0); 8 }, // BIT 0 A

            0x48 => { self.alu_bit(self.get_b(), 1); 8  }, // BIT 1 B
            0x49 => { self.alu_bit(self.get_c(), 1); 8  }, // BIT 1 C
            0x4A => { self.alu_bit(self.get_d(), 1); 8  }, // BIT 1 D
            0x4B => { self.alu_bit(self.get_e(), 1); 8  }, // BIT 1 E
            0x4C => { self.alu_bit(self.get_h(), 1); 8  }, // BIT 1 H
            0x4D => { self.alu_bit(self.get_l(), 1); 8  }, // BIT 1 L
            0x4E => { self.alu_bit(mmu.read_byte(self.get_hl()), 1); 16 }, // BIT 1 (HL)
            0x4F => { self.alu_bit(self.get_a(), 1); 8  }, // BIT 1 A

            0x50 => { self.alu_bit(self.get_b(), 2); 8 }, // BIT 2 B
            0x51 => { self.alu_bit(self.get_c(), 2); 8 }, // BIT 2 C
            0x52 => { self.alu_bit(self.get_d(), 2); 8 }, // BIT 2 D
            0x53 => { self.alu_bit(self.get_e(), 2); 8 }, // BIT 2 E
            0x54 => { self.alu_bit(self.get_h(), 2); 8 }, // BIT 2 H
            0x55 => { self.alu_bit(self.get_l(), 2); 8 }, // BIT 2 L
            0x56 => { self.alu_bit(mmu.read_byte(self.get_hl()), 2); 16 }, // BIT 2 (HL)
            0x57 => { self.alu_bit(self.get_a(), 2); 8 }, // BIT 2 A

            0x58 => { self.alu_bit(self.get_b(), 3); 8  }, // BIT 3 B
            0x59 => { self.alu_bit(self.get_c(), 3); 8  }, // BIT 3 C
            0x5A => { self.alu_bit(self.get_d(), 3); 8  }, // BIT 3 D
            0x5B => { self.alu_bit(self.get_e(), 3); 8  }, // BIT 3 E
            0x5C => { self.alu_bit(self.get_h(), 3); 8  }, // BIT 3 H
            0x5D => { self.alu_bit(self.get_l(), 3); 8  }, // BIT 3 L
            0x5E => { self.alu_bit(mmu.read_byte(self.get_hl()), 3); 16 }, // BIT 3 (HL)
            0x5F => { self.alu_bit(self.get_a(), 3); 8  }, // BIT 3 A

            0x60 => { self.alu_bit(self.get_b(), 4); 8 }, // BIT 4 B
            0x61 => { self.alu_bit(self.get_c(), 4); 8 }, // BIT 4 C
            0x62 => { self.alu_bit(self.get_d(), 4); 8 }, // BIT 4 D
            0x63 => { self.alu_bit(self.get_e(), 4); 8 }, // BIT 4 E
            0x64 => { self.alu_bit(self.get_h(), 4); 8 }, // BIT 4 H
            0x65 => { self.alu_bit(self.get_l(), 4); 8 }, // BIT 4 L
            0x66 => { self.alu_bit(mmu.read_byte(self.get_hl()), 4); 16 }, // BIT 4 (HL)
            0x67 => { self.alu_bit(self.get_a(), 4); 8 }, // BIT 4 A

            0x68 => { self.alu_bit(self.get_b(), 5); 8  }, // BIT 5 B
            0x69 => { self.alu_bit(self.get_c(), 5); 8  }, // BIT 5 C
            0x6A => { self.alu_bit(self.get_d(), 5); 8  }, // BIT 5 D
            0x6B => { self.alu_bit(self.get_e(), 5); 8  }, // BIT 5 E
            0x6C => { self.alu_bit(self.get_h(), 5); 8  }, // BIT 5 H
            0x6D => { self.alu_bit(self.get_l(), 5); 8  }, // BIT 5 L
            0x6E => { self.alu_bit(mmu.read_byte(self.get_hl()), 5); 16 }, // BIT 5 (HL)
            0x6F => { self.alu_bit(self.get_a(), 5); 8  }, // BIT 5 A

            0x70 => { self.alu_bit(self.get_b(), 6); 8 }, // BIT 6 B
            0x71 => { self.alu_bit(self.get_c(), 6); 8 }, // BIT 6 C
            0x72 => { self.alu_bit(self.get_d(), 6); 8 }, // BIT 6 D
            0x73 => { self.alu_bit(self.get_e(), 6); 8 }, // BIT 6 E
            0x74 => { self.alu_bit(self.get_h(), 6); 8 }, // BIT 6 H
            0x75 => { self.alu_bit(self.get_l(), 6); 8 }, // BIT 6 L
            0x76 => { self.alu_bit(mmu.read_byte(self.get_hl()), 6); 16 }, // BIT 6 (HL)
            0x77 => { self.alu_bit(self.get_a(), 6); 8 }, // BIT 6 A

            0x78 => { self.alu_bit(self.get_b(), 7); 8 }, // BIT 7 B
            0x79 => { self.alu_bit(self.get_c(), 7); 8 }, // BIT 7 C
            0x7A => { self.alu_bit(self.get_d(), 7); 8 }, // BIT 7 D
            0x7B => { self.alu_bit(self.get_e(), 7); 8 }, // BIT 7 E
            0x7C => { self.alu_bit(self.get_h(), 7); 8 }, // BIT 7 H
            0x7D => { self.alu_bit(self.get_l(), 7); 8 }, // BIT 7 L
            0x7E => { self.alu_bit(mmu.read_byte(self.get_hl()), 7); 16 }, // BIT 7 (HL)
            0x7F => { self.alu_bit(self.get_a(), 7); 8 }, // BIT 7 A

            0x80 => { self.set_b(self.alu_res(self.get_b(), 0)); 8 }, // RES 0 B
            0x81 => { self.set_c(self.alu_res(self.get_c(), 0)); 8 }, // RES 0 C
            0x82 => { self.set_d(self.alu_res(self.get_d(), 0)); 8 }, // RES 0 D
            0x83 => { self.set_e(self.alu_res(self.get_e(), 0)); 8 }, // RES 0 E
            0x84 => { self.set_h(self.alu_res(self.get_h(), 0)); 8 }, // RES 0 H
            0x85 => { self.set_l(self.alu_res(self.get_l(), 0)); 8 }, // RES 0 L
            0x86 => { mmu.write_byte(self.get_hl(), self.alu_res(mmu.read_byte(self.get_hl()), 0)); 16 }, // RES 0 (HL)
            0x87 => { self.set_a(self.alu_res(self.get_b(), 0)); 8 }, // RES 0 A

            0x88 => { self.set_b(self.alu_res(self.get_b(), 1)); 8 }, // RES 1 B
            0x89 => { self.set_c(self.alu_res(self.get_c(), 1)); 8 }, // RES 1 C
            0x8A => { self.set_d(self.alu_res(self.get_d(), 1)); 8 }, // RES 1 D
            0x8B => { self.set_e(self.alu_res(self.get_e(), 1)); 8 }, // RES 1 E
            0x8C => { self.set_h(self.alu_res(self.get_h(), 1)); 8 }, // RES 1 H
            0x8D => { self.set_l(self.alu_res(self.get_l(), 1)); 8 }, // RES 1 L
            0x8E => { mmu.write_byte(self.get_hl(), self.alu_res(mmu.read_byte(self.get_hl()), 1)); 16 }, // RES 1 (HL)
            0x8F => { self.set_a(self.alu_res(self.get_b(), 1)); 8 }, // RES 1 A

            0x90 => { self.set_b(self.alu_res(self.get_b(), 2)); 8 }, // RES 2 B
            0x91 => { self.set_c(self.alu_res(self.get_c(), 2)); 8 }, // RES 2 C
            0x92 => { self.set_d(self.alu_res(self.get_d(), 2)); 8 }, // RES 2 D
            0x93 => { self.set_e(self.alu_res(self.get_e(), 2)); 8 }, // RES 2 E
            0x94 => { self.set_h(self.alu_res(self.get_h(), 2)); 8 }, // RES 2 H
            0x95 => { self.set_l(self.alu_res(self.get_l(), 2)); 8 }, // RES 2 L
            0x96 => { mmu.write_byte(self.get_hl(), self.alu_res(mmu.read_byte(self.get_hl()), 2)); 16 }, // RES 3 (HL)
            0x97 => { self.set_a(self.alu_res(self.get_b(), 2)); 8 }, // RES 3 A

            0x98 => { self.set_b(self.alu_res(self.get_b(), 3)); 8 }, // RES 3 B
            0x99 => { self.set_c(self.alu_res(self.get_c(), 3)); 8 }, // RES 3 C
            0x9A => { self.set_d(self.alu_res(self.get_d(), 3)); 8 }, // RES 3 D
            0x9B => { self.set_e(self.alu_res(self.get_e(), 3)); 8 }, // RES 3 E
            0x9C => { self.set_h(self.alu_res(self.get_h(), 3)); 8 }, // RES 3 H
            0x9D => { self.set_l(self.alu_res(self.get_l(), 3)); 8 }, // RES 3 L
            0x9E => { mmu.write_byte(self.get_hl(), self.alu_res(mmu.read_byte(self.get_hl()), 3)); 16 }, // RES 3 (HL)
            0x9F => { self.set_a(self.alu_res(self.get_b(), 3)); 8 }, // RES 3 A

            0xA0 => { self.set_b(self.alu_res(self.get_b(), 4)); 8 }, // RES 4 B
            0xA1 => { self.set_c(self.alu_res(self.get_c(), 4)); 8 }, // RES 4 C
            0xA2 => { self.set_d(self.alu_res(self.get_d(), 4)); 8 }, // RES 4 D
            0xA3 => { self.set_e(self.alu_res(self.get_e(), 4)); 8 }, // RES 4 E
            0xA4 => { self.set_h(self.alu_res(self.get_h(), 4)); 8 }, // RES 4 H
            0xA5 => { self.set_l(self.alu_res(self.get_l(), 4)); 8 }, // RES 4 L
            0xA6 => { mmu.write_byte(self.get_hl(), self.alu_res(mmu.read_byte(self.get_hl()), 4)); 16 }, // RES 4 (HL)
            0xA7 => { self.set_a(self.alu_res(self.get_b(), 4)); 8 }, // RES 4 A

            0xA8 => { self.set_b(self.alu_res(self.get_b(), 5)); 8 }, // RES 5 B
            0xA9 => { self.set_c(self.alu_res(self.get_c(), 5)); 8 }, // RES 5 C
            0xAA => { self.set_d(self.alu_res(self.get_d(), 5)); 8 }, // RES 5 D
            0xAB => { self.set_e(self.alu_res(self.get_e(), 5)); 8 }, // RES 5 E
            0xAC => { self.set_h(self.alu_res(self.get_h(), 5)); 8 }, // RES 5 H
            0xAD => { self.set_l(self.alu_res(self.get_l(), 5)); 8 }, // RES 5 L
            0xAE => { mmu.write_byte(self.get_hl(), self.alu_res(mmu.read_byte(self.get_hl()), 5)); 16 }, // RES 5 (HL)
            0xAF => { self.set_a(self.alu_res(self.get_b(), 5)); 8 }, // RES 5 A

            0xB0 => { self.set_b(self.alu_res(self.get_b(), 6)); 8 }, // RES 6 B
            0xB1 => { self.set_c(self.alu_res(self.get_c(), 6)); 8 }, // RES 6 C
            0xB2 => { self.set_d(self.alu_res(self.get_d(), 6)); 8 }, // RES 6 D
            0xB3 => { self.set_e(self.alu_res(self.get_e(), 6)); 8 }, // RES 6 E
            0xB4 => { self.set_h(self.alu_res(self.get_h(), 6)); 8 }, // RES 6 H
            0xB5 => { self.set_l(self.alu_res(self.get_l(), 6)); 8 }, // RES 6 L
            0xB6 => { mmu.write_byte(self.get_hl(), self.alu_res(mmu.read_byte(self.get_hl()), 6)); 16 }, // RES 6 (HL)
            0xB7 => { self.set_a(self.alu_res(self.get_b(), 6)); 8 }, // RES 6 A

            0xB8 => { self.set_b(self.alu_res(self.get_b(), 7)); 8 }, // RES 7 B
            0xB9 => { self.set_c(self.alu_res(self.get_c(), 7)); 8 }, // RES 7 C
            0xBA => { self.set_d(self.alu_res(self.get_d(), 7)); 8 }, // RES 7 D
            0xBB => { self.set_e(self.alu_res(self.get_e(), 7)); 8 }, // RES 7 E
            0xBC => { self.set_h(self.alu_res(self.get_h(), 7)); 8 }, // RES 7 H
            0xBD => { self.set_l(self.alu_res(self.get_l(), 7)); 8 }, // RES 7 L
            0xBE => { mmu.write_byte(self.get_hl(), self.alu_res(mmu.read_byte(self.get_hl()), 7)); 16 }, // RES 7 (HL)
            0xBF => { self.set_a(self.alu_res(self.get_b(), 7)); 8 }, // RES 7 A

            0xC0 => { self.set_b(self.alu_set(self.get_b(), 0)); 8 }, // SET 0 B
            0xC1 => { self.set_c(self.alu_set(self.get_c(), 0)); 8 }, // SET 0 C
            0xC2 => { self.set_d(self.alu_set(self.get_d(), 0)); 8 }, // SET 0 D
            0xC3 => { self.set_e(self.alu_set(self.get_e(), 0)); 8 }, // SET 0 E
            0xC4 => { self.set_h(self.alu_set(self.get_h(), 0)); 8 }, // SET 0 H
            0xC5 => { self.set_l(self.alu_set(self.get_l(), 0)); 8 }, // SET 0 L
            0xC6 => { mmu.write_byte(self.get_hl(), self.alu_set(mmu.read_byte(self.get_hl()), 0)); 16 }, // SET 0 (HL)
            0xC7 => { self.set_a(self.alu_set(self.get_b(), 0)); 8 }, // SET 0 A

            0xC8 => { self.set_b(self.alu_set(self.get_b(), 1)); 8 }, // SET 1 B
            0xC9 => { self.set_c(self.alu_set(self.get_c(), 1)); 8 }, // SET 1 C
            0xCA => { self.set_d(self.alu_set(self.get_d(), 1)); 8 }, // SET 1 D
            0xCB => { self.set_e(self.alu_set(self.get_e(), 1)); 8 }, // SET 1 E
            0xCC => { self.set_h(self.alu_set(self.get_h(), 1)); 8 }, // SET 1 H
            0xCD => { self.set_l(self.alu_set(self.get_l(), 1)); 8 }, // SET 1 L
            0xCE => { mmu.write_byte(self.get_hl(), self.alu_set(mmu.read_byte(self.get_hl()), 1)); 16 }, // SET 1 (HL)
            0xCF => { self.set_a(self.alu_set(self.get_b(), 1)); 8 }, // SET 1 A

            0xD0 => { self.set_b(self.alu_set(self.get_b(), 2)); 8 }, // SET 2 B
            0xD1 => { self.set_c(self.alu_set(self.get_c(), 2)); 8 }, // SET 2 C
            0xD2 => { self.set_d(self.alu_set(self.get_d(), 2)); 8 }, // SET 2 D
            0xD3 => { self.set_e(self.alu_set(self.get_e(), 2)); 8 }, // SET 2 E
            0xD4 => { self.set_h(self.alu_set(self.get_h(), 2)); 8 }, // SET 2 H
            0xD5 => { self.set_l(self.alu_set(self.get_l(), 2)); 8 }, // SET 2 L
            0xD6 => { mmu.write_byte(self.get_hl(), self.alu_set(mmu.read_byte(self.get_hl()), 2)); 16 }, // SET 3 (HL)
            0xD7 => { self.set_a(self.alu_set(self.get_b(), 2)); 8 }, // SET 3 A

            0xD8 => { self.set_b(self.alu_set(self.get_b(), 3)); 8 }, // SET 3 B
            0xD9 => { self.set_c(self.alu_set(self.get_c(), 3)); 8 }, // SET 3 C
            0xDA => { self.set_d(self.alu_set(self.get_d(), 3)); 8 }, // SET 3 D
            0xDB => { self.set_e(self.alu_set(self.get_e(), 3)); 8 }, // SET 3 E
            0xDC => { self.set_h(self.alu_set(self.get_h(), 3)); 8 }, // SET 3 H
            0xDD => { self.set_l(self.alu_set(self.get_l(), 3)); 8 }, // SET 3 L
            0xDE => { mmu.write_byte(self.get_hl(), self.alu_set(mmu.read_byte(self.get_hl()), 3)); 16 }, // SET 3 (HL)
            0xDF => { self.set_a(self.alu_set(self.get_b(), 3)); 8 }, // SET 3 A

            0xE0 => { self.set_b(self.alu_set(self.get_b(), 4)); 8 }, // SET 4 B
            0xE1 => { self.set_c(self.alu_set(self.get_c(), 4)); 8 }, // SET 4 C
            0xE2 => { self.set_d(self.alu_set(self.get_d(), 4)); 8 }, // SET 4 D
            0xE3 => { self.set_e(self.alu_set(self.get_e(), 4)); 8 }, // SET 4 E
            0xE4 => { self.set_h(self.alu_set(self.get_h(), 4)); 8 }, // SET 4 H
            0xE5 => { self.set_l(self.alu_set(self.get_l(), 4)); 8 }, // SET 4 L
            0xE6 => { mmu.write_byte(self.get_hl(), self.alu_set(mmu.read_byte(self.get_hl()), 4)); 16 }, // SET 4 (HL)
            0xE7 => { self.set_a(self.alu_set(self.get_b(), 4)); 8 }, // SET 4 A

            0xE8 => { self.set_b(self.alu_set(self.get_b(), 5)); 8 }, // SET 5 B
            0xE9 => { self.set_c(self.alu_set(self.get_c(), 5)); 8 }, // SET 5 C
            0xEA => { self.set_d(self.alu_set(self.get_d(), 5)); 8 }, // SET 5 D
            0xEB => { self.set_e(self.alu_set(self.get_e(), 5)); 8 }, // SET 5 E
            0xEC => { self.set_h(self.alu_set(self.get_h(), 5)); 8 }, // SET 5 H
            0xED => { self.set_l(self.alu_set(self.get_l(), 5)); 8 }, // SET 5 L
            0xEE => { mmu.write_byte(self.get_hl(), self.alu_set(mmu.read_byte(self.get_hl()), 5)); 16 }, // SET 5 (HL)
            0xEF => { self.set_a(self.alu_set(self.get_b(), 5)); 8 }, // SET 5 A

            0xF0 => { self.set_b(self.alu_set(self.get_b(), 6)); 8 }, // SET 6 B
            0xF1 => { self.set_c(self.alu_set(self.get_c(), 6)); 8 }, // SET 6 C
            0xF2 => { self.set_d(self.alu_set(self.get_d(), 6)); 8 }, // SET 6 D
            0xF3 => { self.set_e(self.alu_set(self.get_e(), 6)); 8 }, // SET 6 E
            0xF4 => { self.set_h(self.alu_set(self.get_h(), 6)); 8 }, // SET 6 H
            0xF5 => { self.set_l(self.alu_set(self.get_l(), 6)); 8 }, // SET 6 L
            0xF6 => { mmu.write_byte(self.get_hl(), self.alu_set(mmu.read_byte(self.get_hl()), 6)); 16 }, // SET 6 (HL)
            0xF7 => { self.set_a(self.alu_set(self.get_b(), 6)); 8 }, // SET 6 A

            0xF8 => { self.set_b(self.alu_set(self.get_b(), 7)); 8 }, // SET 7 B
            0xF9 => { self.set_c(self.alu_set(self.get_c(), 7)); 8 }, // SET 7 C
            0xFA => { self.set_d(self.alu_set(self.get_d(), 7)); 8 }, // SET 7 D
            0xFB => { self.set_e(self.alu_set(self.get_e(), 7)); 8 }, // SET 7 E
            0xFC => { self.set_h(self.alu_set(self.get_h(), 7)); 8 }, // SET 7 H
            0xFD => { self.set_l(self.alu_set(self.get_l(), 7)); 8 }, // SET 7 L
            0xFE => { mmu.write_byte(self.get_hl(), self.alu_set(mmu.read_byte(self.get_hl()), 7)); 16 }, // SET 7 (HL)
            0xFF => { self.set_a(self.alu_set(self.get_b(), 7)); 8 }, // SET 7 A
        }
    }


    // alu operations

    fn alu_inc(&mut self, val: u8) -> u8 {
        self.set_half_flag((val & 0x0F) + 1 == 0x10);
        let val = val.wrapping_add(1);

        self.set_zero_flag(val == 0);
        self.set_neg_flag(false);

        val
    }

    fn alu_add_16b(&mut self, a: u16, b: u16) -> u16 {
        let (new_val, ov) = a.overflowing_add(b);


        self.set_carry_flag(ov);
        self.set_half_flag(((a & 0x7) + (b & 0x7)) & 0x8 == 0x8);
        self.set_neg_flag(false);

        new_val
    }

    fn alu_dec(&mut self, val: u8) -> u8 {
        self.set_half_flag((val & 0x0F) == 0);

        let val = val.wrapping_sub(1);

        self.set_zero_flag(val == 0);
        self.set_neg_flag(true);

        val
    }

    fn alu_rlc(&mut self, val: u8) -> u8 {
        let last_bit_was_on = (val & 0x80) != 0;

        self.set_neg_flag(false);
        self.set_half_flag(false);
        

        self.set_carry_flag(last_bit_was_on);

        let mut val = val << 1;

        if last_bit_was_on {
            val |= 0x1;
        }

        self.set_zero_flag(val == 0);

        val
    }

    fn alu_rrc(&mut self, val: u8) -> u8 {
        let first_bit_was_on = (val & 0x1) != 0;

        let mut val = val >> 1;

        if first_bit_was_on {
            val |= 0x80;
        }


        self.set_neg_flag(false);
        self.set_half_flag(false);

        self.set_zero_flag(val == 0);
        self.set_carry_flag(first_bit_was_on);

        val
    }

    fn alu_rl(&mut self, val: u8) -> u8 {
        let prev_carry = self.get_carry_flag();
        self.set_carry_flag(val & 0x80 != 0);
        let val = (val << 1) | if prev_carry { 1 } else { 0 };

        self.set_zero_flag(val == 0);

        self.set_neg_flag(false);
        self.set_half_flag(false);

        val
    }

    fn alu_rr(&mut self, val: u8) -> u8 {
        let prev_carry = self.get_carry_flag();
        self.set_carry_flag(val & 0x1 != 0);
        let val = (val >> 1) | if prev_carry { 0x80 } else { 0 };

        self.set_neg_flag(false);
        self.set_half_flag(false);
        self.set_zero_flag(val == 0);

        val
    }

    fn alu_daa(&mut self) {
        let mut a = self.get_a();
        let mut correction = 0;
        let mut new_carry = false;

        if self.get_half_flag() || (!self.get_neg_flag() && ((a & 0xF) > 0x9)) {
            correction |= 0x6;
        }

        if self.get_carry_flag() || (!self.get_neg_flag() && a > 0x99) {
            correction |= 0x60;
            new_carry = self.get_carry_flag();
        }


        a = if self.get_neg_flag() { a.wrapping_sub(correction) } else { a.wrapping_add(correction) };

        self.set_carry_flag(new_carry);
        self.set_half_flag(false);
        self.set_zero_flag(a == 0);
        self.set_a(a)

    }

    fn alu_add_a(&mut self, b: u8, add_carry: bool) {
        let carry = if add_carry { 1 } else { 0 };
        let a = self.get_a();

        let res = a.wrapping_add(b).wrapping_add(carry);
        self.set_zero_flag(res == 0);
        self.set_neg_flag(false);
        self.set_carry_flag((a as u16) + (b as u16) + (carry as u16) > 0xFF);
        self.set_half_flag((a & 0xF) + (b & 0xF) + carry > 0xF);

        self.set_a(res);
    }

    fn alu_sub_a(&mut self, b: u8, sub_carry: bool) {
        let carry = if sub_carry { 1 } else { 0 };
        let a = self.get_a();

        let res = a.wrapping_sub(b).wrapping_sub(carry);
        
        self.set_zero_flag(res == 0);
        self.set_neg_flag(true);
        self.set_carry_flag((a as u16) < (b as u16) + (carry as u16));
        self.set_half_flag((a & 0xF) < (b & 0xF) + carry);

        self.set_a(res);
    }

    fn alu_and_a(&mut self, b: u8) {
        self.set_a(self.get_a() & b);

        self.set_zero_flag(self.get_a() == 0);
        self.set_neg_flag(false);
        self.set_half_flag(true);
        self.set_carry_flag(true);
    }

    fn alu_xor_a(&mut self, b: u8) {
        self.set_a(self.get_a() ^ b);

        self.set_zero_flag(self.get_a() == 0);
        self.set_neg_flag(false);
        self.set_half_flag(true);
        self.set_carry_flag(true);
    }

    fn alu_or_a(&mut self, b: u8) {
        self.set_a(self.get_a() | b);

        self.set_zero_flag(self.get_a() == 0);
        self.set_neg_flag(false);
        self.set_half_flag(false);
        self.set_carry_flag(false);
    }

    fn alu_cp_a(&mut self, b: u8) {
        let prev_a = self.get_a();

        self.alu_sub_a(b, false);

        self.set_a(prev_a);
    }

    fn alu_sla(&mut self, val: u8) -> u8 {
        self.set_carry_flag(val & 0x80 == 1);

        let res = val << 1;

        self.set_neg_flag(false);
        self.set_half_flag(false);
        self.set_zero_flag(res == 0);

        res
    }

    fn alu_sra(&mut self, val: u8) -> u8 {
        self.set_carry_flag(val & 0x1 == 1);

        let res = (val >> 1) | (val & 0x80);

        self.set_neg_flag(false);
        self.set_half_flag(false);
        self.set_zero_flag(res == 0);

        res
    }

    fn alu_srl(&mut self, val: u8) -> u8 {
        self.set_carry_flag(val & 0x1 == 1);

        let res = val >> 1;

        self.set_neg_flag(false);
        self.set_half_flag(false);
        self.set_zero_flag(res == 0);

        res
    }


    fn alu_swap(&mut self, val: u8) -> u8 {
        let res = (val >> 4) | (val << 4);

        self.set_carry_flag(false);
        self.set_neg_flag(false);
        self.set_half_flag(false);
        self.set_zero_flag(res == 0);

        res
    }

    fn alu_bit(&mut self, val: u8 , bit: u8) {
        self.set_half_flag(true);
        self.set_neg_flag(false);
        self.set_zero_flag(val & (1 << bit) == 0);
    }


    fn alu_res(&self, val: u8 , bit: u8) -> u8 {
        val & (!(1 << bit))
    }

    fn alu_set(&self, val: u8 , bit: u8) -> u8 {
        val | (1 << bit)
    }



    fn jump_relative(&mut self, mmu: &mut Mmu) {
        let n = self.fetch_byte(mmu) as i8;

        self.set_pc(self.get_pc().wrapping_add(n as u16));
    }

}