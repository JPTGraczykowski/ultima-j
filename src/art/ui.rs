//! Art for the main game screen: the ornate frame around the 3D view, the
//! command icons, compass, power gem, flasks, pull chain, message scroll and
//! mouse cursors - all drawn procedurally in the style of the original.

use super::canvas::{HCanvas, Mat};
use crate::gfx::palette::{c, Ramp, BLACK, CURSOR_DARK, CURSOR_YELLOW, PALE_YELLOW, SCROLL_BG, WHITE};
use crate::gfx::Bitmap;
use crate::layout::*;
use crate::util::{fbm2, hash3, hashf};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(usize)]
pub enum Cursor {
    Cross,
    Forward,
    Back,
    SlideLeft,
    SlideRight,
    TurnLeft,
    TurnRight,
    FwdLeft,
    FwdRight,
    Target,
    Spell,
}

pub struct UiArt {
    pub frame: Bitmap,
    pub icons: Vec<Bitmap>,
    pub compass: Vec<Bitmap>,
    pub gem: Vec<Bitmap>,
    pub flask_glass: Bitmap,
    pub flask_inside: Bitmap,
    pub cursors: Vec<(Bitmap, (i32, i32))>,
    pub slot: Bitmap,
    pub gargoyle_eye: [u8; 4],
    pub chain_bmp: Bitmap,
}

fn inside_rounded(x: i32, y: i32, r: Rect, rad: i32) -> bool {
    if !r.contains((x, y)) {
        return false;
    }
    let cx = if x < r.x + rad { r.x + rad } else if x >= r.x + r.w - rad { r.x + r.w - rad - 1 } else { x };
    let cy = if y < r.y + rad { r.y + rad } else if y >= r.y + r.h - rad { r.y + r.h - rad - 1 } else { y };
    let dx = (x - cx) as f32;
    let dy = (y - cy) as f32;
    dx * dx + dy * dy <= (rad as f32 + 0.3).powi(2)
}

/// Draw a bevelled, outlined region given an "inside" predicate.
fn bevel_region(b: &mut Bitmap, r: Rect, inside: &dyn Fn(i32, i32) -> bool, light: u8, dark: u8, outline: u8) {
    let src = b.clone();
    for y in r.y - 1..=r.y + r.h {
        for x in r.x - 1..=r.x + r.w {
            if !inside(x, y) {
                continue;
            }
            let edge = !inside(x - 1, y) || !inside(x + 1, y) || !inside(x, y - 1) || !inside(x, y + 1);
            if edge {
                b.set(x, y, outline);
                continue;
            }
            let tl = !inside(x - 2, y) || !inside(x, y - 2);
            let br = !inside(x + 2, y) || !inside(x, y + 2);
            if tl && !br {
                b.set(x, y, light);
            } else if br && !tl {
                b.set(x, y, dark);
            } else {
                b.set(x, y, src.get(x, y));
            }
        }
    }
}

/// Mottled copper panel like the original's icon and character panels.
fn copper_panel(b: &mut Bitmap, r: Rect, rad: i32, seed: u32, base: f32, streak: bool) {
    for y in r.y..r.y + r.h {
        for x in r.x..r.x + r.w {
            if !inside_rounded(x, y, r, rad) {
                continue;
            }
            let n = fbm2(x as f32, y as f32, 512.0, 64, 4, seed);
            let fine = hashf(x, y, 1, seed);
            let mut v = base + (n - 0.5) * 2.6 + (fine - 0.5) * 0.7;
            if streak {
                let d = (x - r.x) as f32 * 0.9 + (y - r.y) as f32 * 0.5 - 62.0;
                if d.abs() < 7.0 {
                    v += 1.3 * (1.0 - d.abs() / 7.0);
                }
                let d2 = d + 17.0;
                if d2.abs() < 3.0 {
                    v += 0.9 * (1.0 - d2.abs() / 3.0);
                }
            }
            let mut px = c(Ramp::Copper, v.round() as i32);
            // Occasional dark pits and a few patina flecks (left panel only).
            let p = fbm2(x as f32 + 100.0, y as f32, 512.0, 64, 3, seed + 3);
            if !streak && p > 0.72 && fine > 0.8 {
                px = c(Ramp::Moss, 3 + (fine * 2.0) as i32);
            } else if p < 0.28 && fine > 0.86 {
                px = c(Ramp::Copper, (v as i32 - 2).max(2));
            }
            b.set(x, y, px);
        }
    }
    let inside = |x: i32, y: i32| inside_rounded(x, y, r, rad);
    bevel_region(b, r, &inside, c(Ramp::Copper, 11), c(Ramp::Copper, 4), BLACK);
}

/// Slate metal fill with noise and occasional rust.
fn slate_fill(b: &mut Bitmap, r: Rect, seed: u32, lo: i32, hi: i32, rust: f32) {
    for y in r.y..r.y + r.h {
        for x in r.x..r.x + r.w {
            let n = fbm2(x as f32, y as f32, 512.0, 64, 3, seed);
            let fine = hashf(x, y, 2, seed);
            let v = lo as f32 + (hi - lo) as f32 * (0.5 + (n - 0.5) * 1.3 + (fine - 0.5) * 0.35);
            let mut px = c(Ramp::Slate, (v.round() as i32).clamp(lo, hi));
            let rr = fbm2(x as f32, y as f32 + 50.0, 512.0, 96, 3, seed + 9);
            if rr > 1.0 - rust && fine > 0.35 {
                px = c(Ramp::Copper, 3 + (fine * 4.0) as i32);
            }
            b.set(x, y, px);
        }
    }
}

