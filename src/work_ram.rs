use crate::gb::MemoryMappedDevice;

const SIZE: usize = 0xE000 - 0xC000;

pub struct WorkRam {
    ram: [u8; SIZE],
}

impl WorkRam {
    pub fn new() -> WorkRam {
        WorkRam { ram: [0u8; SIZE] }
    }
}

impl MemoryMappedDevice for WorkRam {
    fn read(&self, addr: u16) -> anyhow::Result<u8> {
        Ok(self.ram[usize::from(addr)])
    }

    fn write(&mut self, addr: u16, val: u8) -> anyhow::Result<()> {
        Ok(self.ram[usize::from(addr)] = val)
    }
}
