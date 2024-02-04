use crate::gb::AccessType::{Direct, Indirect};
use crate::gb::GameBoy;
use anyhow::anyhow;

pub fn execute_next_instruction(gb: &mut GameBoy) -> anyhow::Result<u8> {
    let instruction = gb.read_and_increment_pc();
    let cycles: anyhow::Result<u8> = match instruction {
        0o0 => nop(),
        0o01 | 0o21 | 0o41 | 0o61 => ld_r16_n16(gb, instruction),
        0o02 | 0o22 | 0o42 | 0o62 => ld_ind_r16_a(gb, instruction),
        0o12 | 0o32 | 0o52 | 0o72 => ld_a_ind_r16(gb, instruction),
        0o10 => ld_ind_n16_sp(gb),
        0o03 | 0o23 | 0o43 | 0o63 => inc_r16(gb, instruction),
        0o13 | 0o33 | 0o53 | 0o73 => dec_r16(gb, instruction),
        0o11 | 0o31 | 0o51 | 0o71 => add_hl_r16(gb, instruction),
        0o04 | 0o14 | 0o24 | 0o34 | 0o44 | 0o54 | 0o64 | 0o74 => inc_r8(gb, instruction),
        0o05 | 0o15 | 0o25 | 0o35 | 0o45 | 0o55 | 0o65 | 0o75 => dec_r8(gb, instruction),
        0o06 | 0o16 | 0o26 | 0o36 | 0o46 | 0o56 | 0o66 | 0o76 => ld_r8_n8(gb, instruction),
        0o07 => rlca(gb),
        0o17 => rrca(gb),
        0o27 => rla(gb),
        0o37 => rra(gb),
        0o47 => daa(gb),
        0o57 => cpl(gb),
        0o67 => scf(gb),
        0o77 => ccf(gb),
        0o30 => jr_n8(gb),
        0o40 | 0o50 | 0o60 | 0o70 => jr_cond_n8(gb, instruction),
        0o20 => stop(),
        0o166 => halt(),
        (0o100..=0o177) => ld_r8_r8(gb, instruction),
        0o201..=0o207 => add_a_r8(gb, instruction),
        0o211..=0o217 => adc_a_r8(gb, instruction),
        0o221..=0o227 => sub_a_r8(gb, instruction),
        0o231..=0o237 => sbc_a_r8(gb, instruction),
        0o241..=0o247 => and_a_r8(gb, instruction),
        0o251..=0o257 => xor_a_r8(gb, instruction),
        0o261..=0o267 => or_a_r8(gb, instruction),
        0o271..=0o277 => cp_a_r8(gb, instruction),
        0o306 => add_a_n8(gb),
        0o316 => adc_a_n8(gb),
        0o326 => sub_a_n8(gb),
        0o336 => sbc_a_n8(gb),
        0o346 => and_a_n8(gb),
        0o356 => xor_a_n8(gb),
        0o366 => or_a_n8(gb),
        0o376 => cp_a_n8(gb),
        0o300 | 0o310 | 0o320 | 0o330 => ret_cond(gb, instruction),
        0o311 => ret(gb),
        0o331 => reti(gb),
        0o302 | 0o312 | 0o322 | 0o332 => jp_cond_n16(gb, instruction),
        0o303 => jp_n16(gb),
        0o351 => jp_hl(gb),
        0o304 | 0o314 | 0o324 | 0o334 => call_cond_n16(gb, instruction),
        0o315 => call_n16(gb),
        0o307 | 0o317 | 0o327 | 0o337 | 0o347 | 0o357 | 0o367 | 0o377 => rst(gb, instruction),
        0o301 | 0o321 | 0o341 | 0o361 => pop(gb, instruction),
        0o305 | 0o325 | 0o345 | 0o365 => push(gb, instruction),
        0o313 => prefix(gb),
        0o342 => ldh_ind_c_a(gb),
        0o340 => ldh_ind_n8_a(gb),
        0o352 => ld_ind_n16_a(gb),
        0o362 => ldh_a_ind_c(gb),
        0o360 => ldh_a_ind_n8(gb),
        0o372 => ld_a_ind_n16(gb),
        0o350 => add_sp_n8(gb),
        0o370 => ld_hl_sp_plus_n8(gb),
        0o371 => ld_sp_hl(gb),
        0o363 => di(gb),
        0o373 => ei(gb),
        _ => Err(anyhow!("Unknown instruction {}", instruction)),
    };

    cycles
}

