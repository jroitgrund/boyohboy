mod gb;
mod test;

use crate::gb::GameBoy;

use crate::gb::Color::{Black, DarkGray, LightGray, White};
use anyhow::Result;
use log::{info, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};
use log4rs::Config;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use std::path::Path;
use std::time::Instant;

fn main() -> Result<()> {
    let stdout = ConsoleAppender::builder().build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Warn))
        .unwrap();
    log4rs::init_config(config)?;

    let mut gb = GameBoy::new(Path::new("tetris.gb"))?;
    let sdl_context = sdl2::init().map_err(anyhow::Error::msg)?;
    let video_subsystem = sdl_context.video().map_err(anyhow::Error::msg)?;

    let window = video_subsystem
        .window("boyohboy", 160, 144)
        .position_centered()
        .opengl()
        .build()?;

    let mut canvas = window.into_canvas().build()?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, 160, 144)
        .map_err(anyhow::Error::msg)?;

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut durations: Vec<u128> = vec![];
    let mut event_pump = sdl_context.event_pump().map_err(anyhow::Error::msg)?;
    let mut serial = String::new();
    let mut colors: Vec<u8> = Vec::with_capacity(160 * 144);
    let mut frame_start: Option<Instant> = None;
    'running: loop {
        let (maybe_log, pixels) = gb.step()?;
        if let Some(log) = maybe_log {
            print!("{}", log);
            serial.push_str(&log);
        }
        for pixel in pixels.iter() {
            let intensity: u8 = match &pixel.color {
                White => 255,
                LightGray => 2 * (255 / 3),
                DarkGray => 255 / 3,
                Black => 0,
            };
            colors.push(intensity);

            if pixel.x == 159 && pixel.y == 143 {
                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit { .. }
                        | Event::KeyDown {
                            keycode: Some(Keycode::Escape),
                            ..
                        } => break 'running,
                        _ => {}
                    }
                }

                texture
                    .with_lock(None, |buffer, _| {
                        for (i, color) in colors.iter().enumerate() {
                            buffer[i * 3] = *color;
                            buffer[i * 3 + 1] = *color;
                            buffer[i * 3 + 2] = *color;
                        }
                    })
                    .map_err(anyhow::Error::msg)?;

                canvas
                    .copy(&texture, None, None)
                    .map_err(anyhow::Error::msg)?;
                canvas.present();
                colors.clear();

                if let Some(fs) = frame_start {
                    durations.push(fs.elapsed().as_nanos());
                    let fps: f64 = 1_000_000_000f64
                        / (durations.iter().sum::<u128>() / (durations.len() as u128)) as f64;
                    info!("{:?} fps", fps);
                }
                frame_start = Some(Instant::now());
            }
        }

        if serial.contains("Passed") {
            break;
        }
    }
    Ok(())
}
