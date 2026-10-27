//! Item icons (16x16, used both in the inventory and as world sprites, like
//! the original), larger fixture sprites, rune glyphs and effect sprites.

use super::canvas::{HCanvas, Mat};
use super::sprite3d::{bx, cone, ell, ell_rot, render, CMat, Part, Pattern, View3};
use crate::game::items::ItemKind;
use crate::gfx::palette::{c, Ramp, BLACK, WHITE};
use crate::gfx::Bitmap;
use crate::util::{hashf, M3, V3};

pub const ICON: i32 = 16;

fn canvas() -> HCanvas {
    HCanvas::new(ICON, ICON)
}

fn metal(cv: &mut HCanvas) -> u8 {
    cv.add_mat(Mat::new(Ramp::Silver, 4, 15).spec(0.8))
}

fn wood(cv: &mut HCanvas) -> u8 {
    cv.add_mat(Mat::new(Ramp::Earth, 2, 10).noise(0.25))
}

fn sword(blade: f32, guard_r: Ramp) -> Bitmap {
    let mut cv = canvas();
    let m = metal(&mut cv);
    let g = cv.add_mat(Mat::new(guard_r, 3, 13).spec(0.5));
    let h = cv.add_mat(Mat::new(Ramp::Earth, 1, 7));
    // Blade from the hilt (lower left) towards the upper right.
    let tip = 10.5 * blade.min(1.1);
    let (bx0, by0) = (5.5, 10.5);
    let (tx, ty) = (bx0 + tip * 0.72, by0 - tip * 0.72);
    cv.poly(&[(bx0 - 1.2, by0 - 1.2), (bx0 + 1.2, by0 + 1.2), (tx + 0.9, ty + 0.3), (tx + 0.8, ty - 0.8), (tx - 0.3, ty - 0.9)], 1.0, 1.0, m);
    cv.capsule((3.0, 8.5), (7.5, 13.0), 1.0, 1.0, 2.0, 1.0, g);
    cv.capsule((4.6, 11.4), (2.4, 13.6), 0.9, 0.9, 1.5, 1.0, h);
    cv.ellipse(1.8, 14.2, 1.2, 1.2, 1.5, 1.0, g);
    cv.render(true, true)
}

fn axe(big: bool) -> Bitmap {
    let mut cv = canvas();
    let w = wood(&mut cv);
    let m = metal(&mut cv);
    cv.capsule((3.0, 14.0), (12.0, 3.0), 0.9, 0.9, 0.0, 1.0, w);
    let s: f32 = if big { 1.25 } else { 1.0 };
    cv.poly(&[(9.0, 2.5), (14.5 * s.min(1.05), 1.0), (15.0, 7.0 * s), (11.5, 6.5)], 1.5, 1.0, m);
    cv.render(true, true)
}

fn mace(spiked: bool) -> Bitmap {
    let mut cv = canvas();
    let w = wood(&mut cv);
    let m = if spiked { metal(&mut cv) } else { wood(&mut cv) };
    cv.capsule((3.0, 14.0), (10.0, 6.0), 0.9, if spiked { 0.9 } else { 1.4 }, 0.0, 1.0, w);
    cv.ellipse(11.0, 5.0, 3.4, 3.4, 1.0, 0.9, m);
    let mut b = cv.render(true, true);
    if spiked {
        for (x, y) in [(11, 0), (15, 5), (7, 5), (11, 9)] {
            b.set(x, y, c(Ramp::Silver, 13));
        }
    }
    b
}

fn potion(r: Ramp) -> Bitmap {
    let mut cv = canvas();
    let glass = cv.add_mat(Mat::new(r, 4, 13).spec(0.9));
    let cork = cv.add_mat(Mat::new(Ramp::Earth, 3, 9));
    cv.ellipse(8.0, 11.0, 4.6, 4.2, 0.0, 1.0, glass);
    cv.capsule((8.0, 7.0), (8.0, 4.0), 1.6, 1.4, 1.0, 1.0, glass);
    cv.capsule((8.0, 3.2), (8.0, 1.8), 1.3, 1.3, 2.0, 1.0, cork);
    let mut b = cv.render(true, false);
    b.set(6, 9, WHITE);
    b.set(6, 10, c(r, 14));
    b
}

fn key(r: Ramp) -> Bitmap {
    let mut cv = canvas();
    let m = cv.add_mat(Mat::new(r, 3, 14).spec(0.7));
    cv.ring(4.5, 8.0, 2.4, 2.4, 1.0, 0.0, m);
    cv.capsule((7.0, 8.0), (14.0, 8.0), 0.9, 0.9, 0.0, 1.0, m);
    cv.capsule((12.0, 8.5), (12.0, 11.0), 0.7, 0.7, 0.0, 1.0, m);
    cv.capsule((14.0, 8.5), (14.0, 10.5), 0.7, 0.7, 0.0, 1.0, m);
    cv.render(true, false)
}

fn gem(r: Ramp) -> Bitmap {
    let mut cv = canvas();
    let m = cv.add_mat(Mat::new(r, 4, 15).spec(1.0));
    cv.poly(&[(8.0, 3.5), (12.5, 7.0), (8.0, 13.0), (3.5, 7.0)], 0.0, 2.2, m);
    let mut b = cv.render(true, false);
    b.set(7, 6, WHITE);
    b
}

fn ring(r: Ramp) -> Bitmap {
    let mut cv = canvas();
    let m = cv.add_mat(Mat::new(r, 3, 14).spec(0.8));
    cv.ring(8.0, 9.0, 3.6, 2.6, 1.1, 0.0, m);
    let s = cv.add_mat(Mat::new(Ramp::Red, 5, 14).spec(1.0));
    cv.ellipse(8.0, 5.8, 1.6, 1.4, 2.0, 1.0, s);
    cv.render(true, false)
}

fn armor_mat(cv: &mut HCanvas, kind: u8) -> u8 {
    match kind {
        0 => cv.add_mat(Mat::new(Ramp::Earth, 2, 9).noise(0.2)),
        1 => cv.add_mat(Mat::new(Ramp::Grey, 3, 12).noise(0.45)),
        _ => cv.add_mat(Mat::new(Ramp::Silver, 4, 15).spec(0.7)),
    }
}

fn helm(kind: u8) -> Bitmap {
    let mut cv = canvas();
    let m = armor_mat(&mut cv, kind);
    cv.ellipse(8.0, 9.0, 6.0, 5.5, 0.0, 0.9, m);
    let mut b = cv.render(true, false);
    for x in 1..16 {
        for y in 12..16 {
            b.set(x, y, 0);
        }
    }
    let mut b = b.outlined(BLACK, false);
    if kind == 2 {
        b.fill_rect(4, 8, 8, 2, BLACK);
        b.vline(8, 9, 12, c(Ramp::Silver, 8));
    }
    if kind == 1 {
        for x in 3..13 {
            b.set(x, 12, c(Ramp::Grey, 6 + (x % 2)));
            b.set(x, 13, c(Ramp::Grey, 5));
        }
        b = b.outlined(BLACK, false);
    }
    b
}

fn shirt(kind: u8) -> Bitmap {
    let mut cv = canvas();
    let m = armor_mat(&mut cv, kind);
    cv.poly(&[(4.0, 2.0), (12.0, 2.0), (15.0, 6.0), (13.0, 8.0), (12.0, 6.5), (12.0, 14.5), (4.0, 14.5), (4.0, 6.5), (3.0, 8.0), (1.0, 6.0)], 0.0, 2.0, m);
    let mut b = cv.render(true, false);
    b.hline(6, 9, 2, BLACK);
    b.set(7, 3, BLACK);
    b.set(8, 3, BLACK);
    b
}

fn leggings(kind: u8) -> Bitmap {
    let mut cv = canvas();
    let m = armor_mat(&mut cv, kind);
    cv.poly(&[(4.0, 1.5), (12.0, 1.5), (12.5, 14.5), (9.0, 14.5), (8.0, 5.0), (7.0, 14.5), (3.5, 14.5)], 0.0, 1.6, m);
    cv.render(true, false)
}

