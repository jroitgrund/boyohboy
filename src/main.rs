mod gb;
mod test;

use crate::gb::GameBoy;

use anyhow::Result;
use std::path::Path;

fn main() -> Result<()> {
    let mut gb = GameBoy::new(Path::new(""))?;
    loop {
        gb.step()?;
    }
}