/// Rough grey stone (ledges, compass housing).
fn stone_fill(b: &mut Bitmap, r: Rect, seed: u32, lo: i32, hi: i32) {
    for y in r.y..r.y + r.h {
        for x in r.x..r.x + r.w {
            let n = fbm2(x as f32, y as f32, 256.0, 64, 3, seed);
            let fine = hashf(x, y, 4, seed);
            let v = lo as f32 + (hi - lo) as f32 * (0.5 + (n - 0.5) * 1.2 + (fine - 0.5) * 0.7);
            b.set(x, y, c(Ramp::Grey, (v.round() as i32).clamp(lo, hi)));
        }
    }
}

fn frame_band_v(b: &mut Bitmap, x: i32, y0: i32, y1: i32, seed: u32) {
    // x = left edge of a 11-pixel vertical frame band: black, copper trim, slate, copper trim, black
    for y in y0..=y1 {
        b.set(x, y, BLACK);
        b.set(x + 1, y, c(Ramp::Copper, 9));
        b.set(x + 2, y, c(Ramp::Copper, 5));
        b.set(x + 8, y, c(Ramp::Copper, 10));
        b.set(x + 9, y, c(Ramp::Copper, 5));
        b.set(x + 10, y, BLACK);
    }
    slate_fill(b, Rect::new(x + 3, y0, 5, y1 - y0 + 1), seed, 5, 9, 0.12);
    for y in y0..=y1 {
        b.set(x + 3, y, c(Ramp::Slate, 10));
        b.set(x + 7, y, c(Ramp::Slate, 4));
    }
}

fn frame_band_h(b: &mut Bitmap, x0: i32, x1: i32, y: i32, seed: u32) {
    for x in x0..=x1 {
        b.set(x, y, BLACK);
        b.set(x, y + 1, c(Ramp::Copper, 9));
        b.set(x, y + 2, c(Ramp::Copper, 5));
        b.set(x, y + 8, c(Ramp::Copper, 10));
        b.set(x, y + 9, c(Ramp::Copper, 5));
        b.set(x, y + 10, BLACK);
    }
    slate_fill(b, Rect::new(x0, y + 3, x1 - x0 + 1, 5), seed, 5, 9, 0.12);
    for x in x0..=x1 {
        b.set(x, y + 3, c(Ramp::Slate, 10));
        b.set(x, y + 7, c(Ramp::Slate, 4));
    }
}

fn gargoyle() -> Bitmap {
    let mut cv = HCanvas::new(28, 21);
    let skin = cv.add_mat(Mat::new(Ramp::Copper, 2, 10).noise(0.22));
    let dark = cv.add_mat(Mat::new(Ramp::Earth, 0, 3).noise(0.1));
    let tooth = cv.add_mat(Mat::new(Ramp::Parch, 8, 15));
    // Pointed ears.
    cv.poly(&[(1.0, 4.0), (6.0, 7.0), (5.0, 12.0)], 0.0, 1.2, skin);
    cv.poly(&[(27.0, 4.0), (22.0, 7.0), (23.0, 12.0)], 0.0, 1.2, skin);
    // Broad face.
    cv.ellipse(14.0, 10.5, 10.0, 9.5, 0.0, 0.7, skin);
    // Heavy brow ridge.
    cv.capsule((6.0, 6.5), (13.0, 8.0), 2.3, 2.0, 5.0, 1.0, skin);
    cv.capsule((15.0, 8.0), (22.0, 6.5), 2.0, 2.3, 5.0, 1.0, skin);
    // Cheekbones and nose.
    cv.ellipse(8.0, 12.5, 3.6, 2.6, 3.0, 1.0, skin);
    cv.ellipse(20.0, 12.5, 3.6, 2.6, 3.0, 1.0, skin);
    cv.capsule((14.0, 8.5), (14.0, 13.0), 1.6, 2.8, 6.0, 1.0, skin);
    // Grinning mouth.
    cv.poly(&[(6.5, 15.0), (21.5, 15.0), (19.5, 19.0), (8.5, 19.0)], 2.0, 0.0, dark);
    for k in 0..6 {
        let x = 8.5 + k as f32 * 2.2;
        cv.poly(&[(x, 15.2), (x + 1.6, 15.2), (x + 0.8, 17.2)], 3.0, 0.0, tooth);
    }
    let mut b = cv.render(true, true);
    // Deep eye sockets (the eyes glow during combat).
    for &(x, y) in &[(9, 10), (10, 10), (17, 10), (18, 10)] {
        b.set(x, y, BLACK);
        b.set(x, y + 1, c(Ramp::Copper, 1));
    }
    b.set(10, 9, c(Ramp::Slate, 9));
    b.set(18, 9, c(Ramp::Slate, 9));
    b
}

