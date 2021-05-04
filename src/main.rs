#![allow(dead_code)]

use crate::gameboy::GameBoy;

mod gameboy;

fn main() {
    let mut gb = GameBoy::new();

    let rom_path = match std::env::consts::OS {
        "linux" => "/home/ruben/dev/gb-rs/tetris.gb",
        "windows" => "I:\\Dev\\gb-rs\\tetris.gb",
        _ => panic!("wat?")
    };

    gb.load_rom(rom_path);

    loop {
        gb.tick();
    }
}
