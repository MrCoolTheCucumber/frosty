use common::{CYCLES_PER_SCREEN_DRAW, compare_image_rgb8, get_base_dir};
use gameboy_rs::gameboy::GameBoy;

mod common;

#[test]
fn dmg_acid2() {
    let mut pb = get_base_dir();
    pb.push("tests\\roms\\dmg-acid2.gb");
    let rom_dir = pb.to_str().unwrap();

    let mut s = GameBoy::new(rom_dir, None);

    let cycles_to_run = CYCLES_PER_SCREEN_DRAW * 60 * 5;
    for _ in 0..cycles_to_run {
        s.tick();
    }

    let fb = s.get_frame_buffer();

    pb.pop();
    pb.pop();

    pb.push("expected\\dmg-acid2.png");
    println!("{}", pb.to_str().unwrap());

    let comparison = compare_image_rgb8(fb, pb.to_str().unwrap().to_owned());
    assert!(comparison);
}
