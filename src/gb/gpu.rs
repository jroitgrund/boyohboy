mod obj_data;

use crate::gb::bits::{get_bit, get_bits, test_bit};
use crate::gb::gpu::obj_data::ObjData;
use crate::gb::gpu::GpuState::{Mode0, Mode1, Mode2, Mode3, Stopped};
use crate::gb::memory::Memory;
use crate::gb::Color::{Black, DarkGray, LightGray, White};
use crate::gb::{Color, Pixel};
use anyhow::{anyhow, Result};
use itertools::Itertools;
use std::mem;

const OBJ_TILES_BASE: u16 = 0x8000;
pub const OBJ_ATTRIBUTES_BASE: u16 = 0xFE00;
const OBJ_ATTRIBUTES_SIZE: u16 = 4;
const LCD_CONTROL: u16 = 0xFF40;
const BG_SCY: u16 = 0xFF42;
const BG_SCX: u16 = 0xFF43;
const WINDOW_SCY: u16 = 0xFF4A;
const WINDOW_SCX: u16 = 0xFF4B;
const BG_PALETTE: u16 = 0xFF47;
const OBJ_PALETTE_0: u16 = 0xFF48;
const OBJ_PALETTE_1: u16 = 0xFF49;
pub const LY_REGISTER: u16 = 0xFF44;

const LINE_BYTES: u8 = 2;
const TILE_BYTES: u16 = 16;

const OBJ_X_OFFSET: i32 = 8;
const OBJ_Y_OFFSET: i32 = 16;
const WINDOW_X_OFFSET: i32 = 8;
const WINDOW_AND_BG_SIZE: i32 = 256;
const TILE_SIZE_PX: i32 = 8;
const TILES_PER_LINE: i32 = 32;
const COLOR_ID_TRANSPARENT: u8 = 0;

const MODE2_DOTS: i32 = 80;
const MODE_1_DOTS: i32 = 4560;
const PIXELS_PER_LINE: i32 = 160;
const SCANLINES: i32 = 144;

pub struct Gpu {
    state: GpuState,
}

impl Default for GpuState {
    fn default() -> Self {
        Stopped
    }
}

enum GpuState {
    Stopped,
    Mode2 {
        scanline: i32,
        dots_left: i32,
    },
    Mode3 {
        scanline: i32,
        object_data: Vec<ObjData>,
        pixel: i32,
        dots: i32,
    },
    Mode0 {
        scanline: i32,
        dots_left: i32,
    },
    Mode1 {
        dots_left: i32,
    },
}

impl Gpu {
    pub fn new() -> Gpu {
        Gpu {
            state: Mode2 {
                scanline: 0,
                dots_left: MODE2_DOTS,
            },
        }
    }

    pub fn mode(&self) -> u8 {
        match &self.state {
            Stopped => 2,
            Mode2 { .. } => 2,
            Mode3 { .. } => 3,
            Mode0 { .. } => 0,
            Mode1 { .. } => 1,
        }
    }
}

struct LCDInfo {
    is_ppu_enabled: bool,
    window_tile_map_base: u16,
    is_window_enabled: bool,
    should_use_8000_addressing_mode: bool,
    bg_tile_map_base: u16,
    should_use_16px_objects: bool,
    are_objects_enabled: bool,
    are_bg_and_window_enabled: bool,
    bg_x: i32,
    bg_y: i32,
    window_x: i32,
    window_y: i32,
}

impl LCDInfo {
    fn is_covered_by_window(&self, x: i32, y: i32) -> bool {
        return self.is_window_enabled
            && self.are_bg_and_window_enabled
            && self.window_x - WINDOW_X_OFFSET <= x
            && self.window_x + WINDOW_AND_BG_SIZE - WINDOW_X_OFFSET > x
            && self.window_y <= y
            && self.window_y + WINDOW_AND_BG_SIZE > y;
    }
}

