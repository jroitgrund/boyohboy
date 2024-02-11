use crate::gb::memory::MemoryMappedDevice;

const SIZE: usize = 0xFEA0 - 0xFE00;

pub struct ObjectAttributeMemory {
    ram: [u8; SIZE],
}

impl ObjectAttributeMemory {
    pub fn new() -> ObjectAttributeMemory {
        ObjectAttributeMemory { ram: [0u8; SIZE] }
    }
}

impl MemoryMappedDevice for ObjectAttributeMemory {
    fn read(&self, addr: u16) -> anyhow::Result<u8> {
        Ok(self.ram[usize::from(addr)])
    }

    fn write(&mut self, addr: u16, val: u8) -> anyhow::Result<()> {
        Ok(self.ram[usize::from(addr)] = val)
    }
}
