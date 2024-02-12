mod lcd_status;
mod obj_data;

use crate::gb::bits::{get_bit, get_bits, test_bit};
use crate::gb::cpu::Interrupts;
use crate::gb::gpu::lcd_status::LcdStatus;
use crate::gb::gpu::obj_data::ObjData;
use crate::gb::gpu::GpuState::{Mode0, Mode1, Mode2, Mode3, Stopped};
use crate::gb::memory::map::{
    BGP, LCDC, LY, LYC, OBJ_TILES_BASE, OBP0, OBP1, SCX, SCY, STAT, WX, WY,
};
use crate::gb::memory::Memory;
use crate::gb::Color::{Black, DarkGray, LightGray, White};
use crate::gb::{Color, Pixel};
use anyhow::{anyhow, Result};
use itertools::Itertools;
use log::info;
use std::collections::HashSet;
use std::mem;

const OBJ_ATTRIBUTES_SIZE: u16 = 4;

const LINE_BYTES: u8 = 2;
const TILE_BYTES: u16 = 16;

const OBJ_X_OFFSET: i32 = 8;
const OBJ_Y_OFFSET: i32 = 16;
const WINDOW_X_OFFSET: i32 = 7;
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

#[derive(Default)]
enum GpuState {
    #[default]
    Stopped,
    Mode2 {
        scanline: i32,
        dots_left: i32,
        window_line: i32,
    },
    Mode3 {
        scanline: i32,
        window_line: i32,
        has_window: bool,
        object_data: Vec<ObjData>,
        pixel: i32,
        dots: i32,
    },
    Mode0 {
        scanline: i32,
        window_line: i32,
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
                window_line: 0,
                dots_left: MODE2_DOTS,
            },
        }
    }

    pub fn mode(&self) -> Option<u8> {
        match &self.state {
            Stopped => None,
            Mode2 { .. } => Some(2),
            Mode3 { .. } => Some(3),
            Mode0 { .. } => Some(0),
            Mode1 { .. } => Some(1),
        }
    }

    pub fn scanline(&self) -> Option<i32> {
        match &self.state {
            Stopped => None,
            Mode2 { scanline, .. } => Some(*scanline),
            Mode3 { scanline, .. } => Some(*scanline),
            Mode0 { scanline, .. } => Some(*scanline),
            Mode1 { dots_left, .. } => Some(((4560 - dots_left) / 456) + 144),
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
        self.is_window_enabled
            && self.are_bg_and_window_enabled
            && self.window_x - WINDOW_X_OFFSET <= x
            && self.window_x + WINDOW_AND_BG_SIZE - WINDOW_X_OFFSET > x
            && self.window_y <= y
            && self.window_y + WINDOW_AND_BG_SIZE > y
    }
}

impl Gpu {
    pub fn tick_gpu(&mut self, memory: &mut Memory) -> Result<(Vec<Pixel>, Vec<Interrupts>)> {
        let lcd_info = get_lcdinfo(memory)?;

        if !lcd_info.is_ppu_enabled {
            self.state = Stopped;
        }

        let pixels = (0..4)
            .map(|_| self.state.tick_dot(memory, &lcd_info))
            .filter_map_ok(|f| f)
            .collect::<Result<Vec<Pixel>>>()?;

        let maybe_mode = self.mode();
        let maybe_scanline = self.scanline();
        let current_lyc = (memory.read(STAT)? >> 2) & 1 == 1;

        if maybe_scanline.is_some() {
            memory.write(LY, maybe_scanline.unwrap() as u8)?;
        }

        let stat = memory.read(STAT)?;
        let current_mode = stat & 3;
        let lyc = memory.read(LYC)?;
        memory.write(
            STAT,
            (stat & !7)
                | (if maybe_scanline.is_some() && maybe_scanline.unwrap() as u8 == lyc {
                    1
                } else {
                    0
                } << 2)
                | maybe_mode.unwrap_or(current_mode),
        )?;

        let lcd_status = LcdStatus::from_memory(memory)?;
        let mut interrupts = HashSet::with_capacity(2);

        if let Some(mode) = maybe_mode.filter(|m| *m != current_mode) {
            if mode == 0 && lcd_status.mode_0_interrupt
                || mode == 1 && lcd_status.mode_1_interrupt
                || mode == 2 && lcd_status.mode_2_interrupt
            {
                interrupts.insert(Interrupts::Lcd);
            }

            if mode == 1 {
                interrupts.insert(Interrupts::VBlank);
            }
        }

        if ((memory.read(STAT)? >> 2) & 1 == 1) && !current_lyc && lcd_status.lyc_interrupt {
            interrupts.insert(Interrupts::Lcd);
        }

        Ok((pixels, interrupts.into_iter().collect()))
    }
}

