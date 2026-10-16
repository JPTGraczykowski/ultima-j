//! Procedural wall, floor and ceiling textures in the style of the original:
//! 64x64 walls and 32x32 floors, chunky stone blocks with dark mortar,
//! bevelled edges lit from the upper left, grainy noise and moss.

use crate::gfx::palette::{c, Ramp};
use crate::gfx::Bitmap;
use crate::util::{bayer4, fbm2, hash3, hashf, vnoise2, worley2};

/// All texture kinds. Walls are 64x64, floors/ceilings 32x32 (as in the original).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum Tex {
    // Walls
    RoughStone,
    StoneBlocks,
    MossyStone,
    IrregularStone,
    Cobbles,
    Bricks,
    RedBricks,
    CaveRock,
    WoodPlanks,
    DarkPanels,
    Marble,
    Plaque,
    WallLever,
    CarvedFrieze,
    Sandstone,
    GreyBlocks,
    // Doors
    DoorWood,
    DoorIron,
    DoorPortcullis,
    // Floors / ceilings
    Dirt,
    Flagstones,
    FloorCobbles,
    WoodFloor,
    Grass,
    Gravel,
    MarbleTiles,
    Mud,
    CeilingRock,
    CeilingBeams,
    Water,
    Lava,
    Bridge,
}

pub const TEX_COUNT: usize = Tex::Bridge as usize + 1;
pub const WATER_FRAMES: usize = 8;
pub const LAVA_FRAMES: usize = 8;

pub const ALL_TEX: [Tex; TEX_COUNT] = [
    Tex::RoughStone,
    Tex::StoneBlocks,
    Tex::MossyStone,
    Tex::IrregularStone,
    Tex::Cobbles,
    Tex::Bricks,
    Tex::RedBricks,
    Tex::CaveRock,
    Tex::WoodPlanks,
    Tex::DarkPanels,
    Tex::Marble,
    Tex::Plaque,
    Tex::WallLever,
    Tex::CarvedFrieze,
    Tex::Sandstone,
    Tex::GreyBlocks,
    Tex::DoorWood,
    Tex::DoorIron,
    Tex::DoorPortcullis,
    Tex::Dirt,
    Tex::Flagstones,
    Tex::FloorCobbles,
    Tex::WoodFloor,
    Tex::Grass,
    Tex::Gravel,
    Tex::MarbleTiles,
    Tex::Mud,
    Tex::CeilingRock,
    Tex::CeilingBeams,
    Tex::Water,
    Tex::Lava,
    Tex::Bridge,
];

impl Tex {
    pub fn is_floor(self) -> bool {
        (self as u8) >= (Tex::Dirt as u8)
    }
    pub fn size(self) -> i32 {
        if self.is_floor() { 32 } else { 64 }
    }
    pub fn frames(self) -> usize {
        match self {
            Tex::Water => WATER_FRAMES,
            Tex::Lava => LAVA_FRAMES,
            _ => 1,
        }
    }
    pub fn emissive(self) -> bool {
        matches!(self, Tex::Lava)
    }
    /// The description used by the Look command.
    pub fn describe(self) -> &'static str {
        match self {
            Tex::RoughStone => t!("a rough hewn stone wall"),
            Tex::StoneBlocks => t!("a stone wall"),
            Tex::MossyStone => t!("a mossy stone wall"),
            Tex::IrregularStone => t!("an irregular stone wall"),
            Tex::Cobbles => t!("a cobblestone wall"),
            Tex::Bricks => t!("a brick wall"),
            Tex::RedBricks => t!("a red brick wall"),
            Tex::CaveRock => t!("a rocky cave wall"),
            Tex::WoodPlanks => t!("a wooden wall"),
            Tex::DarkPanels => t!("a panelled wall"),
            Tex::Marble => t!("a marble wall"),
            Tex::Plaque => t!("a wall with writing upon it"),
            Tex::WallLever => t!("a lever set into the wall"),
            Tex::CarvedFrieze => t!("a carved stone frieze"),
            Tex::Sandstone => t!("a sandstone wall"),
            Tex::GreyBlocks => t!("a wall of dressed stone"),
            Tex::DoorWood => t!("a sturdy door"),
            Tex::DoorIron => t!("a massive iron door"),
            Tex::DoorPortcullis => t!("a portcullis"),
            Tex::Dirt => t!("a dirt floor"),
            Tex::Flagstones => t!("a flagstone floor"),
            Tex::FloorCobbles => t!("a cobblestone floor"),
            Tex::WoodFloor => t!("a wooden floor"),
            Tex::Grass => t!("a mossy floor"),
            Tex::Gravel => t!("a gravel floor"),
            Tex::MarbleTiles => t!("a tiled floor"),
            Tex::Mud => t!("a muddy floor"),
            Tex::CeilingRock => t!("a rocky ceiling"),
            Tex::CeilingBeams => t!("a timbered ceiling"),
            Tex::Water => t!("water"),
            Tex::Lava => t!("lava"),
            Tex::Bridge => t!("a wooden bridge"),
        }
    }
}