fn boots(kind: u8) -> Bitmap {
    let mut cv = canvas();
    let m = armor_mat(&mut cv, kind);
    cv.poly(&[(3.0, 2.0), (7.0, 2.0), (7.0, 10.0), (10.0, 12.0), (10.0, 14.5), (2.5, 14.5)], 0.0, 1.3, m);
    cv.poly(&[(8.5, 1.0), (12.5, 1.0), (12.5, 9.0), (15.0, 11.0), (15.0, 13.5), (8.0, 13.5)], 1.0, 1.3, m);
    cv.render(true, true)
}

fn gloves(kind: u8) -> Bitmap {
    let mut cv = canvas();
    let m = armor_mat(&mut cv, kind);
    cv.ellipse(7.0, 9.0, 4.0, 4.5, 0.0, 0.8, m);
    for k in 0..4 {
        let x = 4.0 + k as f32 * 2.1;
        cv.capsule((x, 6.0), (x - 0.5 + k as f32 * 0.2, 1.5 + (k as f32 - 1.5).abs()), 1.0, 0.9, 1.0, 1.0, m);
    }
    cv.capsule((10.0, 9.0), (13.5, 6.0), 1.1, 1.0, 1.0, 1.0, m);
    cv.capsule((7.0, 12.0), (7.0, 15.0), 2.8, 2.8, 0.0, 0.8, m);
    cv.render(true, true)
}

fn shield(kind: u8) -> Bitmap {
    let mut cv = canvas();
    match kind {
        0 => {
            let m = metal(&mut cv);
            cv.ellipse(8.0, 8.0, 6.5, 6.5, 0.0, 0.6, m);
            let b2 = cv.add_mat(Mat::new(Ramp::Gold, 4, 13).spec(0.8));
            cv.ellipse(8.0, 8.0, 1.8, 1.8, 3.0, 1.0, b2);
        }
        1 => {
            let w = wood(&mut cv);
            cv.poly(&[(2.0, 2.0), (14.0, 2.0), (14.0, 8.0), (8.0, 15.0), (2.0, 8.0)], 0.0, 1.5, w);
            let m = metal(&mut cv);
            cv.capsule((8.0, 3.0), (8.0, 12.0), 0.8, 0.8, 1.5, 1.0, m);
            cv.capsule((3.0, 6.0), (13.0, 6.0), 0.8, 0.8, 1.5, 1.0, m);
        }
        _ => {
            let m = cv.add_mat(Mat::new(Ramp::Slate, 3, 12).spec(0.5));
            cv.poly(&[(2.5, 1.0), (13.5, 1.0), (13.5, 11.0), (8.0, 15.5), (2.5, 11.0)], 0.0, 1.5, m);
            let g = cv.add_mat(Mat::new(Ramp::Gold, 4, 13).spec(0.6));
            cv.capsule((8.0, 4.0), (8.0, 11.0), 0.8, 0.8, 1.5, 1.0, g);
            cv.capsule((5.0, 6.5), (11.0, 6.5), 0.8, 0.8, 1.5, 1.0, g);
        }
    }
    cv.render(true, true)
}

fn torch(lit: bool) -> Bitmap {
    let mut cv = canvas();
    let w = wood(&mut cv);
    let rag = cv.add_mat(Mat::new(Ramp::Earth, 1, 6).noise(0.4));
    cv.capsule((4.0, 14.5), (10.0, 6.0), 1.0, 1.3, 0.0, 1.0, w);
    cv.ellipse(10.8, 5.0, 2.2, 2.4, 1.0, 1.0, rag);
    let mut b = cv.render(true, true);
    if lit {
        for (x, y, s) in [(11, 2, 14), (12, 1, 12), (10, 1, 11), (11, 0, 13), (11, 3, 15), (12, 3, 13), (10, 3, 12)] {
            b.set(x, y, c(Ramp::Fire, s));
        }
    }
    b
}

fn candle(lit: bool) -> Bitmap {
    let mut cv = canvas();
    let wax = cv.add_mat(Mat::new(Ramp::Parch, 6, 15));
    cv.capsule((8.0, 14.0), (8.0, 6.0), 1.8, 1.6, 0.0, 1.0, wax);
    let dish = cv.add_mat(Mat::new(Ramp::Gold, 3, 12).spec(0.6));
    cv.ellipse(8.0, 14.5, 4.5, 1.3, 0.0, 1.0, dish);
    let mut b = cv.render(true, true);
    b.set(8, 4, BLACK);
    if lit {
        b.set(8, 3, c(Ramp::Fire, 14));
        b.set(8, 2, c(Ramp::Fire, 12));
        b.set(8, 1, c(Ramp::Fire, 10));
    }
    b
}

fn lantern(lit: bool) -> Bitmap {
    let mut cv = canvas();
    let m = cv.add_mat(Mat::new(Ramp::Gold, 2, 12).spec(0.5));
    let glass = cv.add_mat(if lit { Mat::new(Ramp::Fire, 9, 15) } else { Mat::new(Ramp::Slate, 3, 9) });
    cv.ellipse(8.0, 9.0, 4.0, 4.5, 0.0, 0.8, glass);
    cv.capsule((4.0, 4.5), (12.0, 4.5), 1.2, 1.2, 2.0, 1.0, m);
    cv.capsule((4.0, 13.5), (12.0, 13.5), 1.2, 1.2, 2.0, 1.0, m);
    cv.capsule((8.0, 5.0), (8.0, 13.0), 0.6, 0.6, 3.0, 1.0, m);
    cv.ring(8.0, 2.3, 1.8, 1.4, 0.6, 2.0, m);
    cv.render(true, true)
}

fn bag(kind: u8) -> Bitmap {
    let mut cv = canvas();
    let m = match kind {
        0 => cv.add_mat(Mat::new(Ramp::Earth, 2, 10).noise(0.2)),
        1 => cv.add_mat(Mat::new(Ramp::Moss, 2, 9).noise(0.2)),
        _ => cv.add_mat(Mat::new(Ramp::Copper, 2, 9).noise(0.2)),
    };
    cv.ellipse(8.0, 10.0, 5.5, 5.0, 0.0, 0.9, m);
    cv.capsule((6.5, 5.5), (9.5, 5.5), 1.4, 1.4, 1.0, 1.0, m);
    cv.capsule((8.0, 5.0), (8.0, 2.0), 1.3, 2.0, 0.5, 1.0, m);
    let mut b = cv.render(true, true);
    b.hline(6, 10, 5, c(Ramp::Gold, 9));
    if kind == 2 {
        // rune bag marking
        b.vline(8, 9, 13, c(Ramp::Gold, 12));
        b.set(7, 10, c(Ramp::Gold, 12));
        b.set(9, 10, c(Ramp::Gold, 12));
    }
    b
}

fn wooden_box() -> Bitmap {
    let mut cv = canvas();
    let w = wood(&mut cv);
    cv.poly(&[(1.5, 5.0), (14.5, 5.0), (14.5, 14.5), (1.5, 14.5)], 0.0, 1.0, w);
    cv.poly(&[(1.0, 3.0), (15.0, 3.0), (15.0, 6.0), (1.0, 6.0)], 1.0, 0.8, w);
    let mut b = cv.render(true, true);
    b.hline(2, 13, 9, c(Ramp::Earth, 1));
    b.set(8, 7, c(Ramp::Gold, 12));
    b
}

