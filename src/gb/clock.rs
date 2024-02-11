use crate::gb::bits::{get_bit, get_bits};
use crate::gb::gpu::{Gpu, LY_REGISTER};
use crate::gb::interrupts::Interrupts::{Timer, VBlank, LCD};
use crate::gb::{GameBoyImpl, Pixel};
use anyhow::{anyhow, Result};

const DIV_CYCLES: u8 = 64;
const TIMER_MODULO: u16 = 0xFF06;
const TIMER_CONTROL: u16 = 0xFF07;
const LCD_STATUS: u16 = 0xFF41;
const LY_COMPARE: u16 = 0xFF45;

struct TimerInfo {
    cycles: usize,
    modulo: u8,
    enable: bool,
}

struct LcdStatus {
    lyc_interrupt: bool,
    mode_2_interrupt: bool,
    mode_1_interrupt: bool,
    mode_0_interrupt: bool,
    status: u8,
}

impl GameBoyImpl {
    pub fn tick(&mut self, cycles: usize) -> Result<Vec<Pixel>> {
        let old_mode: u8 = match &self.gpu {
            Gpu::Stopped => 2,
            Gpu::Mode2 { .. } => 2,
            Gpu::Mode3 { .. } => 3,
            Gpu::Mode0 { .. } => 0,
            Gpu::Mode1 { .. } => 1,
        };
        let timer_info = self.get_timer_info()?;

        let mut pixels: Vec<Pixel> = Vec::with_capacity(4 * cycles);

        for _ in 0..cycles {
            pixels.append(&mut self.tick_gpu()?);
            self.cycles = self.cycles.wrapping_add(1);

            if self.cycles % usize::from(DIV_CYCLES) == 0 {
                self.memory.io_registers.tick_div()?;
            }

            if timer_info.enable
                && self.cycles % usize::from(timer_info.cycles) == 0
                && self.memory.io_registers.tick_timer(timer_info.modulo)?
            {
                self.trigger_interrupt(Timer)?;
            }
        }

        let lcd_status = self.get_lcd_status()?;

        let lyc_compare = self.read_8(LY_REGISTER)? == self.read_8(LY_COMPARE)?;
        let mode: u8 = match &self.gpu {
            Gpu::Stopped => 2,
            Gpu::Mode2 { .. } => 2,
            Gpu::Mode3 { .. } => 3,
            Gpu::Mode0 { .. } => 0,
            Gpu::Mode1 { .. } => 1,
        };

        self.write_8(
            LCD_STATUS,
            (lcd_status.status & !7) | (if lyc_compare { 1 } else { 0 } << 2) | mode,
        )?;

        if mode != old_mode
            && (mode == 0 && lcd_status.mode_0_interrupt
                || mode == 1 && lcd_status.mode_1_interrupt
                || mode == 2 && lcd_status.mode_2_interrupt
                || lyc_compare && lcd_status.lyc_interrupt)
        {
            self.trigger_interrupt(LCD)?;
        }

        if mode != old_mode && mode == 1 {
            self.trigger_interrupt(VBlank)?;
        }

        Ok(pixels)
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

        Ok(TimerInfo {
            enable,
            cycles,
            modulo,
        })
    }

    fn get_lcd_status(&mut self) -> Result<LcdStatus> {
        let status = self.read_8(LCD_STATUS)?;
        let lyc_interrupt = get_bit(status, 6) == 1;
        let mode_2_interrupt = get_bit(status, 5) == 1;
        let mode_1_interrupt = get_bit(status, 4) == 1;
        let mode_0_interrupt = get_bit(status, 3) == 1;
        Ok(LcdStatus {
            status,
            lyc_interrupt,
            mode_0_interrupt,
            mode_1_interrupt,
            mode_2_interrupt,
        })
    }
}
