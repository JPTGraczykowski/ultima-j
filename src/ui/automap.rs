//! The automap: explored areas drawn on a torn parchment, like the original.

use super::Ctx;
use crate::game::{DoorKind, Game, TriggerKind};
use crate::gfx::palette::{c, Ramp, BLACK};
use crate::gfx::Bitmap;
use crate::layout::Rect;
use crate::util::{fbm2, hashf};
use crate::world::map::{Liquid, Shape, MAP_SIZE};

pub const CLOSE: Rect = Rect::new(252, 166, 52, 18);
pub const MAP_AREA: Rect = Rect::new(10, 7, 196, 188);

/// Map geometry for a level: (cell size, x offset, y offset).
fn geometry(g: &Game, lvl: usize) -> (i32, i32, i32) {
    let map = &g.levels[lvl].map;
    let (mut x0, mut z0, mut x1, mut z1) = (MAP_SIZE, MAP_SIZE, 0, 0);
    for tz in 0..MAP_SIZE {
        for tx in 0..MAP_SIZE {
            if !map.get(tx, tz).is_solid() {
                x0 = x0.min(tx);
                z0 = z0.min(tz);
                x1 = x1.max(tx);
                z1 = z1.max(tz);
            }
        }
    }
    let (w, h) = ((x1 - x0 + 1).max(1), (z1 - z0 + 1).max(1));
    let cell = (196 / w).min(188 / h).clamp(3, 7);
    (cell, 10 + (196 - w * cell) / 2 - x0 * cell, 7 + (188 - h * cell) / 2 - z0 * cell)
}

/// The tile under a screen position on the map.
pub fn tile_at(g: &Game, lvl: usize, m: (i32, i32)) -> Option<(i32, i32)> {
    let (cell, ox, oy) = geometry(g, lvl);
    let tx = (m.0 - ox).div_euclid(cell);
    let tz = (m.1 - oy).div_euclid(cell);
    if crate::world::map::Map::in_bounds(tx, tz) { Some((tx, tz)) } else { None }
}
pub const UP: Rect = Rect::new(286, 40, 16, 14);
pub const DOWN: Rect = Rect::new(286, 56, 16, 14);

fn background(fb: &mut Bitmap) {
    fb.fill(BLACK);
    for y in 0..200 {
        for x in 0..320 {
            // Torn edge.
            let edge = 4.0 + fbm2(x as f32 + y as f32 * 0.37, 0.0, 512.0, 64, 3, 9) * 6.0;
            let dx = (x as f32).min(319.0 - x as f32);
            let dy = (y as f32).min(199.0 - y as f32);
            if dx.min(dy) < edge {
                continue;
            }
            let n = fbm2(x as f32, y as f32, 256.0, 32, 4, 5);
            let v = 6.0 + (n - 0.5) * 3.0 + (hashf(x, y, 1, 2) - 0.5) * 0.8;
            let mut s = v.round() as i32;
            if dx.min(dy) < edge + 2.0 {
                s -= 2;
            }
            fb.set(x, y, c(Ramp::Parch, s.clamp(2, 9)));
        }
    }
}

