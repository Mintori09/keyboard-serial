use std::path::Path;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let icon_path = Path::new(&out_dir).join("icon.png");

    let w = 64u32;
    let h = 64u32;
    let mut img = image::RgbaImage::new(w, h);

    let transparent = image::Rgba([0, 0, 0, 0]);
    let body = image::Rgba([44, 62, 80, 255]);
    let body_highlight = image::Rgba([64, 82, 100, 255]);
    let key = image::Rgba([189, 195, 199, 255]);
    let key_alt = image::Rgba([52, 152, 219, 255]);

    for y in 0..h {
        for x in 0..w {
            img.put_pixel(x, y, transparent);
        }
    }

    fn rect(img: &mut image::RgbaImage, x: u32, y: u32, rw: u32, rh: u32, color: image::Rgba<u8>) {
        for dy in 0..rh {
            for dx in 0..rw {
                let px = x + dx;
                let py = y + dy;
                if px < img.width() && py < img.height() {
                    img.put_pixel(px, py, color);
                }
            }
        }
    }

    // keyboard body
    rect(&mut img, 4, 6, 56, 52, body);
    rect(&mut img, 4, 6, 56, 1, body_highlight);

    let kw = 4;
    let kh = 6;
    let gap = 2;

    // Row 1: 10 keys
    for i in 0..10 {
        rect(&mut img, 8 + i * (kw + gap), 10, kw, kh, key);
    }
    // Row 2: 9 keys (offset)
    for i in 0..9 {
        rect(&mut img, 11 + i * (kw + gap), 18, kw, kh, key);
    }
    // Row 3: 9 keys
    for i in 0..9 {
        rect(&mut img, 11 + i * (kw + gap), 26, kw, kh, key);
    }
    // Row 4: space bar + modifiers
    rect(&mut img, 8, 34, 4, kh, key_alt);
    rect(&mut img, 14, 34, 28, kh, key);
    rect(&mut img, 44, 34, 4, kh, key_alt);
    rect(&mut img, 50, 34, 4, kh, key);

    img.save(&icon_path).expect("failed to save tray icon");
    println!("cargo:rerun-if-changed=build.rs");
}
