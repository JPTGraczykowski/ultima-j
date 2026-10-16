//! Title screen, introduction, character creation and credits.

use super::{Ctx, ScreenCmd};
use crate::art::creatures::{self, Pose};
use crate::art::sprite3d::{render, View3};
use crate::audio::Sfx;
use crate::game::creatures::Race;
use crate::game::items::{Item, ItemKind, ALL_SKILLS};
use crate::game::magic::rune;
use crate::game::{EqSlot, PClass, Player, ALL_CLASSES};
use crate::gfx::palette::{c, Ramp, BLACK, INK};
use crate::gfx::Bitmap;
use crate::layout::Rect;
use crate::platform::{Input, Key};
use crate::util::{fbm2, hashf, Rng, V3};

fn gold_rows() -> [u8; 12] {
    [
        c(Ramp::Gold, 15),
        c(Ramp::Gold, 14),
        c(Ramp::Gold, 13),
        c(Ramp::Gold, 13),
        c(Ramp::Gold, 12),
        c(Ramp::Gold, 11),
        c(Ramp::Gold, 10),
        c(Ramp::Gold, 9),
        c(Ramp::Gold, 8),
        c(Ramp::Gold, 7),
        c(Ramp::Gold, 6),
        c(Ramp::Gold, 5),
    ]
}

fn bronze_rows() -> [u8; 12] {
    [
        c(Ramp::Copper, 13),
        c(Ramp::Copper, 12),
        c(Ramp::Copper, 12),
        c(Ramp::Copper, 11),
        c(Ramp::Copper, 10),
        c(Ramp::Copper, 10),
        c(Ramp::Copper, 9),
        c(Ramp::Copper, 8),
        c(Ramp::Copper, 7),
        c(Ramp::Copper, 7),
        c(Ramp::Copper, 6),
        c(Ramp::Copper, 5),
    ]
}

/// Swirling purple marble, like the original's title backdrop.
pub fn marble_background() -> Bitmap {
    let mut b = Bitmap::new(320, 200);
    for y in 0..200 {
        for x in 0..320 {
            let fx = x as f32;
            let fy = y as f32;
            let wx = fbm2(fx, fy, 512.0, 16, 4, 11) * 120.0;
            let wy = fbm2(fx + 37.0, fy, 512.0, 16, 4, 12) * 120.0;
            let v = fbm2(fx + wx, fy + wy, 512.0, 8, 5, 13);
            let vein = ((v * 26.0).sin() * 0.5 + 0.5).powf(3.0);
            let base = 0.25 + v * 0.55 - vein * 0.35 + (hashf(x, y, 1, 4) - 0.5) * 0.12;
            let s = (2.0 + base * 9.0).round() as i32;
            let col = if vein < 0.02 && v > 0.55 { c(Ramp::Silver, 8) } else { c(Ramp::Purple, s.clamp(1, 9)) };
            b.set(x, y, col);
        }
    }
    b
}

/// Rough reddish-purple granite with cracks (character creation backdrop).
pub fn granite_background() -> Bitmap {
    let mut b = Bitmap::new(320, 200);
    for y in 0..200 {
        for x in 0..320 {
            let n = fbm2(x as f32, y as f32, 512.0, 32, 5, 21);
            let cell = crate::util::worley2(x as f32, y as f32, 512.0, 5, 0.9, 1.0, 22);
            let crack = cell.f2 - cell.f1 < 0.012;
            let v = 3.0 + n * 4.0 + (hashf(x, y, 2, 3) - 0.5) * 1.2;
            let col = if crack { c(Ramp::Purple, 1) } else { c(Ramp::Red, (v * 0.55) as i32 + 1).max(c(Ramp::Purple, (v * 0.7) as i32)) };
            let col = if (x * 7 + y * 3) % 5 == 0 { c(Ramp::Purple, (v * 0.8) as i32) } else { col };
            b.set(x, y, if crack { col } else { c(Ramp::Purple, (v * 0.75) as i32 + 1) });
        }
    }
    b
}

pub struct Title {
    bg: Bitmap,
    sel: usize,
    t: f32,
    pub note: Option<String>,
}

const TITLE_ITEMS: [&str; 4] = ["Introduction", "Create Character", "Acknowledgements", "Journey Onward"];

fn item_rect(ctx: &Ctx, i: usize) -> Rect {
    let w = ctx.art.font_big.text_width(TITLE_ITEMS[i]);
    Rect::new(160 - w / 2 - 4, 92 + i as i32 * 20, w + 8, 16)
}

impl Title {
    pub fn new() -> Title {
        Title { bg: marble_background(), sel: 1, t: 0.0, note: None }
    }

    pub fn update(&mut self, ctx: &mut Ctx, input: &Input, dt: f32) -> Option<ScreenCmd> {
        self.t += dt;
        if input.hit(Key::Up) {
            self.sel = (self.sel + 3) % 4;
        }
        if input.hit(Key::Down) {
            self.sel = (self.sel + 1) % 4;
        }
        let mut choose = if input.hit(Key::Enter) { Some(self.sel) } else { None };
        for i in 0..4 {
            if item_rect(ctx, i).contains(input.mouse) {
                self.sel = i;
                if input.pressed[0] {
                    choose = Some(i);
                }
            }
        }
        if input.hit(Key::Escape) && self.t > 0.5 {
            return Some(ScreenCmd::Quit);
        }
        match choose {
            Some(0) => {
                ctx.sounds.push(Sfx::Click);
                Some(ScreenCmd::Intro)
            }
            Some(1) => {
                ctx.sounds.push(Sfx::Click);
                Some(ScreenCmd::CharGen)
            }
            Some(2) => Some(ScreenCmd::Credits),
            Some(3) => Some(ScreenCmd::LoadSlot(super::save::latest_slot())),
            _ => None,
        }
    }

