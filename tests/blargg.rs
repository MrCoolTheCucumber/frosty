use std::{fs::File, io::{Read}, path::{PathBuf}};

use gameboy_rs::gameboy::GameBoy;

extern crate gameboy_rs;

const CYCLES_PER_SCREEN_DRAW: u64 = 70_224;

// https://stackoverflow.com/questions/34662713/how-can-i-create-parameterized-tests-in-rust

macro_rules! blargg_test {
    ($($name:ident: $secs:expr,)*) => {
    $(
        #[test]
        fn $name() {
            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            
            let rom_num = &stringify!($name)[6..];
            
            d.push(format!("tests/roms/blargg/{}.gb", rom_num));
            let rom_str = d.to_str().unwrap();

            let mut s = GameBoy::new(rom_str, None);

            let cycles_to_run = CYCLES_PER_SCREEN_DRAW * 60 * $secs;
            for _ in 0..cycles_to_run {
                s.tick();
            }

            let fb = s.get_frame_buffer();

            let mut exp = File::open(format!("./tests/expected/blargg/{}.bin", rom_num)).unwrap();
            let mut buf = Vec::new();
            exp.read_to_end(&mut buf).unwrap();

            for i in 0..fb.len() {
                assert_eq!(fb[i], buf[i]);
            }
        }
    )*
    }
}

blargg_test! {
    blarg_01: (5),
    blarg_02: (5),
    blarg_03: (5),
    blarg_04: (5),
    blarg_05: (5),
    blarg_06: (5),
    blarg_07: (5),
    blarg_08: (5),
    blarg_09: (10),
    blarg_10: (20),
    blarg_11: (20),
}