pub fn rune_glyph(r: usize) -> Bitmap {
    const G: [[&str; 7]; 24] = [
        ["#....", "##...", "#.#..", "##...", "#.#..", "#....", "#...."], // An
        ["##...", "#.#..", "##...", "#.#..", "#.#..", "##...", "....."], // Bet
        ["..#..", ".#...", "#....", ".#...", "..#..", ".....", "....."], // Corp
        ["#...#", "##.##", "#.#.#", "##.##", "#...#", ".....", "....."], // Des
        ["#...#", "##.##", "#.#.#", "#...#", "#...#", ".....", "....."], // Ex
        ["#....", "#.#..", "##.#.", "#.#..", "##...", "#....", "#...."], // Flam
        ["#...#", ".#.#.", "..#..", ".#.#.", "#...#", ".....", "....."], // Grav
        ["#..#.", "#.##.", "##.#.", "#..#.", "#..#.", ".....", "....."], // Hur
        ["..#..", "..#..", "..#..", "..#..", "..#..", ".....", "....."], // In
        [".#...", "#....", ".#.#.", "...#.", ".#...", ".....", "....."], // Jux
        ["#..#.", "#.#..", "##...", "#.#..", "#..#.", ".....", "....."], // Kal
        ["#....", "##...", "#.#..", "#....", "#....", "#....", "....."], // Lor
        ["#...#", "##.##", "#.#.#", "#...#", "#...#", "#...#", "....."], // Mani
        ["#....", "#.#..", "##...", "#....", "#....", "#....", "....."], // Nox
        [".#.#.", "#...#", ".#.#.", "..#..", ".#.#.", "#...#", "....."], // Ort
        ["##.##", "#...#", "#....", "#...#", "##.##", ".....", "....."], // Por
        ["#.#.#", "#.#.#", ".###.", "..#..", "..#..", ".....", "....."], // Quas
        ["##...", "#.#..", "##...", "#.#..", "#..#.", ".....", "....."], // Rel
        ["...#.", "..#..", ".#...", "..#..", ".#...", "#....", "....."], // Sanct
        ["..#..", ".###.", "#.#.#", "..#..", "..#..", ".....", "....."], // Tym
        ["##...", "#.#..", "#..#.", "#..#.", "#..#.", ".....", "....."], // Uus
        ["#...#", "#...#", "#...#", ".#.#.", "..#..", "..#..", "....."], // Vas
        ["#....", "##...", "#.#..", "##...", "#....", "#....", "....."], // Wis
        ["#.#.#", ".###.", "..#..", "..#..", "..#..", ".....", "....."], // Ylem
    ];
    let key = [('#', 1u8)];
    Bitmap::from_ascii(&G[r % 24], &key)
}

fn rune_stone(r: u16) -> Bitmap {
    let mut cv = canvas();
    let m = cv.add_mat(Mat::new(Ramp::Grey, 3, 12).noise(0.15));
    cv.ellipse(8.0, 8.5, 6.2, 5.6, 0.0, 0.8, m);
    let mut b = cv.render(true, false);
    let g = rune_glyph(r as usize);
    for y in 0..g.h {
        for x in 0..g.w {
            if g.get(x, y) != 0 {
                b.set(6 + x, 5 + y, c(Ramp::Copper, 11));
            }
        }
    }
    b
}

fn coin(many: bool) -> Bitmap {
    let mut cv = canvas();
    let m = cv.add_mat(Mat::new(Ramp::Gold, 3, 15).spec(0.8));
    if many {
        for (x, y) in [(5.0, 11.0), (10.5, 11.5), (8.0, 8.5), (6.5, 5.5), (11.0, 6.5)] {
            cv.ellipse(x, y, 3.2, 2.0, 0.0, 0.6, m);
        }
    } else {
        cv.ellipse(8.0, 9.0, 4.5, 3.2, 0.0, 0.5, m);
    }
    cv.render(true, true)
}

fn scroll() -> Bitmap {
    let mut cv = canvas();
    let p = cv.add_mat(Mat::new(Ramp::Parch, 6, 15).noise(0.1));
    cv.poly(&[(3.0, 3.0), (13.0, 3.0), (13.0, 13.0), (3.0, 13.0)], 0.0, 0.5, p);
    cv.capsule((2.0, 3.0), (14.0, 3.0), 1.6, 1.6, 1.0, 1.0, p);
    cv.capsule((2.0, 13.0), (14.0, 13.0), 1.6, 1.6, 1.0, 1.0, p);
    let mut b = cv.render(true, true);
    for y in [6, 8, 10] {
        for x in 5..12 {
            if hashf(x, y, 1, 9) > 0.3 {
                b.set(x, y, c(Ramp::Parch, 2));
            }
        }
    }
    b
}

fn book() -> Bitmap {
    let mut cv = canvas();
    let cover = cv.add_mat(Mat::new(Ramp::Red, 2, 9).noise(0.15));
    let pages = cv.add_mat(Mat::new(Ramp::Parch, 8, 15));
    cv.poly(&[(3.0, 4.0), (13.5, 2.0), (13.5, 12.5), (3.0, 14.5)], 1.0, 0.7, cover);
    cv.poly(&[(2.0, 5.0), (3.5, 4.5), (3.5, 15.0), (2.0, 15.0)], 0.0, 0.3, pages);
    let mut b = cv.render(true, true);
    b.hline(6, 11, 7, c(Ramp::Gold, 11));
    b
}

fn map_icon() -> Bitmap {
    let mut b = scroll();
    for &(x, y) in &[(5, 6), (6, 6), (7, 7), (8, 8), (9, 8), (10, 9), (7, 10), (6, 10)] {
        b.set(x, y, c(Ramp::Blue, 7));
    }
    b.set(10, 6, c(Ramp::Red, 10));
    b
}

fn food(k: ItemKind) -> Bitmap {
    let mut cv = canvas();
    match k {
        ItemKind::Apple => {
            let m = cv.add_mat(Mat::new(Ramp::Red, 3, 14).spec(0.6));
            cv.ellipse(8.0, 9.5, 5.0, 4.6, 0.0, 1.0, m);
            let st = cv.add_mat(Mat::new(Ramp::Earth, 2, 7));
            cv.capsule((8.0, 5.5), (9.0, 2.5), 0.6, 0.5, 2.0, 1.0, st);
            let lf = cv.add_mat(Mat::new(Ramp::Green, 3, 11));
            cv.ellipse(10.5, 3.5, 2.0, 1.0, 2.0, 1.0, lf);
        }
        ItemKind::Bread => {
            let m = cv.add_mat(Mat::new(Ramp::Gold, 2, 12).noise(0.15));
            cv.ellipse(8.0, 9.5, 6.8, 4.2, 0.0, 1.0, m);
            let mut b = cv.render(true, false);
            for x in [5, 8, 11] {
                b.set(x, 7, c(Ramp::Gold, 4));
                b.set(x + 1, 8, c(Ramp::Gold, 4));
            }
            return b;
        }
        ItemKind::Cheese => {
            let m = cv.add_mat(Mat::new(Ramp::Gold, 6, 15));
            cv.poly(&[(2.0, 12.0), (14.0, 12.0), (14.0, 7.0), (5.0, 5.0)], 0.0, 1.2, m);
            let mut b = cv.render(true, false);
            b.set(9, 9, c(Ramp::Gold, 5));
            b.set(12, 10, c(Ramp::Gold, 5));
            b.set(6, 10, c(Ramp::Gold, 6));
            return b;
        }
        ItemKind::Meat => {
            let m = cv.add_mat(Mat::new(Ramp::Red, 2, 10).noise(0.2));
            let bone = cv.add_mat(Mat::new(Ramp::Parch, 8, 15));
            cv.capsule((10.0, 6.0), (14.0, 2.5), 1.0, 1.0, 0.0, 1.0, bone);
            cv.ellipse(7.5, 10.0, 5.5, 4.0, 0.0, 1.0, m);
        }
        ItemKind::Fish => {
            let m = cv.add_mat(Mat::new(Ramp::Silver, 4, 14).spec(0.6));
            cv.ellipse(7.0, 8.0, 5.5, 2.8, 0.0, 1.0, m);
            cv.poly(&[(11.5, 8.0), (15.0, 5.0), (15.0, 11.0)], 0.0, 0.5, m);
            let mut b = cv.render(true, false);
            b.set(3, 7, BLACK);
            return b;
        }
        ItemKind::Mushroom => {
            let cap_m = cv.add_mat(Mat::new(Ramp::Copper, 3, 12));
            let stem = cv.add_mat(Mat::new(Ramp::Parch, 7, 15));
            cv.capsule((8.0, 14.0), (8.0, 8.0), 1.6, 1.4, 0.0, 1.0, stem);
            cv.ellipse(8.0, 7.0, 5.5, 3.2, 1.0, 1.0, cap_m);
            let mut b = cv.render(true, true);
            b.set(6, 5, c(Ramp::Parch, 14));
            b.set(10, 6, c(Ramp::Parch, 14));
            return b;
        }
        ItemKind::WaterFlask => {
            let m = cv.add_mat(Mat::new(Ramp::Earth, 2, 9));
            cv.ellipse(8.0, 10.0, 5.0, 4.8, 0.0, 1.0, m);
            cv.capsule((8.0, 6.0), (8.0, 2.0), 1.4, 1.2, 1.0, 1.0, m);
        }
        _ => {
            // wine bottle
            let m = cv.add_mat(Mat::new(Ramp::Green, 1, 9).spec(0.7));
            cv.capsule((8.0, 14.0), (8.0, 7.0), 3.0, 3.0, 0.0, 1.0, m);
            cv.capsule((8.0, 6.0), (8.0, 1.5), 1.2, 1.1, 1.0, 1.0, m);
        }
    }
    cv.render(true, true)
}