    pub fn draw(&mut self, ctx: &mut Ctx, fb: &mut Bitmap, mouse: (i32, i32)) {
        fb.blit_opaque(&self.bg, 0, 0);
        draw_title_logo(ctx, fb, 22);
        for (i, s) in TITLE_ITEMS.iter().enumerate() {
            let r = item_rect(ctx, i);
            let rows = if i == self.sel { gold_rows() } else { bronze_rows() };
            ctx.art.font_big.draw_fancy(fb, r.x + 4, r.y + 2, s, &rows, BLACK, Some(c(Ramp::Purple, 1)));
        }
        if let Some(n) = &self.note {
            ctx.art.font.draw_centered(fb, 160, 180, n, c(Ramp::Gold, 12));
        }
        ctx.art.font.draw_centered(fb, 160, 192, "A tribute to the 1992 classic. Made with Rust.", c(Ramp::Purple, 11));
        let (b, hs) = &ctx.art.ui.cursors[0];
        fb.blit(b, mouse.0 - hs.0, mouse.1 - hs.1);
    }
}

/// The game logo: "Ultima-J" and the subtitle.
pub fn draw_title_logo(ctx: &Ctx, fb: &mut Bitmap, y: i32) {
    let big = &ctx.art.font_huge;
    let title = "Ultima-J";
    let w = big.text_width(title);
    let rows: Vec<u8> = (0..24).map(|i| c(Ramp::Gold, 15 - (i * 10 / 24))).collect();
    big.draw_fancy(fb, 160 - w / 2, y, title, &rows, BLACK, Some(c(Ramp::Purple, 1)));
    let sub = "The Abyss Below";
    let w2 = ctx.art.font_big.text_width(sub);
    let reds: Vec<u8> = (0..12).map(|i| c(Ramp::Fire, 12 - i / 2)).collect();
    ctx.art.font_big.draw_fancy(fb, 160 - w2 / 2, y + 30, sub, &reds, BLACK, Some(c(Ramp::Purple, 1)));
}

// ------------------------------------------------------- intro and ending

/// A story told in pictures: the introduction, or the ending after the
/// Lord of Souls is struck down.
pub struct Story {
    ending: bool,
    page: usize,
    t: f32,
    pics: Vec<Bitmap>,
}

const INTRO: [(&str, Ramp); 6] = [
    ("Awaken. I speak to thee from beyond the veil of death...", Ramp::Purple),
    ("Long ago, Lord Varian founded a colony deep within the great Abyss, where all peoples might live as one, bound by the light of the Lumen Stone.", Ramp::Gold),
    ("Then the Stone was broken into three shards, and the colony fell into ruin and war. Varian went down to the Throne of Souls, to keep the dead of the Abyss at rest.", Ramp::Gold),
    ("But the Lord of Souls grows weak. The dead no longer rest. One by one they rise from their graves - hollow, nameless, lost.", Ramp::Red),
    ("You open your eyes in cold earth. You remember nothing: not your name, not your death. Only a voice, calling from far below.", Ramp::Copper),
    ("The great doors above are sealed. Find out who you were, and how to live again. The Abyss is waiting...", Ramp::Silver),
];

/// The throne ending: nobody takes the throne, and the peoples go to war.
const FALL: [(&str, Ramp); 5] = [
    ("The Lord of Souls falls. His crown rings once upon the stone. \"So be it, my knight,\" he breathes, and the old king is still.", Ramp::Purple),
    ("You climb onto the Throne of Souls and wait for its power. Nothing comes. The shards flicker in your hand and go dark. There was no law - only an old man's will, and it is spent.", Ramp::Silver),
    ("Far above, the great doors groan. Their seal was Varian's, and it has died with him. For the first time since the Stone broke, daylight falls into the Abyss.", Ramp::Gold),
    ("The Stone is dead, the doors are open, and no one rules the Abyss. The mountain folk march up from the mines, the lizardmen rise from the warrens, the goblins hold the upper halls. Old neighbours kill each other in the ruins of the colony.", Ramp::Red),
    ("At the great doors the three peoples meet, weapons drawn, in the first daylight any of them has seen. Whoever holds the doorway will take the world above - and none of them will let the others go first.", Ramp::Copper),
];

fn framed(pic: &Bitmap) -> Bitmap {
    // Rough-edged frame like the original's intro vignettes.
    let mut b = Bitmap::new(pic.w, pic.h);
    for y in 0..pic.h {
        for x in 0..pic.w {
            let edge = 3.0 + hashf(x / 2, y / 2, 5, 9) * 4.0;
            let d = (x as f32).min((pic.w - 1 - x) as f32).min(y as f32).min((pic.h - 1 - y) as f32);
            if d > edge {
                b.set(x, y, pic.get(x, y).max(1));
            } else if d > edge - 1.5 {
                b.set(x, y, c(Ramp::Grey, 2));
            }
        }
    }
    b
}

