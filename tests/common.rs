use image::{ImageBuffer, RgbImage};

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

pub fn compare_image(fb: &[u8], p: String) -> bool {
    let img = image::io::Reader::open(p).unwrap().decode().unwrap();
    let img = img.as_rgb8().unwrap();

    let index: usize = 0;
    for px in img.enumerate_pixels() {
        if fb[index] != px.2[0] {
            return false;
        }
    }

    true
}