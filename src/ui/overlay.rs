//! Overlays on the main game screen: options, save slots, texts, sleep,
//! death and victory; plus input for conversations and the automap.

use super::play::PlayState;
use super::{Ctx, Overlay, ScreenCmd};
use crate::audio::Sfx;
use crate::game::dialogue;
use crate::gfx::palette::{c, Ramp, BLACK, INK, SCROLL_BG};
use crate::gfx::Bitmap;
use crate::layout::*;
use crate::platform::{Input, Key};
use crate::util::V3;

pub const OPTIONS: [&str; 8] = ["Save Game", "Restore Game", "Music", "Sound", "Help", "Return to Game", "Quit to Title", "Quit Game"];

pub const HELP_TEXT: &str = "MOVING: W run, S walk, X back, A/D turn, Z/C sidestep (or the arrow keys). Hold the left mouse button in the view: the cursor arrow shows the direction, and further from the centre is faster. J or Space jumps. 1/3 look down/up, 2 centres the view. E/Q fly up/down when flying.\n\nCOMMANDS: click an icon on the left (or F2 Talk, F3 Get, F4 Look, F5 Fight, F6 Use), then right-click things in the view. With no icon selected, right-click looks; hold and drag the right button to get, use or talk.\n\nINVENTORY: left-click picks items up and puts them down. Drop armour on your body, weapons and lights in the hand and shoulder circles. Right-click an item to use it: eat, drink, read, light a torch, open a bag, repair worn weapons and armour. Left-click the rune bag to see your runes. Click the chain (or F7) for your statistics; spend skill points there after gaining a level.\n\nFIGHTING: select Fight (F5), aim, hold the right button to draw back and watch the power gem, release to strike. High in the view bashes, middle slashes, low thrusts (or P, ; and . keys).\n\nMAGIC: click runes in the rune bag to put them on the shelf, then click the shelf (or F8) to cast. Some spells ask for a target. Right-click the shelf to clear it.\n\nOTHER: M or Tab opens the map. F9 tracks, F10 sleeps. Use a trap you have spotted to try to disarm it. Click the compass for your condition, the flasks for vitality and mana. Ctrl+S quick-saves, Ctrl+R restores, F1/Esc opens the options. F11 toggles fullscreen, F12 toggles the 4:3 aspect.";


/// The parchment area used for texts and conversations (over the view).
pub const TEXT_AREA: Rect = Rect::new(52, 46, 172, 84);

fn number_key(input: &Input) -> Option<usize> {
    let keys = [Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5, Key::Num6, Key::Num7, Key::Num8, Key::Num9];
    keys.iter().position(|k| input.hit(*k))
}

/// Split text into pages that fit the text area.
pub fn pages(ctx: &Ctx, text: &str, area: Rect) -> Vec<Vec<String>> {
    let lines = ctx.art.font.wrap(text, area.w - 10);
    let per = ((area.h - 10) / 6).max(1) as usize;
    let mut out: Vec<Vec<String>> = lines.chunks(per).map(|c| c.to_vec()).collect();
    if out.is_empty() {
        out.push(Vec::new());
    }
    out
}

