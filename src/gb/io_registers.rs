use crate::gb::MemoryMappedDevice;

const SIZE: usize = 0xFF80 - 0xFF00;
const DIVIDER_REGISTER: u16 = 0xFF04 - 0xFF00;

const TIMER_REGISTER: u16 = 0xFF05 - 0xFF00;

pub struct IORegisters {
    ram: [u8; SIZE],
}

impl IORegisters {
    pub fn new() -> IORegisters {
        IORegisters { ram: [0u8; SIZE] }
    }

    pub fn tick_div(&mut self) -> anyhow::Result<()> {
        self.write(
            DIVIDER_REGISTER,
            self.read(DIVIDER_REGISTER)?.wrapping_add(1),
        )
    }

    pub fn tick_timer(&mut self, modulo: u8) -> anyhow::Result<bool> {
        let incremented = self.read(TIMER_REGISTER)?.wrapping_add(1);
        Ok(match incremented {
            0 => {
                self.write(TIMER_REGISTER, modulo)?;
                true
            }
            _ => {
                self.write(TIMER_REGISTER, incremented)?;
                false
            }
        })
    }
}

impl MemoryMappedDevice for IORegisters {
    fn read(&self, addr: u16) -> anyhow::Result<u8> {
        Ok(self.ram[usize::from(addr)])
    }

    fn write(&mut self, addr: u16, val: u8) -> anyhow::Result<()> {
        Ok(self.ram[usize::from(addr)] = val)
    }
}