/// Shade value 0..1 -> index into a ramp, restricted to [lo, hi].
#[inline]
fn ramp_px(r: Ramp, v: f32, lo: i32, hi: i32) -> u8 {
    let s = lo as f32 + v.clamp(0.0, 1.0) * (hi - lo) as f32;
    c(r, s.round() as i32)
}

/// Same as ramp_px but with ordered dithering for smoother gradients.
#[inline]
fn ramp_px_d(r: Ramp, v: f32, lo: i32, hi: i32, x: i32, y: i32) -> u8 {
    let s = lo as f32 + v.clamp(0.0, 1.0) * (hi - lo) as f32 + (bayer4(x, y) - 0.5) * 0.9;
    c(r, (s.round() as i32).clamp(lo, hi))
}

pub fn generate(t: Tex, frame: usize) -> Bitmap {
    let n = t.size();
    let mut b = Bitmap::new(n, n);
    let seed = t as u32 * 7919 + 13;
    let f = n as f32;
    match t {
        Tex::RoughStone => blocks(&mut b, seed, BlockStyle {
            ramp: Ramp::Grey, rows: 3, min_w: 20, max_w: 40, mortar: 2, lo: 2, hi: 8,
            bevel: 0.30, grain: 0.30, moss: 0.25, irregular: 3.0, base_var: 0.25, base: 0.48,
        }),
        Tex::StoneBlocks => blocks(&mut b, seed, BlockStyle {
            ramp: Ramp::Grey, rows: 4, min_w: 16, max_w: 32, mortar: 1, lo: 2, hi: 8,
            bevel: 0.28, grain: 0.22, moss: 0.0, irregular: 1.0, base_var: 0.22, base: 0.5,
        }),
        Tex::MossyStone => blocks(&mut b, seed, BlockStyle {
            ramp: Ramp::Grey, rows: 4, min_w: 14, max_w: 30, mortar: 1, lo: 2, hi: 8,
            bevel: 0.25, grain: 0.25, moss: 0.65, irregular: 2.0, base_var: 0.25, base: 0.45,
        }),
        Tex::GreyBlocks => blocks(&mut b, seed, BlockStyle {
            ramp: Ramp::Slate, rows: 4, min_w: 22, max_w: 32, mortar: 1, lo: 2, hi: 9,
            bevel: 0.32, grain: 0.15, moss: 0.0, irregular: 0.0, base_var: 0.15, base: 0.55,
        }),
        Tex::Sandstone => blocks(&mut b, seed, BlockStyle {
            ramp: Ramp::Stone, rows: 4, min_w: 18, max_w: 34, mortar: 1, lo: 2, hi: 9,
            bevel: 0.25, grain: 0.25, moss: 0.0, irregular: 1.5, base_var: 0.18, base: 0.55,
        }),
        Tex::IrregularStone => irregular(&mut b, seed, Ramp::Stone, 5, 1.7, 2, 9),
        Tex::Cobbles => cobbles(&mut b, seed, Ramp::Grey, 6, true),
        Tex::Bricks => bricks(&mut b, seed, Ramp::Grey, 2, 8),
        Tex::RedBricks => bricks(&mut b, seed, Ramp::Copper, 2, 8),
        Tex::CaveRock => cave_rock(&mut b, seed),
        Tex::WoodPlanks => planks(&mut b, seed, true, 4),
        Tex::DarkPanels => panels(&mut b, seed),
        Tex::Marble => marble(&mut b, seed, Ramp::Grey, 3, 8),
        Tex::Plaque => {
            blocks(&mut b, seed, BlockStyle {
                ramp: Ramp::Grey, rows: 3, min_w: 20, max_w: 40, mortar: 2, lo: 2, hi: 8,
                bevel: 0.30, grain: 0.30, moss: 0.25, irregular: 3.0, base_var: 0.25, base: 0.48,
            });
            plaque(&mut b, seed);
        }
        Tex::WallLever => {
            blocks(&mut b, seed, BlockStyle {
                ramp: Ramp::Grey, rows: 4, min_w: 16, max_w: 32, mortar: 1, lo: 2, hi: 8,
                bevel: 0.28, grain: 0.22, moss: 0.0, irregular: 1.0, base_var: 0.22, base: 0.5,
            });
            lever_plate(&mut b);
        }
        Tex::CarvedFrieze => frieze(&mut b, seed),
        Tex::DoorWood => door_wood(&mut b, seed),
        Tex::DoorIron => door_iron(&mut b, seed),
        Tex::DoorPortcullis => portcullis(&mut b),
        Tex::Dirt => {
            for y in 0..n {
                for x in 0..n {
                    let (fx, fy) = (x as f32, y as f32);
                    let v = fbm2(fx, fy, f, 4, 4, seed) * 0.75 + hashf(x, y, 0, seed) * 0.35;
                    let mut px = ramp_px(Ramp::Earth, v, 3, 7);
                    let speck = hashf(x, y, 5, seed);
                    if speck > 0.93 {
                        px = c(Ramp::Earth, 8 + (speck * 10.0) as i32 % 2);
                    } else if speck < 0.05 {
                        px = c(Ramp::Earth, 2);
                    }
                    b.set(x, y, px);
                }
            }
        }
        Tex::Gravel => {
            for y in 0..n {
                for x in 0..n {
                    let cell = worley2(x as f32, y as f32, f, 8, 0.9, 1.0, seed);
                    let pebble = (1.0 - cell.f1 * 1.6).max(0.0);
                    let tone = hashf(cell.id as i32, 0, 0, seed);
                    let v = if cell.f2 - cell.f1 < 0.12 { 0.1 } else { 0.25 + pebble * 0.45 + tone * 0.25 };
                    let r = if tone > 0.6 { Ramp::Stone } else { Ramp::Grey };
                    b.set(x, y, ramp_px(r, v, 1, 7));
                }
            }
        }
        Tex::Flagstones => {
            for y in 0..n {
                for x in 0..n {
                    let cell = worley2(x as f32, y as f32, f, 2, 0.55, 1.0, seed);
                    let edge = cell.f2 - cell.f1;
                    let tone = hashf(cell.id as i32, 1, 0, seed);
                    let grain = fbm2(x as f32, y as f32, f, 8, 2, seed + 3);
                    let bevel = (-cell.dx - cell.dy) * 0.25;
                    let v = if edge < 0.07 { 0.05 } else { 0.35 + tone * 0.25 + grain * 0.25 - bevel };
                    b.set(x, y, ramp_px(Ramp::Grey, v, 1, 8));
                }
            }
        }
        Tex::FloorCobbles => cobbles(&mut b, seed, Ramp::Grey, 4, false),
        Tex::WoodFloor => planks(&mut b, seed, false, 4),
        Tex::Bridge => planks(&mut b, seed + 5, false, 3),
        Tex::Grass => {
            for y in 0..n {
                for x in 0..n {
                    let v = fbm2(x as f32, y as f32, f, 4, 3, seed) * 0.6 + hashf(x, y, 0, seed) * 0.4;
                    let r = if fbm2(x as f32, y as f32, f, 2, 2, seed + 9) > 0.55 { Ramp::Green } else { Ramp::Moss };
                    b.set(x, y, ramp_px(r, v, 1, 6));
                }
            }
        }
        Tex::MarbleTiles => {
            for y in 0..n {
                for x in 0..n {
                    let checker = ((x / 16) + (y / 16)) % 2 == 0;
                    let vein = (fbm2(x as f32, y as f32, f, 4, 4, seed) * 9.0).sin().abs();
                    let edge = x % 16 == 0 || y % 16 == 0;
                    let v = if edge { 0.15 } else if checker { 0.7 + vein * 0.2 } else { 0.25 + vein * 0.15 };
                    b.set(x, y, ramp_px(Ramp::Stone, v, 2, 8));
                }
            }
        }
        Tex::Mud => {
            for y in 0..n {
                for x in 0..n {
                    let v = fbm2(x as f32, y as f32, f, 4, 4, seed);
                    let puddle = fbm2(x as f32, y as f32, f, 2, 2, seed + 1) > 0.62;
                    let px = if puddle { ramp_px(Ramp::Moss, v * 0.5 + 0.2, 1, 5) } else { ramp_px(Ramp::Earth, v, 2, 6) };
                    b.set(x, y, px);
                }
            }
        }
        Tex::CeilingRock => {
            for y in 0..n {
                for x in 0..n {
                    let v = fbm2(x as f32, y as f32, f, 4, 4, seed) * 0.8 + hashf(x, y, 0, seed) * 0.25;
                    b.set(x, y, ramp_px(Ramp::Grey, v, 1, 5));
                }
            }
        }
        Tex::CeilingBeams => {
            for y in 0..n {
                for x in 0..n {
                    let beam = (y % 16) < 11;
                    let grain = vnoise2(x as f32 / 8.0, y as f32 * 0.9, 4, seed) * 0.5 + hashf(x, y, 0, seed) * 0.2;
                    let v = if !beam { 0.05 } else if y % 16 == 0 { 0.7 } else { 0.25 + grain };
                    b.set(x, y, ramp_px(Ramp::Earth, v, 1, 7));
                }
            }
        }
        Tex::Water => water(&mut b, seed, frame),
        Tex::Lava => lava(&mut b, seed, frame),
    }
    b
}

