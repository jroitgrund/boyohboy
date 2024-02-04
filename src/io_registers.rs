use crate::gb::MemoryMappedDevice;

const SIZE: usize = 0xFF80 - 0xFF00;

pub struct IORegisters {
    ram: [u8; SIZE],
}

impl IORegisters {
    pub fn new() -> IORegisters {
        IORegisters { ram: [0u8; SIZE] }
    }
}

impl MemoryMappedDevice for IORegisters {
    fn read(&self, addr: u16) -> anyhow::Result<u8> {
        Ok(self.ram[usize::from(addr)])
    }

    fn write(&mut self, addr: u16, val: u8) -> anyhow::Result<()> {
        Ok(self.ram[usize::from(addr)] = val)
    }
}
