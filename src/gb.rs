use crate::cartridge::Cartridge;
use crate::external_ram::ExternalRam;
use crate::gb::AccessType::{Direct, Indirect};
use crate::high_ram::HighRam;
use crate::interrupt_enable_register::InterruptEnableRegister;
use crate::io_registers::IORegisters;
use crate::not_usable::NotUsable;
use crate::object_attribute_memory::ObjectAttributeMemory;
use crate::video_ram::VideoRam;
use crate::work_ram::WorkRam;
use anyhow::anyhow;
use anyhow::Result;

use std::ops;
use std::ops::DerefMut;
use std::path::Path;

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
    pub halted: bool,
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
    cartridge: Cartridge,
    video_ram: VideoRam,
    external_ram: ExternalRam,
    work_ram: WorkRam,
    object_attribute_memory: ObjectAttributeMemory,
    not_usable: NotUsable,
    io_registers: IORegisters,
    high_ram: HighRam,
    interrupt_enable_register: InterruptEnableRegister,
}

pub trait MemoryMappedDevice {
    fn read(&self, addr: u16) -> Result<u8>;
    fn write(&mut self, addr: u16, val: u8) -> Result<()>;
}

impl GameBoy {
    pub fn serial(&mut self) -> Result<Option<String>> {
        if self.read_8(0xff02)? == 0x81 {
            let serial = self.read_8(0xff01)?;
            let log = format!("{}", serial as char);
            self.write_8(0xff02, 0x0)?;
            Ok(Some(log))
        } else {
            Ok(None)
        }
    }
    pub fn log(&mut self) -> Result<String> {
        let pc = self.read_8(self.pc)?;
        let pc_1 = self.read_8(self.pc + 1)?;
        let pc_2 = self.read_8(self.pc + 2)?;
        let pc_3 = self.read_8(self.pc + 3)?;
        Ok(format!(
            "A: {:02X?} F: {:02X?} B: {:02X?} C: {:02X?} D: {:02X?} E: {:02X?} H: {:02X?} L: {:02X?} SP: {:04X?} PC: 00:{:04X?} ({:02X?} {:02X?} {:02X?} {:02X?})\n",
            self.a, self.f, self.b, self.c, self.d, self.e, self.h, self.l, self.sp, self.pc, pc, pc_1, pc_2, pc_3
        ))
    }

    pub fn set_ime(&mut self, enable: bool) -> Result<()> {
        self.interrupt_enable_register.write(
            0,
            match enable {
                true => 1,
                false => 0,
            },
        )
    }