struct BlockStyle {
    ramp: Ramp,
    rows: i32,
    min_w: i32,
    max_w: i32,
    mortar: i32,
    lo: i32,
    hi: i32,
    bevel: f32,
    grain: f32,
    moss: f32,
    irregular: f32,
    base_var: f32,
    base: f32,
}

/// Rows of rectangular stone blocks with bevels and mortar.
fn blocks(b: &mut Bitmap, seed: u32, s: BlockStyle) {
    let n = b.w;
    let f = n as f32;
    let row_h = n / s.rows;
    // Pre-compute block boundaries for each row so they tile horizontally.
    let mut rows: Vec<Vec<i32>> = Vec::new();
    for r in 0..s.rows {
        let mut xs = vec![];
        let mut x = (hash3(r, 0, 0, seed) % 16) as i32;
        let start = x;
        loop {
            xs.push(x);
            let w = s.min_w + (hash3(r, xs.len() as i32, 1, seed) % ((s.max_w - s.min_w).max(1) as u32)) as i32;
            x += w;
            if x >= start + n - s.min_w / 2 {
                break;
            }
        }
        rows.push(xs);
    }
    for y in 0..n {
        for x in 0..n {
            // Irregular mortar lines: perturb sample position.
            let jx = (vnoise2(x as f32 / 6.0, y as f32 / 6.0, (n / 6).max(1), seed + 1) - 0.5) * s.irregular;
            let jy = (vnoise2(x as f32 / 6.0 + 9.0, y as f32 / 6.0, (n / 6).max(1), seed + 2) - 0.5) * s.irregular;
            let sx = (x as f32 + jx).rem_euclid(f);
            let sy = (y as f32 + jy).rem_euclid(f);
            let r = ((sy as i32) / row_h).min(s.rows - 1);
            let ry = sy - (r * row_h) as f32;
            let xs = &rows[r as usize];
            // Find block containing sx (with wrap).
            let mut bi = xs.len() - 1;
            let mut bx0 = xs[bi] - n;
            for (i, &bx) in xs.iter().enumerate() {
                if (sx as i32) >= bx {
                    bi = i;
                    bx0 = bx;
                }
            }
            let bx1 = if bi + 1 < xs.len() { xs[bi + 1] } else { xs[0] + n };
            let lx = sx - bx0 as f32;
            let w = (bx1 - bx0) as f32;
            let m = s.mortar as f32;
            let d_left = lx;
            let d_right = w - lx;
            let d_top = ry;
            let d_bot = row_h as f32 - ry;
            let dmin = d_left.min(d_right).min(d_top).min(d_bot);
            let block_id = hash3(r, bi as i32, 7, seed);
            let base = s.base + ((block_id % 1000) as f32 / 1000.0 - 0.5) * s.base_var * 2.0;
            let grain = (fbm2(x as f32, y as f32, f, 16, 2, seed + 3) - 0.5) * s.grain
                + (hashf(x, y, 3, seed) - 0.5) * s.grain * 0.8;
            let px = if dmin < m {
                c(s.ramp, s.lo.max(1) - 1 + ((hashf(x, y, 9, seed) * 2.0) as i32))
            } else {
                let mut v = base + grain;
                // Bevel: light from upper left.
                if d_top < m + 2.0 || d_left < m + 1.5 {
                    v += s.bevel;
                } else if d_bot < m + 2.0 || d_right < m + 1.5 {
                    v -= s.bevel;
                }
                // Gentle doming
                let cx = (lx / w - 0.5).abs();
                let cy = (ry / row_h as f32 - 0.5).abs();
                v -= (cx * cx + cy * cy) * 0.25;
                ramp_px(s.ramp, v, s.lo, s.hi)
            };
            b.set(x, y, px);
        }
    }
    if s.moss > 0.0 {
        moss(b, seed + 77, s.moss);
    }
}

