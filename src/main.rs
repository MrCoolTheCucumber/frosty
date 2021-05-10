#![allow(dead_code)]

mod gameboy;

use std::{thread, time::Duration};

use ggez::{Context, ContextBuilder, GameResult, conf::{FullscreenType, NumSamples, WindowMode, WindowSetup}};
use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::graphics;

use crate::gameboy::GameBoy;

const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;
const SCALE: u32 = 2;
const SCALED_IMAGE_BUFFER_LENGTH: usize = 160 * 144 * 4 * SCALE as usize;

struct GBState {
    gb: GameBoy,
    prev_cpu_cycles: u64,
    turbo: bool
}

impl GBState {
    pub fn new(_ctx: &mut Context) -> Self {
        let rom_path = match std::env::consts::OS {
            "linux" => "/home/ruben/dev/gb-rs/tetris.gb",
            "windows" => "I:\\Dev\\gb-rs\\dmg-acid2.gb",
            _ => panic!("wat?")
        };

        let gb = GameBoy::new(rom_path);

        Self {
            gb,
            prev_cpu_cycles: 0,
            turbo: false
        }
    }
}

const CYCLES_PER_SCREEN_DRAW: u64 = 70_224;

impl EventHandler for GBState {
    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods, _repeat: bool) {
        if _repeat { return; }
        if keycode == KeyCode::Tab { self.turbo = true; return }
        
        self.gb.key_down(keycode);
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        if keycode == KeyCode::Tab { self.turbo = false; return }
        self.gb.key_up(keycode);
    }

    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        for _ in 0..CYCLES_PER_SCREEN_DRAW {
            self.gb.tick();
        }

        if !self.turbo {
            thread::sleep(Duration::from_millis(5));
        }
        graphics::set_window_title(_ctx, format!("gameboy-rs (FPS: {})", ggez::timer::fps(_ctx) as u64).as_str());

        Ok(())
    }

    fn draw(&mut self, _ctx: &mut Context) -> GameResult {
        graphics::clear(_ctx, graphics::Color::from_rgb(0, 0, 0));

        let frame_buffer = self.gb.get_frame_buffer();

        let mut image_buffer: [u8; 160 * 144 * 4] = [0; 160 * 144 * 4];

        let mut i = 0;
        while i < (WIDTH * HEIGHT * 4) {
            let color = frame_buffer[(i / 4) as usize];

            image_buffer[i as usize] = color;
            image_buffer[(i + 1) as usize] = color;
            image_buffer[(i + 2) as usize] = color;
            image_buffer[(i + 3) as usize] = 255;

            i += 4;
        }

        let mut img = 
            graphics::Image::from_rgba8(_ctx, WIDTH as u16, HEIGHT as u16, &image_buffer)?;
        img.set_filter(graphics::FilterMode::Nearest);

        let pos: [f32; 2] = [0.0; 2];
        let mut dp = graphics::DrawParam::default();
        dp = dp
                .dest(pos)
                .scale([SCALE as f32; 2]);

        graphics::draw(_ctx, &img, dp)?;

        return graphics::present(_ctx);
    }
}

fn main() {  
    let window_setup = WindowSetup {
        title: "gb-rs".to_owned(),
        samples: NumSamples::Zero,
        vsync: false,
        icon: "".to_owned(),
        srgb: true,
    };

    let window_width = (WIDTH * SCALE) as f32;
    let window_height = (HEIGHT * SCALE) as f32;

    let window_mode = WindowMode {
        width: window_width,
        height: window_height,
        maximized: false,
        fullscreen_type: FullscreenType::Windowed,
        borderless: false,
        min_width: window_width,
        max_width: window_width,
        min_height: window_height,
        max_height: window_height,
        resizable: false,
    };

    let (mut ctx, mut event_loop) = 
        ContextBuilder::new("gb-rs", "Ruben")
            .window_setup(window_setup)
            .window_mode(window_mode)
            .build()
            .unwrap();

    let mut gb_state = GBState::new(&mut ctx);

    match event::run(&mut ctx, &mut event_loop, &mut gb_state) {
        Ok(_) => { },
        Err(e) => println!("Error: {}", e)
    }
}
