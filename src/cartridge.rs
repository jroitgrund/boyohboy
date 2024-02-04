use crate::gb::MemoryMappedDevice;
use anyhow::{anyhow, Result};
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
        return Ok(Cartridge { mmap });
    }
}

impl MemoryMappedDevice for Cartridge {
    fn read(&self, addr: u16) -> Result<u8> {
        Ok(self.mmap[usize::from(addr)])
    }

    fn write(&mut self, _addr: u16, _val: u8) -> Result<()> {
        Err(anyhow!("Cannot write to cartridge"))
    }
}
