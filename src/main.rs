//! Ultima-J: a clone of the 1992 first-person dungeon RPG Ultima Underworld,
//! with a software renderer and procedurally generated art in its style.

// Release builds on Windows open only the game window, without a console.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

#[macro_use]
mod lang;
mod art;
mod audio;
mod game;
mod gfx;
mod headless;
mod layout;
mod platform;
mod render;
mod ui;
mod util;
mod world;

use gfx::{palette, Bitmap, Palette, Ramp};

fn dump_art(dir: &str) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let art = art::Art::generate();
    let pal = &art.pal;
    let mut sw = Bitmap::new(16 * 8, 16 * 8);
    for i in 0..256 {
        sw.fill_rect((i % 16) * 8, (i / 16) * 8, 8, 8, i as u8);
    }
    gfx::save_bitmap_png(&format!("{}/palette.png", dir), &sw, pal, 3)?;
    gfx::save_bitmap_png(&format!("{}/textures.png", dir), &art.texture_sheet(), pal, 3)?;
    gfx::save_bitmap_png(&format!("{}/frame.png", dir), &art.ui.frame, pal, 3)?;
    // Item icons and fixtures.
    let kinds = game::dialogue::ALL_ITEM_KINDS;
    let cols = 12;
    let mut sheet = Bitmap::filled(cols * 20, ((kinds.len() as i32 + cols - 1) / cols) * 20, palette::c(Ramp::Copper, 7));
    let mut fixtures = Bitmap::filled(8 * 72, 3 * 80, palette::c(Ramp::Slate, 3));
    let mut fi = 0;
    for (i, k) in kinds.iter().enumerate() {
        if k.is_fixture() {
            let b = art::items::fixture(*k);
            fixtures.blit(&b, (fi % 8) * 72 + (72 - b.w) / 2, (fi / 8) * 80 + 78 - b.h);
            fi += 1;
        } else {
            let b = art::items::icon(*k, 5, false);
            sheet.blit(&b, (i as i32 % cols) * 20 + 2, (i as i32 / cols) * 20 + 2);
        }
    }
    gfx::save_bitmap_png(&format!("{}/items.png", dir), &sheet, pal, 4)?;
    gfx::save_bitmap_png(&format!("{}/fixtures.png", dir), &fixtures, pal, 2)?;
    let mut hands = Bitmap::filled(4 * 100, 2 * 76, palette::c(Ramp::Grey, 3));
    for (i, k) in [Some(game::items::ItemKind::Longsword), Some(game::items::ItemKind::Axe)].iter().enumerate() {
        for ph in 0..4u8 {
            hands.blit(&art::items::hand_weapon(*k, ph), ph as i32 * 100, i as i32 * 76);
        }
    }
    gfx::save_bitmap_png(&format!("{}/hands.png", dir), &hands, pal, 2)?;
    // Paperdolls: no armour, then leather, chain and plate.
    use game::items::ItemKind::*;
    let sets = [
        (false, true, 0, 2, [None, None, None, None, None]),
        (false, false, 1, 1, [Some(LeatherCap), Some(LeatherVest), Some(LeatherLeggings), Some(LeatherBoots), Some(LeatherGloves)]),
        (true, false, 0, 0, [None, None, None, None, None]),
        (true, false, 2, 3, [Some(ChainCowl), Some(ChainShirt), Some(ChainLeggings), None, None]),
        (false, true, 0, 0, [Some(PlateHelm), Some(Breastplate), Some(PlateLeggings), Some(PlateBoots), Some(Gauntlets)]),
        (false, true, 2, 3, [None, Some(LeatherVest), None, None, None]),
    ];
    let mut dolls = Bitmap::filled(sets.len() as i32 * 50, 76, palette::c(Ramp::Copper, 7));
    for (i, (female, beard, skin, hair, kit)) in sets.iter().enumerate() {
        let m = |k: Option<game::items::ItemKind>| k.map(ui::sprites::armor_cmat);
        let o = art::creatures::Outfit { female: *female, beard: *beard, skin: *skin, hair: *hair, head: m(kit[0]), chest: m(kit[1]), legs: m(kit[2]), feet: m(kit[3]), hands: m(kit[4]) };
        dolls.blit(&art::creatures::paperdoll(&o, 46, 72, 80.0), i as i32 * 50 + 2, 2);
    }
    gfx::save_bitmap_png(&format!("{}/paperdolls.png", dir), &dolls, pal, 5)?;
    // Conversation portraits.
    let races = game::creatures::ALL_RACES;
    let mut faces = Bitmap::filled(races.len() as i32 * 36, 36, palette::c(Ramp::Parch, 9));
    for (i, r) in races.iter().enumerate() {
        faces.blit(&art::creatures::portrait(*r, 1, 34), i as i32 * 36 + 1, 1);
    }
    gfx::save_bitmap_png(&format!("{}/portraits.png", dir), &faces, pal, 3)?;
    Ok(())
}