impl Gpu {
    pub fn tick_gpu(&mut self, memory: &mut Memory) -> Result<Vec<Pixel>> {
        let lcd_info = get_lcdinfo(memory)?;

        if !lcd_info.is_ppu_enabled {
            self.state = Stopped;
        }

        Ok((0..4)
            .map(|_| self.state.tick_dot(memory, &lcd_info))
            .filter_map_ok(|f| f)
            .collect::<Result<Vec<Pixel>>>()?)
    }
}

impl GpuState {
    fn tick_dot(&mut self, memory: &mut Memory, lcd_info: &LCDInfo) -> Result<Option<Pixel>> {
        let state: GpuState = mem::take(self);
        let (new_state, pixels) = match state {
            Stopped => (self.tick_stopped(lcd_info), None),
            Mode2 {
                scanline,
                dots_left,
            } => (self.tick_mode_2(memory, dots_left, scanline)?, None),
            Mode3 {
                scanline,
                object_data,
                pixel,
                dots,
            } => {
                let dots = dots + 1;

                self.tick_mode3(memory, &lcd_info, dots, scanline, pixel, object_data)?
            }
            Mode0 {
                scanline,
                dots_left,
            } => self.tick_mode0(memory, scanline, dots_left)?,
            Mode1 { dots_left } => self.tick_mode1(memory, dots_left)?,
        };

        *self = new_state;

        Ok(pixels)
    }

    fn tick_mode3(
        &mut self,
        memory: &mut Memory,
        lcd_info: &&LCDInfo,
        dots: i32,
        scanline: i32,
        pixel: i32,
        object_data: Vec<ObjData>,
    ) -> Result<(GpuState, Option<Pixel>)> {
        let dots = dots + 1;
        if pixel < PIXELS_PER_LINE {
            let mb_obj_and_color_id = invert(
                object_data
                    .iter()
                    .find(|obj_data| obj_data.covers_x(pixel))
                    .filter(|_| lcd_info.are_objects_enabled)
                    .map(|obj| {
                        self.get_object_color_id(memory, obj, lcd_info, pixel, scanline)
                            .map(|color_id| (obj, color_id))
                    }),
            )?;
            let mb_bg_color_id = match (&lcd_info, lcd_info.is_covered_by_window(pixel, scanline)) {
                (
                    LCDInfo {
                        are_bg_and_window_enabled: false,
                        ..
                    },
                    _,
                ) => None,
                (
                    LCDInfo {
                        is_window_enabled: true,
                        ..
                    },
                    true,
                ) => Some(self.get_window_color_id(memory, lcd_info, pixel, scanline)?),
                _ => Some(self.get_bg_color_id(memory, lcd_info, pixel, scanline)?),
            };

            let color = match (mb_obj_and_color_id, mb_bg_color_id) {
                (None, None) => White,
                (None, Some(bg_color_id)) => self.get_bg_color(memory, bg_color_id)?,
                (Some((_, COLOR_ID_TRANSPARENT)), Some(bg_color_id)) => {
                    self.get_bg_color(memory, bg_color_id)?
                }
                (
                    Some((
                        ObjData {
                            priority: true,
                            use_palette_1: obp1_palette,
                            ..
                        },
                        obj_color_id,
                    )),
                    Some(bg_color_id),
                ) => {
                    if bg_color_id != 0 {
                        self.get_bg_color(memory, bg_color_id)?
                    } else {
                        self.get_obj_color(memory, obj_color_id, *obp1_palette)?
                    }
                }
                (
                    Some((
                        ObjData {
                            use_palette_1: obp1_palette,
                            ..
                        },
                        obj_color_id,
                    )),
                    _,
                ) => self.get_obj_color(memory, obj_color_id, *obp1_palette)?,
            };

            Ok((
                (Mode3 {
                    scanline,
                    object_data,
                    pixel: pixel + 1,
                    dots,
                }),
                Some(Pixel {
                    x: pixel as u8,
                    y: scanline as u8,
                    color,
                }),
            ))
        } else {
            Ok((
                Mode0 {
                    scanline,
                    dots_left: 376 - dots,
                },
                None,
            ))
        }
    }

