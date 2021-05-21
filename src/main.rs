use std::time::Duration;

use gameboy_rs::gameboy::GameBoy;
use sdl2::{pixels::PixelFormatEnum, render::{Canvas, Texture}, video::Window};

const SCALE: u32 = 2;
const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;
const CYCLES_PER_SCREEN_DRAW: u64 = 70_224;

fn main() {
    let rom_path = match std::env::consts::OS {
        "linux" => "/home/ruben/dev/gb-rs/tetris.gb",
        "windows" => "I:\\Dev\\gb-rs\\tetris.gb",
        _ => panic!("wat?")
    };

    let mut gb = GameBoy::new(rom_path);

    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let window = video_subsystem
        .window("frosty", WIDTH * SCALE, HEIGHT * SCALE)
        .opengl()
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, WIDTH, HEIGHT)
        .map_err(|e| e.to_string())
        .unwrap();

    let mut frame_count: u64 = 0;
    let timer = sdl.timer().unwrap();
    let mut turbo = false;

    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::KeyDown {
                     timestamp: _, 
                     window_id: _, 
                     keycode, 
                     scancode: _, 
                     keymod: _, 
                     repeat 
                } => {
                    if !repeat && keycode.is_some() {
                        let keycode = keycode.unwrap();
                        match keycode {
                            sdl2::keyboard::Keycode::Tab => turbo = true,
                            _ => gb.key_down(keycode)
                        }
                    }
                }

                sdl2::event::Event::KeyUp {
                    timestamp: _, 
                    window_id: _, 
                    keycode, 
                    scancode: _, 
                    keymod: _, 
                    repeat 
               } => {
                   if !repeat && keycode.is_some() {
                       let keycode = keycode.unwrap();
                       match keycode {
                        sdl2::keyboard::Keycode::Tab => turbo = false,
                        _ => gb.key_up(keycode)
                    }
                   }
               }

                sdl2::event::Event::Quit { .. } => break 'main,
                _ => {}
            }
        }

        let start = timer.performance_counter();

        for _ in 0..CYCLES_PER_SCREEN_DRAW {
            gb.tick();
        }

        update_canvas(&mut texture, &mut canvas, &gb);
        frame_count += 1;

        let end = timer.performance_counter();
        let elapsed = (end - start) as f64 / timer.performance_frequency() as f64 * 1000.0;

        if !turbo && elapsed < 16.6666 {
            let sleep_amount = (16.6666 - elapsed) as u64;
            std::thread::sleep(Duration::from_millis(sleep_amount));
        }

        let end = timer.performance_counter();

        if frame_count % 180 == 0 {
            let elapsed = (end - start) as f64 / timer.performance_frequency() as f64;
            canvas.window_mut().set_title(format!("frosty fps: {:.1}", 1.0f64 / elapsed).as_str()).unwrap()
        }
    }
}

fn update_canvas(texture: &mut Texture, canvas: &mut Canvas<Window>, gb: &GameBoy) {
    let frame_buffer = gb.get_frame_buffer();

    texture.with_lock(None, |buffer: &mut [u8], _pitch: usize| {
        let mut i: usize = 0;
        while i < (WIDTH * HEIGHT * 3) as usize {
            let index = i / 3;
            let color = frame_buffer[index];

            buffer[i] = color;
            buffer[i + 1] = color;
            buffer[i + 2] = color;

            i += 3;
        }
    }).unwrap();

    canvas.clear();
    canvas.copy(&texture, None, None).unwrap();
    canvas.present();
}