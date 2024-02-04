mod gb;
mod instructions;

use crate::gb::GameBoy;
use crate::instructions::execute_next_instruction;
use anyhow::Result;

fn main() -> Result<()> {
    let mut gb = GameBoy::new();

    execute_next_instruction(&mut gb)?;

    Ok(())
}