/// The green dragon that winds around the left side of the view frame.
fn dragon() -> Bitmap {
    let mut cv = HCanvas::new(44, 114);
    let scale = cv.add_mat(Mat::new(Ramp::Green, 2, 10).noise(0.35));
    let belly = cv.add_mat(Mat::new(Ramp::Moss, 3, 11).noise(0.2));
    let claw = cv.add_mat(Mat::new(Ramp::Stone, 4, 13));
    // A thin serpentine body: a sine wave around the frame band (canvas x ~ 10).
    let mut pts: Vec<(f32, f32)> = Vec::new();
    for i in 0..=48 {
        let t = i as f32 / 48.0;
        let y = 4.0 + t * 88.0;
        let x = 10.0 + (t * 3.2 * std::f32::consts::TAU).sin() * 7.5;
        pts.push((x, y));
    }
    let mut visible: Vec<bool> = Vec::new();
    for i in 0..pts.len() {
        // Hide segments where the body passes behind the band (moving right to left).
        let dx = if i + 1 < pts.len() { pts[i + 1].0 - pts[i].0 } else { 1.0 };
        let behind = dx < -0.2 && (pts[i].0 - 10.0).abs() < 4.5 && i < 44;
        visible.push(!behind);
    }
    for i in 0..pts.len() - 1 {
        if !visible[i] || !visible[i + 1] {
            continue;
        }
        let t = i as f32 / 48.0;
        let r0 = 1.2 + t * 2.4;
        let r1 = 1.2 + (t + 1.0 / 48.0) * 2.4;
        cv.capsule(pts[i], pts[i + 1], r0, r1, 0.0, 0.9, scale);
    }
    let end = *pts.last().unwrap();
    // Neck bending towards the centre.
    cv.capsule(end, (14.0, 96.0), 3.6, 3.4, 1.0, 0.9, scale);
    // Back leg and claws
    cv.capsule((6.0, 98.0), (5.0, 108.0), 2.6, 1.8, 0.0, 0.9, scale);
    cv.ellipse(6.5, 109.0, 3.6, 1.8, 1.0, 0.7, claw);
    // Head with open jaws.
    cv.ellipse(18.0, 95.0, 6.5, 5.0, 3.0, 0.9, scale);
    cv.capsule((20.0, 95.5), (33.0, 97.5), 3.4, 2.2, 3.0, 0.9, scale);
    cv.capsule((18.0, 100.0), (31.0, 103.5), 2.0, 1.3, 2.0, 0.9, belly);
    cv.capsule((14.0, 92.0), (22.0, 92.5), 1.5, 1.4, 6.0, 1.0, scale);
    cv.capsule((14.0, 91.0), (7.0, 86.0), 1.6, 0.6, 5.0, 1.0, claw);
    // Front leg gripping the ledge
    cv.capsule((15.0, 100.0), (22.0, 107.0), 2.6, 1.9, 2.0, 0.9, scale);
    cv.ellipse(24.0, 108.5, 3.8, 1.8, 2.0, 0.7, claw);
    let mut b = cv.render(true, true);
    // Orange spots along the spine.
    for i in (2..pts.len() - 1).step_by(3) {
        if visible[i] {
            let (x, y) = pts[i];
            b.set(x.round() as i32, y.round() as i32, c(Ramp::Gold, 11));
        }
    }
    // eye, nostril, teeth
    b.set(20, 93, CURSOR_YELLOW);
    b.set(21, 93, c(Ramp::Gold, 12));
    b.set(20, 94, c(Ramp::Gold, 6));
    b.set(32, 96, BLACK);
    for x in (21..31).step_by(2) {
        b.set(x, 101, WHITE);
    }
    for (x, y) in [(3, 110), (6, 111), (9, 110), (21, 110), (24, 111), (27, 110)] {
        b.set(x, y, c(Ramp::Stone, 14));
    }
    b
}

fn icon_options() -> Bitmap {
    let mut cv = HCanvas::new(ICON_W, ICON_H);
    let m = cv.add_mat(Mat::new(Ramp::Silver, 4, 15).spec(0.4));
    let cx = 15.0;
    let cy = 9.0;
    cv.ring(cx, cy, 11.0, 6.4, 1.7, 0.0, m);
    cv.ring(cx, cy, 5.5, 3.2, 1.1, 0.0, m);
    cv.capsule((4.0, cy), (26.0, cy), 0.9, 0.9, 0.5, 1.0, m);
    cv.capsule((cx, 3.5), (cx, 14.5), 0.9, 0.9, 0.5, 1.0, m);
    cv.ellipse(cx, cy, 1.8, 1.8, 2.0, 1.0, m);
    cv.render(true, true)
}

fn icon_talk() -> Bitmap {
    let mut cv = HCanvas::new(ICON_W, ICON_H);
    let m = cv.add_mat(Mat::new(Ramp::Silver, 3, 15).spec(0.3));
    let upper = [(2.0, 9.0), (7.0, 5.5), (11.5, 4.0), (15.0, 6.0), (18.5, 4.0), (23.0, 5.5), (28.0, 9.0), (15.0, 9.4)];
    cv.poly(&upper, 0.0, 2.5, m);
    cv.ellipse(15.0, 10.6, 11.0, 4.2, 0.0, 0.9, m);
    let mut b = cv.render(true, true);
    for x in 4..27 {
        let y = if (x - 15i32).abs() < 9 { 9 } else { 8 + ((x - 15i32).abs() > 11) as i32 };
        b.set(x, y, BLACK);
    }
    b
}