/// Organic moss patches growing mostly from mortar lines.
fn moss(b: &mut Bitmap, seed: u32, amount: f32) {
    let n = b.w;
    let f = n as f32;
    let src = b.clone();
    for y in 0..n {
        for x in 0..n {
            let m = fbm2(x as f32, y as f32, f, 4, 3, seed);
            let thresh = 1.0 - amount * 0.55;
            if m > thresh {
                let dark = src.get(x, y);
                let lum = crate::gfx::palette::ramp_of(dark).map(|(_, s)| s).unwrap_or(4);
                let k = ((m - thresh) / (1.0 - thresh) * 4.0) as i32;
                let speck = hashf(x, y, 11, seed);
                if speck < 0.75 {
                    let r = if speck < 0.25 { Ramp::Green } else { Ramp::Moss };
                    b.set(x, y, c(r, (lum / 2 + k + 1).clamp(1, 9)));
                }
            }
        }
    }
}

/// Irregular stones from Worley cells, with top-left bevels.
fn irregular(b: &mut Bitmap, seed: u32, ramp: Ramp, cells: i32, aspect: f32, lo: i32, hi: i32) {
    let n = b.w;
    let f = n as f32;
    for y in 0..n {
        for x in 0..n {
            let cell = worley2(x as f32, y as f32, f, cells, 0.8, aspect, seed);
            let edge = cell.f2 - cell.f1;
            let tone = (cell.id % 997) as f32 / 997.0;
            let grain = (fbm2(x as f32, y as f32, f, 16, 2, seed + 3) - 0.5) * 0.3 + (hashf(x, y, 0, seed) - 0.5) * 0.2;
            let px = if edge < 0.08 {
                c(ramp, lo - 1)
            } else {
                // dx,dy point from sample to centre: sample is upper-left of centre when both positive.
                let lit = (cell.dx + cell.dy) * 0.55;
                let rim = if edge < 0.16 { if cell.dx + cell.dy > 0.0 { 0.2 } else { -0.25 } } else { 0.0 };
                ramp_px(ramp, 0.45 + tone * 0.25 + grain + lit * 0.4 + rim, lo, hi)
            };
            b.set(x, y, px);
        }
    }
}

