#[cfg(test)]
mod tests {
    use crate::gb::GameBoy;
    use std::path::Path;

    #[test]
    fn test_blarrg_01() -> anyhow::Result<()> {
        run_rom(Path::new("roms/01-special.gb"), "1")
    }

    #[test]
    fn test_blarrg_02() -> anyhow::Result<()> {
        run_rom(Path::new("roms/02-interrupts.gb"), "2")
    }

    #[test]
    fn test_blarrg_03() -> anyhow::Result<()> {
        run_rom(Path::new("roms/03-op sp,hl.gb"), "3")
    }

    #[test]
    fn test_blarrg_04() -> anyhow::Result<()> {
        run_rom(Path::new("roms/04-op r,imm.gb"), "4")
    }

    #[test]
    fn test_blarrg_05() -> anyhow::Result<()> {
        run_rom(Path::new("roms/05-op rp.gb"), "5")
    }

    #[test]
    fn test_blarrg_06() -> anyhow::Result<()> {
        run_rom(Path::new("roms/06-ld r,r.gb"), "6")
    }

    #[test]
    fn test_blarrg_07() -> anyhow::Result<()> {
        run_rom(Path::new("roms/07-jr,jp,call,ret,rst.gb"), "7")
    }

    #[test]
    fn test_blarrg_08() -> anyhow::Result<()> {
        run_rom(Path::new("roms/08-misc instrs.gb"), "8")
    }

    #[test]
    fn test_blarrg_09() -> anyhow::Result<()> {
        run_rom(Path::new("roms/09-op r,r.gb"), "9")
    }

    #[test]
    fn test_blarrg_10() -> anyhow::Result<()> {
        run_rom(Path::new("roms/10-bit ops.gb"), "10")
    }

    #[test]
    fn test_blarrg_11() -> anyhow::Result<()> {
        run_rom(Path::new("roms/11-op a,(hl).gb"), "11")
    }

    #[test]
    fn test_blarrg_instr_timing() -> anyhow::Result<()> {
        run_rom(Path::new("roms/instr_timing.gb"), "instr_timing")
    }

    #[test]
    fn test_blarrg_interrupt_time() -> anyhow::Result<()> {
        run_rom(Path::new("roms/interrupt_time.gb"), "interrupt_time")
    }

    #[test]
    fn test_blarrg_mem_timing() -> anyhow::Result<()> {
        run_rom(Path::new("roms/mem_timing.gb"), "mem_timing")
    }

    fn run_rom(path: &Path, _id: &str) -> anyhow::Result<()> {
        {
            let mut gb = GameBoy::new(path)?;
            let mut serial = String::new();

            while !serial.contains("Passed") {
                let (maybe_serial_log, _) = gb.step()?;
                if let Some(serial_log) = maybe_serial_log {
                    print!("{}", serial_log);
                    serial.push_str(&serial_log);
                }
            }
        }

        Ok(())
    }
}
