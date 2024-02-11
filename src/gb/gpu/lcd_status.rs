use crate::gb::bits::get_bit;
use crate::gb::memory::map::STAT;
use crate::gb::memory::Memory;

pub struct LcdStatus {
    pub lyc_interrupt: bool,
    pub mode_2_interrupt: bool,
    pub mode_1_interrupt: bool,
    pub mode_0_interrupt: bool,
}

impl LcdStatus {
    pub fn from_memory(memory: &mut Memory) -> anyhow::Result<LcdStatus> {
        let status = memory.read(STAT)?;
        let lyc_interrupt = get_bit(status, 6) == 1;
        let mode_2_interrupt = get_bit(status, 5) == 1;
        let mode_1_interrupt = get_bit(status, 4) == 1;
        let mode_0_interrupt = get_bit(status, 3) == 1;
        Ok(LcdStatus {
            lyc_interrupt,
            mode_0_interrupt,
            mode_1_interrupt,
            mode_2_interrupt,
        })
    }
}
