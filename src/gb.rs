use crate::gb::AccessType::{Direct, Indirect};
use anyhow::anyhow;
use std::ops;

const R16_HL: u8 = 2;

pub enum AccessType {
    Direct,
    Indirect,
}

impl ops::Add<AccessType> for AccessType {
    type Output = AccessType;

    fn add(self, rhs: AccessType) -> AccessType {
        match (self, rhs) {
            (Direct, Direct) => Direct,
            _ => Indirect,
        }
    }
}

pub struct GameBoy {
    pub a: u8,
    f: u8,
    b: u8,
    pub c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    pub sp: u16,
    pub pc: u16,
    pub ime: bool,
    memory: [u8; 10_000_000],
}

impl GameBoy {
    pub fn new() -> GameBoy {
        GameBoy {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
            ime: false,
            memory: [0; 10_000_000],
        }
    }

    pub fn read_8(&self, addr: u16) -> u8 {
        self.memory[usize::from(addr)]
    }

    pub fn write_8(&mut self, addr: u16, val: u8) {
        self.memory[usize::from(addr)] = val
    }

    pub fn write_16(&mut self, addr: u16, val: u16) {
        let bytes = val.to_le_bytes();
        self.write_8(addr, bytes[0]);
        self.write_8(addr + 1, bytes[1]);
    }

    pub fn read_and_increment_pc(&mut self) -> u8 {
        let result = self.read_8(self.pc);
        self.pc += 1;
        result
    }

    pub fn read_r8(&self, r: u8) -> anyhow::Result<(u8, AccessType)> {
        match r {
            0 => Ok((self.b, Direct)),
            1 => Ok((self.c, Direct)),
            2 => Ok((self.d, Direct)),
            3 => Ok((self.e, Direct)),
            4 => Ok((self.h, Direct)),
            5 => Ok((self.l, Direct)),
            6 => Ok((self.read_8(self.read_hl()?), Indirect)),
            7 => Ok((self.a, Direct)),
            _ => Err(anyhow!("Unknown register {}", r)),
        }
    }

    pub fn write_r8(&mut self, r: u8, val: u8) -> anyhow::Result<AccessType> {
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
                self.write_8(self.read_hl()?, val);
                Ok(Indirect)
            }
            7 => {
                self.a = val;
                Ok(Direct)
            }
            _ => Err(anyhow!("Unknown register {}", r)),
        }
    }

    pub fn read_r16(&self, r: u8) -> anyhow::Result<u16> {
        match r {
            0 => Ok(u16::from_le_bytes([self.b, self.c])),
            1 => Ok(u16::from_le_bytes([self.d, self.e])),
            2 => Ok(u16::from_le_bytes([self.h, self.l])),
            3 => Ok(self.sp),
            _ => Err(anyhow!("Unknown register {}", r)),
        }
    }

    pub fn read_hl(&self) -> anyhow::Result<u16> {
        self.read_r16(R16_HL)
    }

    pub fn write_r16(&mut self, r: u8, val: u16) -> anyhow::Result<()> {
        let bytes = val.to_le_bytes();
        match r {
            0 => Ok({
                self.b = bytes[0];
                self.c = bytes[1];
            }),
            1 => Ok({
                self.d = bytes[0];
                self.e = bytes[1];
            }),
            2 => Ok({
                self.h = bytes[0];
                self.l = bytes[1];
            }),
            3 => Ok(self.sp = val),
            _ => Err(anyhow!("Unknown register {}", r)),
        }
    }

    pub fn write_hl(&mut self, val: u16) -> anyhow::Result<()> {
        self.write_r16(R16_HL, val)
    }

    pub fn r16_mem(&mut self, r: u8) -> anyhow::Result<u16> {
        match r {
            0 | 1 => self.read_r16(r),
            2 => {
                let hl = u16::from_le_bytes([self.h, self.l]);
                self.write_hl(hl + 1)?;
                Ok(hl)
            }
            3 => {
                let hl = u16::from_le_bytes([self.h, self.l]);
                self.write_hl(hl - 1)?;
                Ok(hl)
            }
            _ => Err(anyhow!("Unknown register {}", r)),
        }
    }

    pub fn read_r16_stk(&self, r: u8) -> anyhow::Result<u16> {
        match r {
            0 | 1 | 2 => self.read_r16(r),
            3 => Ok(u16::from_le_bytes([self.a, self.f])),
            _ => Err(anyhow!("Unknown register {}", r)),
        }
    }

    pub fn write_r16_stk(&mut self, r: u8, val: u16) -> anyhow::Result<()> {
        match r {
            0 | 1 | 2 => self.write_r16(r, val),
            3 => Ok({
                let bytes = val.to_le_bytes();
                self.a = bytes[0];
                self.f = bytes[1];
            }),
            _ => Err(anyhow!("Unknown register {}", r)),
        }
    }

    pub fn read_n8(&mut self) -> u8 {
        self.read_and_increment_pc()
    }

    pub fn read_n16(&mut self) -> u16 {
        u16::from_le_bytes([self.read_and_increment_pc(), self.read_and_increment_pc()])
    }

    fn get_z(&self) -> bool {
        return self.get_flag(0);
    }

    pub fn get_n(&self) -> bool {
        return self.get_flag(1);
    }

    pub fn get_h(&self) -> bool {
        return self.get_flag(2);
    }

    pub fn get_c(&self) -> bool {
        return self.get_flag(3);
    }

    fn get_flag(&self, flag_i: u8) -> bool {
        let shifts = 7 - flag_i;
        return (self.f >> shifts) & 1 == 1;
    }

    pub fn set_z(&mut self, set: bool) {
        self.set_flag(0, set)
    }

    pub fn set_n(&mut self, set: bool) {
        self.set_flag(1, set)
    }

    pub fn set_h(&mut self, set: bool) {
        self.set_flag(2, set)
    }

    pub fn set_c(&mut self, set: bool) {
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

    pub fn read_cond(&self, cond: u8) -> anyhow::Result<bool> {
        match cond {
            0 => Ok(!self.get_z()),
            1 => Ok(self.get_z()),
            2 => Ok(!self.get_c()),
            3 => Ok(self.get_c()),
            _ => Err(anyhow!("Unknown condition {}", cond)),
        }
    }

    fn push_8(&mut self, val: u8) {
        self.sp -= 1;
        self.write_8(self.sp, val);
    }

    pub fn push_16(&mut self, val: u16) {
        let bytes = val.to_be_bytes();
        self.push_8(bytes[0]);
        self.push_8(bytes[1]);
    }

    fn pop_8(&mut self) -> u8 {
        let res = self.read_8(self.sp);
        self.sp += 1;
        res
    }

    pub fn pop_16(&mut self) -> u16 {
        u16::from_le_bytes([self.pop_8(), self.pop_8()])
    }
}