fn misc(k: ItemKind) -> Bitmap {
    let mut cv = canvas();
    match k {
        ItemKind::Bedroll => {
            let m = cv.add_mat(Mat::new(Ramp::Moss, 2, 10).noise(0.2));
            cv.capsule((2.5, 9.0), (13.5, 9.0), 4.0, 4.0, 0.0, 0.8, m);
            let s = cv.add_mat(Mat::new(Ramp::Earth, 1, 6));
            cv.capsule((5.0, 4.5), (5.0, 13.5), 0.7, 0.7, 4.0, 1.0, s);
            cv.capsule((11.0, 4.5), (11.0, 13.5), 0.7, 0.7, 4.0, 1.0, s);
        }
        ItemKind::Rock => {
            let m = cv.add_mat(Mat::new(Ramp::Grey, 2, 11).noise(0.3));
            cv.poly(&[(3.0, 13.0), (2.5, 8.0), (6.0, 4.0), (11.0, 4.5), (14.0, 9.0), (12.5, 13.5)], 0.0, 3.0, m);
        }
        ItemKind::Bone => {
            let m = cv.add_mat(Mat::new(Ramp::Parch, 7, 15).noise(0.1));
            cv.capsule((4.0, 12.0), (12.0, 4.0), 1.1, 1.1, 0.0, 1.0, m);
            for (x, y) in [(3.0, 11.0), (5.0, 13.0), (11.0, 3.0), (13.0, 5.0)] {
                cv.ellipse(x, y, 1.6, 1.6, 0.5, 1.0, m);
            }
        }
        ItemKind::Skull => {
            let m = cv.add_mat(Mat::new(Ramp::Parch, 7, 15).noise(0.1));
            cv.ellipse(8.0, 7.5, 5.5, 5.0, 0.0, 1.0, m);
            cv.ellipse(8.0, 12.0, 3.0, 2.2, 0.0, 1.0, m);
            let mut b = cv.render(true, false);
            b.fill_rect(5, 7, 2, 2, BLACK);
            b.fill_rect(9, 7, 2, 2, BLACK);
            b.set(8, 10, BLACK);
            b.hline(6, 10, 12, c(Ramp::Parch, 4));
            return b;
        }
        ItemKind::Lockpick => {
            let m = metal(&mut cv);
            cv.capsule((3.0, 13.0), (12.0, 4.0), 0.6, 0.5, 0.0, 1.0, m);
            cv.capsule((12.0, 4.0), (13.5, 4.5), 0.5, 0.5, 0.0, 1.0, m);
            let h = cv.add_mat(Mat::new(Ramp::Earth, 2, 8));
            cv.capsule((2.0, 14.0), (5.0, 11.0), 1.2, 1.2, 1.0, 1.0, h);
        }
        ItemKind::Rope => {
            let m = cv.add_mat(Mat::new(Ramp::Gold, 2, 10).noise(0.3));
            cv.ring(8.0, 9.0, 5.0, 4.0, 1.4, 0.0, m);
            cv.ring(8.0, 9.0, 2.6, 2.0, 1.2, 0.5, m);
        }
        ItemKind::Pole => {
            let w = wood(&mut cv);
            cv.capsule((1.5, 14.5), (14.5, 1.5), 0.9, 0.9, 0.0, 1.0, w);
        }
        _ => {
            // shovel
            let w = wood(&mut cv);
            let m = metal(&mut cv);
            cv.capsule((2.0, 14.0), (10.0, 6.0), 0.8, 0.8, 0.0, 1.0, w);
            cv.poly(&[(9.0, 4.0), (12.0, 1.0), (15.0, 4.0), (12.0, 7.0)], 1.0, 1.2, m);
        }
    }
    cv.render(true, true)
}

fn shard(r: Ramp) -> Bitmap {
    let mut cv = canvas();
    let m = cv.add_mat(Mat::new(r, 5, 15).spec(1.0));
    cv.poly(&[(8.0, 0.5), (12.0, 6.0), (10.5, 15.0), (5.0, 15.0), (3.5, 7.0)], 0.0, 3.0, m);
    let mut b = cv.render(true, false);
    b.set(7, 4, WHITE);
    b.set(6, 6, WHITE);
    b
}

fn sapling() -> Bitmap {
    let mut cv = canvas();
    let m = cv.add_mat(Mat::new(Ramp::Silver, 5, 15).spec(0.8));
    cv.capsule((8.0, 15.0), (8.0, 4.0), 1.0, 0.6, 0.0, 1.0, m);
    cv.capsule((8.0, 9.0), (4.0, 5.0), 0.6, 0.4, 0.5, 1.0, m);
    cv.capsule((8.0, 7.0), (12.0, 3.5), 0.6, 0.4, 0.5, 1.0, m);
    for (x, y) in [(4.0, 4.5), (12.0, 3.0), (8.0, 3.0), (5.5, 7.5), (10.5, 6.5)] {
        cv.ellipse(x, y, 1.5, 1.0, 1.0, 1.0, m);
    }
    cv.render(true, true)
}

fn medallion() -> Bitmap {
    let mut cv = canvas();
    let m = cv.add_mat(Mat::new(Ramp::Gold, 3, 15).spec(0.8));
    cv.ring(8.0, 5.0, 3.0, 2.5, 0.5, 0.0, m);
    cv.ellipse(8.0, 11.0, 4.0, 4.0, 1.0, 0.6, m);
    let mut b = cv.render(true, true);
    b.set(8, 11, c(Ramp::Red, 12));
    b
}

fn wand() -> Bitmap {
    let mut cv = canvas();
    let w = cv.add_mat(Mat::new(Ramp::Purple, 2, 11).spec(0.5));
    cv.capsule((3.0, 13.0), (12.0, 4.0), 0.9, 0.7, 0.0, 1.0, w);
    let tip = cv.add_mat(Mat::new(Ramp::Blue, 8, 15).spec(1.0));
    cv.ellipse(12.5, 3.5, 1.6, 1.6, 1.0, 1.0, tip);
    cv.render(true, true)
}

fn crossbow() -> Bitmap {
    let mut cv = canvas();
    let w = wood(&mut cv);
    let m = metal(&mut cv);
    cv.capsule((3.0, 13.0), (12.0, 4.0), 1.1, 1.1, 0.0, 1.0, w);
    cv.capsule((6.0, 2.0), (14.0, 10.0), 0.8, 0.8, 1.5, 1.0, m);
    cv.render(true, true)
}

fn sling() -> Bitmap {
    let mut cv = canvas();
    let m = cv.add_mat(Mat::new(Ramp::Earth, 2, 9));
    cv.capsule((2.0, 3.0), (8.0, 11.0), 0.6, 0.6, 0.0, 1.0, m);
    cv.capsule((14.0, 3.0), (8.0, 11.0), 0.6, 0.6, 0.0, 1.0, m);
    cv.ellipse(8.0, 12.0, 2.5, 1.8, 1.0, 1.0, m);
    cv.render(true, true)
}

