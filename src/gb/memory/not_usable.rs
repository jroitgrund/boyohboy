use crate::gb::memory::MemoryMappedDevice;
use anyhow::anyhow;
use log::info;

pub struct NotUsable {}

impl MemoryMappedDevice for NotUsable {
    fn read(&self, addr: u16) -> anyhow::Result<u8> {
        Err(anyhow!("Read from unusuable address {}", addr))
    }

    fn write(&mut self, addr: u16, _val: u8) -> anyhow::Result<()> {
        info!("Write to unusuable address {}", addr);
        Ok(())
    }
}
