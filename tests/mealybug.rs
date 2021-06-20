use std::{path::{PathBuf}};

use gameboy_rs::gameboy::GameBoy;
use common::{compare_image_luma8, CYCLES_PER_SCREEN_DRAW};

extern crate gameboy_rs;
extern crate image;

mod common;

macro_rules! mealybug_test {
    ($($name:ident: $path:expr,)*) => {
    $(
        #[test]
        #[ignore]
        fn $name() {
            let path: String = stringify!($path).to_owned();
            let path: String = path[1..path.len()-1].to_owned();
            

            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("./tests/roms/mealybug/");
            d.push(&path);

            let rom_str = d.to_str().unwrap();

            {
                let mut s = GameBoy::new(rom_str, None);

                let cycles_to_run = CYCLES_PER_SCREEN_DRAW * 60 * 10;
                for _ in 0..cycles_to_run {
                    s.tick();
                }

                let fb = s.get_frame_buffer();

                // create file in expected
                let bin_file_path = format!("./tests/expected/mealybug/{}.png", &path[0..path.len()-3]);
                let comparison = compare_image_luma8(fb, bin_file_path);
                assert!(comparison);
            }
        }
    )*
    }
}

mealybug_test! {

    /*
        First line is completely blank when it shouldnt be
    */
    m2_win_en_toggle: "m2_win_en_toggle.gb", 

    m3_bgp_change: "m3_bgp_change.gb", 
    m3_bgp_change_sprites: "m3_bgp_change_sprites.gb", 
    m3_lcdc_bg_en_change: "m3_lcdc_bg_en_change.gb", 
    // m3_lcdc_bg_en_change2: "m3_lcdc_bg_en_change2.gb", 
    m3_lcdc_bg_map_change: "m3_lcdc_bg_map_change.gb", 
    // m3_lcdc_bg_map_change2: "m3_lcdc_bg_map_change2.gb", 
    m3_lcdc_obj_en_change: "m3_lcdc_obj_en_change.gb", 
    m3_lcdc_obj_en_change_variant: "m3_lcdc_obj_en_change_variant.gb", 
    m3_lcdc_obj_size_change: "m3_lcdc_obj_size_change.gb", 
    m3_lcdc_obj_size_change_scx: "m3_lcdc_obj_size_change_scx.gb", 
    m3_lcdc_tile_sel_change: "m3_lcdc_tile_sel_change.gb", 
    // m3_lcdc_tile_sel_change2: "m3_lcdc_tile_sel_change2.gb", 
    m3_lcdc_tile_sel_win_change: "m3_lcdc_tile_sel_win_change.gb", 
    // m3_lcdc_tile_sel_win_change2: "m3_lcdc_tile_sel_win_change2.gb", 
    m3_lcdc_win_en_change_multiple: "m3_lcdc_win_en_change_multiple.gb", 
    m3_lcdc_win_en_change_multiple_wx: "m3_lcdc_win_en_change_multiple_wx.gb", 
    m3_lcdc_win_map_change: "m3_lcdc_win_map_change.gb", 
    // m3_lcdc_win_map_change2: "m3_lcdc_win_map_change2.gb", 
    m3_obp0_change: "m3_obp0_change.gb", 
    m3_scx_high_5_bits: "m3_scx_high_5_bits.gb", 
    // m3_scx_high_5_bits_change2: "m3_scx_high_5_bits_change2.gb", 
    m3_scx_low_3_bits: "m3_scx_low_3_bits.gb", 
    m3_scy_change: "m3_scy_change.gb", 
    // m3_scy_change2: "m3_scy_change2.gb", 
    m3_window_timing: "m3_window_timing.gb", 
    m3_window_timing_wx_0: "m3_window_timing_wx_0.gb", 
    m3_wx_4_change: "m3_wx_4_change.gb", 
    m3_wx_4_change_sprites: "m3_wx_4_change_sprites.gb", 
    m3_wx_5_change: "m3_wx_5_change.gb", 
    m3_wx_6_change: "m3_wx_6_change.gb", 
}