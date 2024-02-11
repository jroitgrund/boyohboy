use crate::gb::memory::MemoryMappedDevice;

const SIZE: usize = 0xA000 - 0x8000;

#[derive(Debug)]
pub struct VideoRam {
    ram: [u8; SIZE],
}

impl VideoRam {
    pub fn new() -> VideoRam {
        VideoRam { ram: [0u8; SIZE] }
    }
}

impl MemoryMappedDevice for VideoRam {
    fn read(&self, addr: u16) -> anyhow::Result<u8> {
        Ok(self.ram[usize::from(addr)])
    }

    fn write(&mut self, addr: u16, val: u8) -> anyhow::Result<()> {
        Ok(self.ram[usize::from(addr)] = val)
    }
}
