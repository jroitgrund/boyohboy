use crate::gb::bits::{clear_bit, get_bits, set_bit};
use crate::gb::AccessType::{Direct, Indirect};
use crate::gb::GameBoyImpl;
use anyhow::anyhow;
use anyhow::Result;

pub struct InstructionResult {
    pub cycles: u8,
    pub is_halt: bool,
}

impl GameBoyImpl {
    pub fn execute_next_instruction_with_halt_bug(&mut self) -> Result<InstructionResult> {
        self.execute_next_instruction_impl(true)
    }
    pub fn execute_next_instruction(&mut self) -> Result<InstructionResult> {
        self.execute_next_instruction_impl(false)
    }

    fn execute_next_instruction_impl(&mut self, halt_bug: bool) -> Result<InstructionResult> {
        let instruction = self.read_and_increment_pc()?;
        if halt_bug {
            self.pc = self.pc - 1;
        }
        let cycles = match instruction {
            0o0 => self.nop(),
            0o01 | 0o21 | 0o41 | 0o61 => self.ld_r16_n16(instruction),
            0o02 | 0o22 | 0o42 | 0o62 => self.ld_ind_r16_a(instruction),
            0o12 | 0o32 | 0o52 | 0o72 => self.ld_a_ind_r16(instruction),
            0o10 => self.ld_ind_n16_sp(),
            0o03 | 0o23 | 0o43 | 0o63 => self.inc_r16(instruction),
            0o13 | 0o33 | 0o53 | 0o73 => self.dec_r16(instruction),
            0o11 | 0o31 | 0o51 | 0o71 => self.add_hl_r16(instruction),
            0o04 | 0o14 | 0o24 | 0o34 | 0o44 | 0o54 | 0o64 | 0o74 => self.inc_r8(instruction),
            0o05 | 0o15 | 0o25 | 0o35 | 0o45 | 0o55 | 0o65 | 0o75 => self.dec_r8(instruction),
            0o06 | 0o16 | 0o26 | 0o36 | 0o46 | 0o56 | 0o66 | 0o76 => self.ld_r8_n8(instruction),
            0o07 => self.rlca(),
            0o17 => self.rrca(),
            0o27 => self.rla(),
            0o37 => self.rra(),
            0o47 => self.daa(),
            0o57 => self.cpl(),
            0o67 => self.scf(),
            0o77 => self.ccf(),
            0o30 => self.jr_n8(),
            0o40 | 0o50 | 0o60 | 0o70 => self.jr_cond_n8(instruction),
            0o20 => self.stop(),
            0o166 => self.halt(),
            (0o100..=0o177) => self.ld_r8_r8(instruction),
            0o200..=0o207 => self.add_a_r8(instruction),
            0o210..=0o217 => self.adc_a_r8(instruction),
            0o220..=0o227 => self.sub_a_r8(instruction),
            0o230..=0o237 => self.sbc_a_r8(instruction),
            0o240..=0o247 => self.and_a_r8(instruction),
            0o250..=0o257 => self.xor_a_r8(instruction),
            0o260..=0o267 => self.or_a_r8(instruction),
            0o270..=0o277 => self.cp_a_r8(instruction),
            0o306 => self.add_a_n8(),
            0o316 => self.adc_a_n8(),
            0o326 => self.sub_a_n8(),
            0o336 => self.sbc_a_n8(),
            0o346 => self.and_a_n8(),
            0o356 => self.xor_a_n8(),
            0o366 => self.or_a_n8(),
            0o376 => self.cp_a_n8(),
            0o300 | 0o310 | 0o320 | 0o330 => self.ret_cond(instruction),
            0o311 => self.ret(),
            0o331 => self.reti(),
            0o302 | 0o312 | 0o322 | 0o332 => self.jp_cond_n16(instruction),
            0o303 => self.jp_n16(),
            0o351 => self.jp_hl(),
            0o304 | 0o314 | 0o324 | 0o334 => self.call_cond_n16(instruction),
            0o315 => self.call_n16(),
            0o307 | 0o317 | 0o327 | 0o337 | 0o347 | 0o357 | 0o367 | 0o377 => self.rst(instruction),
            0o301 | 0o321 | 0o341 | 0o361 => self.pop(instruction),
            0o305 | 0o325 | 0o345 | 0o365 => self.push(instruction),
            0o313 => self.prefix(),
            0o342 => self.ldh_ind_c_a(),
            0o340 => self.ldh_ind_n8_a(),
            0o352 => self.ld_ind_n16_a(),
            0o362 => self.ldh_a_ind_c(),
            0o360 => self.ldh_a_ind_n8(),
            0o372 => self.ld_a_ind_n16(),
            0o350 => self.add_sp_n8(),
            0o370 => self.ld_hl_sp_plus_n8(),
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

    fn ld_r16_n16(&mut self, instruction: u8) -> Result<u8> {
        let operand = self.read_n16()?;
        let reg = get_bits(instruction, 5, 4);
        self.write_r16(reg, operand)?;
        Ok(3)
    }

    fn ld_ind_r16_a(&mut self, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 5, 4);
        let address = self.r16_mem(reg)?;
        self.write_8(address, self.a)?;
        Ok(2)
    }

    fn ld_a_ind_r16(&mut self, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 5, 4);
        let address = self.r16_mem(reg)?;
        self.a = self.read_8(address)?;
        Ok(2)
    }