pub fn update(st: &mut PlayState, ctx: &mut Ctx, input: &Input, dt: f32) -> Option<ScreenCmd> {
    let click = input.pressed[0] || input.pressed[1];
    let m = input.mouse;
    match &mut st.overlay {
        Overlay::None => {}
        Overlay::Options(sel) => {
            if input.hit(Key::Up) {
                *sel = (*sel + OPTIONS.len() - 1) % OPTIONS.len();
            }
            if input.hit(Key::Down) {
                *sel = (*sel + 1) % OPTIONS.len();
            }
            let mut choose = None;
            if input.hit(Key::Enter) {
                choose = Some(*sel);
            }
            if input.hit(Key::Escape) {
                st.overlay = Overlay::None;
                return None;
            }
            for i in 0..OPTIONS.len() {
                let r = option_rect(i);
                if r.contains(m) {
                    *sel = i;
                    if input.pressed[0] {
                        choose = Some(i);
                    }
                }
            }
            if let Some(i) = choose {
                ctx.sounds.push(Sfx::Click);
                match i {
                    0 => st.overlay = Overlay::Slots(true, 0),
                    1 => st.overlay = Overlay::Slots(false, 0),
                    2 => {
                        ctx.music_on = !ctx.music_on;
                        let s = if ctx.music_on { "Music is on." } else { "Music is off." };
                        st.game.sys(s);
                    }
                    3 => {
                        ctx.sound_on = !ctx.sound_on;
                        let s = if ctx.sound_on { "Sound effects are on." } else { "Sound effects are off." };
                        st.game.sys(s);
                    }
                    4 => st.overlay = Overlay::Text(HELP_TEXT.to_string(), 0),
                    5 => st.overlay = Overlay::None,
                    6 => return Some(ScreenCmd::Title),
                    _ => return Some(ScreenCmd::Quit),
                }
            }
        }
        Overlay::Slots(saving, sel) => {
            if input.hit(Key::Escape) {
                st.overlay = Overlay::Options(0);
                return None;
            }
            if input.hit(Key::Up) {
                *sel = (*sel + 3) % 4;
            }
            if input.hit(Key::Down) {
                *sel = (*sel + 1) % 4;
            }
            let mut choose = None;
            if input.hit(Key::Enter) {
                choose = Some(*sel);
            }
            for i in 0..4 {
                if option_rect(i + 1).contains(m) {
                    *sel = i;
                    if input.pressed[0] {
                        choose = Some(i);
                    }
                }
            }
            if input.pressed[0] && option_rect(5).contains(m) {
                st.overlay = Overlay::Options(0);
                return None;
            }
            if let Some(i) = choose {
                let saving = *saving;
                if saving {
                    match super::save::save(&st.game, i) {
                        Ok(()) => st.game.sys("Save game succeeded."),
                        Err(e) => st.game.sys(format!("Save failed: {}", e)),
                    }
                    st.overlay = Overlay::None;
                } else {
                    return Some(ScreenCmd::LoadSlot(i));
                }
            }
        }
        Overlay::Map(lvl) => {
            let n = st.game.levels.len();
            let lvl_now = *lvl;
            if let Some((nl, x, z, text)) = st.ui.note.as_mut() {
                for ch in &input.chars {
                    if text.len() < 24 {
                        text.push(*ch);
                    }
                }
                if input.hit(Key::Backspace) {
                    text.pop();
                }
                if input.hit(Key::Enter) {
                    let (nl, x, z, text) = (*nl, *x, *z, text.clone());
                    if !text.trim().is_empty() {
                        st.game.levels[nl].notes.push((x, z, text));
                    }
                    st.ui.note = None;
                } else if input.hit(Key::Escape) {
                    st.ui.note = None;
                }
                return None;
            }
            if input.pressed[0] && super::automap::MAP_AREA.contains(m) {
                if let Some((tx, tz)) = super::automap::tile_at(&st.game, lvl_now, m) {
                    // Clicking an existing note erases it.
                    let notes = &mut st.game.levels[lvl_now].notes;
                    if let Some(i) = notes.iter().position(|(x, z, _)| (x - tx).abs() <= 1 && (z - tz).abs() <= 1) {
                        notes.remove(i);
                    } else {
                        st.ui.note = Some((lvl_now, tx, tz, String::new()));
                    }
                }
                return None;
            }
            if input.hit(Key::Escape) || input.hit(Key::M) || input.hit(Key::Tab) || (input.pressed[0] && super::automap::CLOSE.contains(m)) {
                st.overlay = Overlay::None;
                return None;
            }
            if input.hit(Key::Up) || input.hit(Key::PageUp) || (input.pressed[0] && super::automap::UP.contains(m)) {
                *lvl = lvl.saturating_sub(1);
            }
            if input.hit(Key::Down) || input.hit(Key::PageDown) || (input.pressed[0] && super::automap::DOWN.contains(m)) {
                *lvl = (*lvl + 1).min(n - 1);
            }
            // Only show levels the player has visited.
            while *lvl > 0 && !st.game.levels[*lvl].map.tiles.iter().any(|t| t.explored) {
                *lvl -= 1;
            }
        }
        Overlay::Text(text, page) => {
            let ps = pages(ctx, text, TEXT_AREA);
            if click || input.any_key() {
                *page += 1;
                if *page >= ps.len() {
                    st.overlay = Overlay::None;
                }
            }
        }
        Overlay::Sleep(t) => {
            *t += dt * 0.6;
            if *t >= 1.0 {
                st.overlay = match st.ui.pending_text.take() {
                    Some(text) => Overlay::Text(text, 0),
                    None => Overlay::None,
                };
            }
        }
        Overlay::Dead(t) => {
            *t += dt;
            if *t > 1.5 && (click || input.any_key()) {
                if st.game.flag("sapling_planted") > 0 {
                    resurrect(st);
                    return None;
                }
                // Restore the most recent save if there is one, else title.
                if input.hit(Key::Escape) {
                    return Some(ScreenCmd::Title);
                }
                if super::save::slot_exists(0) || super::save::slot_exists(1) || super::save::slot_exists(2) || super::save::slot_exists(3) {
                    let slot = super::save::latest_slot();
                    return Some(ScreenCmd::LoadSlot(slot));
                }
                return Some(ScreenCmd::Title);
            }
        }
        Overlay::Victory(t) => {
            *t += dt;
            if *t > 2.0 && (click || input.any_key()) {
                return Some(ScreenCmd::Credits);
            }
        }
        Overlay::Conversation(..) => {
            conversation_input(st, ctx, input);
        }
    }
    None
}

