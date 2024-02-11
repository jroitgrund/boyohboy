use crate::gb::bits::get_bits;
use crate::gb::memory::map::{TAC, TMA};
use crate::gb::memory::Memory;
use anyhow::anyhow;

pub struct TimerInfo {
    pub cycles: usize,
    pub modulo: u8,
    pub enable: bool,
}

impl TimerInfo {
    pub fn from_memory(memory: &mut Memory) -> anyhow::Result<TimerInfo> {
        let timer_control = memory.read(TAC)?;
        let enable = get_bits(timer_control, 2, 2) == 1;
        let cycles = match get_bits(timer_control, 1, 0) {
            0b00 => Ok(64),
            0b01 => Ok(4),
            0b10 => Ok(16),
            0b11 => Ok(64),
            _ => Err(anyhow!("Unknown timer control value {}", timer_control)),
        }?;
        let modulo = memory.read(TMA)?;

        Ok(TimerInfo {
            enable,
            cycles,
            modulo,
        })
    }
}