fn ammo(stone: bool) -> Bitmap {
    let mut cv = canvas();
    if stone {
        let m = cv.add_mat(Mat::new(Ramp::Grey, 3, 12));
        for (x, y) in [(5.0, 10.0), (10.0, 11.0), (8.0, 7.0)] {
            cv.ellipse(x, y, 2.2, 2.0, 0.0, 1.0, m);
        }
    } else {
        let w = wood(&mut cv);
        let m = metal(&mut cv);
        for k in 0..2 {
            let o = k as f32 * 3.0;
            cv.capsule((2.0 + o, 13.0), (11.0 + o, 4.0), 0.5, 0.5, 0.0, 1.0, w);
            cv.ellipse(11.5 + o, 3.5, 1.2, 1.2, 1.0, 1.0, m);
        }
    }
    cv.render(true, true)
}

pub fn icon(kind: ItemKind, data: u16, lit: bool) -> Bitmap {
    use ItemKind::*;
    match kind {
        Dagger => sword(0.45, Ramp::Grey),
        Shortsword => sword(0.7, Ramp::Copper),
        Longsword => sword(1.0, Ramp::Gold),
        Broadsword => sword(1.1, Ramp::Grey),
        JeweledSword => {
            let mut b = sword(1.05, Ramp::Gold);
            b.set(11, 11, c(Ramp::Red, 13));
            b
        }
        HandAxe => axe(false),
        Axe => axe(true),
        Cudgel => mace(false),
        Mace => mace(true),
        Sling => sling(),
        Crossbow => crossbow(),
        SlingStone => ammo(true),
        Bolt => ammo(false),
        LeatherCap => helm(0),
        ChainCowl => helm(1),
        PlateHelm => helm(2),
        LeatherVest => shirt(0),
        ChainShirt => shirt(1),
        Breastplate => shirt(2),
        LeatherLeggings => leggings(0),
        ChainLeggings => leggings(1),
        PlateLeggings => leggings(2),
        LeatherBoots => boots(0),
        PlateBoots => boots(2),
        LeatherGloves => gloves(0),
        Gauntlets => gloves(2),
        Buckler => shield(0),
        WoodenShield => shield(1),
        TowerShield => shield(2),
        GoldRing => ring(Ramp::Gold),
        SilverRing => ring(Ramp::Silver),
        Apple | Bread | Cheese | Meat | Fish | Mushroom | WaterFlask | Wine => food(kind),
        RedPotion => potion(Ramp::Red),
        BluePotion => potion(Ramp::Blue),
        GreenPotion => potion(Ramp::Green),
        YellowPotion => potion(Ramp::Gold),
        Torch => torch(lit),
        Candle => candle(lit),
        Lantern => lantern(lit),
        Bag => bag(0),
        Pack => bag(1),
        RuneBag => bag(2),
        Box => wooden_box(),
        IronKey => key(Ramp::Grey),
        GoldKey => key(Ramp::Gold),
        SilverKey => key(Ramp::Silver),
        RuneStone => rune_stone(data),
        Wand => wand(),
        Coin => coin(data > 1),
        RedGem => gem(Ramp::Red),
        BlueGem => gem(Ramp::Blue),
        GreenGem => gem(Ramp::Green),
        WhiteGem => gem(Ramp::Silver),
        GoldNugget => {
            let mut cv = canvas();
            let m = cv.add_mat(Mat::new(Ramp::Gold, 3, 15).spec(0.8).noise(0.2));
            cv.poly(&[(4.0, 11.0), (5.0, 6.0), (9.0, 4.0), (12.0, 7.0), (11.0, 12.0)], 0.0, 2.5, m);
            cv.render(true, true)
        }
        Scroll => scroll(),
        Book => book(),
        Map => map_icon(),
        Bedroll | Rock | Bone | Skull | Lockpick | Rope | Pole | Shovel => misc(kind),
        ShardOfValor => shard(Ramp::Red),
        ShardOfWisdom => shard(Ramp::Blue),
        ShardOfMercy => shard(Ramp::Green),
        SilverSapling => sapling(),
        Medallion => medallion(),
        Hammer => {
            let mut cv = canvas();
            let w = wood(&mut cv);
            let m = cv.add_mat(Mat::new(Ramp::Gold, 3, 14).spec(0.8));
            cv.capsule((3.0, 14.0), (10.0, 5.0), 1.0, 1.0, 0.0, 1.0, w);
            cv.poly(&[(6.5, 2.0), (12.0, 0.5), (15.5, 6.0), (10.0, 8.5)], 1.5, 1.4, m);
            cv.render(true, true)
        }
        LizardEgg => {
            let mut cv = canvas();
            let m = cv.add_mat(Mat::new(Ramp::Green, 4, 13).spec(0.4).noise(0.25));
            for (x, y) in [(5.5, 10.0), (10.5, 10.5), (8.0, 6.0)] {
                cv.ellipse(x, y, 3.0, 3.8, 0.0, 1.0, m);
            }
            cv.render(true, true)
        }
        RelicBones => {
            let mut cv = canvas();
            let bone = cv.add_mat(Mat::new(Ramp::Parch, 7, 15).noise(0.1));
            let gold = cv.add_mat(Mat::new(Ramp::Gold, 4, 14).spec(0.6));
            cv.capsule((3.0, 12.0), (13.0, 4.0), 1.2, 1.2, 0.0, 1.0, bone);
            cv.capsule((3.0, 5.0), (13.0, 12.0), 1.2, 1.2, 0.5, 1.0, bone);
            cv.ellipse(8.0, 8.0, 2.4, 3.2, 2.0, 1.0, gold);
            cv.render(true, true)
        }
        CaptainsSword => sword(1.0, Ramp::Silver),
        EngravedRing => {
            let mut cv = canvas();
            let m = cv.add_mat(Mat::new(Ramp::Silver, 3, 14).spec(0.8));
            cv.ring(8.0, 9.0, 3.6, 2.6, 1.1, 0.0, m);
            let mut b = cv.render(true, false);
            for x in [6, 8, 10] {
                b.set(x, 12, c(Ramp::Silver, 4));
            }
            b
        }
        ApprenticeHammer => {
            // A small smith's hammer: a plain iron head on a short haft.
            let mut cv = canvas();
            let w = wood(&mut cv);
            let m = metal(&mut cv);
            cv.capsule((4.0, 13.5), (10.0, 6.0), 0.9, 0.9, 0.0, 1.0, w);
            cv.poly(&[(7.5, 3.5), (11.5, 2.0), (14.0, 6.0), (10.0, 7.5)], 1.5, 1.4, m);
            cv.render(true, true)
        }
        ChroniclePage => {
            // A loose leaf, torn along one edge, with a red seal.
            let mut cv = canvas();
            let p = cv.add_mat(Mat::new(Ramp::Parch, 7, 15).noise(0.12));
            cv.poly(&[(3.0, 2.0), (12.0, 3.0), (13.0, 14.0), (11.0, 13.0), (9.0, 14.5), (7.0, 13.5), (4.0, 14.0)], 0.0, 0.4, p);
            let mut b = cv.render(true, true);
            for y in [5, 7, 9, 11] {
                for x in 5..11 {
                    if hashf(x, y, 2, 9) > 0.3 {
                        b.set(x, y, c(Ramp::Parch, 2));
                    }
                }
            }
            b.set(10, 11, c(Ramp::Red, 9));
            b.set(11, 11, c(Ramp::Red, 11));
            b.set(10, 12, c(Ramp::Red, 7));
            b.set(11, 12, c(Ramp::Red, 9));
            b
        }
        _ => Bitmap::new(ICON, ICON),
    }
}