fn scene(page: usize) -> Bitmap {
    let w = 220;
    let h = 120;
    let mut b = Bitmap::filled(w, h, c(Ramp::Grey, 1));
    let mut rng = Rng::new(page as u64 + 7);
    match page {
        0 => {
            b.fill(BLACK);
            let parts = creatures::model(Race::Mage, 0, &Pose::default());
            let v = View3 { yaw: 0.0, pitch: 0.0, scale: 260.0, w: 110, h: 120, center: V3::new(0.0, 0.72, 0.0), center_row: 62.0 };
            let mut face = render(&parts, &v, false);
            // Ghostly purple tint.
            for p in face.px.iter_mut() {
                if *p != 0 {
                    let s = crate::gfx::palette::ramp_of(*p).map(|(_, s)| s).unwrap_or(4);
                    *p = c(Ramp::Purple, (s as f32 * 0.9) as i32 + 1);
                }
            }
            for y in 0..face.h {
                for x in 0..face.w {
                    if face.get(x, y) != 0 && rng.chance(0.35) {
                        face.set(x, y, 0);
                    }
                }
            }
            b.blit(&face, w / 2 - 55, 0);
        }
        1 | 2 => {
            // A great hall of the colony: pillars and a shining stone.
            for y in 0..h {
                for x in 0..w {
                    let v = fbm2(x as f32, y as f32, 256.0, 16, 3, 3);
                    b.set(x, y, c(Ramp::Stone, (1.0 + v * 4.0) as i32));
                }
            }
            for k in 0..5 {
                let x = 20 + k * 45;
                b.fill_rect(x, 10, 12, 100, c(Ramp::Stone, 6));
                b.fill_rect(x + 9, 10, 3, 100, c(Ramp::Stone, 3));
                b.fill_rect(x + 1, 10, 2, 100, c(Ramp::Stone, 9));
            }
            b.fill_rect(0, 105, w, 15, c(Ramp::Stone, 3));
            if page == 1 {
                b.fill_circle_f(110.0, 60.0, 16.0, c(Ramp::Gold, 9));
                b.fill_circle_f(110.0, 60.0, 11.0, c(Ramp::Gold, 13));
                b.fill_circle_f(106.0, 56.0, 4.0, c(Ramp::Gold, 15));
            } else {
                for (dx, dy, r) in [(-26.0, 10.0, Ramp::Red), (0.0, -6.0, Ramp::Blue), (24.0, 12.0, Ramp::Green)] {
                    b.fill_polygon(&[(110.0 + dx, 50.0 + dy), (116.0 + dx, 62.0 + dy), (110.0 + dx, 74.0 + dy), (104.0 + dx, 62.0 + dy)], c(r, 10));
                    b.set((108.0 + dx) as i32, (58.0 + dy) as i32, c(r, 14));
                }
            }
        }
        3 => {
            // The Lord of Souls upon a throne of bones.
            throne_room(&mut b);
            let lord = render(&creatures::model(Race::Demon, 0, &Pose::default()), &View3 { yaw: 0.0, pitch: 0.05, scale: 62.0, w: 80, h: 100, center: V3::ZERO, center_row: 98.0 }, true);
            b.blit(&lord, 70, 6);
            skulls(&mut b);
        }
        4 => {
            // An open grave among the headstones.
            b.fill(c(Ramp::Grey, 1));
            for y in 60..h {
                for x in 0..w {
                    let v = fbm2(x as f32, y as f32, 256.0, 16, 3, 5);
                    b.set(x, y, c(Ramp::Earth, (1.0 + v * 3.0) as i32));
                }
            }
            for (x, hh) in [(18, 34), (52, 28), (160, 30), (196, 36)] {
                b.fill_rect(x, 66 - hh, 16, hh, c(Ramp::Stone, 6));
                b.fill_circle_f(x as f32 + 7.5, 66.0 - hh as f32, 8.0, c(Ramp::Stone, 6));
                b.fill_rect(x + 13, 66 - hh, 3, hh, c(Ramp::Stone, 3));
                b.hline(x + 4, x + 11, 70 - hh, c(Ramp::Stone, 3));
            }
            b.fill_ellipse(110.0, 92.0, 46.0, 16.0, BLACK);
            b.fill_ellipse(110.0, 90.0, 44.0, 13.0, c(Ramp::Earth, 1));
            // A grey hand reaching out of the earth.
            b.fill_rect(104, 74, 9, 14, c(Ramp::Grey, 8));
            for k in 0..4 {
                b.fill_rect(103 + k * 3, 66 + (k % 2) * 2, 2, 9, c(Ramp::Grey, 9));
            }
            b.fill_rect(113, 76, 3, 6, c(Ramp::Grey, 7));
        }
        _ => {
            // The great doors closing.
            b.fill(BLACK);
            great_doors(&mut b, 30, 0, 160, 120, 0);
            for y in 0..h {
                for x in 0..w {
                    if (x < 30 || x >= 190) && hashf(x, y, 1, 1) > 0.97 {
                        b.set(x, y, c(Ramp::Grey, 2));
                    }
                }
            }
        }
    }
    framed(&b)
}

