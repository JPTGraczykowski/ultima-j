//! Procedurally generated art. Everything the game shows is drawn here at
//! start-up from code, styled after the original game's look.

pub mod canvas;
pub mod creatures;
pub mod items;
pub mod sprite3d;
pub mod textures;
pub mod ui;

use crate::gfx::raster::Texture;
use crate::gfx::{Bitmap, Font, Palette};
use textures::{Tex, ALL_TEX};

pub struct Art {
    pub pal: Palette,
    /// textures[tex][frame]
    pub textures: Vec<Vec<Texture>>,
    pub font: Font,
    pub font_big: Font,
    pub font_huge: Font,
    pub ui: ui::UiArt,
}

impl Art {
    pub fn generate() -> Art {
        let pal = Palette::new();
        let textures = ALL_TEX
            .iter()
            .map(|&t| (0..t.frames()).map(|f| Texture::from_bitmap(&textures::generate(t, f))).collect())
            .collect();
        let font = Font::small();
        let font_big = Font::serif();
        let font_huge = font_big.scale2x();
        let ui = ui::UiArt::generate();
        Art { pal, textures, font, font_big, font_huge, ui }
    }

    pub fn tex(&self, t: Tex, frame: usize) -> &Texture {
        let v = &self.textures[t as usize];
        &v[frame % v.len()]
    }

    /// A sheet of all textures for visual review.
    pub fn texture_sheet(&self) -> Bitmap {
        let cols = 8;
        let cell = 68;
        let rows = (ALL_TEX.len() as i32 + cols - 1) / cols;
        let mut sheet = Bitmap::filled(cols * cell, rows * cell, crate::gfx::palette::c(crate::gfx::Ramp::Purple, 2));
        for (i, t) in ALL_TEX.iter().enumerate() {
            let tx = &self.textures[*t as usize][0];
            let bmp = Bitmap { w: tx.w, h: tx.h, px: tx.px.clone() };
            let x = (i as i32 % cols) * cell + 2;
            let y = (i as i32 / cols) * cell + 2;
            if t.is_floor() {
                // Show floors tiled 2x2 so seams are visible.
                for k in 0..4 {
                    sheet.blit_opaque(&bmp, x + (k % 2) * 32, y + (k / 2) * 32);
                }
            } else {
                sheet.blit_opaque(&bmp, x, y);
            }
        }
        sheet
    }
}