fn icon_get() -> Bitmap {
    let mut cv = HCanvas::new(ICON_W, ICON_H);
    let m = cv.add_mat(Mat::new(Ramp::Silver, 3, 15).spec(0.3));
    // forearm from the upper right, palm, and fingers curling down to grab
    cv.capsule((29.0, 2.0), (21.0, 5.0), 3.2, 3.0, 0.0, 0.8, m);
    cv.ellipse(15.0, 6.5, 7.0, 4.6, 1.0, 0.8, m);
    cv.capsule((10.0, 8.0), (6.0, 12.5), 1.8, 1.5, 3.0, 1.0, m); // thumb
    cv.capsule((6.0, 12.5), (8.5, 15.0), 1.5, 1.2, 3.0, 1.0, m);
    cv.capsule((12.0, 10.0), (11.0, 14.0), 1.7, 1.4, 2.0, 1.0, m);
    cv.capsule((11.0, 14.0), (13.0, 16.0), 1.4, 1.1, 2.0, 1.0, m);
    cv.capsule((15.5, 10.5), (15.5, 15.0), 1.7, 1.4, 2.5, 1.0, m);
    cv.capsule((19.0, 10.0), (20.0, 14.5), 1.6, 1.3, 2.0, 1.0, m);
    cv.capsule((22.0, 8.5), (23.5, 12.0), 1.4, 1.2, 1.5, 1.0, m);
    cv.render(true, true)
}

fn icon_look() -> Bitmap {
    let mut cv = HCanvas::new(ICON_W, ICON_H);
    let white = cv.add_mat(Mat::new(Ramp::Silver, 5, 15).spec(0.2));
    let iris = cv.add_mat(Mat::new(Ramp::Silver, 2, 9).spec(0.5));
    let almond = [(1.5, 9.0), (6.0, 5.0), (15.0, 3.2), (24.0, 5.0), (28.5, 9.0), (24.0, 13.0), (15.0, 14.8), (6.0, 13.0)];
    cv.poly(&almond, 0.0, 2.0, white);
    cv.ellipse(15.0, 9.0, 4.6, 4.6, 1.5, 0.8, iris);
    let mut b = cv.render(true, true);
    b.fill_circle_f(15.0, 9.0, 1.9, BLACK);
    b.set(13, 7, WHITE);
    // eyelid crease
    for x in 6..25 {
        let t = (x as f32 - 15.0) / 10.0;
        let y = (2.6 + t * t * 2.5).round() as i32;
        b.set(x, y, c(Ramp::Silver, 5));
    }
    b
}

fn icon_fight() -> Bitmap {
    let mut cv = HCanvas::new(ICON_W, ICON_H);
    let blade = cv.add_mat(Mat::new(Ramp::Silver, 6, 15).spec(0.8));
    let hilt = cv.add_mat(Mat::new(Ramp::Silver, 3, 12).spec(0.3));
    let skin = cv.add_mat(Mat::new(Ramp::Silver, 3, 14).spec(0.2));
    // blade pointing to the upper right
    cv.poly(&[(12.0, 9.0), (27.5, 0.8), (29.0, 1.5), (14.0, 11.0)], 2.0, 1.0, blade);
    // cross guard
    cv.capsule((10.0, 6.0), (15.0, 13.5), 1.2, 1.2, 3.0, 1.0, hilt);
    // fist around the grip
    cv.ellipse(8.5, 11.5, 5.4, 4.6, 2.0, 0.9, skin);
    cv.capsule((5.0, 9.0), (10.0, 8.0), 1.4, 1.4, 4.0, 1.0, skin);
    // pommel
    cv.ellipse(3.0, 16.0, 1.6, 1.6, 1.0, 1.0, hilt);
    let mut b = cv.render(true, true);
    for &(x, y) in &[(7, 11), (8, 12), (9, 13), (6, 13)] {
        b.set(x, y, c(Ramp::Silver, 3));
    }
    b
}

fn icon_use() -> Bitmap {
    let mut cv = HCanvas::new(ICON_W, ICON_H);
    let m = cv.add_mat(Mat::new(Ramp::Silver, 3, 15).spec(0.3));
    let cuff = cv.add_mat(Mat::new(Ramp::Silver, 2, 10).spec(0.2));
    cv.capsule((1.5, 9.5), (6.0, 9.5), 3.8, 3.8, 0.0, 0.8, cuff);
    cv.ellipse(11.0, 10.0, 5.8, 4.6, 1.0, 0.8, m);
    cv.capsule((14.0, 7.5), (28.0, 6.5), 1.7, 1.4, 3.0, 1.0, m); // index finger
    cv.capsule((14.0, 10.5), (19.0, 10.5), 1.5, 1.3, 2.5, 1.0, m);
    cv.capsule((13.5, 13.0), (18.0, 13.2), 1.5, 1.3, 2.0, 1.0, m);
    cv.capsule((12.0, 15.0), (16.0, 15.3), 1.3, 1.1, 1.5, 1.0, m);
    cv.capsule((10.0, 6.0), (16.0, 4.6), 1.4, 1.2, 4.0, 1.0, m); // thumb
    cv.render(true, true)
}