fn resurrect(st: &mut PlayState) {
    let g = &mut st.game;
    let lvl = g.flag("sapling_level") as usize;
    let x = g.flag("sapling_x") as f32 / 100.0;
    let z = g.flag("sapling_z") as f32 / 100.0;
    g.player.dead = false;
    g.player.vit = g.player.max_vit;
    g.player.poison = 0.0;
    if lvl != g.cur {
        g.change_level(lvl, x, z, 0.0);
    } else {
        let y = g.level().map.floor_at(x, z);
        g.player.pos = V3::new(x, y, z);
    }
    g.sys("The silver sapling has called you back from death!");
    st.overlay = Overlay::None;
    st.mesh = crate::render::mesh::LevelMesh::build(&st.game.level().map);
    st.mesh_level = st.game.cur;
}

fn conversation_input(st: &mut PlayState, ctx: &mut Ctx, input: &Input) {
    let Overlay::Conversation(cs, page) = &mut st.overlay else { return };
    let ps = pages(ctx, &cs.text, TEXT_AREA);
    let click = input.pressed[0] || input.pressed[1];
    if *page + 1 < ps.len() {
        if click || input.any_key() {
            *page += 1;
        }
        return;
    }
    if cs.ended || cs.options.is_empty() {
        if click || input.any_key() {
            st.overlay = Overlay::None;
        }
        return;
    }
    let mut choice = number_key(input);
    if input.pressed[0] {
        let lines = option_lines(ctx, &cs.options);
        for (row, (opt_idx, _)) in lines.iter().enumerate().take(SCROLL_LINES as usize) {
            let y = SCROLL_TEXT_Y + row as i32 * 6;
            if input.mouse.1 >= y && input.mouse.1 < y + 6 && input.mouse.0 >= SCROLL.x && input.mouse.0 < SCROLL.x + SCROLL.w {
                choice = Some(*opt_idx);
            }
        }
    }
    if let Some(i) = choice {
        if i < cs.options.len() {
            let opt = cs.options[i].clone();
            ctx.sounds.push(Sfx::Click);
            let mut st2 = std::mem::replace(cs, Box::new(dialogue::ConvState { conv: 0, npc: 0, node: String::new(), text: String::new(), options: Vec::new(), ended: true }));
            st.game.messages.push(crate::game::Msg { text: opt.text.clone(), kind: crate::game::MsgKind::Speech });
            dialogue::enter_node(&mut st.game, &ctx.db, &mut st2, &opt.next);
            if st2.ended && st2.text.is_empty() || opt.next == "end" {
                st.overlay = Overlay::None;
            } else {
                st.overlay = Overlay::Conversation(st2, 0);
            }
            st.handle_events(ctx);
        }
    }
}

