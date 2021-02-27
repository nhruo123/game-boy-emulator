

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

    fn stop(&mut self) {
        unimplemented!("stop is unimplemented");
    }

    fn halt(&mut self) {
        unimplemented!("halt is unimplemented")
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
            0x04 => { let v = self.aul_inc(self.get_b()); self.set_b(v); 4 }, // INC B
            0x05 => { let v = self.aul_dec(self.get_b()); self.set_b(v); 4 }, // DEC B
            0x06 => { let v = self.fetch_byte(mmu); self.set_b(v); 8 }, // LD B,u8
            0x07 => { let v = self.aul_rlc(self.get_a()); self.set_a(v); 4 }, // RLCA
            0x08 => { let v = self.fetch_word(mmu); mmu.write_word(v, self.get_sp()); 8 }, // LD (u16),SP
            0x09 => { self.aul_inc_16b(self.get_bc()) ; 8 }, // ADD HL,BC
            0x0A => { let v = mmu.read_byte(self.get_bc()); self.set_a(v) ; 8 }, // LD A,(BC)
            0x0B => { self.set_bc(self.get_bc().wrapping_add(1)) ; 8 }, // DEC BC
            0x0C => { let v = self.aul_inc(self.get_c()); self.set_c(v); 4 }, // INC C
            0x0D => { let v = self.aul_dec(self.get_c()); self.set_c(v); 4 }, // DEC C
            0x0E => { let v = self.fetch_byte(mmu); self.set_c(v); 8 }, // LD C,u8
            0x0F => { let v = self.aul_rrc(self.get_a()); self.set_a(v); 4 }, // RRCA
            0x10 => { self.stop(); 4 }, // STOP
            0x11 => { let v = self.fetch_word(mmu); self.set_de(v); 12 }, // LD DE,u16
            0x12 => { mmu.write_byte(self.get_de(), self.get_a()); 8 }, // LD (DE),A
            0x13 => { self.set_de(self.get_de().wrapping_add(1)); 8 }, // INC DE
            0x14 => { let v = self.aul_inc(self.get_d()); self.set_d(v); 4 }, // INC D
            0x15 => { let v = self.aul_dec(self.get_d()); self.set_d(v); 4 }, // DEC D
            0x16 => { let v = self.fetch_byte(mmu); self.set_d(v); 4 }, // LD D,u8
            0x17 => { let v = self.aul_rl(self.get_a()); self.set_a(v); 4 }, // RLA
            0x18 => { self.jump_relative(mmu); 12 }, // JR i8
            0x19 => { self.aul_inc_16b(self.get_de()); 8 }, // ADD HL,DE
            0x1A => { self.aul_inc_16b(self.get_de()); 8 }, // LD A,(DE)
            0x1B => { self.set_de(self.get_de().wrapping_add(1)) ; 8 }, // DEC DE
            0x1C => { let v = self.aul_inc(self.get_e()); self.set_e(v); 4 }, // INC E
            0x1D => { let v = self.aul_dec(self.get_e()); self.set_e(v); 4 }, // DEC E
            0x1E => { let v = self.fetch_byte(mmu); self.set_e(v); 8 }, // LD E,u8
            0x1F => { let v = self.aul_rr(self.get_a()); self.set_a(v); 4 }, // RRA
            0x20 => if !self.get_zero_flag() {self.jump_relative(mmu); 12} else { self.set_pc(self.get_pc() + 1); 8 }, // JR NZ,i8
            0x21 => { let v = self.fetch_word(mmu); self.set_hl(v); 12 }, // LD HL,u16
            0x22 => { let v = self.get_hl(); self.set_hl(v.wrapping_add(1)); mmu.write_byte(v, self.get_a()); 8 }, // LD (HL+),A
            0x23 => { self.set_hl(self.get_hl().wrapping_add(1)); 8 }, // INC HL
            0x24 => { let v = self.aul_inc(self.get_h()); self.set_h(v); 4 }, // INC H
            0x25 => { let v = self.aul_dec(self.get_h()); self.set_h(v); 4 }, // DEC H
            0x26 => { let v = self.fetch_byte(mmu); self.set_h(v); 8 }, // LD H,u8
            0x27 => { self.aul_daa(); 4 }, // DAA
            0x28 => if self.get_zero_flag() { self.jump_relative(mmu); 12} else { self.set_pc(self.get_pc() + 1); 8 }, // JR Z,i8
            0x29 => { self.aul_inc_16b(self.get_hl()); 8 }, // ADD HL,HL
            0x2A => { let v = self.get_hl(); self.set_hl(v.wrapping_add(1)); self.set_a(mmu.read_byte(v)); 8 }, // LD (HL+),A
            0x2B => { self.set_hl(self.get_hl().wrapping_add(1)) ; 8 }, // DEC HL
            0x2C => { let v = self.aul_inc(self.get_l()); self.set_l(v); 4 }, // INC L
            0x2D => { let v = self.aul_dec(self.get_l()); self.set_l(v); 4 }, // DEC L
            0x2E => { let v = self.fetch_byte(mmu); self.set_l(v); 8 }, // LD L,u8
            0x2F => { self.set_a(!self.get_a()); self.set_half_flag(true); self.set_neg_flag(true); 4 }, // LD L,u8
            0x30 => if !self.get_carry_flag() { self.jump_relative(mmu); 12 } else { self.set_pc(self.get_pc() + 1); 8 }, // JR NC,i8
            0x31 => { let v = self.fetch_word(mmu); self.set_sp(v); 12 }, // LD SP,u16
            0x32 => { let v = self.get_hl(); self.set_hl(v.wrapping_sub(1)); mmu.write_byte(v, self.get_a()); 8 }, // LD (HL-),A
            0x33 => { self.set_sp(self.get_sp().wrapping_add(1)); 8 }, // INC SP
            0x34 => { let v = mmu.read_byte(self.get_hl()); mmu.write_byte(self.get_hl(), self.aul_inc(v)); 12 }, // INC (HL)
            0x35 => { let v = mmu.read_byte(self.get_hl()); mmu.write_byte(self.get_hl(), self.aul_dec(v)); 12 }, // DEC (HL)
            0x36 => { let v = self.fetch_byte(mmu); mmu.write_byte(self.get_hl(), v); 12 }, // LD (HL),u8
            0x37 => { self.set_carry_flag(true); self.set_neg_flag(false); self.set_half_flag(false); 4 }, // SCF
            0x38 => if self.get_carry_flag() { self.jump_relative(mmu); 12 } else { self.set_pc(self.get_pc() + 1); 8 }, // JR C,i8
            0x39 => { self.aul_inc_16b(self.get_sp()); 8 }, // ADD HL,SP
            0x3A => { let v = self.get_hl(); self.set_hl(v.wrapping_sub(1)); self.set_a(mmu.read_byte(v)); 8 }, // LD A,(HL-)
            0x3B => { self.set_sp(self.get_sp().wrapping_sub(1)); 8 }, // DEC SP
            0x3C => { let v = self.aul_inc(self.get_a()); self.set_a(v); 4 }, // INC A
            0x3D => { let v = self.aul_dec(self.get_a()); self.set_a(v); 4 }, // DEC A
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
            0x80 => { self.aul_add_a(self.get_b(), false); 4 }, // ADD A, B
            0x81 => { self.aul_add_a(self.get_c(), false); 4 }, // ADD A, C
            0x82 => { self.aul_add_a(self.get_d(), false); 4 }, // ADD A, D
            0x83 => { self.aul_add_a(self.get_e(), false); 4 }, // ADD A, E
            0x84 => { self.aul_add_a(self.get_h(), false); 4 }, // ADD A, H
            0x85 => { self.aul_add_a(self.get_l(), false); 4 }, // ADD A, L
            0x86 => { self.aul_add_a(mmu.read_byte(self.get_hl()), false); 8 }, // ADD A, (HL)
            0x87 => { self.aul_add_a(self.get_a(), false); 4 }, // ADD A, A
            0x88 => { self.aul_add_a(self.get_b(), true); 4 }, // ADC A, B
            0x89 => { self.aul_add_a(self.get_c(), true); 4 }, // ADC A, C
            0x8A => { self.aul_add_a(self.get_d(), true); 4 }, // ADC A, D
            0x8B => { self.aul_add_a(self.get_e(), true); 4 }, // ADC A, E
            0x8C => { self.aul_add_a(self.get_h(), true); 4 }, // ADC A, H
            0x8D => { self.aul_add_a(self.get_l(), true); 4 }, // ADC A, L
            0x8E => { self.aul_add_a(mmu.read_byte(self.get_hl()), true); 8 }, // ADC A, (HL)
            0x8F => { self.aul_add_a(self.get_a(), true); 4 }, // ADC A, A
            0x90 => { self.aul_sub_a(self.get_b(), false); 4 }, // SUB A, B
            0x91 => { self.aul_sub_a(self.get_c(), false); 4 }, // SUB A, C
            0x92 => { self.aul_sub_a(self.get_d(), false); 4 }, // SUB A, D
            0x93 => { self.aul_sub_a(self.get_e(), false); 4 }, // SUB A, E
            0x94 => { self.aul_sub_a(self.get_h(), false); 4 }, // SUB A, H
            0x95 => { self.aul_sub_a(self.get_l(), false); 4 }, // SUB A, L
            0x96 => { self.aul_sub_a(mmu.read_byte(self.get_hl()), false); 8 }, // SUB A, (HL)
            0x97 => { self.aul_sub_a(self.get_a(), false); 4 }, // SUB A, A
            0x98 => { self.aul_sub_a(self.get_b(), true); 4 }, // SBC A, B
            0x99 => { self.aul_sub_a(self.get_c(), true); 4 }, // SBC A, C
            0x9A => { self.aul_sub_a(self.get_d(), true); 4 }, // SBC A, D
            0x9B => { self.aul_sub_a(self.get_e(), true); 4 }, // SBC A, E
            0x9C => { self.aul_sub_a(self.get_h(), true); 4 }, // SBC A, H
            0x9D => { self.aul_sub_a(self.get_l(), true); 4 }, // SBC A, L
            0x9E => { self.aul_sub_a(mmu.read_byte(self.get_hl()), true); 8 }, // SBC A, (HL)
            0x9F => { self.aul_sub_a(self.get_a(), true); 4 }, // SBC A, A
            0xA0 => { self.aul_and_a(self.get_b()); 4 }, // AND A, B
            0xA1 => { self.aul_and_a(self.get_c()); 4 }, // AND A, C
            0xA2 => { self.aul_and_a(self.get_d()); 4 }, // AND A, D
            0xA3 => { self.aul_and_a(self.get_e()); 4 }, // AND A, E
            0xA4 => { self.aul_and_a(self.get_h()); 4 }, // AND A, H
            0xA5 => { self.aul_and_a(self.get_l()); 4 }, // AND A, L
            0xA6 => { self.aul_and_a(mmu.read_byte(self.get_hl())); 8 }, // AND A, (HL)
            0xA7 => { self.aul_and_a(self.get_a()); 4 }, // AND A, A
            0xA8 => { self.aul_xor_a(self.get_b()); 4 }, // XOR A, B
            0xA9 => { self.aul_xor_a(self.get_c()); 4 }, // XOR A, C
            0xAA => { self.aul_xor_a(self.get_d()); 4 }, // XOR A, D
            0xAB => { self.aul_xor_a(self.get_e()); 4 }, // XOR A, E
            0xAC => { self.aul_xor_a(self.get_h()); 4 }, // XOR A, H
            0xAD => { self.aul_xor_a(self.get_l()); 4 }, // XOR A, L
            0xAE => { self.aul_xor_a(mmu.read_byte(self.get_hl())); 8 }, // XOR A, (HL)
            0xAF => { self.aul_xor_a(self.get_a()); 4 }, // XOR A, A
            0xB0 => { self.aul_or_a(self.get_b()); 4 }, // OR A, B
            0xB1 => { self.aul_or_a(self.get_c()); 4 }, // OR A, C
            0xB2 => { self.aul_or_a(self.get_d()); 4 }, // OR A, D
            0xB3 => { self.aul_or_a(self.get_e()); 4 }, // OR A, E
            0xB4 => { self.aul_or_a(self.get_h()); 4 }, // OR A, H
            0xB5 => { self.aul_or_a(self.get_l()); 4 }, // OR A, L
            0xB6 => { self.aul_or_a(mmu.read_byte(self.get_hl())); 8 }, // OR A, (HL)
            0xB7 => { self.aul_or_a(self.get_a()); 4 }, // OR A, A
            0xB8 => { self.aul_cp_a(self.get_b()); 4 }, // CP A, B
            0xB9 => { self.aul_cp_a(self.get_c()); 4 }, // CP A, C
            0xBA => { self.aul_cp_a(self.get_d()); 4 }, // CP A, D
            0xBB => { self.aul_cp_a(self.get_e()); 4 }, // CP A, E
            0xBC => { self.aul_cp_a(self.get_h()); 4 }, // CP A, H
            0xBD => { self.aul_cp_a(self.get_l()); 4 }, // CP A, L
            0xBE => { self.aul_cp_a(mmu.read_byte(self.get_hl())); 8 }, // CP A, (HL)
            0xBF => { self.aul_cp_a(self.get_a()); 4 }, // CP A, A
            0xC0 => if !self.get_zero_flag() { let v = self.pop(mmu) ; self.set_pc(v); 20 } else { 8 }, // RET NZ
            0xC1 => { let v = self.pop(mmu); self.set_bc(v) ; 12}, // POP BC
            0xC2 => if !self.get_zero_flag() { let v = self.fetch_word(mmu); self.set_pc(v); 16 } else { self.set_pc(self.get_pc().wrapping_add(2)); 12 }, // JP NZ,u16
            0xC3 => { let v = self.fetch_word(mmu); self.set_pc(v); 16 } // JP u16
            0xC4 => if !self.get_zero_flag() { let v = self.fetch_word(mmu); self.push(mmu, self.pc); self.set_pc(v); 24 } 
                    else { self.set_pc(self.get_pc().wrapping_add(2)); 12 } // CALL NZ,u16
            

            
            


            _other => todo!("Unimplemented opcode {:#x}!", opcode),

        }
    }


    // aul operations

    fn aul_inc(&mut self, val: u8) -> u8 {
        self.set_half_flag((val & 0x0F) + 1 == 0x10);
        let val = val.wrapping_add(1);

        self.set_zero_flag(val == 0);
        self.set_neg_flag(false);

        val
    }

    fn aul_inc_16b(&mut self, val: u16) {
        let (val, ov) = self.get_hl().overflowing_add(val);


        self.set_carry_flag(ov);
        self.set_half_flag(((val & 0x7FF) + (val & 0x7FF)) & 0x800 == 0x800);
        self.set_neg_flag(false);

        self.set_hl(val);
    }

    fn aul_dec(&mut self, val: u8) -> u8 {
        self.set_half_flag((val & 0x0F) == 0);
        let val = val.wrapping_sub(1);

        self.set_zero_flag(val == 0);
        self.set_neg_flag(true);

        val
    }

    fn aul_rlc(&mut self, val: u8) -> u8 {
        let last_bit_was_on = (val & 0x80) != 0;

        self.set_neg_flag(false);
        self.set_half_flag(false);
        self.set_zero_flag(false);

        self.set_carry_flag(last_bit_was_on);

        let mut val = val << 1;

        if last_bit_was_on {
            val |= 0x1;
        }


        val
    }

    fn aul_rrc(&mut self, val: u8) -> u8 {
        let first_bit_was_on = (val & 0x1) != 0;

        self.set_neg_flag(false);
        self.set_half_flag(false);
        self.set_zero_flag(false);

        self.set_carry_flag(first_bit_was_on);

        let mut val = val >> 1;

        if first_bit_was_on {
            val |= 0x80;
        }


        val
    }

    fn aul_rl(&mut self, val: u8) -> u8 {
        let prev_carry = self.get_carry_flag();
        self.set_carry_flag(val & 0x80 != 0);
        let val = (val << 1) | if prev_carry { 1 } else { 0 };

        self.set_neg_flag(false);
        self.set_half_flag(false);
        self.set_zero_flag(false);

        val
    }

    fn aul_rr(&mut self, val: u8) -> u8 {
        let prev_carry = self.get_carry_flag();
        self.set_carry_flag(val & 0x1 != 0);
        let val = (val >> 1) | if prev_carry { 0x80 } else { 0 };

        self.set_neg_flag(false);
        self.set_half_flag(false);
        self.set_zero_flag(false);

        val
    }

    fn aul_daa(&mut self) {
        let mut a = self.get_a();
        let mut correction = 0;
        let mut newCarry = false;

        if self.get_half_flag() || (!self.get_neg_flag() && ((a & 0xF) > 0x9)) {
            correction |= 0x6;
        }

        if self.get_carry_flag() || (!self.get_neg_flag() && a > 0x99) {
            correction |= 0x60;
            newCarry = self.get_carry_flag();
        }


        a = if self.get_neg_flag() { a.wrapping_sub(correction) } else { a.wrapping_add(correction) };

        self.set_carry_flag(newCarry);
        self.set_half_flag(false);
        self.set_zero_flag(a == 0);
        self.set_a(a)

    }

    fn aul_add_a(&mut self, b: u8, add_carry: bool) {
        let carry = if add_carry { 1 } else { 0 };
        let a = self.get_a();

        let res = a.wrapping_add(b).wrapping_add(carry);
        self.set_zero_flag(res == 0);
        self.set_neg_flag(false);
        self.set_carry_flag((a as u16) + (b as u16) + (carry as u16) > 0xFF);
        self.set_half_flag((a & 0xF) + (b & 0xF) + carry > 0xF);

        self.set_a(res);
    }

    fn aul_sub_a(&mut self, b: u8, sub_carry: bool) {
        let carry = if sub_carry { 1 } else { 0 };
        let a = self.get_a();

        let res = a.wrapping_sub(b).wrapping_sub(carry);
        self.set_zero_flag(res == 0);
        self.set_neg_flag(true);
        self.set_carry_flag((a as u16) < (b as u16) + (carry as u16));
        self.set_half_flag((a & 0xF) < (b & 0xF) + carry);

        self.set_a(res);
    }

    fn aul_and_a(&mut self, b: u8) {
        self.set_a(self.get_a() & b);

        self.set_zero_flag(self.get_a() == 0);
        self.set_neg_flag(false);
        self.set_half_flag(true);
        self.set_carry_flag(true);
    }

    fn aul_xor_a(&mut self, b: u8) {
        self.set_a(self.get_a() ^ b);

        self.set_zero_flag(self.get_a() == 0);
        self.set_neg_flag(false);
        self.set_half_flag(true);
        self.set_carry_flag(true);
    }

    fn aul_or_a(&mut self, b: u8) {
        self.set_a(self.get_a() | b);

        self.set_zero_flag(self.get_a() == 0);
        self.set_neg_flag(false);
        self.set_half_flag(true);
        self.set_carry_flag(true);
    }

    fn aul_cp_a(&mut self, b: u8) {
        let prev_a = self.get_a();

        self.aul_sub_a(b, false);

        self.set_a(prev_a);
    }



    fn jump_relative(&mut self, mmu: &mut Mmu) {
        let n = self.fetch_byte(mmu) as i8;

        self.set_sp(self.get_pc().wrapping_add(n as u16));
    }

}