/// Rounded cobbles with strong dome shading.
fn cobbles(b: &mut Bitmap, seed: u32, ramp: Ramp, cells: i32, mossy: bool) {
    let n = b.w;
    let f = n as f32;
    for y in 0..n {
        for x in 0..n {
            let cell = worley2(x as f32, y as f32, f, cells, 0.85, 1.25, seed);
            let edge = cell.f2 - cell.f1;
            let tone = (cell.id % 991) as f32 / 991.0;
            let dome = 1.0 - (cell.f1 * 1.5).min(1.0);
            let lit = (cell.dx + cell.dy) * 0.5;
            let grain = (hashf(x, y, 0, seed) - 0.5) * 0.18;
            let px = if edge < 0.10 {
                c(ramp, 1)
            } else {
                ramp_px(ramp, 0.2 + dome * 0.35 + lit * 0.45 + tone * 0.2 + grain, 1, 8)
            };
            b.set(x, y, px);
        }
    }
    if mossy {
        // Moss in the cracks.
        let src = b.clone();
        for y in 0..n {
            for x in 0..n {
                let cell = worley2(x as f32, y as f32, f, cells, 0.85, 1.25, seed);
                let edge = cell.f2 - cell.f1;
                let m = fbm2(x as f32, y as f32, f, 4, 2, seed + 5);
                if edge < 0.2 && m > 0.5 && hashf(x, y, 4, seed) < 0.7 {
                    let _ = src.get(x, y);
                    b.set(x, y, c(Ramp::Moss, 3 + (m * 6.0) as i32 % 4));
                }
            }
        }
    }
}

fn bricks(b: &mut Bitmap, seed: u32, ramp: Ramp, lo: i32, hi: i32) {
    let n = b.w;
    let f = n as f32;
    let bh = 8;
    let bw = 16;
    for y in 0..n {
        for x in 0..n {
            let row = y / bh;
            let off = if row % 2 == 0 { 0 } else { bw / 2 };
            let col = ((x + off) / bw) % (n / bw);
            let lx = (x + off) % bw;
            let ly = y % bh;
            let id = hash3(row, col, 0, seed);
            let tone = (id % 1000) as f32 / 1000.0;
            let grain = (hashf(x, y, 1, seed) - 0.5) * 0.2 + (fbm2(x as f32, y as f32, f, 8, 2, seed) - 0.5) * 0.2;
            let px = if ly == bh - 1 || lx == bw - 1 {
                c(ramp, lo - 1)
            } else {
                let mut v = 0.35 + tone * 0.3 + grain;
                if ly == 0 || lx == 0 {
                    v += 0.25;
                } else if ly == bh - 2 || lx == bw - 2 {
                    v -= 0.2;
                }
                ramp_px(ramp, v, lo, hi)
            };
            b.set(x, y, px);
        }
    }
}

