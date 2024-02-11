use crate::gb::memory::MemoryMappedDevice;
use anyhow::anyhow;

pub struct Ram {
    mirror_ram: MirrorRam,
}

impl Ram {
    pub fn new() -> Ram {
        Ram {
            mirror_ram: MirrorRam::new(),
        }
    }

    pub fn mirror_ram(&mut self) -> &mut dyn MemoryMappedDevice {
        &mut self.mirror_ram
    }

    pub fn work_ram(&mut self) -> &mut dyn MemoryMappedDevice {
        &mut self.mirror_ram.work_ram
    }
}

struct MirrorRam {
    work_ram: WorkRam,
}

impl MirrorRam {
    fn new() -> MirrorRam {
        MirrorRam {
            work_ram: WorkRam::new(),
        }
    }
}

impl MemoryMappedDevice for MirrorRam {
    fn read(&self, addr: u16) -> anyhow::Result<u8> {
        self.work_ram.read(addr)
    }

    fn write(&mut self, addr: u16, _val: u8) -> anyhow::Result<()> {
        Err(anyhow!("Prohibited write to mirror RAM: {}", addr))
    }
}

const SIZE: usize = 0xE000 - 0xC000;

struct WorkRam {
    ram: [u8; SIZE],
}

impl WorkRam {
    pub fn new() -> WorkRam {
        WorkRam { ram: [0u8; SIZE] }
    }
}

impl MemoryMappedDevice for WorkRam {
    fn read(&self, addr: u16) -> anyhow::Result<u8> {
        Ok(self.ram[usize::from(addr)])
    }

    fn write(&mut self, addr: u16, val: u8) -> anyhow::Result<()> {
        self.ram[usize::from(addr)] = val;
        Ok(())
    }
}
