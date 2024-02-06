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
use sdl2::pixels::Color;
use sdl2::rect::Point;
use std::path::Path;
use std::time::Instant;

fn main() -> Result<()> {
    let stdout = ConsoleAppender::builder().build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        .unwrap();
    log4rs::init_config(config)?;

    let mut gb = GameBoy::new(Path::new("roms/01-special.gb"))?;
    let sdl_context = sdl2::init().map_err(anyhow::Error::msg)?;
    let video_subsystem = sdl_context.video().map_err(anyhow::Error::msg)?;

    let window = video_subsystem
        .window("boyohboy", 160, 144)
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas().build()?;

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut durations: Vec<u128> = vec![];
    let mut event_pump = sdl_context.event_pump().map_err(anyhow::Error::msg)?;
    'running: loop {
        let now = Instant::now();
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

        let (maybeLog, _, pixels) = gb.step()?;
        if let Some(log) = maybeLog {
            print!("{}", log)
        }
        for pixel in pixels.iter() {
            canvas.set_draw_color(match &pixel.color {
                White => Color::RGB(255, 255, 255),
                LightGray => Color::RGB(
                    (2 * 255 / 3) as u8,
                    (2 * 255 / 3) as u8,
                    (2 * 255 / 3) as u8,
                ),
                DarkGray => Color::RGB(255 / 3, 255 / 3, 255 / 3),
                Black => Color::RGB(0, 0, 0),
            });
            canvas
                .draw_point(Point::new(pixel.x as i32, pixel.y as i32))
                .map_err(anyhow::Error::msg)?;
        }
        durations.push(now.elapsed().as_nanos());
        if pixels.iter().any(|px| px.x == 159 && px.y == 143) {
            canvas.present();
            let fps: f64 = 1f64
                / ((((durations.iter().sum::<u128>() / (durations.len() as u128)) / 4) * 70224)
                    as f64
                    / 1000000000f64);
            info!("{:?} fps", fps);
        }
    }
    Ok(())
}
