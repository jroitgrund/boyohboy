use crate::gb::memory::MemoryMappedDevice;
use log::{info, warn};

pub struct NotUsable {}

impl MemoryMappedDevice for NotUsable {
    fn read(&self, addr: u16) -> anyhow::Result<u8> {
        warn!("Read from unusable address {}", addr);
        Ok(0xFF)
    }

    fn write(&mut self, addr: u16, _val: u8) -> anyhow::Result<()> {
        info!("Write to unusuable address {}", addr);
        Ok(())
    }
}
