mod timer_info;

use crate::gb::bits::set_bit;
use crate::gb::clock::timer_info::TimerInfo;
use crate::gb::cpu::Interrupts;
use crate::gb::gpu::Gpu;
use crate::gb::memory::map::{DIV, IF, TIMA};
use crate::gb::memory::Memory;
use crate::gb::Pixel;
use anyhow::Result;

const DIV_CYCLES: u8 = 64;

pub struct Clock {
    cycles: usize,
}

impl Clock {
    pub fn new() -> Clock {
        Clock { cycles: 0 }
    }

    pub fn tick(
        &mut self,
        gpu: &mut Gpu,
        memory: &mut Memory,
        cycles: usize,
    ) -> Result<Vec<Pixel>> {
        let timer_info = TimerInfo::from_memory(memory)?;

        let mut pixels: Vec<Pixel> = Vec::with_capacity(4 * cycles);

        for _ in 0..cycles {
            let (mut new_pixels, interrupts) = gpu.tick_gpu(memory)?;

            pixels.append(&mut new_pixels);
            for interrupt in interrupts {
                trigger_interrupt(memory, interrupt)?;
            }

            self.cycles = self.cycles.wrapping_add(1);

            if self.cycles % usize::from(DIV_CYCLES) == 0 {
                tick_div(memory)?;
            }

            if timer_info.enable
                && self.cycles % timer_info.cycles == 0
                && tick_timer(memory, timer_info.modulo)?
            {
                trigger_interrupt(memory, Interrupts::Timer)?;
            }
        }

        Ok(pixels)
    }
}

pub fn tick_div(memory: &mut Memory) -> Result<()> {
    let div = memory.read(DIV)?;
    memory.write(DIV, div.wrapping_add(1))
}

pub fn tick_timer(memory: &mut Memory, modulo: u8) -> Result<bool> {
    let incremented = memory.read(TIMA)?.wrapping_add(1);
    Ok(match incremented {
        0 => {
            memory.write(TIMA, modulo)?;
            true
        }
        _ => {
            memory.write(TIMA, incremented)?;
            false
        }
    })
}

fn trigger_interrupt(memory: &mut Memory, interrupt: Interrupts) -> anyhow::Result<()> {
    set_interrupt_bit(
        memory,
        match interrupt {
            Interrupts::VBlank => 0,
            Interrupts::Lcd => 1,
            Interrupts::Timer => 2,
            Interrupts::_Serial => 3,
            Interrupts::_Joypad => 4,
        },
    )
}

fn set_interrupt_bit(memory: &mut Memory, bit: u8) -> anyhow::Result<()> {
    let if_reg = memory.read(IF)?;
    memory.write(IF, set_bit(if_reg, bit))
}