fn cave_rock(b: &mut Bitmap, seed: u32) {
    let n = b.w;
    let f = n as f32;
    for y in 0..n {
        for x in 0..n {
            let h = fbm2(x as f32, y as f32, f, 4, 5, seed);
            let hx = fbm2(x as f32 + 1.0, y as f32, f, 4, 5, seed);
            let hy = fbm2(x as f32, y as f32 + 1.0, f, 4, 5, seed);
            // Lit from the upper left using the height gradient.
            let lit = (h - hx) * 6.0 + (h - hy) * 6.0;
            let crack = worley2(x as f32, y as f32, f, 3, 0.9, 0.7, seed + 1);
            let v = 0.35 + (h - 0.5) * 0.6 + lit + (hashf(x, y, 2, seed) - 0.5) * 0.15;
            let px = if crack.f2 - crack.f1 < 0.05 { c(Ramp::Stone, 1) } else { ramp_px(Ramp::Stone, v, 1, 7) };
            b.set(x, y, px);
        }
    }
}

fn planks(b: &mut Bitmap, seed: u32, vertical: bool, count: i32) {
    let n = b.w;
    let pw = n / count;
    for y in 0..n {
        for x in 0..n {
            let (along, across) = if vertical { (y, x) } else { (x, y) };
            let plank = across / pw;
            let la = across % pw;
            let id = hash3(plank, 0, 0, seed);
            let tone = (id % 1000) as f32 / 1000.0;
            let grain = vnoise2(along as f32 / 10.0, across as f32 * 1.3 + plank as f32 * 17.0, 64, seed) * 0.35
                + hashf(x, y, 1, seed) * 0.12;
            let joint = ((along + (id % 64) as i32) % n) == 0;
            let px = if la == pw - 1 || joint {
                c(Ramp::Earth, 1)
            } else if la == 0 {
                c(Ramp::Earth, 8)
            } else {
                ramp_px(Ramp::Earth, 0.3 + tone * 0.2 + grain, 3, 8)
            };
            b.set(x, y, px);
        }
    }
    // Nails
    for p in 0..count {
        let x = p * pw + pw / 2;
        for &yy in &[3, n - 4] {
            let (nx, ny) = if vertical { (x, yy) } else { (yy, x) };
            b.set(nx, ny, c(Ramp::Grey, 10));
            b.set(nx + 1, ny + 1, c(Ramp::Grey, 2));
        }
    }
}

fn panels(b: &mut Bitmap, seed: u32) {
    let n = b.w;
    for y in 0..n {
        for x in 0..n {
            let lx = x % 32;
            let grain = vnoise2(x as f32 * 0.7, y as f32 / 9.0, 64, seed) * 0.25 + hashf(x, y, 0, seed) * 0.1;
            let frame = lx < 3 || lx > 28 || y < 3 || y > 60;
            let v = if frame { 0.55 + grain } else { 0.3 + grain };
            let mut px = ramp_px(Ramp::Red, v * 0.8, 1, 6);
            if lx == 3 || y == 3 {
                px = c(Ramp::Red, 2);
            }
            if lx == 28 || y == 60 {
                px = c(Ramp::Red, 7);
            }
            b.set(x, y, px);
        }
    }
}

fn marble(b: &mut Bitmap, seed: u32, ramp: Ramp, lo: i32, hi: i32) {
    let n = b.w;
    let f = n as f32;
    for y in 0..n {
        for x in 0..n {
            let t = fbm2(x as f32, y as f32, f, 4, 5, seed);
            let vein = ((x as f32 * 0.05 + t * 6.0) * std::f32::consts::PI).sin();
            let v = 0.6 + vein * 0.25 + (hashf(x, y, 1, seed) - 0.5) * 0.1;
            let edge = y % 32 == 0 || (x + if y / 32 % 2 == 0 { 0 } else { 16 }) % 32 == 0;
            b.set(x, y, if edge { c(ramp, lo - 2) } else { ramp_px(ramp, v, lo, hi) });
        }
    }
}

