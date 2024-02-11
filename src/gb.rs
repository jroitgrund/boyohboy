use crate::gb::AccessType::{Direct, Indirect};
use anyhow::Result;

use crate::gb::gpu::Gpu;

use crate::gb::memory::Memory;

use crate::gb::clock::Clock;
use crate::gb::cpu::{Cpu, InstructionResult};
use crate::gb::memory::map::{SB, SC};
use crate::gb::Halt::Running;
use std::ops;
use std::path::Path;
use Halt::{Bug, Halted};

mod bits;
mod clock;
mod cpu;
mod gpu;
mod memory;

const R16_HL: u8 = 2;

#[derive(Debug, PartialEq)]
pub enum Color {
    White,
    LightGray,
    DarkGray,
    Black,
}

#[derive(Debug)]
pub struct Pixel {
    pub x: u8,
    pub y: u8,
    pub color: Color,
}

enum AccessType {
    Direct,
    Indirect,
}

impl ops::Add<AccessType> for AccessType {
    type Output = AccessType;

    fn add(self, rhs: AccessType) -> AccessType {
        match (self, rhs) {
            (Direct, Direct) => Direct,
            _ => Indirect,
        }
    }
}

pub struct GameBoy {
    gb: GameBoyImpl,
}

impl GameBoy {
    pub fn new(cartridge: &Path) -> Result<GameBoy> {
        Ok(GameBoy {
            gb: GameBoyImpl::new(cartridge)?,
        })
    }

    pub fn step(&mut self) -> Result<(Option<String>, Vec<Pixel>)> {
        self.gb.step()
    }
}

struct GameBoyImpl {
    halt: Halt,
    cpu: Cpu,
    clock: Clock,
    gpu: Gpu,
    memory: Memory,
}

#[derive(PartialEq)]
enum Halt {
    Running,
    Halted,
    Bug,
}

impl GameBoyImpl {
    fn step(&mut self) -> Result<(Option<String>, Vec<Pixel>)> {
        let instruction_result = match self.halt {
            Running | Bug => {
                if self.halt == Running {
                    self.cpu.execute_next_instruction(&mut self.memory)?
                } else {
                    self.cpu
                        .execute_next_instruction_with_halt_bug(&mut self.memory)?
                }
            }
            Halted => InstructionResult {
                is_halt: false,
                cycles: 1,
            },
        };
        let mut pixels = Vec::with_capacity(usize::from(instruction_result.cycles) * 4 + 5 * 4);
        let mut instruction_pixels = self.clock.tick(
            &mut self.gpu,
            &mut self.memory,
            usize::from(instruction_result.cycles),
        )?;

        let interrupt_result = self.cpu.handle_interrupts(&mut self.memory)?;
        let mut interrupt_pixels = self.clock.tick(
            &mut self.gpu,
            &mut self.memory,
            usize::from(interrupt_result.cycles),
        )?;

        pixels.append(&mut instruction_pixels);
        pixels.append(&mut interrupt_pixels);

        self.halt = match (
            interrupt_result.interrupt_requested,
            instruction_result.is_halt,
            interrupt_result.interrupts_enabled,
        ) {
            (true, true, false) => Bug,
            (true, _, _) => Running,
            (false, true, _) => Halted,
            (false, false, _) => match self.halt {
                Halted => Halted,
                Bug => Running,
                Running => Running,
            },
        };
        Ok((self.serial()?, pixels))
    }

    pub fn new(cartridge: &Path) -> Result<GameBoyImpl> {
        let gb = GameBoyImpl {
            halt: Running,
            clock: Clock::new(),
            gpu: Gpu::new(),
            memory: Memory::new(cartridge)?,
            cpu: Cpu::new(),
        };

        Ok(gb)
    }

    fn serial(&mut self) -> Result<Option<String>> {
        if self.memory.read(SC)? >> 7 == 1 {
            let serial = self.memory.read(SB)?;
            let log = format!("{}", serial as char);
            self.memory.write(SC, 0)?;
            Ok(Some(log))
        } else {
            Ok(None)
        }
    }
}
