use std::{fs::{self}, path::{PathBuf}};

use gameboy_rs::gameboy::GameBoy;
use common::{CYCLES_PER_SCREEN_DRAW, compare_image};
extern crate gameboy_rs;

mod common;

macro_rules! blargg_test {
    ($($name:ident: $secs:expr,)*) => {
    $(
        #[test]
        fn $name() {
            let mut pb: PathBuf = match std::env::var("CI") {
                Ok(_) => {
                    let github_workspace = std::env::var("GITHUB_WORKSPACE").unwrap();
                    PathBuf::from(&github_workspace)
                }

                Err(_) => {
                    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                }
            };
            
            let rom_num = &stringify!($name)[6..];
            
            pb.push(format!("tests\\roms\\blargg\\{}.gb", rom_num));
            let rom_str = pb.to_str().unwrap();

            let mut sav_file_loc = pb.clone();
            sav_file_loc.set_file_name(format!("{}..sav", rom_num));
            let sav_file_loc = sav_file_loc.to_str().unwrap();

            {
                let mut s = GameBoy::new(rom_str, None);

                let cycles_to_run = CYCLES_PER_SCREEN_DRAW * 60 * $secs;
                for _ in 0..cycles_to_run {
                    s.tick();
                }

                let fb = s.get_frame_buffer();

                pb.pop();
                pb.pop();
                pb.pop();

                pb.push(format!("expected\\blargg\\{}.png", rom_num));

                let comparison = compare_image(fb, pb.to_str().unwrap().to_owned());
                assert!(comparison);
            }

            // gb should be dropped now, which will create a .sav file
            // delete the .sav file
            // for some reason the save file has two periods "." in it
            match fs::remove_file(sav_file_loc) {
                Ok(_) => { },
                Err(_) => { } // don't really care if it fails
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
    blarg_05: (10),
    blarg_06: (5),
    blarg_07: (5),
    blarg_08: (5),
    blarg_09: (20),
    blarg_10: (20),
    blarg_11: (20),
}
