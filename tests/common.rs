use std::path::PathBuf;

use image::{ImageBuffer, RgbImage, RgbaImage, io::Reader};

pub const WIDTH: u32 = 160;
pub const HEIGHT: u32 = 144;
pub const CYCLES_PER_SCREEN_DRAW: u64 = 70_224;

#[allow(dead_code)]
pub fn create_image(fb: &[u8], p: String) {
    let mut img: RgbImage = ImageBuffer::new(WIDTH, HEIGHT);

    let mut img_itr = img.iter_mut();

    for px in fb {
        for _ in 0..3 {
            let img_px = img_itr.next().unwrap();
            *img_px = *px;
        }
    }

    img.save(p).unwrap();
}

pub fn compare_image_rgb8(fb: &[u8], p: String) -> bool {
    let img = image::io::Reader::open(p).unwrap().decode().unwrap();
    let img = img.as_rgb8().unwrap();

    for px in img.enumerate_pixels() {
        if fb[((px.1 * WIDTH) + px.0) as usize] != px.2[0] {
            return false;
        }
    }

    true
}

pub fn compare_image_luma8(fb: &[u8], p: String) -> bool {
    let img = image::io::Reader::open(&p).unwrap().decode().unwrap();
    let img = img.as_luma8().unwrap();

    let mut incorrect_px = 0;
    let mut incorrect_px_list: Vec<(u32, u32)> = Vec::new();

    for px in img.enumerate_pixels() {
        let fb_px = fb[((px.1 * WIDTH) + px.0) as usize];
        let img_px = px.2[0] as i32;
        
        let fb_px = match fb_px {
            0 => 0,
            96 => 0x55,
            192 => 0xAA,
            255 => 255,
            _ => unreachable!()
        } as i32;

        match img_px {
            0 | 0x55 | 0xAA | 255 => {},
            _ => panic!("Invalid luma val??")
        }


        if (fb_px - img_px).abs() > 20 {
            incorrect_px += 1;
            incorrect_px_list.push((px.0, px.1));
        }
    }

    if std::env::var("CI").is_err() {
        let mut img: RgbaImage = ImageBuffer::new(WIDTH, HEIGHT);

        let mut img_itr = img.iter_mut();

        for px in fb {
            for _ in 0..3 {
                let img_px = img_itr.next().unwrap();
                *img_px = *px;
            }

            let img_px = img_itr.next().unwrap();
            *img_px = 255;
        }

        for point in incorrect_px_list {
            let px = img.get_pixel_mut(point.0, point.1);
            px[0] = 118;
            px[1] = 247;
            px[2] = 101;
            px[3] = 50;
        }

        img.save("I:\\temp.png").unwrap();
        create_image(fb, "I:\\result.png".to_owned());
    }

    let correct_px = fb.len() - incorrect_px;
    let accuracy = (correct_px as f32 / fb.len() as f32) * 100.0;
    println!("Accuracy: {}%", accuracy);

    accuracy == 100.0
}

pub fn get_base_dir() -> PathBuf {
    match std::env::var("CI") {
        Ok(_) => {
            let github_workspace = std::env::var("GITHUB_WORKSPACE").unwrap();
            PathBuf::from(&github_workspace)
        }

        Err(_) => {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        }
    }
}