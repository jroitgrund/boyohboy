use crate::gb::bits::{clear_bit, get_bits, get_lsb, set_bit};
use crate::gb::memory::map::{IE, IF};
use crate::gb::memory::Memory;
use crate::gb::AccessType::{Direct, Indirect};
use crate::gb::{AccessType, R16_HL};
use anyhow::anyhow;
use anyhow::Result;
use log::info;

pub enum Interrupts {
    VBlank,
    Lcd,
    Timer,
    _Serial,
    _Joypad,
}

pub struct InterruptResult {
    pub interrupt_requested: bool,
    pub interrupts_enabled: bool,
    pub cycles: u8,
}

pub struct InstructionResult {
    pub cycles: u8,
    pub is_halt: bool,
}

pub struct Cpu {
    pub ime: bool,
    a: u8,
    f: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            ime: false,
            a: 0x01,
            f: 0xB0,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x1,
            l: 0x4D,
            sp: 0xFFFE,
            pc: 0x0100,
        }
    }

    pub fn _log(&mut self, memory: &mut Memory) -> Result<()> {
        let pc = memory.read(self.pc)?;
        let pc_1 = memory.read(self.pc + 1)?;
        let pc_2 = memory.read(self.pc + 2)?;
        let pc_3 = memory.read(self.pc + 3)?;
        info!(
            "A: {:02X?} F: {:02X?} B: {:02X?} C: {:02X?} D: {:02X?} E: {:02X?} H: {:02X?} L: {:02X?} SP: {:04X?} PC: 00:{:04X?} ({:02X?} {:02X?} {:02X?} {:02X?})\n",
            self.a, self.f, self.b, self.c, self.d, self.e, self.h, self.l, self.sp, self.pc, pc, pc_1, pc_2, pc_3
        );
        Ok(())
    }

    pub fn execute_next_instruction_with_halt_bug(
        &mut self,
        memory: &mut Memory,
    ) -> Result<InstructionResult> {
        self.execute_next_instruction_impl(memory, true)
    }

    pub fn execute_next_instruction(&mut self, memory: &mut Memory) -> Result<InstructionResult> {
        self.execute_next_instruction_impl(memory, false)
    }

    pub fn handle_interrupts(&mut self, memory: &mut Memory) -> Result<InterruptResult> {
        let if_reg = memory.read(IF)?;
        let ie_reg = memory.read(IE)?;
        let interrupts = if_reg & ie_reg & 0x1F;

        Ok(if self.ime && interrupts != 0 {
            self.ime = false;
            self.push_16(memory, self.pc)?;
            let lsb = get_lsb(interrupts);
            memory.write(IF, if_reg & (!lsb))?;
            self.pc = match lsb {
                1 => Ok(0x40),
                2 => Ok(0x48),
                4 => Ok(0x50),
                8 => Ok(0x58),
                16 => Ok(0x60),
                _ => Err(anyhow!("Unexpected lsb {}", lsb)),
            }?;
            InterruptResult {
                interrupt_requested: true,
                interrupts_enabled: true,
                cycles: 5,
            }
        } else {
            InterruptResult {
                interrupts_enabled: self.ime,
                interrupt_requested: interrupts != 0,
                cycles: 0,
            }
        })
    }

    fn read_and_increment_pc(&mut self, memory: &mut Memory) -> Result<u8> {
        let result = memory.read(self.pc)?;
        self.pc += 1;
        Ok(result)
    }

    fn read_r8(&mut self, memory: &mut Memory, r: u8) -> Result<(u8, AccessType)> {
        match r {
            0 => Ok((self.b, Direct)),
            1 => Ok((self.c, Direct)),
            2 => Ok((self.d, Direct)),
            3 => Ok((self.e, Direct)),
            4 => Ok((self.h, Direct)),
            5 => Ok((self.l, Direct)),
            6 => Ok((memory.read(self.read_hl()?)?, Indirect)),
            7 => Ok((self.a, Direct)),
            _ => Err(anyhow!("Unknown register {}", r)),
        }
    }

    fn write_r8(&mut self, memory: &mut Memory, r: u8, val: u8) -> anyhow::Result<AccessType> {
        match r {
            0 => {
                self.b = val;
                Ok(Direct)
            }
            1 => {
                self.c = val;
                Ok(Direct)
            }
            2 => {
                self.d = val;
                Ok(Direct)
            }
            3 => {
                self.e = val;
                Ok(Direct)
            }
            4 => {
                self.h = val;
                Ok(Direct)
            }
            5 => {
                self.l = val;
                Ok(Direct)
            }
            6 => {
                memory.write(self.read_hl()?, val)?;
                Ok(Indirect)
            }
            7 => {
                self.a = val;
                Ok(Direct)
            }
            _ => Err(anyhow!("Unknown register {}", r)),
        }
    }

    fn read_r16(&self, r: u8) -> Result<u16> {
        match r {
            0 => Ok(u16::from_be_bytes([self.b, self.c])),
            1 => Ok(u16::from_be_bytes([self.d, self.e])),
            2 => Ok(u16::from_be_bytes([self.h, self.l])),
            3 => Ok(self.sp),
            _ => Err(anyhow!("Unknown register {}", r)),
        }
    }

    fn read_hl(&self) -> Result<u16> {
        self.read_r16(R16_HL)
    }

    fn write_r16(&mut self, r: u8, val: u16) -> Result<()> {
        let bytes = val.to_be_bytes();
        match r {
            0 => {
                self.b = bytes[0];
                self.c = bytes[1];
                Ok(())
            }
            1 => {
                self.d = bytes[0];
                self.e = bytes[1];
                Ok(())
            }
            2 => {
                self.h = bytes[0];
                self.l = bytes[1];
                Ok(())
            }
            3 => {
                self.sp = val;
                Ok(())
            }
            _ => Err(anyhow!("Unknown register {}", r)),
        }
    }

    fn write_hl(&mut self, val: u16) -> Result<()> {
        self.write_r16(R16_HL, val)
    }

    fn r16_mem(&mut self, r: u8) -> anyhow::Result<u16> {
        match r {
            0 | 1 => self.read_r16(r),
            2 => {
                let hl = self.read_r16(R16_HL)?;
                self.write_hl(hl + 1)?;
                Ok(hl)
            }
            3 => {
                let hl = self.read_r16(R16_HL)?;
                self.write_hl(hl - 1)?;
                Ok(hl)
            }
            _ => Err(anyhow!("Unknown register {}", r)),
        }
    }

    fn read_r16_stk(&self, r: u8) -> Result<u16> {
        match r {
            0..=2 => self.read_r16(r),
            3 => Ok(u16::from_be_bytes([self.a, self.f])),
            _ => Err(anyhow!("Unknown register {}", r)),
        }
    }

    fn write_r16_stk(&mut self, r: u8, val: u16) -> Result<()> {
        match r {
            0..=2 => self.write_r16(r, val),
            3 => {
                let bytes = val.to_be_bytes();
                self.a = bytes[0];
                self.f = bytes[1] & 0xF0;
                Ok(())
            }
            _ => Err(anyhow!("Unknown register {}", r)),
        }
    }

    fn read_n8(&mut self, memory: &mut Memory) -> Result<u8> {
        self.read_and_increment_pc(memory)
    }

    fn read_n16(&mut self, memory: &mut Memory) -> Result<u16> {
        Ok(u16::from_le_bytes([
            self.read_and_increment_pc(memory)?,
            self.read_and_increment_pc(memory)?,
        ]))
    }

    fn get_z(&self) -> bool {
        self.get_flag(0)
    }

    fn get_n(&self) -> bool {
        self.get_flag(1)
    }

    fn get_h(&self) -> bool {
        self.get_flag(2)
    }

    fn get_c(&self) -> bool {
        self.get_flag(3)
    }

    fn get_flag(&self, flag_i: u8) -> bool {
        let shifts = 7 - flag_i;
        (self.f >> shifts) & 1 == 1
    }

    fn set_z(&mut self, set: bool) {
        self.set_flag(0, set)
    }

    fn set_n(&mut self, set: bool) {
        self.set_flag(1, set)
    }

    fn set_h(&mut self, set: bool) {
        self.set_flag(2, set)
    }

    fn set_c(&mut self, set: bool) {
        self.set_flag(3, set)
    }

    fn set_flag(&mut self, flag_i: u8, set: bool) {
        let shifts = 7 - flag_i;
        let set_mask = 1u8 << shifts;
        let clear_mask = !set_mask;
        self.f = if set {
            self.f | set_mask
        } else {
            self.f & clear_mask
        }
    }

    fn read_cond(&self, cond: u8) -> anyhow::Result<bool> {
        match cond {
            0 => Ok(!self.get_z()),
            1 => Ok(self.get_z()),
            2 => Ok(!self.get_c()),
            3 => Ok(self.get_c()),
            _ => Err(anyhow!("Unknown condition {}", cond)),
        }
    }

    fn push_8(&mut self, memory: &mut Memory, val: u8) -> Result<()> {
        self.sp -= 1;
        memory.write(self.sp, val)
    }

    fn push_16(&mut self, memory: &mut Memory, val: u16) -> Result<()> {
        let bytes = val.to_be_bytes();
        self.push_8(memory, bytes[0])?;
        self.push_8(memory, bytes[1])
    }

    fn pop_8(&mut self, memory: &mut Memory) -> Result<u8> {
        let res = memory.read(self.sp)?;
        self.sp += 1;
        Ok(res)
    }

    fn pop_16(&mut self, memory: &mut Memory) -> Result<u16> {
        Ok(u16::from_le_bytes([
            self.pop_8(memory)?,
            self.pop_8(memory)?,
        ]))
    }

    fn execute_next_instruction_impl(
        &mut self,
        memory: &mut Memory,
        halt_bug: bool,
    ) -> Result<InstructionResult> {
        let instruction = self.read_and_increment_pc(memory)?;
        if halt_bug {
            self.pc -= 1;
        }
        let cycles = match instruction {
            0o0 => self.nop(),
            0o01 | 0o21 | 0o41 | 0o61 => self.ld_r16_n16(memory, instruction),
            0o02 | 0o22 | 0o42 | 0o62 => self.ld_ind_r16_a(memory, instruction),
            0o12 | 0o32 | 0o52 | 0o72 => self.ld_a_ind_r16(memory, instruction),
            0o10 => self.ld_ind_n16_sp(memory),
            0o03 | 0o23 | 0o43 | 0o63 => self.inc_r16(instruction),
            0o13 | 0o33 | 0o53 | 0o73 => self.dec_r16(instruction),
            0o11 | 0o31 | 0o51 | 0o71 => self.add_hl_r16(instruction),
            0o04 | 0o14 | 0o24 | 0o34 | 0o44 | 0o54 | 0o64 | 0o74 => {
                self.inc_r8(memory, instruction)
            }
            0o05 | 0o15 | 0o25 | 0o35 | 0o45 | 0o55 | 0o65 | 0o75 => {
                self.dec_r8(memory, instruction)
            }
            0o06 | 0o16 | 0o26 | 0o36 | 0o46 | 0o56 | 0o66 | 0o76 => {
                self.ld_r8_n8(memory, instruction)
            }
            0o07 => self.rlca(),
            0o17 => self.rrca(),
            0o27 => self.rla(),
            0o37 => self.rra(),
            0o47 => self.daa(),
            0o57 => self.cpl(),
            0o67 => self.scf(),
            0o77 => self.ccf(),
            0o30 => self.jr_n8(memory),
            0o40 | 0o50 | 0o60 | 0o70 => self.jr_cond_n8(memory, instruction),
            0o20 => self.stop(),
            0o166 => self.halt(),
            (0o100..=0o177) => self.ld_r8_r8(memory, instruction),
            0o200..=0o207 => self.add_a_r8(memory, instruction),
            0o210..=0o217 => self.adc_a_r8(memory, instruction),
            0o220..=0o227 => self.sub_a_r8(memory, instruction),
            0o230..=0o237 => self.sbc_a_r8(memory, instruction),
            0o240..=0o247 => self.and_a_r8(memory, instruction),
            0o250..=0o257 => self.xor_a_r8(memory, instruction),
            0o260..=0o267 => self.or_a_r8(memory, instruction),
            0o270..=0o277 => self.cp_a_r8(memory, instruction),
            0o306 => self.add_a_n8(memory),
            0o316 => self.adc_a_n8(memory),
            0o326 => self.sub_a_n8(memory),
            0o336 => self.sbc_a_n8(memory),
            0o346 => self.and_a_n8(memory),
            0o356 => self.xor_a_n8(memory),
            0o366 => self.or_a_n8(memory),
            0o376 => self.cp_a_n8(memory),
            0o300 | 0o310 | 0o320 | 0o330 => self.ret_cond(memory, instruction),
            0o311 => self.ret(memory),
            0o331 => self.reti(memory),
            0o302 | 0o312 | 0o322 | 0o332 => self.jp_cond_n16(memory, instruction),
            0o303 => self.jp_n16(memory),
            0o351 => self.jp_hl(),
            0o304 | 0o314 | 0o324 | 0o334 => self.call_cond_n16(memory, instruction),
            0o315 => self.call_n16(memory),
            0o307 | 0o317 | 0o327 | 0o337 | 0o347 | 0o357 | 0o367 | 0o377 => {
                self.rst(memory, instruction)
            }
            0o301 | 0o321 | 0o341 | 0o361 => self.pop(memory, instruction),
            0o305 | 0o325 | 0o345 | 0o365 => self.push(memory, instruction),
            0o313 => self.prefix(memory),
            0o342 => self.ldh_ind_c_a(memory),
            0o340 => self.ldh_ind_n8_a(memory),
            0o352 => self.ld_ind_n16_a(memory),
            0o362 => self.ldh_a_ind_c(memory),
            0o360 => self.ldh_a_ind_n8(memory),
            0o372 => self.ld_a_ind_n16(memory),
            0o350 => self.add_sp_n8(memory),
            0o370 => self.ld_hl_sp_plus_n8(memory),
            0o371 => self.ld_sp_hl(),
            0o363 => self.di(),
            0o373 => self.ei(),
            _ => Err(anyhow!("Unknown instruction {}", instruction)),
        }?;

        Ok(InstructionResult {
            cycles,
            is_halt: instruction == 0o166,
        })
    }

    fn nop(&self) -> Result<u8> {
        Ok(2)
    }

    fn ld_r16_n16(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let operand = self.read_n16(memory)?;
        let reg = get_bits(instruction, 5, 4);
        self.write_r16(reg, operand)?;
        Ok(3)
    }

    fn ld_ind_r16_a(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 5, 4);
        let address = self.r16_mem(reg)?;
        memory.write(address, self.a)?;
        Ok(2)
    }

    fn ld_a_ind_r16(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 5, 4);
        let address = self.r16_mem(reg)?;
        self.a = memory.read(address)?;
        Ok(2)
    }

    fn ld_ind_n16_sp(&mut self, memory: &mut Memory) -> Result<u8> {
        let address = self.read_n16(memory)?;
        memory.write_16(address, self.sp)?;
        Ok(5)
    }

    fn inc_r16(&mut self, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 5, 4);
        self.write_r16(reg, self.read_r16(reg)?.wrapping_add(1))?;
        Ok(2)
    }

    fn dec_r16(&mut self, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 5, 4);
        self.write_r16(reg, self.read_r16(reg)?.wrapping_sub(1))?;
        Ok(2)
    }

    fn add_hl_r16(&mut self, instruction: u8) -> Result<u8> {
        let hl = self.read_hl()?;
        let reg = get_bits(instruction, 5, 4);
        let operand = self.read_r16(reg)?;
        let (sum, overflow) = hl.overflowing_add(operand);
        self.set_n(false);
        self.set_h(is_add_half_carry_16(hl, operand));
        self.set_c(overflow);
        self.write_hl(sum)?;
        Ok(2)
    }

    fn inc_r8(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 5, 3);
        let (operand, access_type) = self.read_r8(memory, reg)?;
        let result = operand.wrapping_add(1);
        self.write_r8(memory, reg, result)?;
        self.set_z(result == 0);
        self.set_n(false);
        self.set_h(is_add_half_carry_8(operand, 1));
        Ok(match access_type {
            Direct => 1,
            Indirect => 3,
        })
    }

    fn dec_r8(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 5, 3);
        let (operand, access_type) = self.read_r8(memory, reg)?;
        let result = operand.wrapping_sub(1);
        self.write_r8(memory, reg, result)?;
        self.set_z(result == 0);
        self.set_n(true);
        self.set_h(is_sub_half_carry_8(operand, 1));
        Ok(match access_type {
            Direct => 1,
            Indirect => 3,
        })
    }

    fn ld_r8_n8(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 5, 3);
        let operand = self.read_n8(memory)?;
        Ok(match self.write_r8(memory, reg, operand)? {
            Direct => 2,
            Indirect => 3,
        })
    }

    fn rlca(&mut self) -> Result<u8> {
        let (result, carry) = rotate_left(self.a);
        self.set_z(false);
        self.set_n(false);
        self.set_h(false);
        self.set_c(carry);
        self.a = result;
        Ok(1)
    }

    fn rrca(&mut self) -> Result<u8> {
        let (result, carry) = rotate_right(self.a);
        self.set_z(false);
        self.set_n(false);
        self.set_h(false);
        self.set_c(carry);
        self.a = result;
        Ok(1)
    }

    fn rla(&mut self) -> Result<u8> {
        let (result, carry) = rotate_left_with_carry(self.a, self.get_c());
        self.set_z(false);
        self.set_n(false);
        self.set_h(false);
        self.set_c(carry);
        self.a = result;
        Ok(1)
    }

    fn rra(&mut self) -> Result<u8> {
        let (result, carry) = rotate_right_with_carry(self.a, self.get_c());
        self.set_z(false);
        self.set_n(false);
        self.set_h(false);
        self.set_c(carry);
        self.a = result;
        Ok(1)
    }

    fn daa(&mut self) -> Result<u8> {
        if !self.get_n() {
            if self.get_c() || self.a > 0x99 {
                self.a = self.a.wrapping_add(0x60);
                self.set_c(true);
            }
            if self.get_h() || (self.a & 0x0f) > 0x09 {
                self.a = self.a.wrapping_add(0x6);
            }
        } else {
            if self.get_c() {
                self.a = self.a.wrapping_sub(0x60);
            }
            if self.get_h() {
                self.a = self.a.wrapping_sub(0x6);
            }
        }
        self.set_z(self.a == 0);
        self.set_h(false);
        Ok(1)
    }

    fn cpl(&mut self) -> Result<u8> {
        self.a = !self.a;
        self.set_n(true);
        self.set_h(true);
        Ok(1)
    }

    fn scf(&mut self) -> Result<u8> {
        self.set_n(false);
        self.set_h(false);
        self.set_c(true);
        Ok(1)
    }

    fn ccf(&mut self) -> Result<u8> {
        self.set_n(false);
        self.set_h(false);
        self.set_c(!self.get_c());
        Ok(1)
    }

    fn jr_n8(&mut self, memory: &mut Memory) -> Result<u8> {
        let offset = i16::from(self.read_n8(memory)? as i8);
        self.pc = self.pc.wrapping_add_signed(offset);
        Ok(3)
    }

    fn jr_cond_n8(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let cond = get_bits(instruction, 4, 3);
        let offset = i16::from(self.read_n8(memory)? as i8);
        Ok(if self.read_cond(cond)? {
            self.pc = self.pc.wrapping_add_signed(offset);
            3
        } else {
            2
        })
    }

    fn stop(&mut self) -> Result<u8> {
        Err(anyhow!("Unimplemented"))
    }

    fn halt(&mut self) -> Result<u8> {
        Ok(1)
    }

    fn ld_r8_r8(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let source = get_bits(instruction, 2, 0);
        let dest = get_bits(instruction, 5, 3);
        let (value, access_type) = self.read_r8(memory, source)?;
        Ok(match self.write_r8(memory, dest, value)? + access_type {
            Direct => 1,
            Indirect => 2,
        })
    }

    fn add_a_r8(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(memory, reg)?;
        self.a = self.add_and_set_flags_no_carry(self.a, operand);
        Ok(match access_type {
            Direct => 1,
            Indirect => 2,
        })
    }

    fn adc_a_r8(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(memory, reg)?;
        self.a = self.add_and_set_flags_with_carry(self.a, operand);
        Ok(match access_type {
            Direct => 1,
            Indirect => 2,
        })
    }

    fn sub_a_r8(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(memory, reg)?;
        self.a = self.sub_and_set_flags_no_carry(self.a, operand);
        Ok(match access_type {
            Direct => 1,
            Indirect => 2,
        })
    }
    fn sbc_a_r8(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(memory, reg)?;
        self.a = self.sub_and_set_flags_with_carry(self.a, operand);
        Ok(match access_type {
            Direct => 1,
            Indirect => 2,
        })
    }
    fn and_a_r8(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(memory, reg)?;
        self.a &= operand;
        self.set_z(self.a == 0);
        self.set_n(false);
        self.set_h(true);
        self.set_c(false);
        Ok(match access_type {
            Direct => 1,
            Indirect => 2,
        })
    }
    fn xor_a_r8(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(memory, reg)?;
        self.a ^= operand;
        self.set_z(self.a == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(false);
        Ok(match access_type {
            Direct => 1,
            Indirect => 2,
        })
    }
    fn or_a_r8(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(memory, reg)?;
        self.a |= operand;
        self.set_z(self.a == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(false);
        Ok(match access_type {
            Direct => 1,
            Indirect => 2,
        })
    }
    fn cp_a_r8(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(memory, reg)?;
        self.sub_and_set_flags_no_carry(self.a, operand);
        Ok(match access_type {
            Direct => 1,
            Indirect => 2,
        })
    }

    fn add_a_n8(&mut self, memory: &mut Memory) -> Result<u8> {
        let operand = self.read_n8(memory)?;
        self.a = self.add_and_set_flags_no_carry(self.a, operand);
        Ok(2)
    }

    fn adc_a_n8(&mut self, memory: &mut Memory) -> Result<u8> {
        let operand = self.read_n8(memory)?;
        self.a = self.add_and_set_flags_with_carry(self.a, operand);
        Ok(2)
    }

    fn sub_a_n8(&mut self, memory: &mut Memory) -> Result<u8> {
        let operand = self.read_n8(memory)?;
        self.a = self.sub_and_set_flags_no_carry(self.a, operand);
        Ok(2)
    }
    fn sbc_a_n8(&mut self, memory: &mut Memory) -> Result<u8> {
        let operand = self.read_n8(memory)?;
        self.a = self.sub_and_set_flags_with_carry(self.a, operand);
        Ok(2)
    }
    fn and_a_n8(&mut self, memory: &mut Memory) -> Result<u8> {
        let operand = self.read_n8(memory)?;
        self.a &= operand;
        self.set_z(self.a == 0);
        self.set_n(false);
        self.set_h(true);
        self.set_c(false);
        Ok(2)
    }
    fn xor_a_n8(&mut self, memory: &mut Memory) -> Result<u8> {
        let operand = self.read_n8(memory)?;
        self.a ^= operand;
        self.set_z(self.a == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(false);
        Ok(2)
    }
    fn or_a_n8(&mut self, memory: &mut Memory) -> Result<u8> {
        let operand = self.read_n8(memory)?;
        self.a |= operand;
        self.set_z(self.a == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(false);
        Ok(2)
    }
    fn cp_a_n8(&mut self, memory: &mut Memory) -> Result<u8> {
        let operand = self.read_n8(memory)?;
        self.sub_and_set_flags(self.a, operand, None);
        Ok(2)
    }

    fn ret_cond(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let cond = get_bits(instruction, 4, 3);
        let should_ret = self.read_cond(cond)?;
        Ok(if should_ret {
            let dest = self.pop_16(memory)?;
            self.pc = dest;
            5
        } else {
            2
        })
    }

    fn ret(&mut self, memory: &mut Memory) -> Result<u8> {
        self.pc = self.pop_16(memory)?;
        Ok(4)
    }

    fn reti(&mut self, memory: &mut Memory) -> Result<u8> {
        self.pc = self.pop_16(memory)?;
        self.ime = true;
        Ok(4)
    }

    fn jp_cond_n16(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let cond = get_bits(instruction, 4, 3);
        let dest = self.read_n16(memory)?;
        Ok(if self.read_cond(cond)? {
            self.pc = dest;
            4
        } else {
            3
        })
    }

    fn jp_n16(&mut self, memory: &mut Memory) -> Result<u8> {
        self.pc = self.read_n16(memory)?;
        Ok(4)
    }

    fn jp_hl(&mut self) -> Result<u8> {
        self.pc = self.read_hl()?;
        Ok(1)
    }

    fn call_cond_n16(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let cond = get_bits(instruction, 4, 3);
        let addr = self.read_n16(memory)?;
        Ok(if self.read_cond(cond)? {
            self.push_16(memory, self.pc)?;
            self.pc = addr;
            6
        } else {
            3
        })
    }

    fn call_n16(&mut self, memory: &mut Memory) -> Result<u8> {
        let addr = self.read_n16(memory)?;
        self.push_16(memory, self.pc)?;
        self.pc = addr;
        Ok(6)
    }

    fn rst(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let operand = get_bits(instruction, 5, 3);
        let addr = u16::from(operand) * 8;
        self.push_16(memory, self.pc)?;
        self.pc = addr;
        Ok(4)
    }

    fn pop(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let value = self.pop_16(memory)?;
        let reg = get_bits(instruction, 5, 4);
        self.write_r16_stk(reg, value)?;
        Ok(3)
    }

    fn push(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 5, 4);
        self.push_16(memory, self.read_r16_stk(reg)?)?;
        Ok(4)
    }

    fn ldh_ind_c_a(&mut self, memory: &mut Memory) -> Result<u8> {
        memory.write(0xFF00 + u16::from(self.c), self.a)?;
        Ok(2)
    }

    fn ldh_ind_n8_a(&mut self, memory: &mut Memory) -> Result<u8> {
        let address = 0xFF00 + u16::from(self.read_n8(memory)?);
        memory.write(address, self.a)?;
        Ok(3)
    }

    fn ld_ind_n16_a(&mut self, memory: &mut Memory) -> Result<u8> {
        let address = self.read_n16(memory)?;
        memory.write(address, self.a)?;
        Ok(4)
    }

    fn ldh_a_ind_c(&mut self, memory: &mut Memory) -> Result<u8> {
        self.a = memory.read(0xFF00 + u16::from(self.c))?;
        Ok(2)
    }

    fn ldh_a_ind_n8(&mut self, memory: &mut Memory) -> Result<u8> {
        let address = 0xFF00 + u16::from(self.read_n8(memory)?);
        self.a = memory.read(address)?;
        Ok(3)
    }

    fn ld_a_ind_n16(&mut self, memory: &mut Memory) -> Result<u8> {
        let address = self.read_n16(memory)?;
        self.a = memory.read(address)?;
        Ok(4)
    }

    fn add_sp_n8(&mut self, memory: &mut Memory) -> Result<u8> {
        let operand = self.read_n8(memory)? as i8;
        self.sp = self.add_signed_and_set_flags(self.sp, operand);
        Ok(4)
    }

    fn ld_hl_sp_plus_n8(&mut self, memory: &mut Memory) -> Result<u8> {
        let operand = self.read_n8(memory)? as i8;
        let result = self.add_signed_and_set_flags(self.sp, operand);
        self.write_hl(result)?;
        Ok(3)
    }

    fn ld_sp_hl(&mut self) -> Result<u8> {
        self.sp = self.read_hl()?;
        Ok(2)
    }

    fn di(&mut self) -> Result<u8> {
        self.ime = false;
        Ok(1)
    }

    fn ei(&mut self) -> Result<u8> {
        self.ime = true;
        Ok(1)
    }

    fn rlc(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(memory, reg)?;
        let (result, carry) = rotate_left(operand);
        self.set_z(result == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(carry);
        self.write_r8(memory, reg, result)?;
        Ok(match access_type {
            Direct => 2,
            Indirect => 4,
        })
    }

    fn rrc(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(memory, reg)?;
        let (result, carry) = rotate_right(operand);
        self.set_z(result == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(carry);
        self.write_r8(memory, reg, result)?;
        Ok(match access_type {
            Direct => 2,
            Indirect => 4,
        })
    }

    fn rl(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(memory, reg)?;
        let (result, carry) = rotate_left_with_carry(operand, self.get_c());
        self.set_z(result == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(carry);
        self.write_r8(memory, reg, result)?;
        Ok(match access_type {
            Direct => 2,
            Indirect => 4,
        })
    }

    fn rr(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(memory, reg)?;
        let (result, carry) = rotate_right_with_carry(operand, self.get_c());
        self.set_z(result == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(carry);
        self.write_r8(memory, reg, result)?;
        Ok(match access_type {
            Direct => 2,
            Indirect => 4,
        })
    }

    fn sla(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(memory, reg)?;
        let (result, carry) = shift_left(operand);
        self.set_z(result == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(carry);
        self.write_r8(memory, reg, result)?;
        Ok(match access_type {
            Direct => 2,
            Indirect => 4,
        })
    }

    fn sra(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(memory, reg)?;
        let (result, carry) = shift_right_arithmetic(operand);
        self.set_z(result == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(carry);
        self.write_r8(memory, reg, result)?;
        Ok(match access_type {
            Direct => 2,
            Indirect => 4,
        })
    }

    fn swap(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(memory, reg)?;
        let result = operand.wrapping_shr(4) + operand.wrapping_shl(4);
        self.set_z(result == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(false);
        self.write_r8(memory, reg, result)?;
        Ok(match access_type {
            Direct => 2,
            Indirect => 4,
        })
    }

    fn srl(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(memory, reg)?;
        let (result, carry) = shift_right_logical(operand);
        self.set_z(result == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(carry);
        self.write_r8(memory, reg, result)?;
        Ok(match access_type {
            Direct => 2,
            Indirect => 4,
        })
    }

    fn bit(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let bit = get_bits(instruction, 5, 3);
        let (operand, access_type) = self.read_r8(memory, reg)?;
        self.set_z(get_bits(operand, bit, bit) == 0);
        self.set_n(false);
        self.set_h(true);
        Ok(match access_type {
            Direct => 2,
            Indirect => 3,
        })
    }

    fn res(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let bit = get_bits(instruction, 5, 3);
        let (operand, access_type) = self.read_r8(memory, reg)?;
        self.write_r8(memory, reg, clear_bit(operand, bit))?;
        Ok(match access_type {
            Direct => 2,
            Indirect => 4,
        })
    }

    fn set(&mut self, memory: &mut Memory, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let bit = get_bits(instruction, 5, 3);
        let (operand, access_type) = self.read_r8(memory, reg)?;
        self.write_r8(memory, reg, set_bit(operand, bit))?;
        Ok(match access_type {
            Direct => 2,
            Indirect => 4,
        })
    }

    fn prefix(&mut self, memory: &mut Memory) -> Result<u8> {
        let instruction = self.read_and_increment_pc(memory)?;
        match instruction {
            (0o00..=0o07) => self.rlc(memory, instruction),
            (0o10..=0o17) => self.rrc(memory, instruction),
            (0o20..=0o27) => self.rl(memory, instruction),
            (0o30..=0o37) => self.rr(memory, instruction),
            (0o40..=0o47) => self.sla(memory, instruction),
            (0o50..=0o57) => self.sra(memory, instruction),
            (0o60..=0o67) => self.swap(memory, instruction),
            (0o70..=0o77) => self.srl(memory, instruction),
            (0o100..=0o177) => self.bit(memory, instruction),
            (0o200..=0o277) => self.res(memory, instruction),
            (0o300..=0o377) => self.set(memory, instruction),
        }
    }

    fn add_and_set_flags_no_carry(&mut self, a: u8, b: u8) -> u8 {
        self.add_and_set_flags(a, b, None)
    }

    fn add_and_set_flags_with_carry(&mut self, a: u8, b: u8) -> u8 {
        self.add_and_set_flags(a, b, Some(u8::from(self.get_c())))
    }

    fn add_and_set_flags(&mut self, a: u8, b: u8, carry: Option<u8>) -> u8 {
        let carry_or_0 = carry.unwrap_or(0);
        let (result, is_carry) = a.overflowing_add(b);
        let (result, is_carry_2) = result.overflowing_add(carry_or_0);
        self.set_z(result == 0);
        self.set_n(false);
        self.set_h(is_add_half_carry_8(a, b) || is_add_half_carry_8(a.wrapping_add(b), carry_or_0));
        self.set_c(is_carry | is_carry_2);
        result
    }

    fn sub_and_set_flags_no_carry(&mut self, a: u8, b: u8) -> u8 {
        self.sub_and_set_flags(a, b, None)
    }

    fn sub_and_set_flags_with_carry(&mut self, a: u8, b: u8) -> u8 {
        self.sub_and_set_flags(a, b, Some(u8::from(self.get_c())))
    }

    fn sub_and_set_flags(&mut self, a: u8, b: u8, carry: Option<u8>) -> u8 {
        let carry_or_0 = carry.unwrap_or(0);
        let (result, is_carry) = a.overflowing_sub(b);
        let (result, is_carry_2) = result.overflowing_sub(carry_or_0);
        self.set_z(result == 0);
        self.set_n(true);
        self.set_h(is_sub_half_carry_8(a, b) || is_sub_half_carry_8(a.wrapping_sub(b), carry_or_0));
        self.set_c(is_carry || is_carry_2);
        result
    }

    fn add_signed_and_set_flags(&mut self, a: u16, b: i8) -> u16 {
        self.set_z(false);
        self.set_n(false);
        self.set_h(is_add_half_carry_8(a as u8, b as u8));
        self.set_c((a as u8).overflowing_add(b as u8).1);
        a.wrapping_add_signed(i16::from(b))
    }
}

fn is_add_half_carry_16(a: u16, b: u16) -> bool {
    ((a & 0xFFF) + (b & 0xFFF)) & 0x1000 == 0x1000
}

fn is_add_half_carry_8(a: u8, b: u8) -> bool {
    ((a & 0xF) + (b & 0xF)) & 0x10 == 0x10
}

fn is_sub_half_carry_8(a: u8, b: u8) -> bool {
    ((a & 0xF).wrapping_sub(b & 0xF)) & 0x10 == 0x10
}

fn rotate_left(value: u8) -> (u8, bool) {
    let result = value.wrapping_shl(1) | value.wrapping_shr(7);
    let carry = value.wrapping_shr(7) == 1;
    (result, carry)
}

fn rotate_left_with_carry(value: u8, carry: bool) -> (u8, bool) {
    let result = value.wrapping_shl(1)
        | match carry {
            true => 1,
            false => 0,
        };
    let carry = value.wrapping_shr(7) == 1;
    (result, carry)
}

fn rotate_right(value: u8) -> (u8, bool) {
    let result = value.wrapping_shr(1) | value.wrapping_shl(7);
    let carry = value & 1 == 1;
    (result, carry)
}

fn rotate_right_with_carry(value: u8, carry: bool) -> (u8, bool) {
    let result = value.wrapping_shr(1)
        | (match carry {
            true => 1u8,
            false => 0u8,
        })
        .wrapping_shl(7);
    let carry = value & 1 == 1;
    (result, carry)
}

fn shift_left(value: u8) -> (u8, bool) {
    let result = value.wrapping_shl(1);
    let carry = value.wrapping_shr(7) == 1;
    (result, carry)
}

fn shift_right_arithmetic(value: u8) -> (u8, bool) {
    let result = value >> 1 | (value & 0x80);
    let carry = value & 1 == 1;
    (result, carry)
}

fn shift_right_logical(value: u8) -> (u8, bool) {
    let result = value >> 1;
    let carry = value & 1 == 1;
    (result, carry)
}
