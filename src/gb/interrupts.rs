use crate::gb::bits::{get_lsb, set_bit};
use crate::gb::GameBoyImpl;
use anyhow::anyhow;
use anyhow::Result;

pub enum Interrupts {
    VBlank,
    LCD,
    Timer,
    Serial,
    Joypad,
}

const IF_REGISTER: u16 = 0xFF0F;
const IE_REGISTER: u16 = 0xFFFF;

pub struct InterruptResult {
    pub interrupt_requested: bool,
    pub interrupts_enabled: bool,
    pub cycles: u8,
}

impl GameBoyImpl {
    pub fn handle_interrupts(&mut self) -> Result<InterruptResult> {
        let if_reg = self.read_8(IF_REGISTER)?;
        let ie_reg = self.read_8(IE_REGISTER)?;
        let interrupts = if_reg & ie_reg & 0x1F;

        Ok(if self.ime && interrupts != 0 {
            self.ime = false;
            self.push_16(self.pc)?;
            let lsb = get_lsb(interrupts);
            self.write_8(IF_REGISTER, if_reg & (!lsb))?;
            self.pc = match lsb {
                1 => Ok(0x40),
                2 => Ok(0x48),
                4 => Ok(0x50),
                8 => Ok(0x58),
                16 => Ok(0x60),
                _ => Err(anyhow!("Unexpected lsb {}", lsb)),
            }?;
            InterruptResult {
                interrupt_requested: true,
                interrupts_enabled: true,
                cycles: 5,
            }
        } else {
            InterruptResult {
                interrupts_enabled: self.ime,
                interrupt_requested: interrupts != 0,
                cycles: 0,
            }
        })
    }

    pub fn trigger_interrupt(&mut self, interrupt: Interrupts) -> Result<()> {
        self.set_interrupt_bit(match interrupt {
            Interrupts::VBlank => 0,
            Interrupts::LCD => 1,
            Interrupts::Timer => 2,
            Interrupts::Serial => 3,
            Interrupts::Joypad => 4,
        })
    }

    fn set_interrupt_bit(&mut self, bit: u8) -> Result<()> {
        let if_reg = self.read_8(IF_REGISTER)?;
        self.write_8(IF_REGISTER, set_bit(if_reg, bit))
    }
}
