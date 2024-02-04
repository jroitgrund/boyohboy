use crate::gb::GameBoy;
use crate::instructions::execute_next_instruction;
use anyhow::Result;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
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
    let serial_log_path = format!("logs/serial-{:02?}.txt", num);

    {
        let mut gb = GameBoy::new(path)?;
        let expected_log_lines =
            BufReader::new(File::open(format!("logs/expected-{:02?}.txt", num))?)
                .lines()
                .count();
        let mut log = BufWriter::new(File::create(format!("logs/{:02?}.txt", num))?);

        let mut serial = BufWriter::new(File::create(&serial_log_path)?);

        let mut lines = 0;
        while !gb.halted && lines < expected_log_lines {
            log.write_all(gb.log()?.as_bytes())?;
            if let Some(serial_log) = gb.serial()? {
                serial.write_all(serial_log.as_bytes())?;
            }
            execute_next_instruction(&mut gb)?;
            lines += 1
        }
    }

    let string = fs::read_to_string(serial_log_path)?;
    assert!(string.contains("Passed"));

    Ok(())
}

// fn run_asm(asm: &Path) -> Result<()> {
//     let tmp_dir = TempDir::new("")?;
//     let object = tmp_dir.path().join("object.o");
//     let rom = tmp_dir.path().join("rom.gb");
//
//     assert!(Command::new("rgbasm")
//         .arg("-L")
//         .arg("-o")
//         .arg(object.to_str().unwrap())
//         .arg(asm.to_str().unwrap())
//         .output()
//         .unwrap()
//         .status
//         .success());
//
//     assert!(Command::new("rgblink")
//         .arg("-o")
//         .arg(rom.to_str().unwrap())
//         .arg(object.to_str().unwrap())
//         .output()
//         .unwrap()
//         .status
//         .success());
//
//     assert!(Command::new("rgbfix")
//         .arg("-v")
//         .arg("-p")
//         .arg("0xFF")
//         .arg(rom.to_str().unwrap())
//         .output()
//         .unwrap()
//         .status
//         .success());
//
//     run_rom(&rom)
// }