    fn tick_mode_2(
        &mut self,
        memory: &mut Memory,
        dots_left: i32,
        scanline: i32,
    ) -> Result<GpuState> {
        let dots_left = dots_left - 1;
        Ok(if dots_left == 0 {
            Mode3 {
                scanline,
                object_data: obj_data::get_objects(memory, scanline)?,
                pixel: 0,
                dots: 0,
            }
        } else {
            Mode2 {
                scanline,
                dots_left,
            }
        })
    }

    fn tick_mode0(
        &mut self,
        memory: &mut Memory,
        scanline: i32,
        dots_left: i32,
    ) -> Result<(GpuState, Option<Pixel>)> {
        let dots_left = dots_left - 1;

        Ok((
            if dots_left == 0 {
                let scanline = scanline + 1;
                memory.write(LY_REGISTER, scanline as u8)?;
                if scanline == SCANLINES {
                    Mode1 {
                        dots_left: MODE_1_DOTS,
                    }
                } else {
                    Mode2 {
                        scanline,
                        dots_left: MODE2_DOTS,
                    }
                }
            } else {
                Mode0 {
                    scanline,
                    dots_left,
                }
            },
            None,
        ))
    }

    fn tick_mode1(
        &mut self,
        memory: &mut Memory,
        dots_left: i32,
    ) -> Result<(GpuState, Option<Pixel>)> {
        let dots_left = dots_left - 1;
        let scanline = (((4560 - dots_left) / 456) as u8) + 144;
        memory.write(LY_REGISTER, scanline)?;

        Ok((
            if dots_left == 0 {
                memory.write(LY_REGISTER, 0)?;
                Mode2 {
                    scanline: 0,
                    dots_left: MODE2_DOTS,
                }
            } else {
                Mode1 { dots_left }
            },
            None,
        ))
    }

    fn tick_stopped(&mut self, lcd_info: &LCDInfo) -> GpuState {
        if lcd_info.is_ppu_enabled {
            Mode2 {
                scanline: 0,
                dots_left: MODE2_DOTS,
            }
        } else {
            Stopped
        }
    }

    fn get_object_color_id(
        &mut self,
        memory: &mut Memory,
        obj: &ObjData,
        lcd_info: &LCDInfo,
        x: i32,
        y: i32,
    ) -> Result<u8> {
        let tile_addr = OBJ_TILES_BASE
            + u16::from(obj.tile_index & 0xFE)
                * TILE_BYTES
                * if lcd_info.should_use_16px_objects {
                    2
                } else {
                    1
                }
            + if obj.is_2nd_16_px_tile(i32::from(y)) {
                TILE_BYTES
            } else {
                0
            };

        self.get_color_id(
            memory,
            tile_addr,
            obj.get_tile_x(x),
            obj.get_tile_y(y, lcd_info),
        )
    }

    fn get_bg_color_id(
        &mut self,
        memory: &mut Memory,
        lcd_info: &LCDInfo,
        x: i32,
        y: i32,
    ) -> Result<u8> {
        let bg_x = (x + lcd_info.bg_x) % WINDOW_AND_BG_SIZE;
        let bg_y = (y + lcd_info.bg_y) % WINDOW_AND_BG_SIZE;

        self.get_bg_or_window_color_id(memory, lcd_info, bg_x, bg_y, lcd_info.bg_tile_map_base)
    }

    fn get_window_color_id(
        &mut self,
        memory: &mut Memory,
        lcd_info: &LCDInfo,
        x: i32,
        y: i32,
    ) -> Result<u8> {
        let window_x = x - lcd_info.window_x;
        let window_y = y - lcd_info.window_y;

        self.get_bg_or_window_color_id(
            memory,
            lcd_info,
            window_x,
            window_y,
            lcd_info.window_tile_map_base,
        )
    }