fn get_bits(instruction: u8, high_bit: u8, low_bit: u8) -> u8 {
    return (instruction >> low_bit) & ((1 << (1 + high_bit - low_bit)) - 1);
}

fn is_signed_add_half_carry_16_8(a: u16, b: i8) -> bool {
    return if b > 0 {
        is_add_half_carry_8(a as u8, b.unsigned_abs())
    } else {
        is_sub_half_carry_8(a as u8, b.unsigned_abs())
    };
}

fn is_signed_add_carry_16_8(a: u16, b: i8) -> bool {
    return if b > 0 {
        is_add_carry_16(a, u16::from(b.unsigned_abs()))
    } else {
        is_sub_carry_16(a, u16::from(b.unsigned_abs()))
    };
}

fn is_add_carry_16(a: u16, b: u16) -> bool {
    return a > u16::MAX - b;
}

fn is_sub_carry_16(a: u16, b: u16) -> bool {
    return b > a;
}

fn is_add_half_carry_16(a: u16, b: u16) -> bool {
    return a & 0xFFF + b & 0xFFF > 0xFFF;
}

fn is_add_half_carry_8(a: u8, b: u8) -> bool {
    return a & 0xF + b & 0xF > 0xF;
}

fn is_sub_half_carry_8(a: u8, b: u8) -> bool {
    return b & 0xF > a & 0xF;
}

fn rotate_left(value: u8) -> (u8, bool) {
    let result = value << 1 | value >> 7;
    let carry = value >> 7 == 1;
    (result, carry)
}

fn rotate_left_with_carry(value: u8, carry: bool) -> (u8, bool) {
    let result = value << 1
        | match carry {
            true => 1,
            false => 0,
        };
    let carry = value >> 7 == 1;
    (result, carry)
}

fn rotate_right(value: u8) -> (u8, bool) {
    let result = value >> 1 | value << 7;
    let carry = value & 1 == 1;
    (result, carry)
}

fn rotate_right_with_carry(value: u8, carry: bool) -> (u8, bool) {
    let result = value >> 1
        | (match carry {
            true => 1,
            false => 0,
        }) << 7;
    let carry = value & 1 == 1;
    (result, carry)
}

