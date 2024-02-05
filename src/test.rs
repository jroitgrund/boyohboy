use crate::gb::GameBoy;
use anyhow::Result;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;

#[cfg(test)]
mod tests {
    use crate::test::run_rom;
    use std::path::Path;

    #[test]
    fn test_blarrg_01() -> anyhow::Result<()> {
        run_rom(Path::new("roms/01-special.gb"), 1)
    }

    #[test]
    fn test_blarrg_02() -> anyhow::Result<()> {
        run_rom(Path::new("roms/02-interrupts.gb"), 2)
    }

    #[test]
    fn test_blarrg_03() -> anyhow::Result<()> {
        run_rom(Path::new("roms/03-op sp,hl.gb"), 3)
    }

    #[test]
    fn test_blarrg_04() -> anyhow::Result<()> {
        run_rom(Path::new("roms/04-op r,imm.gb"), 4)
    }

    #[test]
    fn test_blarrg_05() -> anyhow::Result<()> {
        run_rom(Path::new("roms/05-op rp.gb"), 5)
    }

    #[test]
    fn test_blarrg_06() -> anyhow::Result<()> {
        run_rom(Path::new("roms/06-ld r,r.gb"), 6)
    }

    #[test]
    fn test_blarrg_07() -> anyhow::Result<()> {
        run_rom(Path::new("roms/07-jr,jp,call,ret,rst.gb"), 7)
    }

    #[test]
    fn test_blarrg_08() -> anyhow::Result<()> {
        run_rom(Path::new("roms/08-misc instrs.gb"), 8)
    }

    #[test]
    fn test_blarrg_09() -> anyhow::Result<()> {
        run_rom(Path::new("roms/09-op r,r.gb"), 9)
    }

    #[test]
    fn test_blarrg_10() -> anyhow::Result<()> {
        run_rom(Path::new("roms/10-bit ops.gb"), 10)
    }

    #[test]
    fn test_blarrg_11() -> anyhow::Result<()> {
        run_rom(Path::new("roms/11-op a,(hl).gb"), 11)
    }
}

fn run_rom(path: &Path, num: usize) -> Result<()> {
    {
        let mut gb = GameBoy::new(path)?;
        let mut log = BufWriter::new(File::create(format!("logs/{:02?}.txt", num))?);
        let mut serial = String::new();

        while !serial.contains("Passed") {
            let (maybe_serial_log, gb_log) = gb.step()?;
            log.write_all(gb_log.as_bytes())?;
            if let Some(serial_log) = maybe_serial_log {
                print!("{}", serial_log);
                serial.push_str(&serial_log);
            }
        }
    }

    Ok(())
}
