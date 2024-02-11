use crate::gb::memory::MemoryMappedDevice;

const SIZE: usize = 1;

pub struct InterruptEnableRegister {
    ram: [u8; SIZE],
}

impl InterruptEnableRegister {
    pub fn new() -> InterruptEnableRegister {
        InterruptEnableRegister { ram: [0u8; SIZE] }
    }
}

impl MemoryMappedDevice for InterruptEnableRegister {
    fn read(&self, addr: u16) -> anyhow::Result<u8> {
        Ok(self.ram[usize::from(addr)])
    }

    fn write(&mut self, addr: u16, val: u8) -> anyhow::Result<()> {
        self.ram[usize::from(addr)] = val;
        Ok(())
    }
}