/// Compass rose for one of 16 headings (index 0 = facing north).
fn compass_rose(dir: usize) -> Bitmap {
    let w = 64;
    let h = 32;
    let mut cv = HCanvas::new(w, h);
    let gold = cv.add_mat(Mat::new(Ramp::Gold, 2, 12).spec(0.5).noise(0.1));
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    let r = COMPASS_R;
    let squash = 0.46;
    // Heading: the rose rotates so that its north arm points to world north
    // relative to the player's view (north is "away" when facing north).
    let a0 = -(dir as f32) * std::f32::consts::TAU / 16.0;
    let arm: [(f32, f32); 7] = [(-0.2, 0.0), (-0.14, 0.5), (-0.4, 1.0), (0.0, 0.82), (0.4, 1.0), (0.14, 0.5), (0.2, 0.0)];
    for k in 0..4 {
        let a = a0 + k as f32 * std::f32::consts::FRAC_PI_2;
        let (s, co) = a.sin_cos();
        let pts: Vec<(f32, f32)> = arm
            .iter()
            .map(|&(ax, ay)| {
                // local: x across, y along the arm (away from the centre)
                let wx = ax * co + ay * s;
                let wy = -ay * co + ax * s;
                (cx + wx * r, cy + wy * r * squash)
            })
            .collect();
        cv.poly(&pts, 0.0, 2.2, gold);
    }
    cv.ellipse(cx, cy, 6.5, 3.4, 2.0, 1.0, gold);
    let mut b = cv.render(true, true);
    // red north dot at the tip of the north arm
    let (s, co) = a0.sin_cos();
    let nx = cx + s * r * 0.86;
    let ny = cy - co * r * 0.86 * squash;
    b.fill_circle_f(nx, ny, 1.4, c(Ramp::Red, 11));
    b.set(nx as i32, ny as i32, c(Ramp::Red, 14));
    b
}

fn gem(state: usize) -> Bitmap {
    let mut cv = HCanvas::new(32, 14);
    let m = match state {
        0 => Mat::new(Ramp::Slate, 2, 10).spec(0.6),
        1 => Mat::new(Ramp::Red, 3, 13).spec(0.7),
        2 => Mat::new(Ramp::Gold, 4, 14).spec(0.7),
        _ => Mat::new(Ramp::Green, 3, 13).spec(0.7),
    };
    let mm = cv.add_mat(m);
    cv.ellipse(16.0, 9.0, 14.5, 7.5, 0.0, 0.7, mm);
    let mut b = cv.render(false, false);
    // keep only the top part (the dome sits in the rim)
    for y in 11..14 {
        for x in 0..32 {
            b.set(x, y, 0);
        }
    }
    if state == 4 {
        for &(x, y) in &[(10, 4), (17, 3), (22, 6), (13, 7), (19, 8)] {
            b.set(x, y, WHITE);
            b.set(x + 1, y, PALE_YELLOW);
            b.set(x - 1, y, PALE_YELLOW);
            b.set(x, y - 1, PALE_YELLOW);
            b.set(x, y + 1, PALE_YELLOW);
        }
    }
    b
}

fn flask_parts() -> (Bitmap, Bitmap) {
    // Glass overlay (highlights, neck, stopper, outline) and the inside mask.
    let w = 28;
    let h = 36;
    let mut glass = Bitmap::new(w, h);
    let mut inside = Bitmap::new(w, h);
    let cx = 14.0;
    let cy = 23.0;
    let r = FLASK_R as f32;
    for y in 0..h {
        for x in 0..w {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            let d = (dx * dx + dy * dy).sqrt();
            let in_neck = (dx.abs() < 3.0) && y >= 3 && y < 14;
            if d < r - 0.6 || (in_neck && y >= 5) {
                inside.set(x, y, 1);
            }
        }
    }
    // outline
    let o = inside.outlined(1, false);
    for y in 0..h {
        for x in 0..w {
            if o.get(x, y) == 1 && inside.get(x, y) == 0 {
                glass.set(x, y, c(Ramp::Slate, 3));
            }
        }
    }
    // stopper / rim
    for x in 10..18 {
        glass.set(x, 3, c(Ramp::Slate, 6));
        glass.set(x, 4, c(Ramp::Slate, 9));
    }
    // highlights on the glass
    for &(x, y) in &[(8, 17), (7, 18), (7, 19), (6, 21), (6, 22), (12, 7), (12, 8), (12, 9)] {
        glass.set(x, y, c(Ramp::Silver, 14));
    }
    // gold tripod stand
    for x in 4..25 {
        glass.set(x, 33, c(Ramp::Gold, 9));
        glass.set(x, 34, c(Ramp::Gold, 5));
    }
    for &x in &[5, 14, 23] {
        glass.set(x, 35, c(Ramp::Gold, 7));
        glass.set(x, 32, c(Ramp::Gold, 11));
    }
    (glass, inside)
}

fn chain() -> Bitmap {
    let mut cv = HCanvas::new(CHAIN.w, CHAIN.h);
    let m = cv.add_mat(Mat::new(Ramp::Slate, 3, 13).spec(0.5));
    let mx = CHAIN.w as f32 / 2.0;
    cv.ring(mx, 2.5, 1.8, 2.3, 0.8, 0.0, m);
    cv.capsule((mx, 5.0), (mx, 10.0), 0.9, 0.9, 0.0, 1.0, m);
    cv.ring(mx, 11.0, 1.6, 2.4, 0.8, 0.0, m);
    cv.ring(mx, 20.0, 4.0, 4.8, 1.0, 0.0, m);
    cv.render(true, false)
}