fn shift_left(value: u8) -> (u8, bool) {
    let result = value << 1;
    let carry = value >> 7 == 1;
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

fn nop() -> anyhow::Result<u8> {
    Ok(2)
}

fn ld_r16_n16(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let operand = gb.read_n16();
    let reg = get_bits(instruction, 5, 4);
    gb.write_r16(reg, operand)?;
    Ok(3)
}

fn ld_ind_r16_a(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let reg = get_bits(instruction, 5, 4);
    let address = gb.r16_mem(reg)?;
    gb.write_8(address, gb.a);
    Ok(2)
}

fn ld_a_ind_r16(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let reg = get_bits(instruction, 5, 4);
    let address = gb.r16_mem(reg)?;
    gb.a = gb.read_8(address);
    Ok(2)
}

fn ld_ind_n16_sp(gb: &mut GameBoy) -> anyhow::Result<u8> {
    let address = gb.read_n16();
    gb.write_16(address, gb.sp);
    Ok(5)
}

fn inc_r16(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let reg = get_bits(instruction, 5, 4);
    gb.write_r16(reg, gb.read_r16(reg)? + 1)?;
    Ok(2)
}

fn dec_r16(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let reg = get_bits(instruction, 5, 4);
    gb.write_r16(reg, gb.read_r16(reg)? + 1)?;
    Ok(2)
}

fn add_hl_r16(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let hl = gb.read_hl()?;
    let reg = get_bits(instruction, 5, 4);
    let operand = gb.read_r16(reg)?;
    let (sum, overflow) = hl.overflowing_add(operand);
    gb.set_n(false);
    gb.set_h(is_add_half_carry_16(hl, operand));
    gb.set_c(overflow);
    gb.write_hl(sum)?;
    Ok(2)
}

fn inc_r8(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let reg = get_bits(instruction, 5, 3);
    let (operand, access_type) = gb.read_r8(reg)?;
    let result = operand + 1;
    gb.write_r8(reg, result)?;
    gb.set_z(result == 0);
    gb.set_n(false);
    gb.set_h(is_add_half_carry_8(operand, 1));
    Ok(match access_type {
        Direct => 1,
        Indirect => 3,
    })
}

fn dec_r8(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let reg = get_bits(instruction, 5, 3);
    let (operand, access_type) = gb.read_r8(reg)?;
    let result = operand - 1;
    gb.write_r8(reg, result)?;
    gb.set_z(result == 0);
    gb.set_n(true);
    gb.set_h(is_sub_half_carry_8(operand, 1));
    Ok(match access_type {
        Direct => 1,
        Indirect => 3,
    })
}

fn ld_r8_n8(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let reg = get_bits(instruction, 5, 3);
    let operand = gb.read_n8();
    Ok(match gb.write_r8(reg, operand)? {
        Direct => 2,
        Indirect => 3,
    })
}

fn rlca(gb: &mut GameBoy) -> anyhow::Result<u8> {
    let (result, carry) = rotate_left(gb.a);
    gb.set_z(false);
    gb.set_n(false);
    gb.set_h(false);
    gb.set_c(carry);
    gb.a = result;
    Ok(1)
}

fn rrca(gb: &mut GameBoy) -> anyhow::Result<u8> {
    let (result, carry) = rotate_right(gb.a);
    gb.set_z(false);
    gb.set_n(false);
    gb.set_h(false);
    gb.set_c(carry);
    gb.a = result;
    Ok(1)
}

fn rla(gb: &mut GameBoy) -> anyhow::Result<u8> {
    let (result, carry) = rotate_left_with_carry(gb.a, gb.get_c());
    gb.set_z(false);
    gb.set_n(false);
    gb.set_h(false);
    gb.set_c(carry);
    gb.a = result;
    Ok(1)
}

fn rra(gb: &mut GameBoy) -> anyhow::Result<u8> {
    let (result, carry) = rotate_right_with_carry(gb.a, gb.get_c());
    gb.set_z(false);
    gb.set_n(false);
    gb.set_h(false);
    gb.set_c(carry);
    gb.a = result;
    Ok(1)
}

fn daa(gb: &mut GameBoy) -> anyhow::Result<u8> {
    if gb.get_n() {
        if gb.get_c() || gb.a > 0x99 {
            gb.a += 0x60;
            gb.set_c(true);
        }
        if gb.get_h() || (gb.a & 0x0f) > 0x09 {
            gb.a += 0x6;
        }
    } else {
        if gb.get_c() {
            gb.a -= 0x60;
        }
        if gb.get_h() {
            gb.a -= 0x6;
        }
    }
    gb.set_z(gb.a == 0);
    gb.set_h(false);
    Ok(1)
}

fn cpl(gb: &mut GameBoy) -> anyhow::Result<u8> {
    gb.a = !gb.a;
    gb.set_n(true);
    gb.set_h(true);
    Ok(1)
}

fn scf(gb: &mut GameBoy) -> anyhow::Result<u8> {
    gb.set_n(false);
    gb.set_h(false);
    gb.set_c(true);
    Ok(1)
}

fn ccf(gb: &mut GameBoy) -> anyhow::Result<u8> {
    gb.set_n(false);
    gb.set_h(false);
    gb.set_c(!gb.get_c());
    Ok(1)
}

fn jr_n8(gb: &mut GameBoy) -> anyhow::Result<u8> {
    gb.pc = gb.pc.wrapping_add_signed(gb.read_n8() as i16);
    Ok(3)
}

fn jr_cond_n8(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let cond = get_bits(instruction, 4, 3);
    let offset = gb.read_n8() as i16;
    Ok(if gb.read_cond(cond)? {
        gb.pc = gb.pc.wrapping_add_signed(offset);
        3
    } else {
        2
    })
}

fn stop() -> anyhow::Result<u8> {
    Ok(1)
}

fn halt() -> anyhow::Result<u8> {
    Ok(1)
}

fn ld_r8_r8(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let source = get_bits(instruction, 2, 0);
    let dest = get_bits(instruction, 5, 3);
    let (value, access_type) = gb.read_r8(source)?;
    Ok(match gb.write_r8(dest, value)? + access_type {
        Direct => 1,
        Indirect => 2,
    })
}

fn add_a_r8(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let reg = get_bits(instruction, 2, 0);
    let (operand, access_type) = gb.read_r8(reg)?;
    gb.a = add_and_set_flags_no_carry(gb, gb.a, operand);
    Ok(match access_type {
        Direct => 1,
        Indirect => 2,
    })
}

fn adc_a_r8(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let reg = get_bits(instruction, 2, 0);
    let (operand, access_type) = gb.read_r8(reg)?;
    gb.a = add_and_set_flags_with_carry(gb, gb.a, operand);
    Ok(match access_type {
        Direct => 1,
        Indirect => 2,
    })
}

fn sub_a_r8(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let reg = get_bits(instruction, 2, 0);
    let (operand, access_type) = gb.read_r8(reg)?;
    gb.a = sub_and_set_flags_no_carry(gb, gb.a, operand);
    Ok(match access_type {
        Direct => 1,
        Indirect => 2,
    })
}
fn sbc_a_r8(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let reg = get_bits(instruction, 2, 0);
    let (operand, access_type) = gb.read_r8(reg)?;
    gb.a = sub_and_set_flags_with_carry(gb, gb.a, operand);
    Ok(match access_type {
        Direct => 1,
        Indirect => 2,
    })
}
fn and_a_r8(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let reg = get_bits(instruction, 2, 0);
    let (operand, access_type) = gb.read_r8(reg)?;
    gb.a = gb.a & operand;
    gb.set_z(gb.a == 0);
    gb.set_n(false);
    gb.set_h(true);
    gb.set_c(false);
    Ok(match access_type {
        Direct => 1,
        Indirect => 2,
    })
}
fn xor_a_r8(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let reg = get_bits(instruction, 2, 0);
    let (operand, access_type) = gb.read_r8(reg)?;
    gb.a = gb.a ^ operand;
    gb.set_z(gb.a == 0);
    gb.set_n(false);
    gb.set_h(false);
    gb.set_c(false);
    Ok(match access_type {
        Direct => 1,
        Indirect => 2,
    })
}
fn or_a_r8(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let reg = get_bits(instruction, 2, 0);
    let (operand, access_type) = gb.read_r8(reg)?;
    gb.a = gb.a | operand;
    gb.set_z(gb.a == 0);
    gb.set_n(false);
    gb.set_h(false);
    gb.set_c(false);
    Ok(match access_type {
        Direct => 1,
        Indirect => 2,
    })
}
fn cp_a_r8(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let reg = get_bits(instruction, 2, 0);
    let (operand, access_type) = gb.read_r8(reg)?;
    sub_and_set_flags_no_carry(gb, gb.a, operand);
    Ok(match access_type {
        Direct => 1,
        Indirect => 2,
    })
}

fn add_a_n8(gb: &mut GameBoy) -> anyhow::Result<u8> {
    let operand = gb.read_n8();
    gb.a = add_and_set_flags_no_carry(gb, gb.a, operand);
    Ok(2)
}

fn adc_a_n8(gb: &mut GameBoy) -> anyhow::Result<u8> {
    let operand = gb.read_n8();
    gb.a = add_and_set_flags_with_carry(gb, gb.a, operand);
    Ok(2)
}

fn sub_a_n8(gb: &mut GameBoy) -> anyhow::Result<u8> {
    let operand = gb.read_n8();
    gb.a = sub_and_set_flags_no_carry(gb, gb.a, operand);
    Ok(2)
}
fn sbc_a_n8(gb: &mut GameBoy) -> anyhow::Result<u8> {
    let operand = gb.read_n8();
    gb.a = sub_and_set_flags_with_carry(gb, gb.a, operand);
    Ok(2)
}
fn and_a_n8(gb: &mut GameBoy) -> anyhow::Result<u8> {
    let operand = gb.read_n8();
    gb.a = gb.a & operand;
    gb.set_z(gb.a == 0);
    gb.set_n(false);
    gb.set_h(true);
    gb.set_c(false);
    Ok(2)
}
fn xor_a_n8(gb: &mut GameBoy) -> anyhow::Result<u8> {
    let operand = gb.read_n8();
    gb.a = gb.a ^ operand;
    gb.set_z(gb.a == 0);
    gb.set_n(false);
    gb.set_h(false);
    gb.set_c(false);
    Ok(2)
}
fn or_a_n8(gb: &mut GameBoy) -> anyhow::Result<u8> {
    let operand = gb.read_n8();
    gb.a = gb.a | operand;
    gb.set_z(gb.a == 0);
    gb.set_n(false);
    gb.set_h(false);
    gb.set_c(false);
    Ok(2)
}
fn cp_a_n8(gb: &mut GameBoy) -> anyhow::Result<u8> {
    let operand = gb.read_n8();
    sub_and_set_flags(gb, gb.a, operand, None);
    Ok(2)
}

fn ret_cond(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let cond = get_bits(instruction, 4, 3);
    let dest = gb.pop_16();
    Ok(if gb.read_cond(cond)? {
        gb.pc = dest;
        5
    } else {
        2
    })
}

fn ret(gb: &mut GameBoy) -> anyhow::Result<u8> {
    gb.pc = gb.pop_16();
    Ok(4)
}

fn reti(gb: &mut GameBoy) -> anyhow::Result<u8> {
    gb.pc = gb.pop_16();
    gb.ime = true;
    Ok(4)
}

fn jp_cond_n16(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let cond = get_bits(instruction, 4, 3);
    let dest = gb.read_n16();
    Ok(if gb.read_cond(cond)? {
        gb.pc = dest;
        4
    } else {
        3
    })
}

fn jp_n16(gb: &mut GameBoy) -> anyhow::Result<u8> {
    gb.pc = gb.read_n16();
    Ok(4)
}

fn jp_hl(gb: &mut GameBoy) -> anyhow::Result<u8> {
    gb.pc = gb.read_hl()?;
    Ok(1)
}

fn call_cond_n16(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let cond = get_bits(instruction, 4, 3);
    let addr = gb.read_n16();
    Ok(if gb.read_cond(cond)? {
        gb.push_16(gb.pc);
        gb.pc = addr;
        6
    } else {
        3
    })
}

fn call_n16(gb: &mut GameBoy) -> anyhow::Result<u8> {
    let addr = gb.read_n16();
    gb.push_16(gb.pc);
    gb.pc = addr;
    Ok(6)
}

fn rst(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let operand = get_bits(instruction, 5, 3);
    let addr = u16::from(operand) * 8;
    gb.push_16(gb.pc);
    gb.pc = addr;
    Ok(4)
}

fn pop(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let value = gb.pop_16();
    let reg = get_bits(instruction, 5, 4);
    gb.write_r16_stk(reg, value)?;
    Ok(3)
}

fn push(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let reg = get_bits(instruction, 5, 4);
    gb.push_16(gb.read_r16_stk(reg)?);
    Ok(4)
}

fn ldh_ind_c_a(gb: &mut GameBoy) -> anyhow::Result<u8> {
    gb.write_8(0xFF00 + u16::from(gb.c), gb.a);
    Ok(2)
}

fn ldh_ind_n8_a(gb: &mut GameBoy) -> anyhow::Result<u8> {
    let address = 0xFF00 + u16::from(gb.read_n8());
    gb.write_8(address, gb.a);
    Ok(3)
}

fn ld_ind_n16_a(gb: &mut GameBoy) -> anyhow::Result<u8> {
    let address = gb.read_n16();
    gb.write_8(address, gb.a);
    Ok(4)
}

fn ldh_a_ind_c(gb: &mut GameBoy) -> anyhow::Result<u8> {
    gb.a = gb.read_8(0xFF00 + u16::from(gb.c));
    Ok(2)
}

fn ldh_a_ind_n8(gb: &mut GameBoy) -> anyhow::Result<u8> {
    let address = 0xFF00 + u16::from(gb.read_n8());
    gb.a = gb.read_8(address);
    Ok(3)
}

fn ld_a_ind_n16(gb: &mut GameBoy) -> anyhow::Result<u8> {
    let address = gb.read_n16();
    gb.a = gb.read_8(address);
    Ok(4)
}

fn add_sp_n8(gb: &mut GameBoy) -> anyhow::Result<u8> {
    let operand = gb.read_n8() as i8;
    gb.sp = add_signed_and_set_flags(gb, gb.sp, operand);
    Ok(4)
}

fn ld_hl_sp_plus_n8(gb: &mut GameBoy) -> anyhow::Result<u8> {
    let operand = gb.read_n8() as i8;
    let result = add_signed_and_set_flags(gb, gb.sp, operand);
    gb.write_hl(result)?;
    Ok(3)
}

fn ld_sp_hl(gb: &mut GameBoy) -> anyhow::Result<u8> {
    gb.write_hl(gb.sp)?;
    Ok(2)
}

fn di(gb: &mut GameBoy) -> anyhow::Result<u8> {
    gb.ime = false;
    Ok(1)
}

fn ei(gb: &mut GameBoy) -> anyhow::Result<u8> {
    gb.ime = true;
    Ok(1)
}

fn rlc(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let reg = get_bits(instruction, 2, 0);
    let (operand, access_type) = gb.read_r8(reg)?;
    let (result, carry) = rotate_left(operand);
    gb.set_z(result == 0);
    gb.set_n(false);
    gb.set_h(false);
    gb.set_c(carry);
    gb.write_r8(reg, result)?;
    Ok(match access_type {
        Direct => 2,
        Indirect => 4,
    })
}

fn rrc(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let reg = get_bits(instruction, 2, 0);
    let (operand, access_type) = gb.read_r8(reg)?;
    let (result, carry) = rotate_right(operand);
    gb.set_z(result == 0);
    gb.set_n(false);
    gb.set_h(false);
    gb.set_c(carry);
    gb.write_r8(reg, result)?;
    Ok(match access_type {
        Direct => 2,
        Indirect => 4,
    })
}

fn rl(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let reg = get_bits(instruction, 2, 0);
    let (operand, access_type) = gb.read_r8(reg)?;
    let (result, carry) = rotate_left_with_carry(operand, gb.get_c());
    gb.set_z(result == 0);
    gb.set_n(false);
    gb.set_h(false);
    gb.set_c(carry);
    gb.write_r8(reg, result)?;
    Ok(match access_type {
        Direct => 2,
        Indirect => 4,
    })
}

fn rr(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let reg = get_bits(instruction, 2, 0);
    let (operand, access_type) = gb.read_r8(reg)?;
    let (result, carry) = rotate_right_with_carry(operand, gb.get_c());
    gb.set_z(result == 0);
    gb.set_n(false);
    gb.set_h(false);
    gb.set_c(carry);
    gb.write_r8(reg, result)?;
    Ok(match access_type {
        Direct => 2,
        Indirect => 4,
    })
}

fn sla(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let reg = get_bits(instruction, 2, 0);
    let (operand, access_type) = gb.read_r8(reg)?;
    let (result, carry) = shift_left(operand);
    gb.set_z(result == 0);
    gb.set_n(false);
    gb.set_h(false);
    gb.set_c(carry);
    Ok(match access_type {
        Direct => 2,
        Indirect => 4,
    })
}

fn sra(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let reg = get_bits(instruction, 2, 0);
    let (operand, access_type) = gb.read_r8(reg)?;
    let (result, carry) = shift_right_arithmetic(operand);
    gb.set_z(result == 0);
    gb.set_n(false);
    gb.set_h(false);
    gb.set_c(carry);
    Ok(match access_type {
        Direct => 2,
        Indirect => 4,
    })
}

fn swap(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let reg = get_bits(instruction, 2, 0);
    let (operand, access_type) = gb.read_r8(reg)?;
    let result = operand >> 4 + operand << 4;
    gb.set_z(result == 0);
    gb.set_n(false);
    gb.set_h(false);
    gb.set_c(false);
    Ok(match access_type {
        Direct => 2,
        Indirect => 4,
    })
}

fn srl(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let reg = get_bits(instruction, 2, 0);
    let (operand, access_type) = gb.read_r8(reg)?;
    let (result, carry) = shift_right_logical(operand);
    gb.set_z(result == 0);
    gb.set_n(false);
    gb.set_h(false);
    gb.set_c(carry);
    Ok(match access_type {
        Direct => 2,
        Indirect => 4,
    })
}

fn bit(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let reg = get_bits(instruction, 2, 0);
    let bit = get_bits(instruction, 5, 3);
    let (operand, access_type) = gb.read_r8(reg)?;
    gb.set_z((operand >> bit) & 1 == 1);
    gb.set_n(false);
    gb.set_h(true);
    Ok(match access_type {
        Direct => 2,
        Indirect => 3,
    })
}

fn res(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let reg = get_bits(instruction, 2, 0);
    let bit = get_bits(instruction, 5, 3);
    let (operand, access_type) = gb.read_r8(reg)?;
    gb.write_r8(reg, operand & !(1 << bit))?;
    Ok(match access_type {
        Direct => 2,
        Indirect => 4,
    })
}

fn set(gb: &mut GameBoy, instruction: u8) -> anyhow::Result<u8> {
    let reg = get_bits(instruction, 2, 0);
    let bit = get_bits(instruction, 5, 3);
    let (operand, access_type) = gb.read_r8(reg)?;
    gb.write_r8(reg, operand | (1 << bit))?;
    Ok(match access_type {
        Direct => 2,
        Indirect => 4,
    })
}

fn prefix(gb: &mut GameBoy) -> anyhow::Result<u8> {
    let instruction = gb.read_and_increment_pc();
    match instruction {
        (0o00..=0o07) => rlc(gb, instruction),
        (0o10..=0o17) => rrc(gb, instruction),
        (0o20..=0o27) => rl(gb, instruction),
        (0o30..=0o37) => rr(gb, instruction),
        (0o40..=0o47) => sla(gb, instruction),
        (0o50..=0o57) => sra(gb, instruction),
        (0o60..=0o67) => swap(gb, instruction),
        (0o70..=0o77) => srl(gb, instruction),
        (0o100..=0o177) => bit(gb, instruction),
        (0o200..=0o277) => res(gb, instruction),
        (0o300..=0o377) => set(gb, instruction),
    }
}

fn add_and_set_flags_no_carry(gb: &mut GameBoy, a: u8, b: u8) -> u8 {
    add_and_set_flags(gb, a, b, None)
}

fn add_and_set_flags_with_carry(gb: &mut GameBoy, a: u8, b: u8) -> u8 {
    add_and_set_flags(gb, a, b, Some(u8::from(gb.get_c())))
}

fn add_and_set_flags(gb: &mut GameBoy, a: u8, b: u8, carry: Option<u8>) -> u8 {
    let carry_or_0 = carry.unwrap_or(0);
    let (result, is_carry) = a.overflowing_add(b);
    let (result, is_carry_2) = result.overflowing_add(carry_or_0);
    gb.set_z(result == 0);
    gb.set_n(false);
    gb.set_h(is_add_half_carry_8(a, b) || is_add_half_carry_8(a + b, carry_or_0));
    gb.set_c(is_carry | is_carry_2);
    result
}

fn sub_and_set_flags_no_carry(gb: &mut GameBoy, a: u8, b: u8) -> u8 {
    sub_and_set_flags(gb, a, b, None)
}

fn sub_and_set_flags_with_carry(gb: &mut GameBoy, a: u8, b: u8) -> u8 {
    sub_and_set_flags(gb, a, b, Some(u8::from(gb.get_c())))
}

fn sub_and_set_flags(gb: &mut GameBoy, a: u8, b: u8, carry: Option<u8>) -> u8 {
    let carry_or_0 = carry.unwrap_or(0);
    let (result, is_carry) = a.overflowing_sub(b);
    let (result, is_carry_2) = result.overflowing_sub(carry_or_0);
    gb.set_z(result == 0);
    gb.set_n(true);
    gb.set_h(is_sub_half_carry_8(a, b) || is_sub_half_carry_8(a - b, carry_or_0));
    gb.set_c(is_carry || is_carry_2);
    result
}

fn add_signed_and_set_flags(gb: &mut GameBoy, a: u16, b: i8) -> u16 {
    gb.set_z(false);
    gb.set_n(false);
    gb.set_h(is_signed_add_half_carry_16_8(a, b));
    gb.set_c(is_signed_add_carry_16_8(a, b));
    a.wrapping_add_signed(i16::from(b))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_get_bits() {
        assert_eq!(super::get_bits(0b00110010, 5, 1), 0b11001);
    }
}
