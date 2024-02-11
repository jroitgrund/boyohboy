use crate::gb::gpu::OBJ_ATTRIBUTES_BASE;
use crate::gb::memory::cartridge::Cartridge;
use crate::gb::memory::external_ram::ExternalRam;
use crate::gb::memory::high_ram::HighRam;
use crate::gb::memory::interrupt_enable_register::InterruptEnableRegister;
use crate::gb::memory::io_registers::IORegisters;
use crate::gb::memory::not_usable::NotUsable;
use crate::gb::memory::object_attribute_memory::ObjectAttributeMemory;
use crate::gb::memory::ram::Ram;
use crate::gb::memory::video_ram::VideoRam;
use std::path::Path;

mod cartridge;
mod external_ram;
mod high_ram;
mod interrupt_enable_register;
mod io_registers;
mod not_usable;
mod object_attribute_memory;
mod ram;
mod video_ram;

const DMA_REGISTER: u16 = 0xFF46;

pub trait MemoryMappedDevice {
    fn read(&self, addr: u16) -> anyhow::Result<u8>;
    fn write(&mut self, addr: u16, val: u8) -> anyhow::Result<()>;
}

pub struct Memory {
    cartridge: Cartridge,
    video_ram: VideoRam,
    external_ram: ExternalRam,
    ram: Ram,
    object_attribute_memory: ObjectAttributeMemory,
    not_usable: NotUsable,
    pub io_registers: IORegisters,
    high_ram: HighRam,
    interrupt_enable_register: InterruptEnableRegister,
}

impl Memory {
    pub fn new(cartridge: &Path) -> anyhow::Result<Memory> {
        Ok(Memory {
            cartridge: Cartridge::new(cartridge)?,
            video_ram: VideoRam::new(),
            external_ram: ExternalRam::new(),
            ram: Ram::new(),
            object_attribute_memory: ObjectAttributeMemory::new(),
            not_usable: NotUsable {},
            io_registers: IORegisters::new(),
            high_ram: HighRam::new(),
            interrupt_enable_register: InterruptEnableRegister::new(),
        })
    }
    pub fn read(&mut self, addr: u16) -> anyhow::Result<u8> {
        let (device, offset) = self.get_device_and_offset(addr)?;
        device.read(offset)
    }
    pub fn write(&mut self, addr: u16, val: u8) -> anyhow::Result<()> {
        match addr {
            DMA_REGISTER => {
                for offset in 0..0xFF {
                    let byte = self.read((u16::from(val) << 2) + offset)?;
                    self.write(OBJ_ATTRIBUTES_BASE + offset, byte)?;
                }
                Ok(())
            }
            _ => {
                let (device, offset) = self.get_device_and_offset(addr)?;
                device.write(offset, val)
            }
        }
    }

    fn get_device_and_offset(
        &mut self,
        addr: u16,
    ) -> anyhow::Result<(&mut dyn MemoryMappedDevice, u16)> {
        match addr {
            0x0000..=0x7FFF => Ok((&mut self.cartridge, addr)),
            0x8000..=0x9FFF => Ok((&mut self.video_ram, addr - 0x8000)),
            0xA000..=0xBFFF => Ok((&mut self.external_ram, addr - 0xA000)),
            0xC000..=0xDFFF => Ok((self.ram.work_ram(), addr - 0xC000)),
            0xE000..=0xFDFF => Ok((self.ram.mirror_ram(), addr - 0xE000)),
            0xFE00..=0xFE9F => Ok((&mut self.object_attribute_memory, addr - 0xFE00)),
            0xFEA0..=0xFEFF => Ok((&mut self.not_usable, addr - 0xFEA0)),
            0xFF00..=0xFF7F => Ok((&mut self.io_registers, addr - 0xFF00)),
            0xFF80..=0xFFFE => Ok((&mut self.high_ram, addr - 0xFF80)),
            0xFFFF..=0xFFFF => Ok((&mut self.interrupt_enable_register, addr - 0xFFFF)),
        }
    }
}