fn slot_ring() -> Bitmap {
    let mut b = Bitmap::new(17, 17);
    b.ellipse_outline(8.5, 8.5, 8.0, 8.0, c(Ramp::Copper, 4));
    b.ellipse_outline(9.0, 9.0, 7.5, 7.5, c(Ramp::Copper, 9));
    b.ellipse_outline(8.5, 8.5, 7.0, 7.0, c(Ramp::Copper, 5));
    b
}

fn cursor_from(rows: &[&str]) -> Bitmap {
    let key = [
        ('#', CURSOR_DARK),
        ('k', BLACK),
        ('Y', c(Ramp::Gold, 14)),
        ('y', c(Ramp::Gold, 11)),
        ('o', c(Ramp::Gold, 8)),
        ('w', WHITE),
        ('r', c(Ramp::Red, 10)),
        ('b', c(Ramp::Blue, 10)),
    ];
    Bitmap::from_ascii(rows, &key)
}

fn cursors() -> Vec<(Bitmap, (i32, i32))> {
    let cross = cursor_from(&[
        "...kkk...",
        "...kYk...",
        "...kyk...",
        "kkkk.kkkk",
        "kYy...yok",
        "kkkk.kkkk",
        "...kyk...",
        "...kok...",
        "...kkk...",
    ]);
    let up = cursor_from(&[
        ".....k.....",
        "....kYk....",
        "...kYyok...",
        "..kYyyook..",
        ".kYyyyyook.",
        "kkkkyyokkkk",
        "...kyyok...",
        "...kyyok...",
        "...kyook...",
        "...kkkkk...",
    ]);
    let down = up.sub(0, 0, up.w, up.h);
    let mut down_f = Bitmap::new(down.w, down.h);
    for y in 0..down.h {
        for x in 0..down.w {
            down_f.set(x, down.h - 1 - y, down.get(x, y));
        }
    }
    let right = cursor_from(&[
        ".....k....",
        ".....kk...",
        "kkkkkkYk..",
        "kYyyyyyyk.",
        "kyyyyyyook",
        "koooooook.",
        "kkkkkkok..",
        ".....kk...",
        ".....k....",
    ]);
    let fwd_right = cursor_from(&[
        "......k....",
        "......kk...",
        ".kkkkkkYk..",
        "kYyyyyyyyk.",
        "kyyyyyyyook",
        "kyookkkkok.",
        "kyok..kkk..",
        "kyok..k....",
        "kyok.......",
        "kyok.......",
        "kkkk.......",
    ]);
    let turn_right = cursor_from(&[
        "...kkkkk.....",
        "..kYyyyyk....",
        ".kYykkkyyk...",
        "kYyk...kyyk.k",
        "kyk.....kyykk",
        "kyk....kkyyok",
        "kyk....kyyyok",
        "kok....kooook",
        "kkk....kkkkkk",
    ]);
    let target = cursor_from(&[
        "...rrrrr...",
        "..rr...rr..",
        ".rr.....rr.",
        "rr...k...rr",
        "r...kwk...r",
        "rr...k...rr",
        ".rr.....rr.",
        "..rr...rr..",
        "...rrrrr...",
    ]);
    let spell = cursor_from(&[
        "....b....",
        "....b....",
        "....b....",
        "....w....",
        "bbbw.wbbb",
        "....w....",
        "....b....",
        "....b....",
        "....b....",
    ]);
    let mk = |b: Bitmap| {
        let hs = (b.w / 2, b.h / 2);
        (b, hs)
    };
    vec![
        mk(cross),
        mk(up),
        mk(down_f),
        mk(right.flipped_x()),
        mk(right),
        mk(turn_right.flipped_x()),
        mk(turn_right),
        mk(fwd_right.flipped_x()),
        mk(fwd_right),
        mk(target),
        mk(spell),
    ]
}

/// Small arrows on the compass base (easy move buttons).
fn move_arrow(kind: usize) -> Bitmap {
    let rows: &[&str] = match kind {
        0 => &[
            "....kk.....",
            "...kwk.....",
            "..kwwkkkkk.",
            ".kwwwwwwwk.",
            "..kwwkkkwk.",
            "...kwk.kwk.",
            "....kk.kwk.",
            ".......kwk.",
            ".......kkk.",
        ],
        1 => &[
            "....k....",
            "...kwk...",
            "..kwwwk..",
            ".kwwwwwk.",
            "kkkwwwkkk",
            "..kwwwk..",
            "..kwwwk..",
            "..kwwwk..",
            "..kkkkk..",
        ],
        _ => &[
            ".....kk....",
            ".....kwk...",
            ".kkkkkwwk..",
            ".kwwwwwwwk.",
            ".kwkkkwwk..",
            ".kwk.kwk...",
            ".kwk.kk....",
            ".kwk.......",
            ".kkk.......",
        ],
    };
    let key = [('k', BLACK), ('w', c(Ramp::Silver, 14))];
    Bitmap::from_ascii(rows, &key)
}

