use crate::processor::decode::Condition;
use crate::mmu::Mmu;
use crate::processor::registers::Reg16;
use crate::processor::*;
use crate::utils::UIntExt;

impl Processor {
    // 8 bit operations

    pub fn load<I: Copy, O: Copy>(&mut self, mmu: &mut Mmu, out8: O, in8: I) -> TCycles
    where
        Self: In8<I> + Out8<O>,
    {
        let (val, cycles) = self.read(in8, mmu);

        self.write(out8, mmu, val) + cycles + FETCH_T_CYCLES
    }

    pub fn add<I: Copy>(&mut self, mmu: &mut Mmu, in8: I) -> TCycles
    where
        Self: In8<I>,
    {
        let (val, cycles) = self.read(in8, mmu);

        self.registers.a = self.alu_add(val, false);

        cycles + FETCH_T_CYCLES
    }

    pub fn adc<I: Copy>(&mut self, mmu: &mut Mmu, in8: I) -> TCycles
    where
        Self: In8<I>,
    {
        let (val, cycles) = self.read(in8, mmu);

        self.registers.a = self.alu_add(val, self.registers.cf());

        cycles + FETCH_T_CYCLES
    }

    pub fn sub<I: Copy>(&mut self, mmu: &mut Mmu, in8: I) -> TCycles
    where
        Self: In8<I>,
    {
        let (val, cycles) = self.read(in8, mmu);

        self.registers.a = self.alu_sub(val, false);

        cycles + FETCH_T_CYCLES
    }

    pub fn sbc<I: Copy>(&mut self, mmu: &mut Mmu, in8: I) -> TCycles
    where
        Self: In8<I>,
    {
        let (val, cycles) = self.read(in8, mmu);

        self.registers.a = self.alu_sub(val, self.registers.cf());

        cycles + FETCH_T_CYCLES
    }

    pub fn cp<I: Copy>(&mut self, mmu: &mut Mmu, in8: I) -> TCycles
    where
        Self: In8<I>,
    {
        let (val, cycles) = self.read(in8, mmu);

        self.alu_sub(val, false);

        cycles + FETCH_T_CYCLES
    }

    pub fn and<I: Copy>(&mut self, mmu: &mut Mmu, in8: I) -> TCycles
    where
        Self: In8<I>,
    {
        let (val, cycles) = self.read(in8, mmu);

        self.registers.a &= val;

        self.registers.set_zf(self.registers.a == 0);
        self.registers.set_nf(false);
        self.registers.set_hf(true);
        self.registers.set_cf(false);

        cycles + FETCH_T_CYCLES
    }

    pub fn or<I: Copy>(&mut self, mmu: &mut Mmu, in8: I) -> TCycles
    where
        Self: In8<I>,
    {
        let (val, cycles) = self.read(in8, mmu);

        self.registers.a |= val;

        self.registers.set_zf(self.registers.a == 0);
        self.registers.set_nf(false);
        self.registers.set_hf(false);
        self.registers.set_cf(false);

        cycles + FETCH_T_CYCLES
    }

    pub fn xor<I: Copy>(&mut self, mmu: &mut Mmu, in8: I) -> TCycles
    where
        Self: In8<I>,
    {
        let (val, cycles) = self.read(in8, mmu);

        self.registers.a ^= val;

        self.registers.set_zf(self.registers.a == 0);
        self.registers.set_nf(false);
        self.registers.set_hf(false);
        self.registers.set_cf(false);

        cycles + FETCH_T_CYCLES
    }

    pub fn inc<IO: Copy>(&mut self, mmu: &mut Mmu, io: IO) -> TCycles
    where
        Self: In8<IO> + Out8<IO>,
    {
        let (val, cycles) = self.read(io, mmu);

        let result = val.wrapping_add(1);

        let cycles = self.write(io, mmu, result) + cycles;

        self.registers.set_zf(result == 0);
        self.registers.set_nf(false);
        self.registers.set_hf(val & 0xF == 0xF);

        cycles + FETCH_T_CYCLES
    }

    pub fn dec<IO: Copy>(&mut self, mmu: &mut Mmu, io: IO) -> TCycles
    where
        Self: In8<IO> + Out8<IO>,
    {
        let (val, cycles) = self.read(io, mmu);

        let result = val.wrapping_sub(1);

        let cycles = self.write(io, mmu, result) + cycles;

        self.registers.set_zf(result == 0);
        self.registers.set_nf(true);
        self.registers.set_hf(val & 0xF == 0);

        cycles + FETCH_T_CYCLES
    }

