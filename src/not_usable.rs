use crate::gb::MemoryMappedDevice;
use anyhow::anyhow;

pub struct NotUsable {}

impl MemoryMappedDevice for NotUsable {
    fn read(&self, addr: u16) -> anyhow::Result<u8> {
        Err(anyhow!("Read from unusuable address {}", addr))
    }

    fn write(&mut self, addr: u16, _val: u8) -> anyhow::Result<()> {
        Err(anyhow!("Write to unusuable address {}", addr))
    }
}