/// A throne of bones in a purple gloom.
fn throne_room(b: &mut Bitmap) {
    b.fill(c(Ramp::Purple, 1));
    b.fill_rect(80, 20, 60, 90, c(Ramp::Parch, 5));
    b.fill_rect(85, 25, 50, 80, c(Ramp::Purple, 2));
    b.fill_rect(0, 100, b.w, 20, c(Ramp::Grey, 2));
    for k in 0..9 {
        let x = 84.0 + k as f32 * 6.5;
        b.fill_circle_f(x, 19.0, 3.5, c(Ramp::Parch, 9));
        b.set(x as i32 - 1, 19, BLACK);
        b.set(x as i32 + 1, 19, BLACK);
    }
}

/// Skulls scattered on the throne room floor.
fn skulls(b: &mut Bitmap) {
    for (x, y) in [(20, 104), (34, 108), (178, 106), (194, 103), (60, 110), (150, 110)] {
        b.fill_circle_f(x as f32, y as f32, 4.0, c(Ramp::Parch, 8));
        b.set(x - 1, y, BLACK);
        b.set(x + 1, y, BLACK);
    }
}

/// The two leaves of the great doors in the rectangle, `gap` pixels apart,
/// with daylight between them when they are open.
fn great_doors(b: &mut Bitmap, x: i32, y: i32, w: i32, h: i32, gap: i32) {
    let door = crate::art::textures::generate(crate::art::textures::Tex::DoorIron, 0);
    let leaf = (w - gap) / 2;
    b.blit_scaled(&door, x, y, leaf, h);
    b.blit_scaled(&door, x + leaf + gap, y, leaf, h);
    if gap == 0 {
        for xx in x + leaf - 2..x + leaf + 2 {
            b.vline(xx, y, y + h - 1, BLACK);
        }
        return;
    }
    let mid = x + w / 2;
    for xx in x + leaf..x + leaf + gap {
        let d = (xx - mid).abs() as f32 / (gap as f32 / 2.0);
        b.vline(xx, y, y + h - 1, c(Ramp::Gold, (15.0 - d * 3.0) as i32));
    }
}

/// A creature from the game, rendered for a vignette with its feet at the
/// bottom centre of the bitmap. `yaw` turns it (0 faces the viewer).
fn figure(race: Race, look: u8, yaw: f32, pose: &Pose, scale: f32) -> Bitmap {
    let w = (scale * 2.0) as i32;
    let h = (scale * 1.5) as i32;
    render(&creatures::model(race, look, pose), &View3 { yaw, pitch: 0.08, scale, w, h, center: V3::ZERO, center_row: h as f32 - 2.0 }, true)
}

/// Blit a figure so that its feet stand at (x, y).
fn stand(b: &mut Bitmap, fig: &Bitmap, x: i32, y: i32) {
    b.blit(fig, x - fig.w / 2, y - fig.h);
}

/// Darken a figure to a silhouette against firelight.
fn silhouette(fig: &Bitmap) -> Bitmap {
    let mut s = fig.clone();
    for p in s.px.iter_mut() {
        if *p != 0 {
            *p = c(Ramp::Red, 1);
        }
    }
    s
}

