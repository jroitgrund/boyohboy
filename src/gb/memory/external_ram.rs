use crate::gb::memory::MemoryMappedDevice;

const SIZE: usize = 0xC000 - 0xA000;

pub struct ExternalRam {
    ram: [u8; SIZE],
}

impl ExternalRam {
    pub fn new() -> ExternalRam {
        ExternalRam { ram: [0u8; SIZE] }
    }
}

impl MemoryMappedDevice for ExternalRam {
    fn read(&self, addr: u16) -> anyhow::Result<u8> {
        Ok(self.ram[usize::from(addr)])
    }

    fn write(&mut self, addr: u16, val: u8) -> anyhow::Result<()> {
        Ok(self.ram[usize::from(addr)] = val)
    }
}