impl GpuState {
    fn tick_dot(&mut self, memory: &mut Memory, lcd_info: &LCDInfo) -> Result<Option<Pixel>> {
        let state: GpuState = mem::take(self);
        let (new_state, pixels) = match state {
            Stopped => (self.tick_stopped(lcd_info), None),
            Mode2 {
                scanline,
                window_line,
                dots_left,
            } => (
                self.tick_mode_2(memory, dots_left, scanline, window_line)?,
                None,
            ),
            Mode3 {
                scanline,
                window_line,
                has_window,
                object_data,
                pixel,
                dots,
            } => self.tick_mode3(
                memory,
                &lcd_info,
                dots,
                scanline,
                window_line,
                has_window,
                pixel,
                object_data,
            )?,
            Mode0 {
                scanline,
                window_line,
                dots_left,
            } => (self.tick_mode0(scanline, window_line, dots_left)?, None),
            Mode1 { dots_left } => (self.tick_mode1(dots_left)?, None),
        };

        *self = new_state;

        Ok(pixels)
    }

    fn tick_mode3(
        &mut self,
        memory: &mut Memory,
        lcd_info: &LCDInfo,
        dots: i32,
        scanline: i32,
        window_line: i32,
        has_window: bool,
        pixel: i32,
        object_data: Vec<ObjData>,
    ) -> Result<(GpuState, Option<Pixel>)> {
        let dots = dots + 1;
        if pixel < PIXELS_PER_LINE {
            let (color, is_window_pixel) =
                self.get_pixel_color(memory, lcd_info, scanline, window_line, pixel, &object_data)?;

            Ok((
                (Mode3 {
                    scanline,
                    window_line,
                    has_window: has_window || is_window_pixel,
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
                    window_line: if has_window {
                        window_line + 1
                    } else {
                        window_line
                    },
                    dots_left: 376 - dots - 1,
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
        window_line: i32,
    ) -> Result<GpuState> {
        let dots_left = dots_left - 1;
        Ok(if dots_left == 0 {
            Mode3 {
                scanline,
                has_window: false,
                window_line,
                object_data: ObjData::from_memory(memory, scanline)?,
                pixel: 0,
                dots: 0,
            }
        } else {
            Mode2 {
                scanline,
                window_line,
                dots_left,
            }
        })
    }

    fn tick_mode0(&mut self, scanline: i32, window_line: i32, dots_left: i32) -> Result<GpuState> {
        let dots_left = dots_left - 1;

        Ok(if dots_left == 0 {
            let scanline = scanline + 1;
            if scanline == SCANLINES {
                Mode1 {
                    dots_left: MODE_1_DOTS,
                }
            } else {
                Mode2 {
                    scanline,
                    window_line,
                    dots_left: MODE2_DOTS,
                }
            }
        } else {
            Mode0 {
                scanline,
                window_line,
                dots_left,
            }
        })
    }

    fn tick_mode1(&mut self, dots_left: i32) -> Result<GpuState> {
        let dots_left = dots_left - 1;

        Ok(if dots_left == 0 {
            Mode2 {
                scanline: 0,
                window_line: 0,
                dots_left: MODE2_DOTS,
            }
        } else {
            Mode1 { dots_left }
        })
    }

    fn tick_stopped(&mut self, lcd_info: &LCDInfo) -> GpuState {
        if lcd_info.is_ppu_enabled {
            Mode2 {
                scanline: 0,
                window_line: 0,
                dots_left: MODE2_DOTS,
            }
        } else {
            Stopped
        }
    }

    fn get_pixel_color(
        &mut self,
        memory: &mut Memory,
        lcd_info: &LCDInfo,
        scanline: i32,
        window_line: i32,
        pixel: i32,
        object_data: &[ObjData],
    ) -> Result<(Color, bool)> {
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

        if lcd_info.are_objects_enabled && !object_data.is_empty() {
            info!("{}", object_data.len());
        }

        let (mb_bg_color_id, is_window) =
            match (&lcd_info, lcd_info.is_covered_by_window(pixel, scanline)) {
                (
                    LCDInfo {
                        are_bg_and_window_enabled: false,
                        ..
                    },
                    _,
                ) => (None, false),
                (
                    LCDInfo {
                        is_window_enabled: true,
                        ..
                    },
                    true,
                ) => (
                    Some(self.get_window_color_id(memory, lcd_info, pixel, window_line)?),
                    true,
                ),
                _ => (
                    Some(self.get_bg_color_id(memory, lcd_info, pixel, scanline)?),
                    false,
                ),
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
                        priority,
                        use_palette_1,
                        ..
                    },
                    obj_color_id,
                )),
                Some(bg_color_id),
            ) => {
                if *priority && bg_color_id != 0 {
                    self.get_bg_color(memory, bg_color_id)?
                } else {
                    self.get_obj_color(memory, obj_color_id, *use_palette_1)?
                }
            }
            (Some((ObjData { use_palette_1, .. }, obj_color_id)), None) => {
                self.get_obj_color(memory, obj_color_id, *use_palette_1)?
            }
        };

        Ok((color, is_window))
    }

    fn get_object_color_id(
        &mut self,
        memory: &mut Memory,
        obj: &ObjData,
        lcd_info: &LCDInfo,
        x: i32,
        y: i32,
    ) -> Result<u8> {
        let tile_index = if lcd_info.should_use_16px_objects {
            (obj.tile_index & 0xFE) + if obj.is_2nd_16_px_tile(y) { 1 } else { 0 }
        } else {
            obj.tile_index
        };
        let tile_addr = OBJ_TILES_BASE + u16::from(tile_index) * TILE_BYTES;

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
        let window_x = x - lcd_info.window_x + 7;
        let window_y = y;

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
        let palette = memory.read(BGP)?;
        get_color(palette, color_id)
    }

    fn get_obj_color(
        &mut self,
        memory: &mut Memory,
        color_id: u8,
        obp1_palette: bool,
    ) -> Result<Color> {
        let palette = memory.read(if obp1_palette { OBP1 } else { OBP0 })?;
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
        is_ppu_enabled: test_bit(memory.read(LCDC)?, 7),
        window_tile_map_base: if test_bit(memory.read(LCDC)?, 6) {
            0x9C00
        } else {
            0x9800
        },
        is_window_enabled: test_bit(memory.read(LCDC)?, 5),
        should_use_8000_addressing_mode: test_bit(memory.read(LCDC)?, 4),
        bg_tile_map_base: if test_bit(memory.read(LCDC)?, 3) {
            0x9C00
        } else {
            0x9800
        },
        should_use_16px_objects: test_bit(memory.read(LCDC)?, 2),
        are_objects_enabled: test_bit(memory.read(LCDC)?, 1),
        are_bg_and_window_enabled: test_bit(memory.read(LCDC)?, 0),
        bg_x: i32::from(memory.read(SCX)?),
        bg_y: i32::from(memory.read(SCY)?),
        window_x: i32::from(memory.read(WX)?),
        window_y: i32::from(memory.read(WY)?),
    })
}