fn fall_scene(page: usize) -> Bitmap {
    let w = 220;
    let h = 120;
    let mut b = Bitmap::filled(w, h, c(Ramp::Grey, 1));
    let side = std::f32::consts::FRAC_PI_2;
    match page {
        0 => {
            // The old king dead at the foot of his throne.
            throne_room(&mut b);
            skulls(&mut b);
            let pose = Pose { death: 1.0, ..Default::default() };
            let v = View3 { yaw: 0.0, pitch: 0.45, scale: 40.0, w: 150, h: 50, center: V3::ZERO, center_row: 44.0 };
            let dead = render(&creatures::model(Race::Demon, 0, &pose), &v, true);
            stand(&mut b, &dead, 104, 116);
        }
        1 => {
            // Three dead shards in a grey, open hand.
            b.fill(c(Ramp::Purple, 1));
            for y in 0..h {
                for x in 0..w {
                    if hashf(x, y, 3, 7) > 0.985 {
                        b.set(x, y, c(Ramp::Purple, 3));
                    }
                }
            }
            let (skin, lit) = (c(Ramp::Grey, 6), c(Ramp::Grey, 8));
            b.fill_rect(94, 92, 34, 28, skin);
            b.fill_rect(98, 92, 26, 28, lit);
            for (k, len) in [14.0f32, 18.0, 17.0, 13.0].into_iter().enumerate() {
                let x = 78.0 + k as f32 * 15.0;
                let top = 46.0 - len + (k as f32 - 1.5).abs() * 4.0;
                b.fill_ellipse(x, top + len / 2.0 + 4.0, 7.0, len / 2.0 + 6.0, skin);
                b.fill_ellipse(x + 1.0, top + len / 2.0 + 4.0, 5.0, len / 2.0 + 4.0, lit);
                b.hline(x as i32 - 4, x as i32 + 4, (top + len * 0.6) as i32 + 4, skin);
            }
            b.fill_ellipse(150.0, 82.0, 20.0, 8.0, skin);
            b.fill_ellipse(151.0, 81.0, 17.0, 6.0, lit);
            b.fill_ellipse(110.0, 76.0, 38.0, 24.0, skin);
            b.fill_ellipse(111.0, 74.0, 33.0, 19.0, lit);
            b.line(84, 68, 136, 64, skin);
            b.line(88, 84, 130, 90, skin);
            for (dx, dy) in [(-18.0, 4.0), (0.0, -4.0), (18.0, 4.0)] {
                let (x, y) = (111.0 + dx, 76.0 + dy);
                b.fill_polygon(&[(x, y - 13.0), (x + 7.0, y), (x, y + 9.0), (x - 7.0, y)], c(Ramp::Slate, 3));
                b.fill_polygon(&[(x, y - 11.0), (x + 5.0, y), (x, y + 7.0), (x - 2.0, y - 1.0)], c(Ramp::Slate, 5));
                b.line(x as i32 - 2, y as i32 - 5, x as i32 + 2, y as i32 + 3, c(Ramp::Grey, 1));
            }
        }
        2 => {
            // The great doors opening on daylight.
            b.fill(BLACK);
            for y in 96..h {
                let spread = (y - 96) as f32 * 1.6 + 12.0;
                for x in 0..w {
                    let d = (x as f32 - 110.0).abs();
                    if d < spread {
                        let s = 9.0 - d / spread * 6.0 - (y - 96) as f32 * 0.12;
                        b.set(x, y, c(Ramp::Gold, s as i32));
                    } else {
                        b.set(x, y, c(Ramp::Stone, 1 + (hashf(x, y, 4, 2) * 2.0) as i32));
                    }
                }
            }
            great_doors(&mut b, 34, 0, 152, 96, 24);
            // Dust in the light.
            for k in 0..40 {
                let x = 98 + (hashf(k, 1, 6, 1) * 24.0) as i32;
                let y = (hashf(k, 2, 6, 1) * 110.0) as i32;
                b.set(x, y, c(Ramp::Gold, 15));
            }
        }
        3 => {
            // The colony burns; its peoples fight in the ruins.
            for y in 0..h {
                for x in 0..w {
                    let v = fbm2(x as f32, y as f32, 256.0, 16, 3, 9);
                    let glow = (y as f32 / h as f32).powf(1.5) * 9.0;
                    b.set(x, y, c(Ramp::Fire, (v * 4.0 + glow) as i32 - 2));
                }
            }
            for (k, x) in [12, 64, 150, 196].into_iter().enumerate() {
                let top = if k % 2 == 0 { 14 } else { 34 };
                b.fill_rect(x, top, 12, 110 - top, c(Ramp::Stone, 1));
                b.fill_polygon(&[(x as f32, top as f32), (x as f32 + 12.0, top as f32 + 5.0), (x as f32 + 6.0, top as f32 - 6.0)], c(Ramp::Stone, 1));
            }
            b.fill_rect(0, 108, w, 12, c(Ramp::Red, 1));
            let attack = Pose { attack: 0.6, ..Default::default() };
            let fighters = [
                (Race::Dwarf, 0, side, 40),
                (Race::GreenGoblin, 0, -side, 66),
                (Race::Lizardman, 0, side, 112),
                (Race::GrayGoblin, 0, -side, 138),
                (Race::Dwarf, 1, side, 168),
                (Race::Lizardman, 1, -side, 194),
            ];
            for (race, look, yaw, x) in fighters {
                let f = silhouette(&figure(race, look, yaw, &attack, 40.0));
                stand(&mut b, &f, x, 112);
            }
            // Embers.
            for k in 0..30 {
                let x = (hashf(k, 3, 5, 1) * w as f32) as i32;
                let y = (hashf(k, 4, 5, 1) * 90.0) as i32;
                b.set(x, y, c(Ramp::Fire, 14));
            }
        }
        _ => {
            // The three peoples at the open doors, ready to fight their way out.
            b.fill(BLACK);
            for y in 80..h {
                for x in 0..w {
                    let d = (x as f32 - 110.0).abs();
                    let spread = 10.0 + (y - 80) as f32 * 1.2;
                    let s = if d < spread { 7.0 - d / spread * 4.0 } else { 1.0 + hashf(x, y, 4, 2) * 2.0 };
                    b.set(x, y, if d < spread { c(Ramp::Gold, s as i32) } else { c(Ramp::Stone, s as i32) });
                }
            }
            great_doors(&mut b, 60, 0, 100, 80, 20);
            let ready = Pose { attack: 0.25, ..Default::default() };
            // Mountain folk on the left, lizardmen on the right, both turned
            // toward the goblins who hold the doorway.
            for (i, (x, y)) in [(16, 108), (36, 116), (24, 96)].into_iter().enumerate() {
                let f = figure(Race::Dwarf, i as u8, side * 0.7, &ready, 44.0);
                stand(&mut b, &f, x, y);
            }
            for (i, (x, y)) in [(204, 108), (184, 116), (196, 96)].into_iter().enumerate() {
                let f = figure(Race::Lizardman, i as u8, -side * 0.7, &ready, 40.0);
                stand(&mut b, &f, x, y);
            }
            for (race, x, y) in [(Race::GrayGoblin, 92, 98), (Race::GreenGoblin, 128, 98), (Race::GrayGoblin, 110, 114)] {
                let f = figure(race, 0, std::f32::consts::PI, &ready, 44.0);
                stand(&mut b, &f, x, y);
            }
        }
    }
    framed(&b)
}

