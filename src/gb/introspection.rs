use crate::gb::GameBoyImpl;

impl GameBoyImpl {
    pub fn serial(&mut self) -> anyhow::Result<Option<String>> {
        if self.read_8(0xff02)? == 0x81 {
            let serial = self.read_8(0xff01)?;
            let log = format!("{}", serial as char);
            self.write_8(0xff02, 0x0)?;
            Ok(Some(log))
        } else {
            Ok(None)
        }
    }
    pub fn log(&mut self) -> anyhow::Result<String> {
        let pc = self.read_8(self.pc)?;
        let pc_1 = self.read_8(self.pc + 1)?;
        let pc_2 = self.read_8(self.pc + 2)?;
        let pc_3 = self.read_8(self.pc + 3)?;
        Ok(format!(
            "A: {:02X?} F: {:02X?} B: {:02X?} C: {:02X?} D: {:02X?} E: {:02X?} H: {:02X?} L: {:02X?} SP: {:04X?} PC: 00:{:04X?} ({:02X?} {:02X?} {:02X?} {:02X?})\n",
            self.a, self.f, self.b, self.c, self.d, self.e, self.h, self.l, self.sp, self.pc, pc, pc_1, pc_2, pc_3
        ))
    }
}