/// Fixture sprites rendered from small 3D models.
pub fn fixture(kind: ItemKind) -> Bitmap {
    use Pattern::*;
    let m = |r: Ramp, lo: i32, hi: i32, p: Pattern| CMat::new(r, lo, hi, p);
    let wood = m(Ramp::Earth, 1, 8, Wood);
    let stone = m(Ramp::Grey, 2, 10, Noise(0.3));
    let iron = m(Ramp::Slate, 2, 10, Metal);
    let mut parts: Vec<Part> = Vec::new();
    match kind {
        ItemKind::Barrel => {
            parts.push(cone(V3::new(0.0, 0.03, 0.0), V3::new(0.0, 0.27, 0.0), 0.17, 0.2, wood));
            parts.push(cone(V3::new(0.0, 0.27, 0.0), V3::new(0.0, 0.5, 0.0), 0.2, 0.17, wood));
            for y in [0.08, 0.27, 0.45] {
                parts.push(ell(V3::new(0.0, y, 0.0), V3::new(0.205, 0.015, 0.205), iron));
            }
        }
        ItemKind::Table => {
            parts.push(bx(V3::new(0.0, 0.4, 0.0), V3::new(0.38, 0.03, 0.25), M3::IDENTITY, wood));
            for (x, z) in [(-0.33, -0.2), (0.33, -0.2), (-0.33, 0.2), (0.33, 0.2)] {
                parts.push(cone(V3::new(x, 0.0, z), V3::new(x, 0.38, z), 0.025, 0.025, wood));
            }
        }
        ItemKind::Chest => {
            parts.push(bx(V3::new(0.0, 0.15, 0.0), V3::new(0.25, 0.14, 0.16), M3::IDENTITY, wood));
            parts.push(ell(V3::new(0.0, 0.29, 0.0), V3::new(0.25, 0.08, 0.16), wood));
            parts.push(bx(V3::new(0.0, 0.2, 0.165), V3::new(0.04, 0.05, 0.01), M3::IDENTITY, m(Ramp::Gold, 4, 13, Metal)));
            for x in [-0.17, 0.17] {
                parts.push(bx(V3::new(x, 0.18, 0.0), V3::new(0.02, 0.17, 0.165), M3::IDENTITY, iron));
            }
        }
        ItemKind::Fountain => {
            let water = m(Ramp::Blue, 3, 10, Noise(0.25));
            parts.push(cone(V3::new(0.0, 0.0, 0.0), V3::new(0.0, 0.12, 0.0), 0.36, 0.38, stone));
            parts.push(cone(V3::new(0.0, 0.12, 0.0), V3::new(0.0, 0.2, 0.0), 0.39, 0.39, stone));
            parts.push(ell(V3::new(0.0, 0.2, 0.0), V3::new(0.34, 0.012, 0.34), water));
            parts.push(cone(V3::new(0.0, 0.18, 0.0), V3::new(0.0, 0.48, 0.0), 0.05, 0.04, stone));
            parts.push(ell(V3::new(0.0, 0.5, 0.0), V3::new(0.13, 0.04, 0.13), stone));
            parts.push(cone(V3::new(0.0, 0.52, 0.0), V3::new(0.0, 0.66, 0.0), 0.03, 0.01, m(Ramp::Blue, 7, 13, Plain)));
            for k in 0..4 {
                let a = k as f32 * 1.57 + 0.4;
                parts.push(cone(V3::new(a.sin() * 0.1, 0.5, a.cos() * 0.1), V3::new(a.sin() * 0.25, 0.22, a.cos() * 0.25), 0.012, 0.008, m(Ramp::Blue, 8, 13, Plain)));
            }
        }
        ItemKind::BonePile => {
            let bone = m(Ramp::Parch, 6, 15, Bone);
            parts.push(ell(V3::new(0.05, 0.06, 0.0), V3::new(0.08, 0.07, 0.08), bone));
            parts.push(cone(V3::new(-0.2, 0.02, 0.05), V3::new(0.15, 0.02, -0.1), 0.025, 0.02, bone));
            parts.push(cone(V3::new(-0.1, 0.02, -0.12), V3::new(0.2, 0.03, 0.1), 0.02, 0.025, bone));
            parts.push(ell(V3::new(-0.12, 0.04, 0.08), V3::new(0.09, 0.03, 0.05), bone));
        }
        ItemKind::Plant => {
            let leaf = m(Ramp::Green, 2, 10, Noise(0.3));
            parts.push(cone(V3::new(0.0, 0.0, 0.0), V3::new(0.0, 0.25, 0.0), 0.02, 0.015, leaf));
            for k in 0..6 {
                let a = k as f32 * 1.05;
                parts.push(ell_rot(V3::new(a.sin() * 0.1, 0.12 + (k % 3) as f32 * 0.06, a.cos() * 0.1), V3::new(0.09, 0.025, 0.04), M3::rot_y(a).mul(&M3::rot_z(0.5)), leaf));
            }
            parts.push(ell(V3::new(0.03, 0.3, 0.04), V3::new(0.03, 0.03, 0.03), m(Ramp::Red, 6, 13, Plain)));
        }
        ItemKind::Shrine => {
            let gold = m(Ramp::Gold, 3, 13, Metal);
            parts.push(bx(V3::new(0.0, 0.08, 0.0), V3::new(0.26, 0.08, 0.2), M3::IDENTITY, stone));
            parts.push(cone(V3::new(0.0, 0.16, 0.0), V3::new(0.0, 0.62, 0.0), 0.05, 0.05, gold));
            parts.push(cone(V3::new(-0.17, 0.5, 0.0), V3::new(0.17, 0.5, 0.0), 0.04, 0.04, gold));
            parts.push(Part { prim: super::sprite3d::Prim::Ellipsoid { c: V3::new(0.0, 0.73, 0.0), r: V3::new(0.1, 0.13, 0.03), rot: M3::IDENTITY }, mat: gold, face: false });
            parts.push(ell(V3::new(0.0, 0.73, 0.0), V3::new(0.055, 0.08, 0.035), m(Ramp::Grey, 0, 1, Plain)));
        }
        ItemKind::Lever => {
            parts.push(bx(V3::new(0.0, 0.05, 0.0), V3::new(0.08, 0.05, 0.06), M3::IDENTITY, iron));
            parts.push(cone(V3::new(0.0, 0.08, 0.0), V3::new(0.06, 0.3, 0.0), 0.015, 0.015, iron));
            parts.push(ell(V3::new(0.065, 0.31, 0.0), V3::new(0.03, 0.03, 0.03), m(Ramp::Red, 3, 11, Plain)));
        }
        ItemKind::Campfire => {
            for k in 0..3 {
                let a = k as f32 * 2.1;
                parts.push(cone(V3::new(a.sin() * 0.18, 0.03, a.cos() * 0.18), V3::new(-a.sin() * 0.05, 0.08, -a.cos() * 0.05), 0.03, 0.03, wood));
            }
            parts.push(cone(V3::new(0.0, 0.03, 0.0), V3::new(0.0, 0.32, 0.0), 0.11, 0.0, m(Ramp::Fire, 6, 12, Plain).glow()));
            parts.push(cone(V3::new(0.04, 0.03, 0.06), V3::new(0.06, 0.22, 0.07), 0.07, 0.0, m(Ramp::Fire, 9, 14, Plain).glow()));
        }
        ItemKind::Bed => {
            parts.push(bx(V3::new(0.0, 0.12, 0.0), V3::new(0.42, 0.06, 0.22), M3::IDENTITY, wood));
            parts.push(bx(V3::new(0.0, 0.2, 0.0), V3::new(0.4, 0.04, 0.2), M3::IDENTITY, m(Ramp::Moss, 2, 9, Cloth)));
            parts.push(ell(V3::new(-0.3, 0.26, 0.0), V3::new(0.08, 0.04, 0.14), m(Ramp::Parch, 6, 14, Cloth)));
            for (x, z) in [(-0.38, -0.18), (0.38, -0.18), (-0.38, 0.18), (0.38, 0.18)] {
                parts.push(cone(V3::new(x, 0.0, z), V3::new(x, 0.1, z), 0.025, 0.025, wood));
            }
        }
        ItemKind::Brazier => {
            parts.push(cone(V3::new(0.0, 0.0, 0.0), V3::new(0.0, 0.38, 0.0), 0.04, 0.03, iron));
            parts.push(ell(V3::new(0.0, 0.42, 0.0), V3::new(0.17, 0.07, 0.17), iron));
            parts.push(cone(V3::new(0.0, 0.45, 0.0), V3::new(0.0, 0.66, 0.0), 0.12, 0.0, m(Ramp::Fire, 6, 12, Plain).glow()));
            parts.push(cone(V3::new(0.02, 0.45, 0.05), V3::new(0.03, 0.6, 0.06), 0.07, 0.0, m(Ramp::Fire, 9, 14, Plain).glow()));
        }
        ItemKind::Statue => {
            let marble = m(Ramp::Stone, 3, 13, Noise(0.15));
            parts.push(bx(V3::new(0.0, 0.08, 0.0), V3::new(0.2, 0.08, 0.2), M3::IDENTITY, marble));
            parts.push(cone(V3::new(0.0, 0.16, 0.0), V3::new(0.0, 0.7, 0.0), 0.13, 0.1, marble));
            parts.push(ell(V3::new(0.0, 0.8, 0.0), V3::new(0.07, 0.09, 0.07), marble));
            parts.push(cone(V3::new(0.1, 0.6, 0.0), V3::new(0.18, 0.95, 0.05), 0.03, 0.025, marble));
            parts.push(cone(V3::new(-0.1, 0.6, 0.0), V3::new(-0.12, 0.35, 0.05), 0.03, 0.025, marble));
        }
        ItemKind::Bookshelf => {
            parts.push(bx(V3::new(0.0, 0.42, 0.0), V3::new(0.32, 0.42, 0.1), M3::IDENTITY, wood));
            for (k, r) in [Ramp::Red, Ramp::Blue, Ramp::Moss, Ramp::Copper, Ramp::Purple].iter().enumerate() {
                for row in 0..3 {
                    let x = -0.24 + k as f32 * 0.12;
                    parts.push(bx(V3::new(x, 0.18 + row as f32 * 0.26, 0.08), V3::new(0.05, 0.09, 0.04), M3::IDENTITY, m(*r, 2, 9, Cloth)));
                }
            }
        }
        ItemKind::Pillar => {
            let marble = m(Ramp::Stone, 2, 11, Noise(0.15));
            parts.push(bx(V3::new(0.0, 0.05, 0.0), V3::new(0.16, 0.05, 0.16), M3::IDENTITY, marble));
            parts.push(cone(V3::new(0.0, 0.1, 0.0), V3::new(0.0, 1.05, 0.0), 0.11, 0.11, marble));
            parts.push(bx(V3::new(0.0, 1.1, 0.0), V3::new(0.16, 0.05, 0.16), M3::IDENTITY, marble));
        }
        ItemKind::Gravestone => {
            parts.push(bx(V3::new(0.0, 0.2, 0.0), V3::new(0.15, 0.2, 0.04), M3::IDENTITY, stone));
            parts.push(ell(V3::new(0.0, 0.4, 0.0), V3::new(0.15, 0.08, 0.04), stone));
        }
        ItemKind::Cauldron => {
            parts.push(ell(V3::new(0.0, 0.18, 0.0), V3::new(0.2, 0.16, 0.2), iron));
            parts.push(ell(V3::new(0.0, 0.31, 0.0), V3::new(0.16, 0.02, 0.16), m(Ramp::Green, 4, 12, Noise(0.3))));
        }
        ItemKind::MoonOrb => {
            parts.push(cone(V3::new(0.0, 0.0, 0.0), V3::new(0.0, 0.3, 0.0), 0.08, 0.04, stone));
            parts.push(ell(V3::new(0.0, 0.4, 0.0), V3::new(0.1, 0.1, 0.1), m(Ramp::Blue, 6, 15, Metal)));
        }
        ItemKind::Stalagmite => {
            let rock = m(Ramp::Stone, 1, 8, Noise(0.35));
            parts.push(cone(V3::new(0.0, 0.0, 0.0), V3::new(0.02, 0.58, 0.0), 0.14, 0.01, rock));
            parts.push(cone(V3::new(0.1, 0.0, 0.05), V3::new(0.12, 0.25, 0.05), 0.07, 0.01, rock));
        }
        ItemKind::Corpse => {
            let cloth = m(Ramp::Earth, 1, 6, Cloth);
            let bone = m(Ramp::Parch, 5, 13, Bone);
            parts.push(ell(V3::new(0.0, 0.06, 0.0), V3::new(0.3, 0.06, 0.12), cloth));
            parts.push(ell(V3::new(0.33, 0.07, 0.0), V3::new(0.07, 0.06, 0.06), bone));
        }
        _ => {}
    }
    let size = kind.def().size;
    let scale = 64.0;
    let w = (size.0 * scale * 1.1).ceil() as i32 + 4;
    let h = (size.1 * scale * 1.05).ceil() as i32 + 4;
    let v = View3 { yaw: 0.35, pitch: 0.18, scale, w, h, center: V3::ZERO, center_row: h as f32 - 3.0 };
    match kind {
        ItemKind::Blood => {
            let mut b = Bitmap::new(24, 6);
            for (x, y, r) in [(12.0, 3.0, 6.0), (6.0, 3.5, 3.0), (18.0, 2.5, 3.5)] {
                b.fill_ellipse(x, y, r, r * 0.35, c(Ramp::Red, 4));
            }
            b.set(11, 2, c(Ramp::Red, 7));
            b
        }
        ItemKind::Ward => {
            let mut b = Bitmap::new(26, 8);
            b.fill_ellipse(13.0, 4.0, 12.0, 3.6, c(Ramp::Blue, 6));
            b.fill_ellipse(13.0, 4.0, 9.0, 2.6, c(Ramp::Blue, 3));
            b.ellipse_outline(13.0, 4.0, 12.0, 3.6, c(Ramp::Blue, 13));
            // Rune strokes.
            b.line(9, 4, 17, 4, c(Ramp::Blue, 14));
            b.line(13, 2, 13, 6, c(Ramp::Blue, 14));
            b.line(10, 2, 16, 6, c(Ramp::Blue, 11));
            b
        }
        ItemKind::Web => {
            let mut b = Bitmap::new(44, 44);
            let col = c(Ramp::Silver, 11);
            for k in 0..8 {
                let a = k as f32 * std::f32::consts::PI / 4.0;
                b.line(22, 22, 22 + (a.cos() * 21.0) as i32, 22 + (a.sin() * 21.0) as i32, col);
            }
            for r in [6.0f32, 11.0, 16.0, 20.0] {
                b.ellipse_outline(22.0, 22.0, r, r, c(Ramp::Silver, 8));
            }
            b
        }
        _ => render(&parts, &v, true),
    }
}