pub fn build_frame() -> Bitmap {
    let mut b = Bitmap::filled(320, 200, BLACK);
    let seed = 4242;

    // Dark wall behind the flasks and power gem.
    for y in 118..150 {
        for x in 0..320 {
            if (x + y) % 2 == 0 && hashf(x, y, 5, seed) > 0.55 {
                b.set(x, y, c(Ramp::Grey, 1 + (hashf(x, y, 6, seed) * 2.0) as i32));
            }
        }
    }
    // Stone ledges (left under the gem, right under the flasks).
    for &(x0, x1) in &[(0, 42), (233, 320)] {
        stone_fill(&mut b, Rect::new(x0, 150, x1 - x0, 16), seed + x0 as u32, 2, 6);
        for x in x0..x1 {
            b.set(x, 150, c(Ramp::Grey, 7));
            for y in 158..161 {
                let v = 7 + (hashf(x, y, 9, seed) * 3.0) as i32;
                b.set(x, y, c(Ramp::Grey, v));
            }
            b.set(x, 161, c(Ramp::Grey, 3));
            b.set(x, 165, BLACK);
        }
    }

    // Copper panels.
    copper_panel(&mut b, ICON_PANEL, 4, seed + 1, 6.4, false);
    copper_panel(&mut b, PANEL, 4, seed + 2, 7.6, true);
    // Icon dimples on the left panel.
    for &(cx, cy) in ICON_CENTERS.iter() {
        let (fx, fy) = (cx as f32 + 0.5, cy as f32 + 0.5);
        for y in cy - 10..=cy + 10 {
            for x in cx - 17..=cx + 17 {
                let dx = (x as f32 + 0.5 - fx) / 16.0;
                let dy = (y as f32 + 0.5 - fy) / 9.5;
                let d = dx * dx + dy * dy;
                if d < 1.0 && d > 0.78 {
                    let col = if dx + dy * 1.4 < 0.0 { c(Ramp::Copper, 4) } else { c(Ramp::Copper, 10) };
                    b.set(x, y, col);
                }
            }
        }
    }

    // Lower frame panel (below the view, around the compass).
    slate_fill(&mut b, Rect::new(42, 129, 192, 37), seed + 3, 5, 9, 0.18);
    let lower = Rect::new(42, 129, 192, 37);
    let inside = |x: i32, y: i32| lower.contains((x, y));
    bevel_region(&mut b, lower, &inside, c(Ramp::Slate, 11), c(Ramp::Slate, 3), BLACK);

    // The view frame bands.
    frame_band_v(&mut b, 41, 8, 136, seed + 4);
    frame_band_v(&mut b, 224, 8, 136, seed + 5);
    frame_band_h(&mut b, 41, 234, 8, seed + 6);
    frame_band_h(&mut b, 41, 234, 128, seed + 7);
    // Corner bosses.
    for &(x, y) in &[(41, 8), (224, 8), (41, 128), (224, 128)] {
        for yy in 0..11 {
            for xx in 0..11 {
                let edge = xx == 0 || yy == 0 || xx == 10 || yy == 10;
                let col = if edge {
                    BLACK
                } else if xx == 1 || yy == 1 {
                    c(Ramp::Copper, 11)
                } else if xx == 9 || yy == 9 {
                    c(Ramp::Copper, 4)
                } else {
                    c(Ramp::Copper, 7 + ((xx + yy) % 3 == 0) as i32)
                };
                b.set(x + xx, y + yy, col);
            }
        }
        b.fill_circle_f(x as f32 + 5.5, y as f32 + 5.5, 1.8, c(Ramp::Slate, 9));
        b.set(x + 4, y + 4, c(Ramp::Slate, 13));
    }
    // Rivets along the bands.
    for k in 0..12 {
        let y = 22 + k * 9;
        for &x in &[46, 229] {
            b.set(x, y, c(Ramp::Slate, 13));
            b.set(x + 1, y + 1, c(Ramp::Slate, 2));
        }
    }
    for k in 0..18 {
        let x = 56 + k * 10;
        if (120..152).contains(&x) {
            continue;
        }
        b.set(x, 13, c(Ramp::Slate, 13));
        b.set(x + 1, 14, c(Ramp::Slate, 2));
    }

    // Iron rods along the top edge.
    for x in 2..318 {
        if (124..149).contains(&x) {
            continue;
        }
        b.set(x, 2, c(Ramp::Slate, 7));
        b.set(x, 3, c(Ramp::Slate, 4));
        b.set(x, 1, c(Ramp::Slate, 9));
    }
    for &x in &[40, 120, 152, 234] {
        b.fill_rect(x - 2, 0, 5, 6, c(Ramp::Copper, 5));
        b.rect(x - 2, 0, 5, 6, BLACK);
        b.set(x - 1, 1, c(Ramp::Copper, 10));
    }

    // Gargoyle.
    let g = gargoyle();
    b.blit(&g, GARGOYLE_C.0 - g.w / 2, 0);

    // Slots: current spells (left) and rune shelf (right).
    for r in [SPELLS, RUNE_SHELF] {
        b.fill_rect(r.x, r.y, r.w, r.h, c(Ramp::Slate, 3));
        for x in r.x..r.x + r.w {
            b.set(x, r.y, BLACK);
            b.set(x, r.y + r.h - 1, c(Ramp::Slate, 10));
        }
        for y in r.y..r.y + r.h {
            b.set(r.x, y, BLACK);
            b.set(r.x + r.w - 1, y, c(Ramp::Slate, 10));
        }
    }
    // Rune shelf dividers.
    for k in 1..3 {
        let x = RUNE_SHELF.x + k * RUNE_SHELF.w / 3;
        b.vline(x, RUNE_SHELF.y + 1, RUNE_SHELF.y + RUNE_SHELF.h - 2, c(Ramp::Slate, 1));
    }

    // Compass housing: a grey stone block with an oval top.
    let (ccx, ccy) = COMPASS_C;
    let housing = Rect::new(ccx - 32, 131, 64, 35);
    stone_fill(&mut b, housing, seed + 8, 3, 8);
    let hin = |x: i32, y: i32| {
        if !housing.contains((x, y)) {
            return false;
        }
        // chamfered top corners
        let dx = (x - housing.x).min(housing.x + housing.w - 1 - x);
        let dy = y - housing.y;
        dx + dy >= 5
    };
    // clear chamfer corners back to slate panel
    for y in housing.y..housing.y + 6 {
        for x in housing.x..housing.x + housing.w {
            if !hin(x, y) {
                b.set(x, y, c(Ramp::Slate, 6));
            }
        }
    }
    bevel_region(&mut b, housing, &hin, c(Ramp::Grey, 10), c(Ramp::Grey, 2), BLACK);
    // oval recess for the rose
    for y in housing.y + 2..housing.y + 24 {
        for x in housing.x + 2..housing.x + housing.w - 2 {
            let dx = (x as f32 + 0.5 - ccx as f32) / 29.0;
            let dy = (y as f32 + 0.5 - ccy as f32) / 12.0;
            let d = dx * dx + dy * dy;
            if d < 1.0 {
                let v = if d > 0.85 { if dy < 0.0 { 2 } else { 9 } } else { 4 + (hashf(x, y, 3, seed) * 3.0) as i32 };
                b.set(x, y, c(Ramp::Grey, v));
            }
        }
    }
    // front face line and move arrows
    b.hline(housing.x + 2, housing.x + housing.w - 3, 152, c(Ramp::Grey, 10));
    b.hline(housing.x + 2, housing.x + housing.w - 3, 153, c(Ramp::Grey, 3));
    for (i, r) in MOVE_BUTTONS.iter().enumerate() {
        let a = move_arrow(i);
        b.blit(&a, r.x + (r.w - a.w) / 2, r.y + (r.h - a.h) / 2);
    }

    // Dragons on both sides of the frame.
    let d = dragon();
    b.blit(&d, 35, 54);
    b.blit_flip_x(&d, 241 - d.w, 54);

    // Power gem pedestal.
    let (gx, gy) = GEM_C;
    let ped = Rect::new(gx - 17, gy + 6, 35, 15);
    stone_fill(&mut b, ped, seed + 10, 3, 9);
    bevel_region(&mut b, ped, &|x, y| ped.contains((x, y)), c(Ramp::Grey, 11), c(Ramp::Grey, 2), BLACK);
    for x in gx - 16..gx + 17 {
        let t = (x - gx) as f32 / 16.0;
        let yy = gy + 6 - (2.0 * (1.0 - t * t).max(0.0).sqrt()) as i32;
        b.set(x, yy, c(Ramp::Gold, 12));
        b.set(x, yy + 1, c(Ramp::Gold, 7));
        b.set(x, gy + 7, c(Ramp::Gold, 5));
    }

    // Message scroll.
    b.fill_rect(0, 166, 320, 34, BLACK);
    b.fill_rect(12, 167, 296, 33, SCROLL_BG);
    for x in 12..308 {
        b.set(x, 167, c(Ramp::Parch, 4));
        let speck = hash3(x, 0, 0, seed) % 7 == 0;
        if speck {
            b.set(x, 168, c(Ramp::Parch, 5));
        }
    }
    for &x0 in &[2, 308] {
        for y in 166..200 {
            for x in x0..x0 + 10 {
                let t = (x - x0) as f32 / 9.0;
                let v = 3.0 + (1.0 - (t * 2.0 - 1.0).powi(2)) * 4.0 + (hashf(x, y, 2, seed) - 0.5);
                b.set(x, y, c(Ramp::Earth, v as i32));
            }
            b.set(x0, y, BLACK);
            b.set(x0 + 9, y, BLACK);
        }
        for x in x0 - 1..x0 + 11 {
            b.set(x, 166, BLACK);
        }
    }

    b
}

impl UiArt {
    pub fn generate() -> UiArt {
        let frame = build_frame();
        let icons = vec![icon_options(), icon_talk(), icon_get(), icon_look(), icon_fight(), icon_use()];
        let compass = (0..16).map(compass_rose).collect();
        let gem = (0..5).map(gem).collect();
        let (flask_glass, flask_inside) = flask_parts();
        let cursors = cursors();
        UiArt {
            frame,
            icons,
            compass,
            gem,
            flask_glass,
            flask_inside,
            cursors,
            slot: slot_ring(),
            gargoyle_eye: [c(Ramp::Copper, 2), c(Ramp::Green, 12), c(Ramp::Gold, 13), c(Ramp::Red, 12)],
            chain_bmp: chain(),
        }
    }

}