    fn ld_ind_n16_sp(&mut self) -> Result<u8> {
        let address = self.read_n16()?;
        self.write_16(address, self.sp)?;
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

    fn inc_r8(&mut self, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 5, 3);
        let (operand, access_type) = self.read_r8(reg)?;
        let result = operand.wrapping_add(1);
        self.write_r8(reg, result)?;
        self.set_z(result == 0);
        self.set_n(false);
        self.set_h(is_add_half_carry_8(operand, 1));
        Ok(match access_type {
            Direct => 1,
            Indirect => 3,
        })
    }

    fn dec_r8(&mut self, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 5, 3);
        let (operand, access_type) = self.read_r8(reg)?;
        let result = operand.wrapping_sub(1);
        self.write_r8(reg, result)?;
        self.set_z(result == 0);
        self.set_n(true);
        self.set_h(is_sub_half_carry_8(operand, 1));
        Ok(match access_type {
            Direct => 1,
            Indirect => 3,
        })
    }

    fn ld_r8_n8(&mut self, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 5, 3);
        let operand = self.read_n8()?;
        Ok(match self.write_r8(reg, operand)? {
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

    fn jr_n8(&mut self) -> Result<u8> {
        let offset = i16::from(self.read_n8()? as i8);
        self.pc = self.pc.wrapping_add_signed(offset);
        Ok(3)
    }

    fn jr_cond_n8(&mut self, instruction: u8) -> Result<u8> {
        let cond = get_bits(instruction, 4, 3);
        let offset = i16::from(self.read_n8()? as i8);
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

    fn ld_r8_r8(&mut self, instruction: u8) -> Result<u8> {
        let source = get_bits(instruction, 2, 0);
        let dest = get_bits(instruction, 5, 3);
        let (value, access_type) = self.read_r8(source)?;
        Ok(match self.write_r8(dest, value)? + access_type {
            Direct => 1,
            Indirect => 2,
        })
    }

    fn add_a_r8(&mut self, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(reg)?;
        self.a = self.add_and_set_flags_no_carry(self.a, operand);
        Ok(match access_type {
            Direct => 1,
            Indirect => 2,
        })
    }

    fn adc_a_r8(&mut self, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(reg)?;
        self.a = self.add_and_set_flags_with_carry(self.a, operand);
        Ok(match access_type {
            Direct => 1,
            Indirect => 2,
        })
    }

    fn sub_a_r8(&mut self, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(reg)?;
        self.a = self.sub_and_set_flags_no_carry(self.a, operand);
        Ok(match access_type {
            Direct => 1,
            Indirect => 2,
        })
    }
    fn sbc_a_r8(&mut self, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(reg)?;
        self.a = self.sub_and_set_flags_with_carry(self.a, operand);
        Ok(match access_type {
            Direct => 1,
            Indirect => 2,
        })
    }
    fn and_a_r8(&mut self, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(reg)?;
        self.a = self.a & operand;
        self.set_z(self.a == 0);
        self.set_n(false);
        self.set_h(true);
        self.set_c(false);
        Ok(match access_type {
            Direct => 1,
            Indirect => 2,
        })
    }
    fn xor_a_r8(&mut self, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(reg)?;
        self.a = self.a ^ operand;
        self.set_z(self.a == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(false);
        Ok(match access_type {
            Direct => 1,
            Indirect => 2,
        })
    }
    fn or_a_r8(&mut self, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(reg)?;
        self.a = self.a | operand;
        self.set_z(self.a == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(false);
        Ok(match access_type {
            Direct => 1,
            Indirect => 2,
        })
    }
    fn cp_a_r8(&mut self, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(reg)?;
        self.sub_and_set_flags_no_carry(self.a, operand);
        Ok(match access_type {
            Direct => 1,
            Indirect => 2,
        })
    }

    fn add_a_n8(&mut self) -> Result<u8> {
        let operand = self.read_n8()?;
        self.a = self.add_and_set_flags_no_carry(self.a, operand);
        Ok(2)
    }

    fn adc_a_n8(&mut self) -> Result<u8> {
        let operand = self.read_n8()?;
        self.a = self.add_and_set_flags_with_carry(self.a, operand);
        Ok(2)
    }

    fn sub_a_n8(&mut self) -> Result<u8> {
        let operand = self.read_n8()?;
        self.a = self.sub_and_set_flags_no_carry(self.a, operand);
        Ok(2)
    }
    fn sbc_a_n8(&mut self) -> Result<u8> {
        let operand = self.read_n8()?;
        self.a = self.sub_and_set_flags_with_carry(self.a, operand);
        Ok(2)
    }
    fn and_a_n8(&mut self) -> Result<u8> {
        let operand = self.read_n8()?;
        self.a = self.a & operand;
        self.set_z(self.a == 0);
        self.set_n(false);
        self.set_h(true);
        self.set_c(false);
        Ok(2)
    }
    fn xor_a_n8(&mut self) -> Result<u8> {
        let operand = self.read_n8()?;
        self.a = self.a ^ operand;
        self.set_z(self.a == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(false);
        Ok(2)
    }
    fn or_a_n8(&mut self) -> Result<u8> {
        let operand = self.read_n8()?;
        self.a = self.a | operand;
        self.set_z(self.a == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(false);
        Ok(2)
    }
    fn cp_a_n8(&mut self) -> Result<u8> {
        let operand = self.read_n8()?;
        self.sub_and_set_flags(self.a, operand, None);
        Ok(2)
    }

    fn ret_cond(&mut self, instruction: u8) -> Result<u8> {
        let cond = get_bits(instruction, 4, 3);
        let should_ret = self.read_cond(cond)?;
        Ok(if should_ret {
            let dest = self.pop_16()?;
            self.pc = dest;
            5
        } else {
            2
        })
    }

    fn ret(&mut self) -> Result<u8> {
        self.pc = self.pop_16()?;
        Ok(4)
    }

    fn reti(&mut self) -> Result<u8> {
        self.pc = self.pop_16()?;
        self.ime = true;
        Ok(4)
    }

    fn jp_cond_n16(&mut self, instruction: u8) -> Result<u8> {
        let cond = get_bits(instruction, 4, 3);
        let dest = self.read_n16()?;
        Ok(if self.read_cond(cond)? {
            self.pc = dest;
            4
        } else {
            3
        })
    }

    fn jp_n16(&mut self) -> Result<u8> {
        self.pc = self.read_n16()?;
        Ok(4)
    }

    fn jp_hl(&mut self) -> Result<u8> {
        self.pc = self.read_hl()?;
        Ok(1)
    }

    fn call_cond_n16(&mut self, instruction: u8) -> Result<u8> {
        let cond = get_bits(instruction, 4, 3);
        let addr = self.read_n16()?;
        Ok(if self.read_cond(cond)? {
            self.push_16(self.pc)?;
            self.pc = addr;
            6
        } else {
            3
        })
    }

    fn call_n16(&mut self) -> Result<u8> {
        let addr = self.read_n16()?;
        self.push_16(self.pc)?;
        self.pc = addr;
        Ok(6)
    }

    fn rst(&mut self, instruction: u8) -> Result<u8> {
        let operand = get_bits(instruction, 5, 3);
        let addr = u16::from(operand) * 8;
        self.push_16(self.pc)?;
        self.pc = addr;
        Ok(4)
    }

    fn pop(&mut self, instruction: u8) -> Result<u8> {
        let value = self.pop_16()?;
        let reg = get_bits(instruction, 5, 4);
        self.write_r16_stk(reg, value)?;
        Ok(3)
    }

    fn push(&mut self, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 5, 4);
        self.push_16(self.read_r16_stk(reg)?)?;
        Ok(4)
    }

    fn ldh_ind_c_a(&mut self) -> Result<u8> {
        self.write_8(0xFF00 + u16::from(self.c), self.a)?;
        Ok(2)
    }

    fn ldh_ind_n8_a(&mut self) -> Result<u8> {
        let address = 0xFF00 + u16::from(self.read_n8()?);
        self.write_8(address, self.a)?;
        Ok(3)
    }

    fn ld_ind_n16_a(&mut self) -> Result<u8> {
        let address = self.read_n16()?;
        self.write_8(address, self.a)?;
        Ok(4)
    }

    fn ldh_a_ind_c(&mut self) -> Result<u8> {
        self.a = self.read_8(0xFF00 + u16::from(self.c))?;
        Ok(2)
    }

    fn ldh_a_ind_n8(&mut self) -> Result<u8> {
        let address = 0xFF00 + u16::from(self.read_n8()?);
        self.a = self.read_8(address)?;
        Ok(3)
    }

    fn ld_a_ind_n16(&mut self) -> Result<u8> {
        let address = self.read_n16()?;
        self.a = self.read_8(address)?;
        Ok(4)
    }

    fn add_sp_n8(&mut self) -> Result<u8> {
        let operand = self.read_n8()? as i8;
        self.sp = self.add_signed_and_set_flags(self.sp, operand);
        Ok(4)
    }

    fn ld_hl_sp_plus_n8(&mut self) -> Result<u8> {
        let operand = self.read_n8()? as i8;
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

    fn rlc(&mut self, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(reg)?;
        let (result, carry) = rotate_left(operand);
        self.set_z(result == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(carry);
        self.write_r8(reg, result)?;
        Ok(match access_type {
            Direct => 2,
            Indirect => 4,
        })
    }

    fn rrc(&mut self, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(reg)?;
        let (result, carry) = rotate_right(operand);
        self.set_z(result == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(carry);
        self.write_r8(reg, result)?;
        Ok(match access_type {
            Direct => 2,
            Indirect => 4,
        })
    }

    fn rl(&mut self, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(reg)?;
        let (result, carry) = rotate_left_with_carry(operand, self.get_c());
        self.set_z(result == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(carry);
        self.write_r8(reg, result)?;
        Ok(match access_type {
            Direct => 2,
            Indirect => 4,
        })
    }

    fn rr(&mut self, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(reg)?;
        let (result, carry) = rotate_right_with_carry(operand, self.get_c());
        self.set_z(result == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(carry);
        self.write_r8(reg, result)?;
        Ok(match access_type {
            Direct => 2,
            Indirect => 4,
        })
    }

    fn sla(&mut self, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(reg)?;
        let (result, carry) = shift_left(operand);
        self.set_z(result == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(carry);
        self.write_r8(reg, result)?;
        Ok(match access_type {
            Direct => 2,
            Indirect => 4,
        })
    }

    fn sra(&mut self, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(reg)?;
        let (result, carry) = shift_right_arithmetic(operand);
        self.set_z(result == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(carry);
        self.write_r8(reg, result)?;
        Ok(match access_type {
            Direct => 2,
            Indirect => 4,
        })
    }

    fn swap(&mut self, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(reg)?;
        let result = operand.wrapping_shr(4) + operand.wrapping_shl(4);
        self.set_z(result == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(false);
        self.write_r8(reg, result)?;
        Ok(match access_type {
            Direct => 2,
            Indirect => 4,
        })
    }

    fn srl(&mut self, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let (operand, access_type) = self.read_r8(reg)?;
        let (result, carry) = shift_right_logical(operand);
        self.set_z(result == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_c(carry);
        self.write_r8(reg, result)?;
        Ok(match access_type {
            Direct => 2,
            Indirect => 4,
        })
    }

    fn bit(&mut self, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let bit = get_bits(instruction, 5, 3);
        let (operand, access_type) = self.read_r8(reg)?;
        self.set_z(get_bits(operand, bit, bit) == 0);
        self.set_n(false);
        self.set_h(true);
        Ok(match access_type {
            Direct => 2,
            Indirect => 3,
        })
    }

    fn res(&mut self, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let bit = get_bits(instruction, 5, 3);
        let (operand, access_type) = self.read_r8(reg)?;
        self.write_r8(reg, clear_bit(operand, bit))?;
        Ok(match access_type {
            Direct => 2,
            Indirect => 4,
        })
    }

    fn set(&mut self, instruction: u8) -> Result<u8> {
        let reg = get_bits(instruction, 2, 0);
        let bit = get_bits(instruction, 5, 3);
        let (operand, access_type) = self.read_r8(reg)?;
        self.write_r8(reg, set_bit(operand, bit))?;
        Ok(match access_type {
            Direct => 2,
            Indirect => 4,
        })
    }

    fn prefix(&mut self) -> Result<u8> {
        let instruction = self.read_and_increment_pc()?;
        match instruction {
            (0o00..=0o07) => self.rlc(instruction),
            (0o10..=0o17) => self.rrc(instruction),
            (0o20..=0o27) => self.rl(instruction),
            (0o30..=0o37) => self.rr(instruction),
            (0o40..=0o47) => self.sla(instruction),
            (0o50..=0o57) => self.sra(instruction),
            (0o60..=0o67) => self.swap(instruction),
            (0o70..=0o77) => self.srl(instruction),
            (0o100..=0o177) => self.bit(instruction),
            (0o200..=0o277) => self.res(instruction),
            (0o300..=0o377) => self.set(instruction),
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