/// A carved stone tablet with runic "writing" in the middle of the wall.
fn plaque(b: &mut Bitmap, seed: u32) {
    let (x0, y0, w, h) = (12, 18, 40, 26);
    for y in y0..y0 + h {
        for x in x0..x0 + w {
            let edge = x == x0 || y == y0 || x == x0 + w - 1 || y == y0 + h - 1;
            let v = if edge { 0.1 } else { 0.55 + (hashf(x, y, 0, seed) - 0.5) * 0.15 };
            b.set(x, y, ramp_px(Ramp::Stone, v, 2, 12));
        }
    }
    b.hline(x0 + 1, x0 + w - 2, y0 + 1, c(Ramp::Stone, 13));
    b.vline(x0 + 1, y0 + 1, y0 + h - 2, c(Ramp::Stone, 13));
    // Lines of glyphs
    for line in 0..4 {
        let y = y0 + 4 + line * 5;
        let mut x = x0 + 4;
        while x < x0 + w - 5 {
            let g = hash3(x, y, 3, seed) % 5;
            let col = c(Ramp::Stone, 3);
            match g {
                0 => {
                    b.vline(x, y, y + 3, col);
                }
                1 => {
                    b.vline(x, y, y + 3, col);
                    b.set(x + 1, y + 1, col);
                }
                2 => {
                    b.line(x, y + 3, x + 2, y, col);
                }
                3 => {
                    b.vline(x, y, y + 3, col);
                    b.vline(x + 2, y, y + 3, col);
                    b.set(x + 1, y + 1, col);
                }
                _ => {}
            }
            x += 3 + (g as i32 % 2);
        }
    }
}

/// An iron plate with a lever handle, set into the wall.
fn lever_plate(b: &mut Bitmap) {
    let (x0, y0, w, h) = (22, 22, 20, 24);
    for y in y0..y0 + h {
        for x in x0..x0 + w {
            let edge = x == x0 || y == y0 || x == x0 + w - 1 || y == y0 + h - 1;
            let v = if edge { 1 } else if x == x0 + 1 || y == y0 + 1 { 9 } else { 5 + ((x + y) % 3 == 0) as i32 };
            b.set(x, y, c(Ramp::Slate, v));
        }
    }
    for (x, y) in [(x0 + 3, y0 + 3), (x0 + w - 4, y0 + 3), (x0 + 3, y0 + h - 4), (x0 + w - 4, y0 + h - 4)] {
        b.set(x, y, c(Ramp::Slate, 12));
    }
    // Slot, handle and knob.
    for y in y0 + 6..y0 + h - 6 {
        b.set(x0 + w / 2, y, c(Ramp::Slate, 0));
    }
    let (px, py) = (x0 + w / 2, y0 + h / 2 + 2);
    for k in 0..12 {
        let x = px + k / 2;
        let y = py - k;
        b.set(x, y, c(Ramp::Grey, 9));
        b.set(x + 1, y, c(Ramp::Grey, 4));
    }
    b.fill_circle_f(px as f32 + 6.5, py as f32 - 12.0, 2.2, c(Ramp::Red, 7));
    b.set(px + 6, py - 13, c(Ramp::Red, 11));
    b.fill_circle_f(px as f32 + 0.5, py as f32 + 0.5, 1.6, c(Ramp::Slate, 10));
}

fn frieze(b: &mut Bitmap, seed: u32) {
    marble(b, seed, Ramp::Stone, 3, 8);
    // A band of carved circles and a sun motif.
    for x in 0..64 {
        b.set(x, 20, c(Ramp::Stone, 1));
        b.set(x, 21, c(Ramp::Stone, 9));
        b.set(x, 44, c(Ramp::Stone, 1));
        b.set(x, 45, c(Ramp::Stone, 9));
    }
    for k in 0..4 {
        let cx = 8.0 + k as f32 * 16.0;
        b.ellipse_outline(cx, 32.5, 6.0, 6.0, c(Ramp::Stone, 2));
        b.ellipse_outline(cx + 0.7, 33.2, 6.0, 6.0, c(Ramp::Stone, 9));
        b.fill_circle_f(cx, 32.5, 2.0, c(Ramp::Stone, 3));
    }
}

fn door_wood(b: &mut Bitmap, seed: u32) {
    planks(b, seed, true, 4);
    // Iron bands and studs.
    for &by in &[10, 50] {
        for y in by..by + 5 {
            for x in 0..64 {
                let v = if y == by { 0.8 } else if y == by + 4 { 0.1 } else { 0.4 + hashf(x, y, 1, seed) * 0.15 };
                b.set(x, y, ramp_px(Ramp::Slate, v, 1, 11));
            }
        }
        for x in (4..64).step_by(10) {
            b.set(x, by + 2, c(Ramp::Grey, 13));
            b.set(x + 1, by + 3, c(Ramp::Grey, 3));
        }
    }
    // Handle / ring
    b.ellipse_outline(50.0, 33.0, 3.5, 3.5, c(Ramp::Slate, 9));
    b.set(50, 29, c(Ramp::Slate, 12));
    // Frame shading
    for y in 0..64 {
        b.set(0, y, c(Ramp::Earth, 1));
        b.set(63, y, c(Ramp::Earth, 1));
    }
}

