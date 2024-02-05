use crate::gb::MemoryMappedDevice;

const SIZE: usize = 0xFFFF - 0xFF80;

pub struct HighRam {
    ram: [u8; SIZE],
}

impl HighRam {
    pub fn new() -> HighRam {
        HighRam { ram: [0u8; SIZE] }
    }
}

impl MemoryMappedDevice for HighRam {
    fn read(&self, addr: u16) -> anyhow::Result<u8> {
        Ok(self.ram[usize::from(addr)])
    }

    fn write(&mut self, addr: u16, val: u8) -> anyhow::Result<()> {
        Ok(self.ram[usize::from(addr)] = val)
    }
}