    pub fn swap<IO: Copy>(&mut self, mmu: &mut Mmu, io: IO) -> TCycles
    where
        Self: In8<IO> + Out8<IO>,
    {
        let (val, cycles) = self.read(io, mmu);
        let res = val >> 4 | val << 4;
        
        let cycles = self.write(io, mmu, res) + cycles;

        self.registers.set_zf(res == 0);
        self.registers.set_nf(false);
        self.registers.set_hf(false);
        self.registers.set_cf(false);

        cycles + FETCH_T_CYCLES
    }

    pub fn daa(&mut self) -> TCycles
    {
        // shameless ripoff...
        let mut val = self.registers.a;
        let mut correction = if self.registers.cf() { 0x60 } else { 0x00 };

        if self.registers.hf() { correction |= 0x06; };

        if !self.registers.nf() {
            if val & 0x0F > 0x09 { correction |= 0x06; };
            if val > 0x99 { correction |= 0x60; };
            val = val.wrapping_add(correction);
        } else {
            val = val.wrapping_sub(correction);
        }

        self.registers.set_zf(val == 0);
        self.registers.set_hf(false);
        self.registers.set_cf(correction >= 0x60);

        self.registers.a = val;

        FETCH_T_CYCLES
    }

    pub fn cpl(&mut self) -> TCycles {
        self.registers.a = !self.registers.a;

        self.registers.set_nf(true);
        self.registers.set_hf(true);

        FETCH_T_CYCLES
    }

    pub fn ccf(&mut self) -> TCycles {
        self.registers.set_cf(!self.registers.cf());

        self.registers.set_nf(false);
        self.registers.set_hf(false);

        FETCH_T_CYCLES
    }

    pub fn scf(&mut self) -> TCycles {
        self.registers.set_cf(true);

        self.registers.set_nf(false);
        self.registers.set_hf(false);

        FETCH_T_CYCLES
    }

    pub fn nop(&mut self) -> TCycles {
        FETCH_T_CYCLES
    }

    
    pub fn halt(&mut self) -> TCycles {
        self.halt = true;

        FETCH_T_CYCLES
    }

    pub fn stop(&mut self) -> TCycles {
        unimplemented!("stop is unimplemented");
    }

    pub fn di(&mut self) -> TCycles {
        self.ime = false;
        
        FETCH_T_CYCLES
    }

    pub fn ei(&mut self) -> TCycles {
        self.ime = true;
        
        FETCH_T_CYCLES
    }

    pub fn rlca(&mut self) -> TCycles {
        self.registers.a = self.alu_rlc(self.registers.a);
        self.registers.set_zf(false);

        FETCH_T_CYCLES
    }

    pub fn rla(&mut self) -> TCycles {
        self.registers.a = self.alu_rl(self.registers.a);
        self.registers.set_zf(false);

        FETCH_T_CYCLES
    }

    pub fn rrca(&mut self) -> TCycles {
        self.registers.a = self.alu_rrc(self.registers.a);
        self.registers.set_zf(false);

        FETCH_T_CYCLES
    }

    pub fn rra(&mut self) -> TCycles {
        self.registers.a = self.alu_rr(self.registers.a);
        self.registers.set_zf(false);

        FETCH_T_CYCLES
    }

    pub fn rlc<IO: Copy>(&mut self, mmu: &mut Mmu, io: IO) -> TCycles
    where
        Self: In8<IO> + Out8<IO>
    {
        let (val, cycles) = self.read(io, mmu);

        let result = self.alu_rlc(val);

        let cycles = self.write(io, mmu, result) + cycles;


        cycles + FETCH_T_CYCLES
    }

    pub fn rl<IO: Copy>(&mut self, mmu: &mut Mmu, io: IO) -> TCycles
    where
        Self: In8<IO> + Out8<IO>
    {
        let (val, cycles) = self.read(io, mmu);

        let result = self.alu_rl(val);

        let cycles = self.write(io, mmu, result) + cycles;


        cycles + FETCH_T_CYCLES
    }

    pub fn rrc<IO: Copy>(&mut self, mmu: &mut Mmu, io: IO) -> TCycles
    where
        Self: In8<IO> + Out8<IO>
    {
        let (val, cycles) = self.read(io, mmu);

        let result = self.alu_rrc(val);

        let cycles = self.write(io, mmu, result) + cycles;


        cycles + FETCH_T_CYCLES
    }