impl Story {
    pub fn intro() -> Story {
        Story { ending: false, page: 0, t: 0.0, pics: (0..INTRO.len()).map(scene).collect() }
    }

    /// The ending after the Lord of Souls is struck down.
    pub fn fall() -> Story {
        Story { ending: true, page: 0, t: 0.0, pics: (0..FALL.len()).map(fall_scene).collect() }
    }

    fn pages(&self) -> &'static [(&'static str, Ramp)] {
        if self.ending { &FALL } else { &INTRO }
    }

    pub fn update(&mut self, _ctx: &mut Ctx, input: &Input, dt: f32) -> Option<ScreenCmd> {
        self.t += dt;
        let done = if self.ending { ScreenCmd::Credits } else { ScreenCmd::CharGen };
        if input.hit(Key::Escape) {
            return Some(if self.ending { done } else { ScreenCmd::Title });
        }
        if (self.t > 0.4 && (input.any_click() || input.any_key())) || self.t > 12.0 {
            self.page += 1;
            self.t = 0.0;
            if self.page >= self.pages().len() {
                return Some(done);
            }
        }
        None
    }

    pub fn draw(&mut self, ctx: &mut Ctx, fb: &mut Bitmap) {
        fb.fill(BLACK);
        let pages = self.pages();
        let p = self.page.min(pages.len() - 1);
        let pic = &self.pics[p];
        fb.blit(pic, 160 - pic.w / 2, 18);
        let (text, ramp) = pages[p];
        let lines = ctx.art.font.wrap(text, 260);
        for (i, l) in lines.iter().enumerate() {
            ctx.art.font.draw_centered(fb, 160, 146 + i as i32 * 8, l, c(ramp, 12));
        }
        if self.ending && p == pages.len() - 1 && self.t > 1.0 {
            ctx.art.font.draw_centered(fb, 160, 192, "- The End -", c(Ramp::Red, 10));
        }
    }
}

// --------------------------------------------------------- character creation

pub struct CharGen {
    bg: Bitmap,
    step: usize,
    female: bool,
    class: usize,
    look: usize,
    name: String,
    sel: usize,
    rng: Rng,
    player: Player,
}

const LOOKS: [(u8, u8); 4] = [(0, 0), (0, 1), (1, 2), (2, 3)];

impl CharGen {
    pub fn new() -> CharGen {
        let mut cg = CharGen {
            bg: granite_background(),
            step: 0,
            female: false,
            class: 0,
            look: 0,
            name: String::new(),
            sel: 0,
            rng: Rng::new(42),
            player: Player::new("", PClass::Fighter, false),
        };
        cg.reroll();
        cg
    }

    fn reroll(&mut self) {
        let class = ALL_CLASSES[self.class];
        let mut p = Player::new(&self.name, class, self.female);
        p.st += self.rng.range(-2, 4);
        p.dex += self.rng.range(-2, 4);
        p.int += self.rng.range(-2, 4);
        p.max_vit = 20 + p.st / 2 + self.rng.range(0, 6);
        p.vit = p.max_vit;
        let (skin, hair) = LOOKS[self.look];
        p.skin = skin;
        p.hair = hair;
        self.player = p;
    }

    fn options(&self) -> Vec<String> {
        match self.step {
            0 => vec!["Male".into(), "Female".into()],
            1 => ALL_CLASSES.iter().map(|c| c.name().to_string()).collect(),
            2 => vec!["Fair".into(), "Golden".into(), "Dusky".into(), "Bronze".into()],
            3 => vec![],
            _ => vec!["Yes".into(), "No".into()],
        }
    }