pub fn draw(ctx: &mut Ctx, fb: &mut Bitmap, g: &Game, lvl: usize, note: Option<(usize, i32, i32, String)>) {
    background(fb);
    let lv = &g.levels[lvl];
    let map = &lv.map;
    let ink = c(Ramp::Parch, 2);
    let floor = c(Ramp::Parch, 10);
    // Fit the level's extent into the map area.
    let (cell, ox, oy) = geometry(g, lvl);
    let px = |tx: i32| ox + tx * cell;
    let pz = |tz: i32| oy + tz * cell;
    for tz in 0..MAP_SIZE {
        for tx in 0..MAP_SIZE {
            let t = map.get(tx, tz);
            if t.is_solid() || !t.explored {
                continue;
            }
            for yy in 0..cell {
                for xx in 0..cell {
                    let fxx = (xx as f32 + 0.5) / cell as f32;
                    let fyy = (yy as f32 + 0.5) / cell as f32;
                    let (sx, sy) = (px(tx) + xx, pz(tz) + yy);
                    if t.point_open(fxx, fyy) {
                        let col = match t.liquid {
                            Liquid::Water => {
                                let wave = ((sx + sy * 2) % 5 == 0) || ((sx * 3 + sy) % 7 == 0);
                                c(Ramp::Blue, if wave { 8 } else { 6 })
                            }
                            Liquid::Lava => c(Ramp::Fire, if (sx + sy) % 3 == 0 { 8 } else { 6 }),
                            Liquid::None => {
                                if hashf(sx, sy, 7, 1) > 0.9 { c(Ramp::Parch, 9) } else { floor }
                            }
                        };
                        fb.set(sx, sy, col);
                    } else if t.is_diag() {
                        fb.set(sx, sy, ink);
                    }
                }
            }
            if matches!(t.shape, Shape::SlopeN | Shape::SlopeS | Shape::SlopeE | Shape::SlopeW) {
                for k in 1..cell - 1 {
                    fb.set(px(tx) + k, pz(tz) + cell / 2, c(Ramp::Parch, 6));
                }
            }
            // Walls: dark lines along edges that face solid tiles; water gets darker banks.
            let dirs = [(0, -1), (1, 0), (0, 1), (-1, 0)];
            for (e, (dx, dz)) in dirs.iter().enumerate() {
                let nb = map.get(tx + dx, tz + dz);
                let wall = nb.is_solid() || !nb.edge_open((e + 2) % 4);
                let bank = t.liquid == Liquid::Water && nb.liquid != Liquid::Water && !wall;
                if !(wall || bank) {
                    continue;
                }
                let col = if wall { ink } else { c(Ramp::Blue, 3) };
                for k in 0..cell {
                    let (x, y) = match e {
                        0 => (px(tx) + k, pz(tz)),
                        1 => (px(tx) + cell - 1, pz(tz) + k),
                        2 => (px(tx) + k, pz(tz) + cell - 1),
                        _ => (px(tx), pz(tz) + k),
                    };
                    if !t.is_diag() || t.point_open((x - px(tx)) as f32 / cell as f32 + 0.1, (y - pz(tz)) as f32 / cell as f32 + 0.1) {
                        fb.set(x, y, col);
                    }
                }
            }
        }
    }
    // Doors and stairs.
    for d in &lv.doors {
        if !map.get(d.tx, d.tz).explored || (d.kind == DoorKind::Secret && !d.found) {
            continue;
        }
        let col = c(Ramp::Earth, 4);
        match d.axis {
            crate::game::DoorAxis::NS => fb.fill_rect(px(d.tx), pz(d.tz) + cell / 2 - 1, cell, 2, col),
            crate::game::DoorAxis::EW => fb.fill_rect(px(d.tx) + cell / 2 - 1, pz(d.tz), 2, cell, col),
        }
    }
    for t in &lv.triggers {
        if let TriggerKind::Stairs { .. } = t.kind {
            if map.get(t.tx, t.tz).explored {
                for k in 0..cell {
                    if k % 2 == 0 {
                        fb.hline(px(t.tx), px(t.tx) + cell - 1, pz(t.tz) + k, c(Ramp::Earth, 3));
                    }
                }
            }
        }
    }
    // Traps the player has spotted.
    for t in lv.triggers.iter().filter(|t| t.found && t.is_armed_trap()) {
        let (x, y) = (px(t.tx) + cell / 2, pz(t.tz) + cell / 2);
        let col = c(Ramp::Red, 5);
        fb.line(x - 2, y - 2, x + 2, y + 2, col);
        fb.line(x + 2, y - 2, x - 2, y + 2, col);
    }
    // The player.
    if lvl == g.cur {
        let p = &g.player;
        let x = ox as f32 + p.pos.x * cell as f32;
        let y = oy as f32 + p.pos.z * cell as f32;
        let (s, co) = p.yaw.sin_cos();
        fb.line(x as i32, y as i32, (x + s * 4.0) as i32, (y - co * 4.0) as i32, c(Ramp::Red, 10));
        fb.fill_circle_f(x, y, 1.5, c(Ramp::Red, 11));
    }
    // Notes written by the player.
    for (x, z, text) in &lv.notes {
        let sx = px(*x);
        let sy = pz(*z);
        ctx.art.font.draw(fb, sx, sy, text, c(Ramp::Blue, 3));
    }
    if let Some((nl, x, z, text)) = note {
        if nl == lvl {
            let s = format!("{}_", text);
            fb.fill_rect(px(x) - 1, pz(z) - 1, ctx.art.font.text_width(&s) + 2, 7, c(Ramp::Parch, 11));
            ctx.art.font.draw(fb, px(x), pz(z), &s, c(Ramp::Blue, 3));
        }
    }
    // Side panel: title, level arrows, compass rose, close button.
    let f = &ctx.art.font;
    let big = &ctx.art.font_big;
    let ink_cols = [c(Ramp::Parch, 2), c(Ramp::Parch, 2), c(Ramp::Parch, 3), c(Ramp::Parch, 3), c(Ramp::Parch, 2)];
    big.draw_rows(fb, 214, 10, &format!("Level {}", lvl + 1), &ink_cols);
    let name = &lv.name;
    for (i, l) in f.wrap(name, 100).iter().enumerate() {
        f.draw(fb, 214, 28 + i as i32 * 6, l, c(Ramp::Parch, 2));
    }
    for (r, up) in [(UP, true), (DOWN, false)] {
        fb.rect(r.x, r.y, r.w, r.h, c(Ramp::Parch, 3));
        let cx = r.x + r.w / 2;
        for k in 0..5 {
            let y = if up { r.y + 4 + k } else { r.y + r.h - 5 - k };
            fb.hline(cx - k, cx + k, y, c(Ramp::Parch, 2));
        }
    }
    // compass rose
    let (cx, cy) = (258, 110);
    for (dx, dy, l) in [(0, -14, "N"), (14, 0, "E"), (0, 14, "S"), (-14, 0, "W")] {
        fb.line(cx, cy, cx + dx * 2 / 3, cy + dy * 2 / 3, c(Ramp::Parch, 3));
        f.draw(fb, cx + dx - 1, cy + dy - 2, l, c(Ramp::Parch, 2));
    }
    fb.ellipse_outline(cx as f32 + 0.5, cy as f32 + 0.5, 6.0, 6.0, c(Ramp::Parch, 3));
    // explored percentage
    let open = map.tiles.iter().filter(|t| !t.is_solid()).count().max(1);
    let seen = map.tiles.iter().filter(|t| !t.is_solid() && t.explored).count();
    f.draw(fb, 214, 140, &format!("Explored: {}%", seen * 100 / open), c(Ramp::Parch, 2));
    f.draw(fb, 214, 150, "Click the map to", c(Ramp::Parch, 4));
    f.draw(fb, 214, 156, "write a note.", c(Ramp::Parch, 4));
    // close scroll button
    fb.fill_rect(CLOSE.x, CLOSE.y, CLOSE.w, CLOSE.h, c(Ramp::Parch, 10));
    fb.rect(CLOSE.x, CLOSE.y, CLOSE.w, CLOSE.h, c(Ramp::Parch, 3));
    for k in [CLOSE.x - 2, CLOSE.x + CLOSE.w] {
        fb.fill_rect(k, CLOSE.y - 1, 3, CLOSE.h + 2, c(Ramp::Parch, 6));
    }
    f.draw_centered(fb, CLOSE.x + CLOSE.w / 2, CLOSE.y + 6, "CLOSE", c(Ramp::Parch, 2));
    // quill
    for k in 0..30 {
        fb.set(214 + k, 190 - k / 3, c(Ramp::Silver, 11));
        fb.set(214 + k, 191 - k / 3, c(Ramp::Silver, 7));
    }
}
