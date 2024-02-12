use crate::gb::bits::test_bit;
use crate::gb::gpu::{get_lcdinfo, LCDInfo, OBJ_ATTRIBUTES_SIZE, OBJ_X_OFFSET, OBJ_Y_OFFSET};
use crate::gb::memory::map::OBJ_ATTRIBUTES_BASE;
use crate::gb::memory::Memory;
use itertools::Itertools;
use std::cmp::Ordering;

#[derive(PartialEq, Eq)]
pub struct ObjData {
    pub index: u8,
    pub y: i32,
    pub x: i32,
    pub tile_index: u8,
    pub priority: bool,
    pub y_flip: bool,
    pub x_flip: bool,
    pub use_palette_1: bool,
}

impl ObjData {
    fn compare_tuple(&self) -> (i32, u8) {
        (self.x, self.index)
    }

    pub fn from_memory(memory: &mut Memory, scanline: i32) -> anyhow::Result<Vec<ObjData>> {
        let lcd_info = get_lcdinfo(memory)?;
        Ok((0..40)
            .map(|index: u8| {
                let attributes_base = OBJ_ATTRIBUTES_BASE + u16::from(index) * OBJ_ATTRIBUTES_SIZE;
                let y = i32::from(memory.read(attributes_base)?);
                let x = i32::from(memory.read(attributes_base + 1)?);
                let tile_index = memory.read(attributes_base + 2)?;
                let flags = memory.read(attributes_base + 3)?;
                let priority = test_bit(flags, 7);
                let y_flip = test_bit(flags, 6);
                let x_flip = test_bit(flags, 5);
                let use_palette_1 = test_bit(flags, 4);
                Ok(ObjData {
                    index,
                    y,
                    x,
                    tile_index,
                    priority,
                    y_flip,
                    x_flip,
                    use_palette_1,
                })
            })
            .filter_ok(|obj| obj.covers_y(scanline, &lcd_info))
            .take(10)
            .collect::<anyhow::Result<Vec<ObjData>>>()?
            .into_iter()
            .sorted()
            .collect())
    }
}

impl PartialOrd<ObjData> for ObjData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ObjData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.compare_tuple().cmp(&other.compare_tuple())
    }
}

impl ObjData {
    pub fn covers_x(&self, x: i32) -> bool {
        self.x - OBJ_X_OFFSET <= x && self.x + 8 - OBJ_X_OFFSET > x
    }

    pub fn get_tile_x(&self, x: i32) -> u8 {
        let x = (x - (self.x - OBJ_X_OFFSET)) as u8;
        if self.x_flip {
            7 - x
        } else {
            x
        }
    }

    pub fn covers_y(&self, y: i32, lcd_info: &LCDInfo) -> bool {
        self.y - OBJ_Y_OFFSET <= y
            && self.y
                + (if lcd_info.should_use_16px_objects {
                    16
                } else {
                    8
                })
                - OBJ_Y_OFFSET
                > y
    }

    pub fn get_tile_y(&self, y: i32, lcd_info: &LCDInfo) -> u8 {
        let y = (y - (self.y - OBJ_Y_OFFSET) - if self.is_2nd_16_px_tile(y) { 8 } else { 0 }) as u8;
        if self.y_flip {
            (if lcd_info.should_use_16px_objects {
                15
            } else {
                7
            }) - y
        } else {
            y
        }
    }

    pub fn is_2nd_16_px_tile(&self, y: i32) -> bool {
        y - (self.y - OBJ_Y_OFFSET) > 8
    }
}