/// Option lines for the message scroll: (option index, text line).
pub fn option_lines(ctx: &Ctx, opts: &[dialogue::Opt]) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (i, o) in opts.iter().enumerate() {
        let text = format!("{}. {}", i + 1, o.text);
        for (k, l) in ctx.art.font.wrap(&text, SCROLL.w - 8).into_iter().enumerate() {
            out.push((i, if k == 0 { l } else { format!("   {}", l) }));
        }
    }
    out
}

fn option_rect(i: usize) -> Rect {
    Rect::new(TEXT_AREA.x + 30, TEXT_AREA.y + 10 + i as i32 * 9, TEXT_AREA.w - 60, 8)
}

pub fn parchment(fb: &mut Bitmap, r: Rect) {
    fb.fill_rect(r.x, r.y, r.w, r.h, SCROLL_BG);
    for x in r.x..r.x + r.w {
        for y in r.y..r.y + r.h {
            let n = crate::util::hashf(x, y, 3, 77);
            if n > 0.93 {
                fb.set(x, y, c(Ramp::Parch, 9));
            } else if n < 0.04 {
                fb.set(x, y, c(Ramp::Parch, 7));
            }
        }
    }
    fb.rect(r.x - 1, r.y - 1, r.w + 2, r.h + 2, BLACK);
    fb.hline(r.x, r.x + r.w - 1, r.y, c(Ramp::Parch, 11));
    fb.vline(r.x, r.y, r.y + r.h - 1, c(Ramp::Parch, 11));
    fb.hline(r.x, r.x + r.w - 1, r.y + r.h - 1, c(Ramp::Parch, 5));
    fb.vline(r.x + r.w - 1, r.y, r.y + r.h - 1, c(Ramp::Parch, 5));
}

