use crate::gb::memory::map::MBC_TYPE;
use crate::gb::memory::MemoryMappedDevice;
use anyhow::Result;
use log::warn;
use memmap::{Mmap, MmapOptions};
use std::fs::File;
use std::path::Path;

pub struct Cartridge {
    mmap: Mmap,
}

impl Cartridge {
    pub fn new(cartridge: &Path) -> Result<Cartridge> {
        let file = File::open(cartridge)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        let mbc = mmap[usize::from(MBC_TYPE)];
        warn!("MBC: {}", mbc);
        Ok(Cartridge { mmap })
    }
}

impl MemoryMappedDevice for Cartridge {
    fn read(&self, addr: u16) -> Result<u8> {
        Ok(self.mmap[usize::from(addr)])
    }

    fn write(&mut self, _addr: u16, _val: u8) -> Result<()> {
        warn!("Cannot write to cartridge ({})", _addr);
        Ok(())
    }
}
