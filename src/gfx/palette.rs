//! The 256-colour VGA-style palette and the light (shade) tables.
//!
//! The original game ran in VGA mode 13h: 256 colours, 6 bits per channel.
//! Our palette follows the same structure the original used: a handful of
//! special colours followed by ramps that run from near-black to a highlight.
//! Every ramp ends in near-black so distance shading can stay inside a ramp
//! and keep the hue, exactly like the original's light tables did.

pub const SHADES: usize = 32;
pub const MAX_LIGHT: usize = SHADES - 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Ramp {
    Grey,
    Slate,
    Stone,
    Earth,
    Parch,
    Copper,
    Gold,
    Red,
    Flesh,
    Green,
    Moss,
    Blue,
    Purple,
    Silver,
    Fire,
}

pub const RAMP_COUNT: usize = 15;
pub const RAMP_LEN: usize = 16;

/// Palette index of shade `s` (0 = darkest, 15 = brightest) of ramp `r`.
#[inline]
pub const fn c(r: Ramp, s: i32) -> u8 {
    let s = if s < 0 {
        0
    } else if s > 15 {
        15
    } else {
        s
    };
    16 + (r as u8) * 16 + s as u8
}

/// Which ramp a palette index belongs to (None for the special colours).
pub fn ramp_of(i: u8) -> Option<(Ramp, i32)> {
    if i < 16 {
        return None;
    }
    let r = (i - 16) / 16;
    let s = ((i - 16) % 16) as i32;
    Some((RAMPS_ORDER[r as usize], s))
}

pub const RAMPS_ORDER: [Ramp; RAMP_COUNT] = [
    Ramp::Grey,
    Ramp::Slate,
    Ramp::Stone,
    Ramp::Earth,
    Ramp::Parch,
    Ramp::Copper,
    Ramp::Gold,
    Ramp::Red,
    Ramp::Flesh,
    Ramp::Green,
    Ramp::Moss,
    Ramp::Blue,
    Ramp::Purple,
    Ramp::Silver,
    Ramp::Fire,
];

// Special colours (indices 0..16).
#[allow(dead_code)]
pub const TRANSPARENT: u8 = 0;
pub const BLACK: u8 = 1;
pub const WHITE: u8 = 2;
pub const CURSOR_YELLOW: u8 = 3;
#[allow(dead_code)]
pub const BRIGHT_RED: u8 = 4;
#[allow(dead_code)]
pub const BRIGHT_GREEN: u8 = 5;
pub const INK: u8 = 6; // message scroll text colour
pub const SCROLL_BG: u8 = 7; // message scroll parchment
pub const MSG_GREEN: u8 = 8; // system messages
#[allow(dead_code)]
pub const NEAR_BLACK: u8 = 9;
#[allow(dead_code)]
pub const BRIGHT_BLUE: u8 = 10;
#[allow(dead_code)]
pub const DARK_OUTLINE: u8 = 11;
#[allow(dead_code)]
pub const INK_RED: u8 = 12;
pub const PALE_YELLOW: u8 = 13;
pub const CURSOR_DARK: u8 = 14;
pub const INK_BLUE: u8 = 15;

const SPECIALS: [[u8; 3]; 16] = [
    [0, 0, 0],       // transparent (draws black)
    [0, 0, 0],       // black
    [252, 252, 252], // white
    [252, 216, 0],   // cursor yellow
    [252, 0, 0],     // bright red
    [0, 252, 0],     // bright green
    [60, 40, 32],    // ink
    [140, 104, 84],  // scroll parchment
    [72, 192, 72],   // message green
    [8, 8, 12],      // near black
    [64, 128, 252],  // bright blue
    [20, 12, 8],     // dark outline (warm black)
    [140, 20, 12],   // red ink
    [252, 240, 160], // pale yellow
    [100, 60, 0],    // cursor dark edge
    [32, 40, 112],   // blue ink
];

