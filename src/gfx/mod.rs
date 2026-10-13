pub mod bitmap;
pub mod font;
pub mod palette;
pub mod raster;

pub use bitmap::Bitmap;
pub use font::Font;
pub use palette::{c, Palette, Ramp};

/// The original game's screen resolution.
pub const SCREEN_W: i32 = 320;
pub const SCREEN_H: i32 = 200;

/// Write RGBA pixels to a PNG file.
pub fn save_png_rgba(path: &str, w: u32, h: u32, rgba: &[u8]) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|e| format!("{}: {}", path, e))?;
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut writer = enc.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(rgba).map_err(|e| e.to_string())?;
    Ok(())
}

/// Write an indexed bitmap through the palette to a PNG, optionally scaled up.
pub fn save_bitmap_png(path: &str, bmp: &Bitmap, pal: &Palette, scale: i32) -> Result<(), String> {
    let s = scale.max(1);
    let (w, h) = (bmp.w * s, bmp.h * s);
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let p = bmp.get(x / s, y / s);
            let c = pal.rgb[p as usize];
            let i = ((y * w + x) * 4) as usize;
            rgba[i] = c[0];
            rgba[i + 1] = c[1];
            rgba[i + 2] = c[2];
            rgba[i + 3] = if p == 0 { 0 } else { 255 };
        }
    }
    save_png_rgba(path, w as u32, h as u32, &rgba)
}