    pub fn rr<IO: Copy>(&mut self, mmu: &mut Mmu, io: IO) -> TCycles
    where
        Self: In8<IO> + Out8<IO>
    {
        let (val, cycles) = self.read(io, mmu);

        let result = self.alu_rr(val);

        let cycles = self.write(io, mmu, result) + cycles;


        cycles + FETCH_T_CYCLES
    }

    pub fn sla<IO: Copy>(&mut self, mmu: &mut Mmu, io: IO) -> TCycles
    where
        Self: In8<IO> + Out8<IO>
    {
        let (val, cycles) = self.read(io, mmu);

        let result = self.alu_sla(val);

        let cycles = self.write(io, mmu, result) + cycles;


        cycles + FETCH_T_CYCLES
    }

    pub fn sra<IO: Copy>(&mut self, mmu: &mut Mmu, io: IO) -> TCycles
    where
        Self: In8<IO> + Out8<IO>
    {
        let (val, cycles) = self.read(io, mmu);

        let result = self.alu_sra(val);

        let cycles = self.write(io, mmu, result) + cycles;


        cycles + FETCH_T_CYCLES
    }

    pub fn srl<IO: Copy>(&mut self, mmu: &mut Mmu, io: IO) -> TCycles
    where
        Self: In8<IO> + Out8<IO>
    {
        let (val, cycles) = self.read(io, mmu);

        let result = self.alu_srl(val);

        let cycles = self.write(io, mmu, result) + cycles;


        cycles + FETCH_T_CYCLES
    }

    pub fn bit<IO: Copy>(&mut self, mmu: &mut Mmu, io: IO, bit: u8) -> TCycles
    where
        Self: In8<IO> + Out8<IO>
    {
        let (val, cycles) = self.read(io, mmu);

        self.registers.set_zf(val & (1 << bit) == 0);
        self.registers.set_nf(false);
        self.registers.set_hf(true);


        cycles + (FETCH_T_CYCLES * 2)
    }

    pub fn set<IO: Copy>(&mut self, mmu: &mut Mmu, io: IO, bit: u8) -> TCycles
    where
        Self: In8<IO> + Out8<IO>
    {
        let (val, cycles) = self.read(io, mmu);
        let cycles = self.write(io, mmu, val | (1 << bit)) + cycles;


        cycles + (FETCH_T_CYCLES * 2)
    }

    pub fn res<IO: Copy>(&mut self, mmu: &mut Mmu, io: IO, bit: u8) -> TCycles
    where
        Self: In8<IO> + Out8<IO>
    {
        let (val, cycles) = self.read(io, mmu);
        let cycles = self.write(io, mmu, val & !(1 << bit)) + cycles;


        cycles + (FETCH_T_CYCLES * 2)
    }

    pub fn jp(&mut self, mmu: &mut Mmu) -> TCycles {
        let addr = self.fetch_word(mmu);
        self.registers.pc = addr;

        (FETCH_T_CYCLES * 2) + MEM_ACCESS_T_CYCLES_16
    }

    pub fn jp_cc(&mut self, mmu: &mut Mmu, condition: Condition) -> TCycles {
        let addr = self.fetch_word(mmu);
        
        if self.test_condition(condition) {
            self.registers.pc = addr;
            (FETCH_T_CYCLES * 2) + MEM_ACCESS_T_CYCLES_16
        } else {
            FETCH_T_CYCLES + MEM_ACCESS_T_CYCLES_16
        }
    }

    pub fn jp_hl(&mut self) -> TCycles {
        self.registers.pc = self.registers.read16(Reg16::HL);

        FETCH_T_CYCLES
    }

    pub fn jr(&mut self, mmu: &mut Mmu) -> TCycles {
        let addr = self.fetch_byte(mmu) as i8 as u16;

        self.registers.pc = self.registers.pc.wrapping_add(addr);

        (FETCH_T_CYCLES * 2) + MEM_ACCESS_T_CYCLES_8
    }


    pub fn jr_cc(&mut self, mmu: &mut Mmu, condition: Condition) -> TCycles {
        let addr = self.fetch_byte(mmu) as i8 as u16;

        
        if self.test_condition(condition) {
            self.registers.pc = self.registers.pc.wrapping_add(addr);
            (FETCH_T_CYCLES * 2) + MEM_ACCESS_T_CYCLES_8
        } else {
            FETCH_T_CYCLES + MEM_ACCESS_T_CYCLES_8
        }
    }

    pub fn call(&mut self, mmu: &mut Mmu) -> TCycles {
        self.ctrl_call(mmu, true)
    }