    pub fn new(cartridge: &Path) -> Result<GameBoy> {
        let mut gb = GameBoy {
            halted: false,
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
            cartridge: Cartridge::new(cartridge)?,
            video_ram: VideoRam::new(),
            external_ram: ExternalRam::new(),
            work_ram: WorkRam::new(),
            object_attribute_memory: ObjectAttributeMemory::new(),
            not_usable: NotUsable {},
            io_registers: IORegisters::new(),
            high_ram: HighRam::new(),
            interrupt_enable_register: InterruptEnableRegister::new(),
        };

        gb.write_8(0xFF00, 0xCF)?;
        gb.write_8(0xFF01, 0x00)?;
        gb.write_8(0xFF02, 0x7E)?;
        gb.write_8(0xFF04, 0xAB)?;
        gb.write_8(0xFF05, 0x00)?;
        gb.write_8(0xFF06, 0x00)?;
        gb.write_8(0xFF07, 0xF8)?;
        gb.write_8(0xFF0F, 0xE1)?;
        gb.write_8(0xFF10, 0x80)?;
        gb.write_8(0xFF11, 0xBF)?;
        gb.write_8(0xFF12, 0xF3)?;
        gb.write_8(0xFF13, 0xFF)?;
        gb.write_8(0xFF14, 0xBF)?;
        gb.write_8(0xFF16, 0x3F)?;
        gb.write_8(0xFF17, 0x00)?;
        gb.write_8(0xFF18, 0xFF)?;
        gb.write_8(0xFF19, 0xBF)?;
        gb.write_8(0xFF1A, 0x7F)?;
        gb.write_8(0xFF1B, 0xFF)?;
        gb.write_8(0xFF1C, 0x9F)?;
        gb.write_8(0xFF1D, 0xFF)?;
        gb.write_8(0xFF1E, 0xBF)?;
        gb.write_8(0xFF20, 0xFF)?;
        gb.write_8(0xFF21, 0x00)?;
        gb.write_8(0xFF22, 0x00)?;
        gb.write_8(0xFF23, 0xBF)?;
        gb.write_8(0xFF24, 0x77)?;
        gb.write_8(0xFF25, 0xF3)?;
        gb.write_8(0xFF26, 0xF1)?;
        gb.write_8(0xFF40, 0x91)?;
        gb.write_8(0xFF41, 0x85)?;
        gb.write_8(0xFF42, 0x00)?;
        gb.write_8(0xFF43, 0x00)?;
        gb.write_8(0xFF44, 0x90)?;
        gb.write_8(0xFF45, 0x00)?;
        gb.write_8(0xFF46, 0xFF)?;
        gb.write_8(0xFF47, 0xFC)?;
        gb.write_8(0xFF48, 0xFF)?;
        gb.write_8(0xFF49, 0xFF)?;
        gb.write_8(0xFF4A, 0x00)?;
        gb.write_8(0xFF4B, 0x00)?;
        gb.write_8(0xFF4D, 0xFF)?;
        gb.write_8(0xFF4F, 0xFF)?;
        gb.write_8(0xFF51, 0xFF)?;
        gb.write_8(0xFF52, 0xFF)?;
        gb.write_8(0xFF53, 0xFF)?;
        gb.write_8(0xFF54, 0xFF)?;
        gb.write_8(0xFF55, 0xFF)?;
        gb.write_8(0xFF56, 0xFF)?;
        gb.write_8(0xFF68, 0xFF)?;
        gb.write_8(0xFF69, 0xFF)?;
        gb.write_8(0xFF6A, 0xFF)?;
        gb.write_8(0xFF6B, 0xFF)?;
        gb.write_8(0xFF70, 0xFF)?;
        gb.write_8(0xFFFF, 0x00)?;
        Ok(gb)
    }

    fn get_device_and_offset(
        &mut self,
        addr: u16,
    ) -> Result<(Box<&mut dyn MemoryMappedDevice>, u16)> {
        match addr {
            0x0000..=0x7FFF => Ok((Box::new(&mut self.cartridge), addr)),
            0x8000..=0x9FFF => Ok((Box::new(&mut self.video_ram), addr - 0x8000)),
            0xA000..=0xBFFF => Ok((Box::new(&mut self.external_ram), addr - 0xA000)),
            0xC000..=0xDFFF => Ok((Box::new(&mut self.work_ram), addr - 0xC000)),
            0xE000..=0xFDFF => Ok((Box::new(&mut self.work_ram), addr - 0xE000)),
            0xFE00..=0xFE9F => Ok((Box::new(&mut self.object_attribute_memory), addr - 0xFE00)),
            0xFEA0..=0xFEFF => Ok((Box::new(&mut self.not_usable), addr - 0xFEA0)),
            0xFF00..=0xFF7F => Ok((Box::new(&mut self.io_registers), addr - 0xFF00)),
            0xFF80..=0xFFFE => Ok((Box::new(&mut self.high_ram), addr - 0xFF80)),
            0xFFFF..=0xFFFF => Ok((Box::new(&mut self.interrupt_enable_register), addr - 0xFFFF)),
        }
    }

    pub fn read_8(&mut self, addr: u16) -> Result<u8> {
        let (device, offset) = self.get_device_and_offset(addr)?;
        if addr == 0xFF44 {
            println!();
        }
        device.read(offset)
    }