/// Key colours for each ramp, dark to light. Interpolated to 16 shades.
fn ramp_stops(r: Ramp) -> &'static [[u8; 3]] {
    match r {
        Ramp::Grey => &[[3, 3, 4], [40, 40, 42], [88, 88, 88], [140, 140, 138], [196, 196, 192], [244, 244, 240]],
        Ramp::Slate => &[[3, 5, 8], [28, 38, 44], [56, 72, 78], [88, 108, 112], [136, 156, 158], [204, 220, 220]],
        Ramp::Stone => &[[5, 4, 3], [44, 40, 34], [92, 84, 72], [140, 128, 108], [188, 176, 152], [236, 228, 208]],
        Ramp::Earth => &[[6, 3, 0], [48, 26, 6], [100, 58, 14], [148, 96, 32], [196, 144, 72], [236, 200, 140]],
        Ramp::Parch => &[[14, 9, 6], [60, 40, 32], [100, 72, 56], [140, 104, 84], [192, 158, 124], [244, 224, 188]],
        Ramp::Copper => &[[10, 4, 0], [64, 32, 8], [112, 56, 20], [160, 84, 48], [200, 128, 92], [248, 196, 156]],
        Ramp::Gold => &[[18, 8, 0], [92, 52, 0], [168, 112, 8], [216, 164, 24], [244, 216, 88], [255, 248, 200]],
        Ramp::Red => &[[10, 0, 0], [72, 6, 4], [140, 16, 10], [204, 44, 28], [240, 112, 84], [255, 196, 176]],
        Ramp::Flesh => &[[16, 6, 3], [84, 40, 24], [156, 88, 60], [208, 140, 108], [240, 188, 156], [255, 232, 212]],
        Ramp::Green => &[[0, 6, 0], [14, 40, 10], [32, 88, 20], [72, 144, 40], [136, 204, 80], [212, 252, 168]],
        Ramp::Moss => &[[5, 5, 0], [36, 36, 8], [76, 74, 22], [120, 116, 48], [168, 160, 88], [220, 212, 148]],
        Ramp::Blue => &[[0, 0, 10], [8, 14, 64], [20, 40, 132], [52, 88, 200], [120, 156, 240], [208, 228, 255]],
        Ramp::Purple => &[[5, 0, 8], [34, 14, 56], [72, 40, 116], [120, 84, 172], [172, 140, 220], [232, 216, 255]],
        Ramp::Silver => &[[4, 4, 8], [40, 40, 52], [84, 84, 104], [124, 124, 148], [176, 176, 200], [240, 240, 252]],
        Ramp::Fire => &[[28, 0, 0], [112, 12, 0], [196, 52, 0], [244, 128, 8], [252, 204, 48], [255, 252, 208]],
    }
}

fn q6(v: f32) -> u8 {
    // Quantise to a 6-bit VGA DAC value, expressed in 8 bits (value * 4,
    // which is how DOS screenshots of the era store it).
    let v = v.round().clamp(0.0, 255.0) as u32;
    let v6 = ((v + 2) / 4).min(63);
    (v6 * 4) as u8
}

pub struct Palette {
    pub rgb: [[u8; 3]; 256],
    /// shade[level][index] -> index; level 0 = black, MAX_LIGHT = unchanged.
    pub shade: Vec<[u8; 256]>,
    nearest_cube: Vec<u8>, // 32x32x32 cube lookup for nearest()
}

fn dist2(a: [u8; 3], b: [i32; 3]) -> i32 {
    let dr = a[0] as i32 - b[0];
    let dg = a[1] as i32 - b[1];
    let db = a[2] as i32 - b[2];
    dr * dr * 3 + dg * dg * 4 + db * db * 2
}

impl Palette {
    pub fn new() -> Palette {
        let mut rgb = [[0u8; 3]; 256];
        for (i, s) in SPECIALS.iter().enumerate() {
            rgb[i] = *s;
        }
        for (ri, r) in RAMPS_ORDER.iter().enumerate() {
            let stops = ramp_stops(*r);
            let n = stops.len() - 1;
            for s in 0..RAMP_LEN {
                let t = s as f32 / (RAMP_LEN - 1) as f32 * n as f32;
                let k = (t.floor() as usize).min(n - 1);
                let f = t - k as f32;
                let a = stops[k];
                let b = stops[k + 1];
                let mut col = [0u8; 3];
                for ch in 0..3 {
                    col[ch] = q6(a[ch] as f32 + (b[ch] as f32 - a[ch] as f32) * f);
                }
                rgb[16 + ri * 16 + s] = col;
            }
        }
        let mut pal = Palette { rgb, shade: Vec::new(), nearest_cube: Vec::new() };
        pal.build_nearest_cube();
        pal.build_shades();
        pal
    }