    fn question(&self) -> &'static str {
        match self.step {
            0 => "Choose thy sex:",
            1 => "Choose thy profession:",
            2 => "Choose thine appearance:",
            3 => "What is thy name?",
            _ => "Keep this character?",
        }
    }

    fn opt_rect(&self, i: usize) -> Rect {
        let n = self.options().len() as i32;
        let cols = if n > 4 { 2 } else { 1 };
        let rows = (n + cols - 1) / cols;
        let col = i as i32 / rows;
        let row = i as i32 % rows;
        let w = if cols == 2 { 64 } else { 90 };
        let x0 = if cols == 2 { 172 } else { 196 };
        Rect::new(x0 + col * (w + 6), 92 + row * 16, w, 13)
    }

    pub fn update(&mut self, ctx: &mut Ctx, input: &Input, _dt: f32) -> Option<ScreenCmd> {
        if input.hit(Key::Escape) {
            if self.step == 0 {
                return Some(ScreenCmd::Title);
            }
            self.step -= 1;
            self.sel = 0;
            return None;
        }
        let opts = self.options();
        let mut choose: Option<usize> = None;
        if self.step == 3 {
            for ch in &input.chars {
                if (ch.is_alphanumeric() || *ch == ' ' || *ch == '\'' || *ch == '-') && self.name.len() < 14 {
                    self.name.push(*ch);
                }
            }
            if input.hit(Key::Backspace) {
                self.name.pop();
            }
            if input.hit(Key::Enter) || (input.pressed[0] && Rect::new(200, 130, 80, 14).contains(input.mouse)) {
                if self.name.trim().is_empty() {
                    self.name = if self.female { "Avatara".into() } else { "Avatar".into() };
                }
                self.player.name = self.name.trim().to_string();
                self.step += 1;
                self.sel = 0;
                ctx.sounds.push(Sfx::Click);
            }
            return None;
        }
        if !opts.is_empty() {
            if input.hit(Key::Up) {
                self.sel = (self.sel + opts.len() - 1) % opts.len();
            }
            if input.hit(Key::Down) {
                self.sel = (self.sel + 1) % opts.len();
            }
            if input.hit(Key::Enter) {
                choose = Some(self.sel);
            }
            for i in 0..opts.len() {
                if self.opt_rect(i).contains(input.mouse) {
                    self.sel = i;
                    if input.pressed[0] {
                        choose = Some(i);
                    }
                }
            }
        }
        if let Some(i) = choose {
            ctx.sounds.push(Sfx::Click);
            match self.step {
                0 => self.female = i == 1,
                1 => self.class = i,
                2 => self.look = i,
                _ => {
                    if i == 0 {
                        let mut p = self.player.clone();
                        starting_kit(&mut p);
                        return Some(ScreenCmd::NewGame(Box::new(p)));
                    }
                    self.step = 0;
                    self.sel = 0;
                    self.reroll();
                    return None;
                }
            }
            self.step += 1;
            self.sel = 0;
            self.reroll();
        }
        None
    }

    pub fn draw(&mut self, ctx: &mut Ctx, fb: &mut Bitmap, mouse: (i32, i32)) {
        fb.blit_opaque(&self.bg, 0, 0);
        let f = &ctx.art.font;
        let big = &ctx.art.font_big;
        let text_rows = [c(Ramp::Silver, 13), c(Ramp::Silver, 13), c(Ramp::Silver, 12), c(Ramp::Silver, 12), c(Ramp::Silver, 11), c(Ramp::Silver, 11), c(Ramp::Silver, 10), c(Ramp::Silver, 10), c(Ramp::Silver, 9), c(Ramp::Silver, 9), c(Ramp::Silver, 8), c(Ramp::Silver, 8)];
        let panel = |fb: &mut Bitmap, r: Rect| {
            fb.rect(r.x, r.y, r.w, r.h, BLACK);
            fb.rect(r.x + 1, r.y + 1, r.w - 2, r.h - 2, c(Ramp::Gold, 9));
            fb.rect(r.x + 2, r.y + 2, r.w - 4, r.h - 4, c(Ramp::Gold, 5));
            fb.rect(r.x + 3, r.y + 3, r.w - 6, r.h - 6, BLACK);
        };
        // Divider between left and right halves.
        fb.vline(160, 0, 199, BLACK);
        fb.vline(161, 0, 199, c(Ramp::Purple, 7));
        let p = &self.player;
        // Name / sex / class panel.
        let r1 = Rect::new(10, 6, 140, 30);
        panel(fb, r1);
        let name = if self.step >= 4 { p.name.clone() } else if self.step == 3 { format!("{}_", self.name) } else { "".into() };
        big.draw_fancy(fb, 80 - big.text_width(&name) / 2, 9, &name, &text_rows, BLACK, None);
        if self.step > 0 {
            big.draw_fancy(fb, 16, 22, if self.female { "Female" } else { "Male" }, &text_rows, BLACK, None);
        }
        if self.step > 1 {
            let cn = ALL_CLASSES[self.class].name();
            big.draw_fancy(fb, 144 - big.text_width(cn), 22, cn, &text_rows, BLACK, None);
        }
        // Portrait (paperdoll).
        let r2 = Rect::new(10, 40, 62, 78);
        panel(fb, r2);
        let doll = ctx.cache.paperdoll(p).clone();
        fb.blit(&doll, r2.x + r2.w / 2 - doll.w / 2, r2.y + 4);
        // Stats.
        let r3 = Rect::new(76, 40, 74, 78);
        panel(fb, r3);
        if self.step > 1 {
            for (i, (l, v)) in [("Str:", p.st), ("Dex:", p.dex), ("Int:", p.int), ("Vit:", p.max_vit)].iter().enumerate() {
                big.draw_fancy(fb, 84, 48 + i as i32 * 17, l, &text_rows, BLACK, None);
                let s = v.to_string();
                big.draw_fancy(fb, 142 - big.text_width(&s), 48 + i as i32 * 17, &s, &text_rows, BLACK, None);
            }
        }
        // Skills.
        let r4 = Rect::new(10, 122, 140, 72);
        panel(fb, r4);
        if self.step > 1 {
            let mut y = 128;
            for s in ALL_SKILLS.iter().filter(|s| p.skills[**s as usize] > 0).take(6) {
                f.draw(fb, 20, y, s.name(), c(Ramp::Silver, 12));
                let v = p.skills[*s as usize].to_string();
                f.draw(fb, 130 - f.text_width(&v), y, &v, c(Ramp::Silver, 12));
                y += 10;
            }
        }
        // Question and options.
        let q = self.question();
        for (i, l) in f.wrap(q, 140).iter().enumerate() {
            let w = big.text_width(l);
            if w <= 150 {
                big.draw_fancy(fb, 240 - w / 2, 62 + i as i32 * 14, l, &text_rows, BLACK, None);
            } else {
                f.draw_centered(fb, 240, 66 + i as i32 * 8, l, c(Ramp::Silver, 13));
            }
        }
        if self.step == 3 {
            let r = Rect::new(200, 130, 80, 14);
            fb.fill_rect(r.x, r.y, r.w, r.h, c(Ramp::Purple, 3));
            fb.rect(r.x, r.y, r.w, r.h, BLACK);
            f.draw_centered(fb, 240, r.y + 4, "Done", c(Ramp::Silver, 13));
            f.draw_centered(fb, 240, 100, "Type a name and press Enter.", c(Ramp::Silver, 10));
        }
        for (i, o) in self.options().iter().enumerate() {
            let r = self.opt_rect(i);
            fb.fill_rect(r.x, r.y, r.w, r.h, if i == self.sel { c(Ramp::Purple, 6) } else { c(Ramp::Purple, 3) });
            fb.rect(r.x, r.y, r.w, r.h, BLACK);
            f.draw_centered(fb, r.x + r.w / 2, r.y + 4, o, c(Ramp::Silver, 13));
            if i == self.sel {
                let (cb, _) = &ctx.art.ui.cursors[0];
                fb.blit(cb, r.x + r.w - 11, r.y + 2);
            }
        }
        if self.step == 1 {
            let desc = match ALL_CLASSES[self.sel] {
                PClass::Fighter => "Strong and skilled with weapons.",
                PClass::Mage => "A wielder of rune magic.",
                PClass::Bard => "Charming, learned, a little of all.",
                PClass::Tinker => "Clever with locks, traps and tools.",
                PClass::Druid => "Magic, lore and the wild.",
                PClass::Paladin => "A holy warrior.",
                PClass::Ranger => "Tracker, archer, swordsman.",
                PClass::Shepherd => "Humble, hardy and resourceful.",
            };
            f.draw_centered(fb, 240, 170, desc, c(Ramp::Silver, 11));
        }
        let (b, hs) = &ctx.art.ui.cursors[0];
        fb.blit(b, mouse.0 - hs.0, mouse.1 - hs.1);
        let _ = INK;
    }
}

