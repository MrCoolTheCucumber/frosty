use std::{path::{PathBuf}};

use gameboy_rs::gameboy::GameBoy;
use common::{compare_image_rgb8, CYCLES_PER_SCREEN_DRAW};

extern crate gameboy_rs;
extern crate image;

mod common;

macro_rules! mooneye_test {
    ($($name:ident: $path:expr,)*) => {
    $(
        #[test]
        fn $name() {
            let path: String = stringify!($path).to_owned();
            let path: String = path[1..path.len()-1].to_owned();
            

            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("./tests/roms/mooneye/acceptance/");
            d.push(&path);

            let rom_str = d.to_str().unwrap();

            {
                let mut s = GameBoy::new(rom_str, None);

                let cycles_to_run = CYCLES_PER_SCREEN_DRAW * 60 * 5;
                for _ in 0..cycles_to_run {
                    s.tick();
                }

                let fb = s.get_frame_buffer();

                // create file in expected
                let bin_file_path = format!("./tests/expected/mooneye/acceptance/{}.png", &path);
                let comparison = compare_image_rgb8(fb, bin_file_path);
                assert!(comparison);
            }
        }
    )*
    }
}

mooneye_test! {
    add_sp_e_timing: "add_sp_e_timing.gb",
    mem_oam: "bits/mem_oam.gb",
    reg_f: "bits/reg_f.gb",
    // unused_hwio: "bits/unused_hwio-GS.gb"
    boot_div: "boot_div-dmgABCmgb.gb",
    boot_regs: "boot_regs-dmgABC.gb",
    call_cc_timing: "call_cc_timing.gb",
    call_cc_timing2: "call_cc_timing2.gb",
    call_timing: "call_timing.gb",
    call_timing2: "call_timing2.gb",
    div_timing: "div_timing.gb",
    di_timing: "di_timing-GS.gb",
    ei_sequence: "ei_sequence.gb",
    ei_timing: "ei_timing.gb",
    halt_ime0_ei: "halt_ime0_ei.gb",
    halt_ime0_nointr_timing: "halt_ime0_nointr_timing.gb",
    halt_ime1_timing: "halt_ime1_timing.gb",
    halt_ime1_timing2: "halt_ime1_timing2-GS.gb",
    if_ie_registers: "if_ie_registers.gb",
    daa: "instr/daa.gb",
    // ie_push: "interrupts/ie_push.gb",
    intr_timing: "intr_timing.gb",
    jp_cc_timing: "jp_cc_timing.gb",
    jp_timing: "jp_timing.gb",
    ld_hl_sp_e_timing: "ld_hl_sp_e_timing.gb",
    basic: "oam_dma/basic.gb",
    reg_read: "oam_dma/reg_read.gb",
    // sources: "oam_dma/sources-GS.gb",
    oam_dma_restart: "oam_dma_restart.gb",
    oam_dma_start: "oam_dma_start.gb",
    oam_dma_timing: "oam_dma_timing.gb",
    pop_timing: "pop_timing.gb",
    hblank_ly_scx_timing: "ppu/hblank_ly_scx_timing-GS.gb",
    intr_1_2_timing: "ppu/intr_1_2_timing-GS.gb",
    intr_2_0_timing: "ppu/intr_2_0_timing.gb",
    intr_2_mode0_timing: "ppu/intr_2_mode0_timing.gb",
    // intr_2_mode0_timing_sprites: "ppu/intr_2_mode0_timing_sprites.gb",
    intr_2_mode3_timing: "ppu/intr_2_mode3_timing.gb",
    intr_2_oam_ok_timing: "ppu/intr_2_oam_ok_timing.gb",
    lcdon_timing: "ppu/lcdon_timing-GS.gb",
    // lcdon_write_timing: "ppu/lcdon_write_timing-GS.gb",
    stat_irq_blocking: "ppu/stat_irq_blocking.gb",
    // stat_lyc_onoff: "ppu/stat_lyc_onoff.gb",
    // vblank_stat_intr: "ppu/vblank_stat_intr-GS.gb",
    push_timing: "push_timing.gb",
    rapid_di_ei: "rapid_di_ei.gb",
    reti_intr_timing: "reti_intr_timing.gb",
    reti_timing: "reti_timing.gb",
    ret_cc_timing: "ret_cc_timing.gb",
    ret_timing: "ret_timing.gb",
    rst_timing: "rst_timing.gb",
    // boot_sclk_align: "serial/boot_sclk_align-dmgABCmgb.gb",
    div_write: "timer/div_write.gb",
    rapid_toggle: "timer/rapid_toggle.gb",
    tim00: "timer/tim00.gb",
    tim00_div_trigger: "timer/tim00_div_trigger.gb",
    tim01: "timer/tim01.gb",
    tim01_div_trigger: "timer/tim01_div_trigger.gb",
    tim10: "timer/tim10.gb",
    tim10_div_trigger: "timer/tim10_div_trigger.gb",
    tim11: "timer/tim11.gb",
    tim11_div_trigger: "timer/tim11_div_trigger.gb",
    tima_reload: "timer/tima_reload.gb",
    tima_write_reloading: "timer/tima_write_reloading.gb",
    tma_write_reloading: "timer/tma_write_reloading.gb",
}
