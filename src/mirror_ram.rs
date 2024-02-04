use crate::gb::MemoryMappedDevice;
use crate::work_ram::WorkRam;
use anyhow::anyhow;

pub struct MirrorRam<'a> {
    work_ram: Option<&'a mut WorkRam>,
}

impl MemoryMappedDevice for MirrorRam<'_> {
    fn read(&self, addr: u16) -> anyhow::Result<u8> {
        self.work_ram.as_ref().unwrap().read(addr)
    }

    fn write(&mut self, addr: u16, _val: u8) -> anyhow::Result<()> {
        Err(anyhow!("Prohibited write to mirror RAM: {}", addr))
    }
}
