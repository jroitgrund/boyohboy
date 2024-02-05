use crate::gb::bits::get_bits;
use crate::gb::interrupts::Interrupts::Timer;
use crate::gb::GameBoyImpl;
use anyhow::{anyhow, Result};

const DIV_CYCLES: u8 = 64;
const TIMER_MODULO: u16 = 0xFF06;
const TIMER_CONTROL: u16 = 0xFF07;

struct TimerInfo {
    cycles: usize,
    modulo: u8,
    enable: bool,
}

impl GameBoyImpl {
    pub fn tick(&mut self, cycles: usize) -> Result<()> {
        let timer_info = self.get_timer_info()?;

        for _i in 0..cycles {
            self.cycles = self.cycles.wrapping_add(1);

            if self.cycles % usize::from(DIV_CYCLES) == 0 {
                self.io_registers.tick_div()?;
            }

            if timer_info.enable && self.cycles % usize::from(timer_info.cycles) == 0 {
                if self.io_registers.tick_timer(timer_info.modulo)? {
                    self.trigger_interrupt(Timer)?;
                }
            }
        }

        Ok(())
    }

    fn get_timer_info(&mut self) -> Result<TimerInfo> {
        let timer_control = self.read_8(TIMER_CONTROL)?;
        let enable = get_bits(timer_control, 2, 2) == 1;
        let cycles = match get_bits(timer_control, 1, 0) {
            0b00 => Ok(64),
            0b01 => Ok(4),
            0b10 => Ok(16),
            0b11 => Ok(64),
            _ => Err(anyhow!("Unknown timer control value {}", timer_control)),
        }?;
        let modulo = self.read_8(TIMER_MODULO)?;

        return Ok(TimerInfo {
            enable,
            cycles,
            modulo,
        });
    }
}