    fn build_nearest_cube(&mut self) {
        let mut cube = vec![0u8; 32 * 32 * 32];
        for r in 0..32usize {
            for g in 0..32usize {
                for b in 0..32usize {
                    let t = [(r * 8 + 4) as i32, (g * 8 + 4) as i32, (b * 8 + 4) as i32];
                    cube[(r * 32 + g) * 32 + b] = self.nearest_slow(t);
                }
            }
        }
        self.nearest_cube = cube;
    }

    fn nearest_slow(&self, t: [i32; 3]) -> u8 {
        let mut best = BLACK;
        let mut bd = i32::MAX;
        for i in 1..256usize {
            // Skip specials that are UI-only so art does not pick them up.
            if (3..16).contains(&i) {
                continue;
            }
            let d = dist2(self.rgb[i], t);
            if d < bd {
                bd = d;
                best = i as u8;
            }
        }
        best
    }

    fn build_shades(&mut self) {
        let mut tables = vec![[0u8; 256]; SHADES];
        for (level, table) in tables.iter_mut().enumerate() {
            let f = level as f32 / MAX_LIGHT as f32;
            for i in 0..256usize {
                if i == 0 {
                    table[i] = 0;
                    continue;
                }
                if level == MAX_LIGHT {
                    table[i] = i as u8;
                    continue;
                }
                let src = self.rgb[i];
                let t = [
                    (src[0] as f32 * f) as i32,
                    (src[1] as f32 * f) as i32,
                    (src[2] as f32 * f) as i32,
                ];
                if i < 16 {
                    table[i] = self.nearest_slow(t);
                    continue;
                }
                let base = 16 + ((i - 16) / 16) * 16;
                let mut best = BLACK;
                let mut bd = dist2(self.rgb[BLACK as usize], t);
                for s in 0..RAMP_LEN {
                    let d = dist2(self.rgb[base + s], t);
                    if d < bd {
                        bd = d;
                        best = (base + s) as u8;
                    }
                }
                table[i] = best;
            }
        }
        self.shade = tables;
    }

    /// Convert an indexed image to RGBA, with an optional tint (r,g,b,amount 0..1).
    pub fn to_rgba(&self, src: &[u8], out: &mut [u8], tint: Option<([f32; 3], f32)>) {
        match tint {
            None => {
                for (i, &p) in src.iter().enumerate() {
                    let c = self.rgb[p as usize];
                    out[i * 4] = c[0];
                    out[i * 4 + 1] = c[1];
                    out[i * 4 + 2] = c[2];
                    out[i * 4 + 3] = 255;
                }
            }
            Some((t, a)) => {
                let mut lut = [[0u8; 3]; 256];
                for (i, l) in lut.iter_mut().enumerate() {
                    for ch in 0..3 {
                        let v = self.rgb[i][ch] as f32 * (1.0 - a) + t[ch] * a;
                        l[ch] = v.clamp(0.0, 255.0) as u8;
                    }
                }
                for (i, &p) in src.iter().enumerate() {
                    let c = lut[p as usize];
                    out[i * 4] = c[0];
                    out[i * 4 + 1] = c[1];
                    out[i * 4 + 2] = c[2];
                    out[i * 4 + 3] = 255;
                }
            }
        }
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ramps_are_dark_to_light() {
        let p = Palette::new();
        for r in RAMPS_ORDER {
            let lum = |i: u8| {
                let c = p.rgb[i as usize];
                c[0] as i32 * 3 + c[1] as i32 * 6 + c[2] as i32
            };
            for s in 1..16 {
                assert!(lum(c(r, s)) >= lum(c(r, s - 1)), "ramp {:?} shade {}", r, s);
            }
        }
    }

    #[test]
    fn shade_tables_keep_transparency_and_identity() {
        let p = Palette::new();
        assert_eq!(p.shade[MAX_LIGHT][123], 123);
        for l in 0..SHADES {
            assert_eq!(p.shade[l][0], 0);
        }
        // Darkest level should map everything to (near) black.
        let c0 = p.rgb[p.shade[0][c(Ramp::Gold, 15) as usize] as usize];
        assert!(c0[0] < 24 && c0[1] < 24 && c0[2] < 24);
    }
}