fn dump_creatures(out: &str) -> Result<(), String> {
    let pal = Palette::new();
    let races = game::creatures::ALL_RACES;
    let t0 = std::time::Instant::now();
    let sprites: Vec<_> = races.iter().map(|r| art::creatures::render_creature(*r, 0)).collect();
    eprintln!("creatures rendered in {:.2}s", t0.elapsed().as_secs_f64());
    let cell = 100;
    let cols = 10;
    let mut sheet = Bitmap::filled(cols * cell, races.len() as i32 * cell, gfx::c(Ramp::Slate, 3));
    for (ri, s) in sprites.iter().enumerate() {
        let pick: [(usize, usize); 10] = [(0, 0), (0, 1), (0, 2), (0, 3), (0, 4), (2, 0), (6, 0), (8, 0), (10, 0), (11, 0)];
        for (ci, (f, v)) in pick.iter().enumerate() {
            let b = &s.frames[*f][*v];
            let x = ci as i32 * cell + (cell - b.w) / 2;
            let y = ri as i32 * cell + cell - b.h;
            sheet.blit(b, x, y);
        }
    }
    gfx::save_bitmap_png(out, &sheet, &pal, 1)
}

/// Top-down renders of every level for design review: area names, tile
/// coordinates every 8 tiles (for `goto` and `level` in scripts) and a legend.
fn dump_maps(dir: &str) -> Result<(), String> {
    use world::map::{Liquid, MAP_SIZE};
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let pal = Palette::new();
    let small = gfx::Font::small();
    let big = small.scale2x();
    let levels = game::levels::build_all();
    let s = 8;
    let (left, top, bottom) = (16, 34, 40);
    let (w, h) = (left + MAP_SIZE * s + 6, top + MAP_SIZE * s + bottom);
    let water = gfx::c(Ramp::Blue, 7);
    let lava = gfx::c(Ramp::Fire, 8);
    let door = gfx::c(Ramp::Earth, 9);
    let stairs = gfx::c(Ramp::Gold, 13);
    let item = gfx::c(Ramp::Green, 12);
    let named = gfx::c(Ramp::Purple, 13);
    let creature = gfx::c(Ramp::Red, 11);
    let grey = gfx::c(Ramp::Grey, 8);
    for (li, lv) in levels.iter().enumerate() {
        let mut b = Bitmap::filled(w, h, palette::BLACK);
        let (ox, oy) = (left, top);
        for z in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                let t = lv.map.get(x, z);
                if t.is_solid() {
                    continue;
                }
                let col = match t.liquid {
                    Liquid::Water => water,
                    Liquid::Lava => lava,
                    Liquid::None => gfx::c(Ramp::Stone, (6 + t.floor as i32).clamp(2, 14)),
                };
                for yy in 0..s {
                    for xx in 0..s {
                        if t.point_open((xx as f32 + 0.5) / s as f32, (yy as f32 + 0.5) / s as f32) {
                            b.set(ox + x * s + xx, oy + z * s + yy, col);
                        }
                    }
                }
                if t.shape != world::map::Shape::Open && !t.is_diag() {
                    b.set(ox + x * s + 4, oy + z * s + 4, palette::BLACK);
                }
            }
        }
        // Grid, a brighter line and a coordinate every 8 tiles.
        for k in 0..=MAP_SIZE {
            let col = if k % 8 == 0 { gfx::c(Ramp::Grey, 5) } else { gfx::c(Ramp::Grey, 2) };
            b.vline(ox + k * s, oy, oy + MAP_SIZE * s, col);
            b.hline(ox, ox + MAP_SIZE * s, oy + k * s, col);
            if k % 8 == 0 && k < MAP_SIZE {
                small.draw(&mut b, ox + k * s + 1, oy - 7, &k.to_string(), grey);
                let n = k.to_string();
                small.draw(&mut b, ox - 2 - small.text_width(&n), oy + k * s + 1, &n, grey);
            }
        }
        for d in &lv.doors {
            b.fill_rect(ox + d.tx * s + 2, oy + d.tz * s + 2, s - 4, s - 4, door);
        }
        for t in &lv.triggers {
            if let game::TriggerKind::Stairs { .. } = t.kind {
                b.fill_rect(ox + t.tx * s + 1, oy + t.tz * s + 1, s - 2, s - 2, stairs);
            }
        }
        let at = |x: f32| x * s as f32;
        for it in &lv.items {
            b.fill_rect(ox + at(it.pos.x) as i32 - 1, oy + at(it.pos.z) as i32 - 1, 3, 3, item);
        }
        for n in &lv.npcs {
            b.fill_circle_f(ox as f32 + at(n.pos.x), oy as f32 + at(n.pos.z), 2.5, if n.name.is_some() { named } else { creature });
        }
        b.fill_circle_f(ox as f32 + at(lv.start.0), oy as f32 + at(lv.start.1), 3.0, palette::WHITE);
        // Names: areas in white, characters in lilac under their dot.
        for n in &lv.npcs {
            if let Some(name) = &n.name {
                let x = ox + at(n.pos.x) as i32 - small.text_width(name) / 2;
                small.draw_fancy(&mut b, x, oy + at(n.pos.z) as i32 + 4, name, &[named], palette::BLACK, None);
            }
        }
        for (x, z, name) in game::levels::map_labels(lv, li) {
            let cx = ox + x * s + s / 2 - small.text_width(&name) / 2;
            small.draw_fancy(&mut b, cx, oy + z * s + 1, &name, &[palette::WHITE], palette::BLACK, None);
        }
        let title = format!("Level {}: {}", li + 1, lv.name);
        big.draw(&mut b, left, 4, &title, palette::WHITE);
        // Legend.
        let ly = top + MAP_SIZE * s + 8;
        let mut x = left;
        let key = |b: &mut Bitmap, x: &mut i32, y: i32, col: u8, round: bool, text: &str| {
            if round {
                b.fill_circle_f(*x as f32 + 2.5, y as f32 + 2.5, 2.5, col);
            } else {
                b.fill_rect(*x, y, 6, 6, col);
            }
            *x = small.draw(b, *x + 9, y, text, grey) + 12;
        };
        key(&mut b, &mut x, ly, palette::WHITE, true, "start");
        key(&mut b, &mut x, ly, stairs, false, "stairs");
        key(&mut b, &mut x, ly, door, false, "door");
        key(&mut b, &mut x, ly, item, false, "item");
        key(&mut b, &mut x, ly, named, true, "named NPC");
        key(&mut b, &mut x, ly, creature, true, "creature");
        key(&mut b, &mut x, ly, water, false, "water");
        key(&mut b, &mut x, ly, lava, false, "lava");
        small.draw(&mut b, left, ly + 12, "Brighter floor is higher. A dot in a tile marks a slope.", grey);
        small.draw(&mut b, left, ly + 20, "Numbers are tile coordinates: goto X Z puts you on tile X, Z (add .5 for its middle).", grey);
        gfx::save_bitmap_png(&format!("{}/level{}.png", dir, li + 1), &b, &pal, 2)?;
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("");
    let result = match cmd {
        "--dump-art" if args.len() >= 3 => dump_art(&args[2]),
        "--creatures" if args.len() >= 3 => dump_creatures(&args[2]),
        "--script" if args.len() >= 3 => headless::run(&args[2]),
        "--maps" if args.len() >= 3 => dump_maps(&args[2]),
        "--help" | "-h" => {
            println!("ultima-j            start the game");
            println!("ultima-j --play     skip the title and start playing");
            println!("ultima-j --script \"cmd; cmd; ...\"   run headless (see src/headless.rs)");
            println!("ultima-j --maps DIR   write a labelled top-down map of every level");
            println!("ultima-j --dump-art DIR | --creatures FILE   write generated art for review");
            println!("More in DEVELOPING.md.");
            Ok(())
        }
        "--play" => {
            platform::run_window(Box::new(ui::GameApp::new_playing_in(1, ui::save::load_settings().language).with_saved_settings()));
            Ok(())
        }
        _ => {
            platform::run_window(Box::new(ui::GameApp::new().with_saved_settings()));
            Ok(())
        }
    };
    if let Err(e) = result {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}