/// Give a new character their starting possessions.
pub fn starting_kit(p: &mut Player) {
    let mut bag = Item::new(ItemKind::RuneBag);
    bag.contents.clear();
    p.pack[0] = Some(bag);
    p.pack[1] = Some(Item::new(ItemKind::Apple));
    match p.class {
        PClass::Fighter | PClass::Paladin => {
            *p.equip.get_mut(EqSlot::Chest) = Some(Item::new(ItemKind::LeatherVest));
            *p.equip.get_mut(EqSlot::Weapon) = Some(Item::new(ItemKind::Dagger));
        }
        PClass::Mage | PClass::Druid => {
            p.runes[rune::IN as usize] = true;
            p.runes[rune::LOR as usize] = true;
            p.runes[rune::ORT as usize] = true;
            p.runes[rune::JUX as usize] = true;
            p.pack[2] = Some(Item::new(ItemKind::Candle));
        }
        PClass::Tinker => {
            p.pack[2] = Some(Item::new(ItemKind::Lockpick));
            *p.equip.get_mut(EqSlot::Weapon) = Some(Item::new(ItemKind::Cudgel));
        }
        PClass::Ranger => {
            *p.equip.get_mut(EqSlot::Weapon) = Some(Item::new(ItemKind::Sling));
            p.pack[2] = Some(Item::new(ItemKind::SlingStone).qty(12));
            *p.equip.get_mut(EqSlot::Feet) = Some(Item::new(ItemKind::LeatherBoots));
        }
        PClass::Bard => {
            p.pack[2] = Some(Item::new(ItemKind::Coin).qty(20));
            *p.equip.get_mut(EqSlot::Weapon) = Some(Item::new(ItemKind::Dagger));
        }
        PClass::Shepherd => {
            *p.equip.get_mut(EqSlot::Weapon) = Some(Item::new(ItemKind::Cudgel));
            p.pack[2] = Some(Item::new(ItemKind::Bread));
        }
    }
}

pub fn draw_credits(ctx: &mut Ctx, fb: &mut Bitmap, t: f32) {
    fb.fill(BLACK);
    draw_title_logo(ctx, fb, 8);
    let lines = [
        "",
        "A loving tribute to Ultima Underworld: The Stygian Abyss",
        "(Blue Sky Productions / ORIGIN Systems, 1992).",
        "",
        "Engine, art and sound: all generated procedurally in Rust.",
        "Software renderer: 320x200, 256 colours, light tables.",
        "Creatures: ray-cast clay models, pixelated and outlined.",
        "",
        "Story, characters and dialogue are original to this game.",
        "",
        "Thank you for playing.",
    ];
    let scroll = (t * 6.0) as i32;
    for (i, l) in lines.iter().enumerate() {
        let y = 70 + i as i32 * 9 - scroll.min(0);
        ctx.art.font.draw_centered(fb, 160, y, l, c(Ramp::Gold, 12));
    }
    ctx.art.font.draw_centered(fb, 160, 188, "(click to return)", c(Ramp::Purple, 10));
}