    pub fn write_8(&mut self, addr: u16, val: u8) -> Result<()> {
        let (mut device, offset) = self.get_device_and_offset(addr)?;
        if addr == 0xFF44 {
            println!();
        }
        device.deref_mut().write(offset, val)
    }

    pub fn write_16(&mut self, addr: u16, val: u16) -> Result<()> {
        let bytes = val.to_le_bytes();
        self.write_8(addr, bytes[0])?;
        self.write_8(addr + 1, bytes[1])
    }

    pub fn read_and_increment_pc(&mut self) -> Result<u8> {
        let result = self.read_8(self.pc)?;
        self.pc += 1;
        Ok(result)
    }

    pub fn read_r8(&mut self, r: u8) -> anyhow::Result<(u8, AccessType)> {
        match r {
            0 => Ok((self.b, Direct)),
            1 => Ok((self.c, Direct)),
            2 => Ok((self.d, Direct)),
            3 => Ok((self.e, Direct)),
            4 => Ok((self.h, Direct)),
            5 => Ok((self.l, Direct)),
            6 => Ok((self.read_8(self.read_hl()?)?, Indirect)),
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
                self.write_8(self.read_hl()?, val)?;
                Ok(Indirect)
            }
            7 => {
                self.a = val;
                Ok(Direct)
            }
            _ => Err(anyhow!("Unknown register {}", r)),
        }
    }

    pub fn read_r16(&self, r: u8) -> Result<u16> {
        match r {
            0 => Ok(u16::from_be_bytes([self.b, self.c])),
            1 => Ok(u16::from_be_bytes([self.d, self.e])),
            2 => Ok(u16::from_be_bytes([self.h, self.l])),
            3 => Ok(self.sp),
            _ => Err(anyhow!("Unknown register {}", r)),
        }
    }

    pub fn read_hl(&self) -> Result<u16> {
        self.read_r16(R16_HL)
    }

    pub fn write_r16(&mut self, r: u8, val: u16) -> Result<()> {
        let bytes = val.to_be_bytes();
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

    pub fn write_hl(&mut self, val: u16) -> Result<()> {
        self.write_r16(R16_HL, val)
    }

    pub fn r16_mem(&mut self, r: u8) -> anyhow::Result<u16> {
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

    pub fn read_r16_stk(&self, r: u8) -> anyhow::Result<u16> {
        match r {
            0 | 1 | 2 => self.read_r16(r),
            3 => Ok(u16::from_be_bytes([self.a, self.f])),
            _ => Err(anyhow!("Unknown register {}", r)),
        }
    }

    pub fn write_r16_stk(&mut self, r: u8, val: u16) -> Result<()> {
        match r {
            0 | 1 | 2 => self.write_r16(r, val),
            3 => Ok({
                let bytes = val.to_be_bytes();
                self.a = bytes[0];
                self.f = bytes[1] & 0xF0;
            }),
            _ => Err(anyhow!("Unknown register {}", r)),
        }
    }

    pub fn read_n8(&mut self) -> Result<u8> {
        self.read_and_increment_pc()
    }

    pub fn read_n16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes([
            self.read_and_increment_pc()?,
            self.read_and_increment_pc()?,
        ]))
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

    fn push_8(&mut self, val: u8) -> Result<()> {
        self.sp -= 1;
        self.write_8(self.sp, val)
    }

    pub fn push_16(&mut self, val: u16) -> Result<()> {
        let bytes = val.to_be_bytes();
        self.push_8(bytes[0])?;
        self.push_8(bytes[1])
    }

    fn pop_8(&mut self) -> Result<u8> {
        let res = self.read_8(self.sp)?;
        self.sp += 1;
        Ok(res)
    }

    pub fn pop_16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes([self.pop_8()?, self.pop_8()?]))
    }
}
