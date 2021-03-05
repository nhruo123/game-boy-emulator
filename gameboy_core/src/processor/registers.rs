const ZERO_FLAG_MASK: u8 = 0b1 << 7;
const NEG_FLAG_MASK: u8 = 0b1 << 6;
const HALF_CARRY_FLAG_MASK: u8 = 0b1 << 5;
const CARRY_FLAG_MASK: u8 = 0b1 << 4;

#[derive(Clone, Copy, Debug)]
pub struct Registers {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16, // stack ptr
    pub pc: u16, // program counter
}

#[derive(Clone, Copy, Debug)]
pub enum Reg8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Clone, Copy, Debug)]
pub enum Reg16 {
    AF,
    BC,
    DE,
    HL,
    SP,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            pc: 0,
            sp: 0,
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
        }
    }

    pub fn read16(&self, reg: Reg16) -> u16 {
        use self::Reg16::*;
        use crate::utils::*;

        match reg {
            AF => build_u16(self.a, self.f),
            BC => build_u16(self.b, self.c),
            DE => build_u16(self.d, self.e),
            HL => build_u16(self.h, self.l),
            SP => self.sp,
        }
    }

    pub fn write16(&mut self, reg: Reg16, val: u16) {
        use self::Reg16::*;
        use crate::utils::*;

        match reg {
            AF => {
                self.a = get_u16_high(val);
                self.f = get_u16_low(val) & 0xF0;
            }
            BC => {
                self.b = get_u16_high(val);
                self.c = get_u16_low(val);
            }
            DE => {
                self.d = get_u16_high(val);
                self.e = get_u16_low(val);
            }
            HL => {
                self.h = get_u16_high(val);
                self.l = get_u16_low(val);
            }
            SP => self.sp = val,
        }
    }

    #[inline]
    pub fn zf(&self) -> bool {
        (self.f & ZERO_FLAG_MASK) != 0
    }

    #[inline]
    pub fn nf(&self) -> bool {
        (self.f & NEG_FLAG_MASK) != 0
    }

    #[inline]
    pub fn cf(&self) -> bool {
        (self.f & CARRY_FLAG_MASK) != 0
    }

    #[inline]
    pub fn hf(&self) -> bool {
        (self.f & HALF_CARRY_FLAG_MASK) != 0
    }

    #[inline]
    pub fn set_zf(&mut self, zf: bool) {
        if zf {
            self.f |= ZERO_FLAG_MASK;
        } else {
            self.f &= !ZERO_FLAG_MASK;
        }
    }

    #[inline]
    pub fn set_nf(&mut self, nf: bool) {
        if nf {
            self.f |= NEG_FLAG_MASK;
        } else {
            self.f &= !NEG_FLAG_MASK;
        }
    }

    #[inline]
    pub fn set_cf(&mut self, cf: bool) {
        if cf {
            self.f |= CARRY_FLAG_MASK;
        } else {
            self.f &= !CARRY_FLAG_MASK;
        }
    }

    #[inline]
    pub fn set_hf(&mut self, hf: bool) {
        if hf {
            self.f |= HALF_CARRY_FLAG_MASK;
        } else {
            self.f &= !HALF_CARRY_FLAG_MASK;
        }
    }
}


impl std::fmt::Display for Registers {

    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "PC:[{:04X}] SP:[{:04X}] \
            A:[{:02X}], F:[{:02X}], B:[{:02X}], C:[{:02X}], \
            D:[{:02X}], E:[{:02X}], H:[{:02X}], L:[{:02X}], flags:[{}{}{}{}]]",
            self.pc, self.sp, self.a, self.f, self.b, self.c, self.d, self.e, self.h, self.l,
            if self.zf() { "z" } else { "_" },
            if self.nf() { "n" } else { "_" },
            if self.hf() { "h" } else { "_" },
            if self.cf() { "c" } else { "_" },
        )
    }
}