    fn get_bg_or_window_color_id(
        &mut self,
        memory: &mut Memory,
        lcd_info: &LCDInfo,
        x: i32,
        y: i32,
        tile_map_base: u16,
    ) -> Result<u8> {
        let tile_row = y / TILE_SIZE_PX;
        let tile_col = x / TILE_SIZE_PX;

        let tile_map_index = (tile_row * TILES_PER_LINE + tile_col) as u16;

        let tile_index = memory.read(tile_map_base + tile_map_index)?;
        let tile_addr = self.get_bg_or_window_tile_addr(tile_index, lcd_info)?;
        let tile_x = (x % TILE_SIZE_PX) as u8;
        let tile_y = (y % TILE_SIZE_PX) as u8;

        self.get_color_id(memory, tile_addr, tile_x, tile_y)
    }

    fn get_bg_or_window_tile_addr(&mut self, tile_index: u8, lcd_info: &LCDInfo) -> Result<u16> {
        if lcd_info.should_use_8000_addressing_mode {
            Ok(0x8000 + u16::from(tile_index) * TILE_BYTES)
        } else {
            Ok(0x9000u16.wrapping_add_signed((TILE_BYTES as i16) * i16::from(tile_index as i8)))
        }
    }

    fn get_color_id(
        &mut self,
        memory: &mut Memory,
        tile_addr: u16,
        tile_x: u8,
        tile_y: u8,
    ) -> Result<u8> {
        let line_offset = u16::from(tile_y * LINE_BYTES);

        let line_1 = memory.read(tile_addr + line_offset)?;
        let line_2 = memory.read(tile_addr + line_offset + 1)?;

        Ok(get_bit(line_1, 7 - tile_x) | (get_bit(line_2, 7 - tile_x) << 1))
    }

    fn get_bg_color(&mut self, memory: &mut Memory, color_id: u8) -> Result<Color> {
        let palette = memory.read(BG_PALETTE)?;
        get_color(palette, color_id)
    }

    fn get_obj_color(
        &mut self,
        memory: &mut Memory,
        color_id: u8,
        obp1_palette: bool,
    ) -> Result<Color> {
        let palette = memory.read(if obp1_palette {
            OBJ_PALETTE_1
        } else {
            OBJ_PALETTE_0
        })?;
        get_color(palette, color_id)
    }
}

fn get_color(palette: u8, color_id: u8) -> Result<Color> {
    let color = get_bits(palette, color_id * 2 + 1, color_id * 2);
    match color {
        0 => Ok(White),
        1 => Ok(LightGray),
        2 => Ok(DarkGray),
        3 => Ok(Black),
        _ => Err(anyhow!("Unknown color: {}", color)),
    }
}

fn invert<T, E>(x: Option<Result<T, E>>) -> Result<Option<T>, E> {
    x.map_or(Ok(None), |v| v.map(Some))
}

fn get_lcdinfo(memory: &mut Memory) -> Result<LCDInfo> {
    Ok(LCDInfo {
        is_ppu_enabled: test_bit(memory.read(LCD_CONTROL)?, 7),
        window_tile_map_base: if test_bit(memory.read(LCD_CONTROL)?, 6) {
            0x9C00
        } else {
            0x9800
        },
        is_window_enabled: test_bit(memory.read(LCD_CONTROL)?, 5),
        should_use_8000_addressing_mode: test_bit(memory.read(LCD_CONTROL)?, 4),
        bg_tile_map_base: if test_bit(memory.read(LCD_CONTROL)?, 3) {
            0x9C00
        } else {
            0x9800
        },
        should_use_16px_objects: test_bit(memory.read(LCD_CONTROL)?, 2),
        are_objects_enabled: test_bit(memory.read(LCD_CONTROL)?, 1),
        are_bg_and_window_enabled: test_bit(memory.read(LCD_CONTROL)?, 0),
        bg_x: i32::from(memory.read(BG_SCX)?),
        bg_y: i32::from(memory.read(BG_SCY)?),
        window_x: i32::from(memory.read(WINDOW_SCX)?),
        window_y: i32::from(memory.read(WINDOW_SCY)?),
    })
}