fn door_iron(b: &mut Bitmap, seed: u32) {
    for y in 0..64 {
        for x in 0..64 {
            let panel = (y % 21) == 0 || x == 0 || x == 63 || x == 31 || x == 32;
            let grain = fbm2(x as f32, y as f32, 64.0, 8, 3, seed) * 0.3 + hashf(x, y, 0, seed) * 0.1;
            let rust = fbm2(x as f32, y as f32, 64.0, 4, 3, seed + 1) > 0.66;
            let v = if panel { 0.12 } else { 0.35 + grain };
            let px = if rust && !panel { ramp_px(Ramp::Copper, v * 0.8, 1, 6) } else { ramp_px(Ramp::Slate, v, 1, 10) };
            b.set(x, y, px);
            if (y % 21) == 1 && !panel {
                b.set(x, y, c(Ramp::Slate, 9));
            }
        }
    }
    for y in (5..64).step_by(21) {
        for x in [4, 27, 36, 59] {
            b.set(x, y, c(Ramp::Grey, 13));
            b.set(x, y + 1, c(Ramp::Grey, 3));
        }
    }
}

fn portcullis(b: &mut Bitmap) {
    for y in 0..64 {
        for x in 0..64 {
            let bar_v = x % 12 < 3;
            let bar_h = y % 14 < 3;
            if bar_v || bar_h {
                let edge = x % 12 == 0 || y % 14 == 0;
                b.set(x, y, c(Ramp::Slate, if edge { 9 } else { 5 }));
            }
        }
    }
}

fn water(b: &mut Bitmap, seed: u32, frame: usize) {
    let n = b.w;
    let f = n as f32;
    let ph = frame as f32 / WATER_FRAMES as f32 * std::f32::consts::TAU;
    for y in 0..n {
        for x in 0..n {
            let fx = x as f32;
            let fy = y as f32;
            let w1 = ((fx / f * 2.0 + fy / f) * std::f32::consts::TAU + ph).sin();
            let w2 = ((fy / f * 3.0 - fx / f) * std::f32::consts::TAU - ph).sin();
            let nn = fbm2(fx, fy, f, 4, 3, seed);
            let v = 0.45 + w1 * 0.12 + w2 * 0.1 + (nn - 0.5) * 0.4;
            let mut px = ramp_px_d(Ramp::Blue, v, 1, 5, x, y);
            if w1 + w2 > 1.55 && nn > 0.45 {
                px = c(Ramp::Blue, 7);
            }
            b.set(x, y, px);
        }
    }
}

fn lava(b: &mut Bitmap, seed: u32, frame: usize) {
    let n = b.w;
    let f = n as f32;
    let ph = frame as f32 / LAVA_FRAMES as f32 * std::f32::consts::TAU;
    for y in 0..n {
        for x in 0..n {
            let fx = x as f32;
            let fy = y as f32;
            let cell = worley2(fx, fy, f, 3, 0.9, 1.0, seed);
            let edge = cell.f2 - cell.f1;
            let pulse = (ph + (cell.id % 7) as f32).sin() * 0.5 + 0.5;
            let nn = fbm2(fx, fy, f, 4, 3, seed + 1);
            let glow = 0.07 + pulse * 0.03;
            let px = if edge < glow * 0.45 {
                c(Ramp::Fire, 9 + (pulse * 2.0) as i32)
            } else if edge < glow {
                c(Ramp::Fire, 6 + (pulse * 2.0) as i32)
            } else if edge < glow * 1.9 {
                c(Ramp::Red, 5 + (pulse * 2.0) as i32)
            } else {
                // Dark crust with a few smouldering spots.
                let v = nn + (hashf(x, y, 3, seed) - 0.5) * 0.2;
                if v > 0.68 {
                    c(Ramp::Red, 3 + (pulse * 2.0) as i32)
                } else {
                    ramp_px(Ramp::Earth, v * 0.8, 1, 4)
                }
            };
            b.set(x, y, px);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_textures_generate_with_correct_size_and_no_holes() {
        for t in ALL_TEX {
            for fr in 0..t.frames() {
                let b = generate(t, fr);
                assert_eq!(b.w, t.size());
                assert_eq!(b.h, t.size());
                if t != Tex::DoorPortcullis {
                    assert!(b.px.iter().all(|&p| p != 0), "{:?} has transparent pixels", t);
                }
            }
        }
    }
}