    pub fn call_cc(&mut self, mmu: &mut Mmu, condition: Condition) -> TCycles {
        self.ctrl_call(mmu, self.test_condition(condition))
    }

    pub fn rst(&mut self, mmu: &mut Mmu, addr: u8) -> TCycles {
        self.push(mmu, self.registers.pc);

        self.registers.pc = addr as u16;

        (FETCH_T_CYCLES * 2) + MEM_ACCESS_T_CYCLES_16
    }

    pub fn ret(&mut self, mmu: &mut Mmu) -> TCycles {
        self.ctrl_ret(mmu, true);

        (FETCH_T_CYCLES * 2) + MEM_ACCESS_T_CYCLES_16
    }

    pub fn ret_cc(&mut self, mmu: &mut Mmu, condition: Condition) -> TCycles {
        self.ctrl_ret(mmu, self.test_condition(condition))
    }

    pub fn reti(&mut self, mmu: &mut Mmu) -> TCycles {
        self.ctrl_ret(mmu, true);
        self.ime = true;

        (FETCH_T_CYCLES * 2) + MEM_ACCESS_T_CYCLES_16
    }






    // 16 bit operations
    pub fn load16_imm(&mut self, mmu: &mut Mmu, reg: Reg16) -> TCycles {
        let immediate_val = self.fetch_word(mmu);

        self.registers.write16(reg, immediate_val);

        MEM_ACCESS_T_CYCLES_16 + FETCH_T_CYCLES
    }

    pub fn load16_sp_hl(&mut self) -> TCycles {
        self.registers
            .write16(Reg16::SP, self.registers.read16(Reg16::HL));

        8
    }

    pub fn load16_hl_sp_n(&mut self, mmu: &mut Mmu) -> TCycles {
        let imm_val = self.fetch_byte(mmu) as i8 as u16;

        let sp = self.registers.sp;

        self.registers.write16(Reg16::HL, sp.wrapping_add(imm_val));

        self.registers.set_zf(false);
        self.registers.set_nf(false);
        self.registers
            .set_hf(u16::test_add_carry_bit(3, sp, imm_val));
        self.registers
            .set_cf(u16::test_add_carry_bit(7, sp, imm_val));

        12
    }

    pub fn load16_nn_sp(&mut self, mmu: &mut Mmu) -> TCycles {
        let addr = self.fetch_word(mmu);
        let val = self.registers.read16(Reg16::SP);

        mmu.write_word(addr, val);

        20
    }

    pub fn push16(&mut self, mmu: &mut Mmu, reg: Reg16) -> TCycles {
        let val = self.registers.read16(reg);
        self.push(mmu, val);

        16
    }

    pub fn pop16(&mut self, mmu: &mut Mmu, reg: Reg16) -> TCycles {
        let val = self.pop(mmu);

        self.registers.write16(reg, val);

        16
    }

    // add 16 bit reg to HL
    pub fn add16(&mut self, reg: Reg16) -> TCycles {
        let hl = self.registers.read16(Reg16::HL);

        let val = self.registers.read16(reg);

        let (res, c) = hl.overflowing_add(val);

        self.registers.set_nf(false);
        self.registers.set_hf(u16::test_add_carry_bit(11, hl, val));
        self.registers.set_cf(c);

        self.registers.write16(Reg16::HL, res);

        8
    }

    pub fn add16_sp_n(&mut self, mmu: &mut Mmu) -> TCycles {
        let sp = self.registers.read16(Reg16::SP);

        let val = self.fetch_byte(mmu) as i8 as i16 as u16;

        let res = sp.wrapping_add(val);

        self.registers.set_zf(false);
        self.registers.set_nf(false);
        self.registers.set_hf(u16::test_add_carry_bit(3, sp, val));
        self.registers.set_cf(u16::test_add_carry_bit(7, sp, val));

        self.registers.write16(Reg16::SP, res);

        16
    }

    pub fn inc16(&mut self, reg: Reg16) -> TCycles {
        let val = self.registers.read16(reg).wrapping_add(1);
        self.registers.write16(reg, val);

        8
    }

    pub fn dec16(&mut self, reg: Reg16) -> TCycles {
        let val = self.registers.read16(reg).wrapping_sub(1);
        self.registers.write16(reg, val);

        8
    }

    pub fn undefined(&mut self, opcode: u8) -> ! {
        panic!("Undefined opcode {:04X}", opcode)
    }

    pub fn cb_opcodes(&mut self, mmu: &mut Mmu) -> TCycles {
        self.fetch_byte(mmu);

        todo!()
    }
}