pub fn draw(st: &mut PlayState, ctx: &mut Ctx, fb: &mut Bitmap) {
    let f = &ctx.art.font;
    let gold = [c(Ramp::Gold, 14), c(Ramp::Gold, 13), c(Ramp::Gold, 12), c(Ramp::Gold, 11), c(Ramp::Gold, 10), c(Ramp::Gold, 8)];
    match &st.overlay {
        Overlay::Options(sel) => {
            parchment(fb, TEXT_AREA);
            f.draw_centered(fb, TEXT_AREA.x + TEXT_AREA.w / 2, TEXT_AREA.y + 3, "- Game Options -", INK);
            for (i, o) in OPTIONS.iter().enumerate() {
                let r = option_rect(i);
                if i == *sel {
                    fb.fill_rect(r.x, r.y, r.w, r.h, c(Ramp::Parch, 9));
                }
                let mut label = o.to_string();
                if i == 2 {
                    label = format!("Music: {}", if ctx.music_on { "On" } else { "Off" });
                }
                if i == 3 {
                    label = format!("Sound: {}", if ctx.sound_on { "On" } else { "Off" });
                }
                f.draw_centered(fb, r.x + r.w / 2, r.y + 2, &label, INK);
            }
        }
        Overlay::Slots(saving, sel) => {
            parchment(fb, TEXT_AREA);
            let title = if *saving { "- Save Game -" } else { "- Restore Game -" };
            f.draw_centered(fb, TEXT_AREA.x + TEXT_AREA.w / 2, TEXT_AREA.y + 3, title, INK);
            for i in 0..4 {
                let r = option_rect(i + 1);
                if i == *sel {
                    fb.fill_rect(r.x - 20, r.y, r.w + 40, r.h, c(Ramp::Parch, 9));
                }
                let desc = super::save::describe(i).unwrap_or_else(|| "(empty)".into());
                let numeral = ["I", "II", "III", "IV"][i];
                f.draw(fb, r.x - 16, r.y + 2, numeral, INK);
                f.draw(fb, r.x + 2, r.y + 2, &desc, INK);
            }
            let r = option_rect(5);
            f.draw_centered(fb, r.x + r.w / 2, r.y + 2, "Cancel", INK);
        }
        Overlay::Text(text, page) => {
            parchment(fb, TEXT_AREA);
            let ps = pages(ctx, text, TEXT_AREA);
            let p = (*page).min(ps.len() - 1);
            for (i, l) in ps[p].iter().enumerate() {
                f.draw(fb, TEXT_AREA.x + 5, TEXT_AREA.y + 5 + i as i32 * 6, l, INK);
            }
            if p + 1 < ps.len() {
                f.draw(fb, TEXT_AREA.x + TEXT_AREA.w - 30, TEXT_AREA.y + TEXT_AREA.h - 8, "[MORE]", c(Ramp::Red, 6));
            }
        }
        Overlay::Sleep(t) => {
            if *t > 0.25 && *t < 0.75 {
                ctx.art.font_big.draw_fancy(fb, 120, 66, "Z z z . . .", &gold, BLACK, None);
            }
        }
        Overlay::Dead(t) => {
            fb.fill_rect(VIEW.x, VIEW.y, VIEW.w, VIEW.h, c(Ramp::Red, 1));
            for y in VIEW.y..VIEW.y + VIEW.h {
                for x in VIEW.x..VIEW.x + VIEW.w {
                    if (x + y) % 3 == 0 && crate::util::hashf(x, y, 1, 3) > 0.6 {
                        fb.set(x, y, c(Ramp::Red, 3));
                    }
                }
            }
            ctx.art.font_big.draw_fancy(fb, VIEW.x + 30, VIEW.y + 30, "You have died.", &[c(Ramp::Red, 13), c(Ramp::Red, 12), c(Ramp::Red, 11), c(Ramp::Red, 10), c(Ramp::Red, 9), c(Ramp::Red, 8)], BLACK, None);
            if *t > 1.5 {
                let msg = if st.game.flag("sapling_planted") > 0 { "Click to return to the silver sapling." } else { "Click to restore your last game." };
                f.draw_centered(fb, VIEW.x + VIEW.w / 2, VIEW.y + 70, msg, c(Ramp::Parch, 12));
                f.draw_centered(fb, VIEW.x + VIEW.w / 2, VIEW.y + 80, "(Esc returns to the title)", c(Ramp::Parch, 9));
            }
        }
        Overlay::Victory(t) => {
            parchment(fb, TEXT_AREA);
            let text = "The Shadow Fiend is banished, and the Lumen Stone shines whole once more. As its light floods the abyss, the great doors above grind open. With Lady Rowena at your side, you climb toward the daylight - a hero of the Abyss.";
            let lines = f.wrap(text, TEXT_AREA.w - 10);
            for (i, l) in lines.iter().enumerate() {
                f.draw(fb, TEXT_AREA.x + 5, TEXT_AREA.y + 5 + i as i32 * 6, l, INK);
            }
            if *t > 2.0 {
                f.draw_centered(fb, TEXT_AREA.x + TEXT_AREA.w / 2, TEXT_AREA.y + TEXT_AREA.h - 9, "- The End -", c(Ramp::Red, 6));
            }
        }
        _ => {}
    }
}