/// Small sprites for projectiles and effects.
pub fn effect_sprite(kind: u8, frame: usize) -> Bitmap {
    let mut b = Bitmap::new(16, 16);
    let t = frame as f32;
    match kind {
        // magic arrow / lightning / fireball / poison / stone
        0 => {
            b.fill_circle_f(8.0, 8.0, 4.0, c(Ramp::Blue, 9));
            b.fill_circle_f(8.0, 8.0, 2.5, c(Ramp::Blue, 13));
            b.fill_circle_f(7.5, 7.5, 1.0, WHITE);
        }
        1 => {
            let mut x = 8;
            for y in 0..16 {
                x += if (y + frame as i32) % 3 == 0 { 1 } else { -1 };
                b.set(x.clamp(0, 15), y, c(Ramp::Blue, 14));
                b.set((x + 1).clamp(0, 15), y, WHITE);
            }
        }
        2 => {
            b.fill_circle_f(8.0, 8.0, 6.0 + (t * 1.3).sin(), c(Ramp::Fire, 6));
            b.fill_circle_f(8.0, 8.0, 4.5, c(Ramp::Fire, 10));
            b.fill_circle_f(8.0, 8.0, 2.5, c(Ramp::Fire, 14));
        }
        3 => {
            b.fill_circle_f(8.0, 8.0, 4.0, c(Ramp::Green, 8));
            b.fill_circle_f(7.0, 7.0, 1.5, c(Ramp::Green, 13));
        }
        4 => {
            b.fill_circle_f(8.0, 8.0, 2.5, c(Ramp::Grey, 9));
            b.set(7, 7, c(Ramp::Grey, 13));
        }
        // effects: blood, sparks, explosion, splash, smoke
        5 => {
            for k in 0..9 {
                let a = k as f32 * 0.7;
                let r = 1.5 + t * 1.6;
                b.fill_circle_f(8.0 + a.cos() * r, 8.0 + a.sin() * r + t * 0.8, 1.3, c(Ramp::Red, 6 + (k % 3)));
            }
        }
        6 => {
            for k in 0..8 {
                let a = k as f32 * 0.785;
                let r = 1.0 + t * 1.8;
                b.set((8.0 + a.cos() * r) as i32, (8.0 + a.sin() * r) as i32, c(Ramp::Gold, 14));
            }
        }
        7 => {
            let r = 3.0 + t * 2.0;
            b.fill_circle_f(8.0, 8.0, r.min(7.5), c(Ramp::Fire, (12 - frame as i32 * 2).max(4)));
            b.fill_circle_f(8.0, 8.0, (r - 2.0).max(1.0), c(Ramp::Fire, (15 - frame as i32 * 2).max(6)));
        }
        8 => {
            for k in 0..7 {
                let a = 3.4 + k as f32 * 0.45;
                let r = 1.0 + t * 1.5;
                b.set((8.0 + a.cos() * r) as i32, (12.0 + a.sin() * r) as i32, c(Ramp::Blue, 11));
            }
        }
        _ => {
            b.fill_circle_f(8.0, 10.0 - t, 2.0 + t, c(Ramp::Grey, 6));
        }
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::dialogue::ALL_ITEM_KINDS;

    #[test]
    fn every_item_has_art() {
        for k in ALL_ITEM_KINDS {
            let b = if k.is_fixture() { fixture(*k) } else { icon(*k, 2, false) };
            let n = b.px.iter().filter(|&&p| p != 0).count();
            assert!(n > 10, "{:?} has no art", k);
        }
    }
}

/// The big first-person weapon shown in the 3D view in fight mode.
/// `phase`: 0 ready, 1 drawn back, 2 mid-swing, 3 follow-through.
pub fn hand_weapon(kind: Option<ItemKind>, phase: u8) -> Bitmap {
    let w = 104;
    let h = 80;
    let mut cv = HCanvas::new(w, h);
    let ang: f32 = match phase {
        0 => -0.42,
        1 => 0.3,
        2 => -1.0,
        _ => -1.65,
    };
    let grip = (66.0f32, 62.0f32);
    let dir = (ang.sin(), -ang.cos());
    let perp = (ang.cos(), ang.sin());
    let p = |u: f32, v: f32| (grip.0 + perp.0 * u + dir.0 * v, grip.1 + perp.1 * u + dir.1 * v);
    let skin = cv.add_mat(Mat::new(Ramp::Flesh, 3, 13).noise(0.08));
    let sleeve = cv.add_mat(Mat::new(Ramp::Earth, 2, 9).noise(0.2));
    use ItemKind::*;
    let is_weapon = kind.map(|k| k.def().class == crate::game::items::Class::Weapon).unwrap_or(false);
    match kind {
        Some(Dagger) | Some(Shortsword) | Some(Longsword) | Some(Broadsword) | Some(JeweledSword) | Some(CaptainsSword) => {
            let len = match kind {
                Some(Dagger) => 28.0,
                Some(Shortsword) => 42.0,
                Some(Broadsword) => 60.0,
                _ => 54.0,
            };
            let half = if kind == Some(Broadsword) { 4.8 } else { 4.0 };
            let blade = cv.add_mat(Mat::new(Ramp::Silver, 5, 15).spec(0.9));
            let guard = cv.add_mat(Mat::new(if kind == Some(JeweledSword) { Ramp::Gold } else { Ramp::Slate }, 3, 13).spec(0.6));
            let grip_m = cv.add_mat(Mat::new(Ramp::Earth, 1, 7));
            cv.poly(&[p(-half, 9.0), p(half, 9.0), p(half * 0.7, len), p(0.0, len + 9.0), p(-half * 0.7, len)], 4.0, 2.2, blade);
            cv.capsule(p(-12.0, 9.0), p(12.0, 9.0), 2.6, 2.6, 7.0, 1.0, guard);
            cv.capsule(p(0.0, -10.0), p(0.0, 8.0), 2.4, 2.4, 3.0, 1.0, grip_m);
            let pm = p(0.0, -12.0);
            cv.ellipse(pm.0, pm.1, 3.4, 3.4, 5.0, 1.0, guard);
        }
        Some(HandAxe) | Some(Axe) | Some(Hammer) => {
            let wood = cv.add_mat(Mat::new(Ramp::Earth, 2, 10).noise(0.2));
            let head = cv.add_mat(if kind == Some(Hammer) { Mat::new(Ramp::Gold, 3, 14).spec(0.8) } else { Mat::new(Ramp::Silver, 4, 14).spec(0.8) });
            cv.capsule(p(0.0, -12.0), p(0.0, 50.0), 2.8, 2.8, 3.0, 1.0, wood);
            if kind == Some(Hammer) {
                cv.poly(&[p(-10.0, 40.0), p(10.0, 40.0), p(10.0, 54.0), p(-10.0, 54.0)], 5.0, 2.0, head);
            } else {
                cv.poly(&[p(1.0, 36.0), p(18.0, 29.0), p(21.0, 46.0), p(16.0, 55.0), p(1.0, 48.0)], 5.0, 2.0, head);
            }
        }
        Some(Cudgel) | Some(Mace) => {
            let wood = cv.add_mat(Mat::new(Ramp::Earth, 2, 10).noise(0.2));
            cv.capsule(p(0.0, -12.0), p(0.0, 40.0), 2.8, if kind == Some(Cudgel) { 6.0 } else { 2.8 }, 3.0, 1.0, wood);
            if kind == Some(Mace) {
                let head = cv.add_mat(Mat::new(Ramp::Slate, 3, 13).spec(0.7));
                let c0 = p(0.0, 44.0);
                cv.ellipse(c0.0, c0.1, 8.5, 8.5, 5.0, 1.0, head);
                for k in 0..6 {
                    let a = k as f32 * 1.047;
                    let q = p(a.cos() * 10.0, 44.0 + a.sin() * 10.0);
                    cv.ellipse(q.0, q.1, 2.0, 2.0, 6.0, 1.0, head);
                }
            }
        }
        _ => {}
    }
    // Forearm rising from the bottom edge, then the fist around the grip.
    let (gx, gy) = grip;
    cv.capsule((gx + 22.0, gy + 30.0), (gx + 6.0, gy + 6.0), 11.0, 8.5, 0.0, 0.7, sleeve);
    cv.capsule((gx + 6.0, gy + 7.0), (gx + 1.0, gy + 2.0), 8.0, 7.5, 2.0, 0.8, skin);
    if is_weapon {
        // Fingers wrapped around the grip.
        for k in 0..4 {
            let q = p(-5.5 + k as f32 * 3.7, -1.0 + (k as f32 - 1.5).abs() * 0.4);
            cv.ellipse(q.0, q.1, 3.0, 3.4, 9.0, 1.0, skin);
        }
        let t = p(4.5, 4.0);
        cv.ellipse(t.0, t.1, 3.0, 2.6, 10.0, 1.0, skin);
    } else {
        cv.ellipse(gx, gy - 6.0, 11.0, 9.0, 4.0, 1.0, skin);
        for k in 0..4 {
            cv.ellipse(gx - 8.0 + k as f32 * 5.2, gy - 14.0, 3.0, 3.4, 8.0, 1.0, skin);
        }
    }
    cv.render(true, true)
}
