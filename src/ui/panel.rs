//! The character panel: inventory with paperdoll, statistics, and runes.

use super::play::{draw_rune_tile, Mode, Panel, PlayState};
use super::Ctx;
use crate::audio::Sfx;
use crate::game::items::{Class, Item, ItemKind, Slot, ALL_SKILLS};
use crate::game::magic::RUNE_NAMES;
use crate::game::{EqSlot, Equip};
use crate::gfx::palette::{c, Ramp, BLACK, INK};
use crate::gfx::Bitmap;
use crate::layout::*;

const EQ_CIRCLES: [(EqSlot, (i32, i32)); 4] = [
    (EqSlot::ShoulderL, EQ_SHOULDER_L),
    (EqSlot::ShoulderR, EQ_SHOULDER_R),
    (EqSlot::Weapon, EQ_HAND_L),
    (EqSlot::Shield, EQ_HAND_R),
];
const RINGS: [(EqSlot, (i32, i32)); 2] = [(EqSlot::RingL, EQ_RING_L), (EqSlot::RingR, EQ_RING_R)];
const WEIGHT_C: (i32, i32) = (307, 63);

fn near(m: (i32, i32), c: (i32, i32), r: i32) -> bool {
    (m.0 - c.0).pow(2) + (m.1 - c.1).pow(2) <= r * r
}

/// Body region of the paperdoll under the mouse (armour slots).
fn body_slot(m: (i32, i32)) -> Option<EqSlot> {
    let (px, py) = (PAPERDOLL_C.0, 9);
    if (m.0 - px).abs() > 15 {
        return None;
    }
    let y = m.1 - py;
    if (m.0 - px).abs() > 9 && (30..44).contains(&y) {
        return Some(EqSlot::Hands);
    }
    match y {
        0..=13 => Some(EqSlot::Head),
        14..=35 => Some(EqSlot::Chest),
        36..=55 => Some(EqSlot::Legs),
        56..=72 => Some(EqSlot::Feet),
        _ => None,
    }
}

fn armor_slot_for(item: &Item) -> Option<EqSlot> {
    match item.def().armor {
        Some((Slot::Head, _)) => Some(EqSlot::Head),
        Some((Slot::Chest, _)) => Some(EqSlot::Chest),
        Some((Slot::Legs, _)) => Some(EqSlot::Legs),
        Some((Slot::Feet, _)) => Some(EqSlot::Feet),
        Some((Slot::Hands, _)) => Some(EqSlot::Hands),
        _ => None,
    }
}

/// The container currently opened, if any.
fn open_container<'a>(p: &'a mut crate::game::Player, path: &[usize]) -> Option<&'a mut Item> {
    let first = *path.first()?;
    let mut cur = p.pack.get_mut(first)?.as_mut()?;
    for &i in &path[1..] {
        cur = cur.contents.get_mut(i)?;
    }
    Some(cur)
}

pub fn click(st: &mut PlayState, ctx: &mut Ctx, m: (i32, i32), right: bool) {
    match st.ui.panel {
        Panel::Inventory => click_inventory(st, ctx, m, right),
        Panel::Stats => click_stats(st, ctx, m),
        Panel::Runes => click_runes(st, ctx, m, right),
    }
}

fn click_stats(st: &mut PlayState, ctx: &mut Ctx, m: (i32, i32)) {
    // Scroll buttons.
    if near(m, (PANEL.x + 33, PANEL.y + 110), 5) {
        st.ui.stats_scroll = (st.ui.stats_scroll - 3).max(0);
        ctx.sounds.push(Sfx::Click);
        return;
    }
    if near(m, (PANEL.x + 49, PANEL.y + 110), 5) {
        st.ui.stats_scroll = (st.ui.stats_scroll + 3).min(ALL_SKILLS.len() as i32 - 6);
        ctx.sounds.push(Sfx::Click);
        return;
    }
    // Spend skill points by clicking a skill line.
    let y0 = PANEL.y + 62;
    if m.1 >= y0 && m.1 < y0 + 6 * 7 && st.game.player.skill_points > 0 {
        let row = (m.1 - y0) / 7;
        let idx = (st.ui.stats_scroll + row) as usize;
        if idx < ALL_SKILLS.len() {
            let p = &mut st.game.player;
            let s = ALL_SKILLS[idx];
            if p.skills[s as usize] < 30 {
                p.skills[s as usize] += 1;
                p.skill_points -= 1;
                if s == crate::game::items::Skill::Mana {
                    p.max_mana += 1;
                }
                let v = p.skills[s as usize];
                st.game.sys(format!("Your {} skill is now {}.", s.name(), v));
                ctx.sounds.push(Sfx::LevelUp);
            }
        }
    }
}

fn click_runes(st: &mut PlayState, ctx: &mut Ctx, m: (i32, i32), right: bool) {
    for r in 0..24usize {
        let (x, y) = rune_cell(r);
        if m.0 >= x && m.0 < x + 17 && m.1 >= y && m.1 < y + 14 {
            if !st.game.player.runes[r] {
                return;
            }
            if right || st.ui.mode == Mode::Look {
                st.game.msg(format!("You see the {} rune, meaning '{}'.", RUNE_NAMES[r], crate::game::magic::RUNE_MEANINGS[r]));
                return;
            }
            let shelf = &mut st.game.player.shelf;
            if shelf.len() >= 3 {
                shelf.remove(0);
            }
            shelf.push(r as u8);
            ctx.sounds.push(Sfx::Click);
            return;
        }
    }
    // Bottom area: back to the inventory.
    if m.1 > PANEL.y + 100 {
        st.ui.panel = Panel::Inventory;
    }
}

fn rune_cell(r: usize) -> (i32, i32) {
    let col = (r % 4) as i32;
    let row = (r / 4) as i32;
    (PANEL.x + 5 + col * 19, PANEL.y + 8 + row * 15)
}

fn click_inventory(st: &mut PlayState, ctx: &mut Ctx, m: (i32, i32), right: bool) {
    // Weight read-out.
    if near(m, WEIGHT_C, 6) {
        let p = &st.game.player;
        let left = (p.capacity() - p.load()) as f32 / 10.0;
        st.game.msg(format!("You can carry {:.1} more stones.", left.max(0.0)));
        return;
    }
    // Equipment circles and rings.
    for (slot, cpos) in EQ_CIRCLES.iter().chain(RINGS.iter()) {
        let r = if matches!(slot, EqSlot::RingL | EqSlot::RingR) { 4 } else { 8 };
        if near(m, *cpos, r) {
            equip_click(st, ctx, *slot, right);
            return;
        }
    }
    if let Some(slot) = body_slot(m) {
        if let Some(h) = &st.ui.held {
            if let Some(s) = armor_slot_for(h) {
                equip_click(st, ctx, s, right);
                return;
            }
        }
        equip_click(st, ctx, slot, right);
        return;
    }
    // Pack slots (or an opened container's contents).
    for (i, &cpos) in PACK_SLOTS.iter().enumerate() {
        if near(m, cpos, 8) {
            pack_click(st, ctx, i, right);
            return;
        }
    }
}

fn equip_click(st: &mut PlayState, ctx: &mut Ctx, slot: EqSlot, right: bool) {
    let p = &mut st.game.player;
    if right {
        if st.ui.mode == Mode::Look {
            if let Some(mut it) = p.equip.get_mut(slot).take() {
                st.game.examine(&mut it);
                *st.game.player.equip.get_mut(slot) = Some(it);
            }
            return;
        }
        if let Some(it) = p.equip.get_mut(slot).take() {
            let back = st.game.use_item(it, Some(slot));
            *st.game.player.equip.get_mut(slot) = back;
        }
        return;
    }
    match st.ui.held.take() {
        Some(h) => {
            if !Equip::accepts(slot, &h) {
                st.game.msg("That does not fit there.");
                st.ui.held = Some(h);
                return;
            }
            let old = p.equip.get_mut(slot).replace(h);
            st.ui.held = old;
            ctx.sounds.push(Sfx::Pickup);
        }
        None => {
            st.ui.held = p.equip.get_mut(slot).take();
            if st.ui.held.is_some() {
                ctx.sounds.push(Sfx::Pickup);
            }
        }
    }
}

fn pack_click(st: &mut PlayState, ctx: &mut Ctx, i: usize, right: bool) {
    let path = st.ui.open.clone();
    if !path.is_empty() {
        // Slot 0 shows the open container itself: click closes it.
        if i == 0 {
            if right || st.ui.held.is_none() {
                st.ui.open.pop();
                ctx.sounds.push(Sfx::Click);
                return;
            }
            // Dropping onto the container icon puts the item inside.
            let h = st.ui.held.take().unwrap();
            let cont = open_container(&mut st.game.player, &path).unwrap();
            if (cont.contents.len() as i32) < cont.def().capacity {
                cont.contents.push(h);
                ctx.sounds.push(Sfx::Drop);
            } else {
                st.game.msg("It is full.");
                st.ui.held = Some(h);
            }
            return;
        }
        let idx = i - 1 + st.ui.pack_page * 7;
        let held = st.ui.held.take();
        let mode = st.ui.mode;
        let cont = open_container(&mut st.game.player, &path).unwrap();
        let cap = cont.def().capacity as usize;
        if right {
            st.ui.held = held;
            if let Some(it) = cont.contents.get(idx).cloned() {
                if mode == Mode::Look {
                    let mut it = it;
                    st.game.examine(&mut it);
                    if let Some(c) = open_container(&mut st.game.player, &path).and_then(|c| c.contents.get_mut(idx)) {
                        c.identified = it.identified;
                    }
                } else if it.def().capacity > 0 {
                    st.ui.open.push(idx);
                } else {
                    let it = cont.contents.remove(idx);
                    let back = st.game.use_item(it, None);
                    if let Some(b) = back {
                        let cont = open_container(&mut st.game.player, &path).unwrap();
                        let at = idx.min(cont.contents.len());
                        cont.contents.insert(at, b);
                    }
                }
            }
            return;
        }
        match held {
            Some(h) => {
                if idx < cont.contents.len() {
                    let old = std::mem::replace(&mut cont.contents[idx], h);
                    st.ui.held = Some(old);
                } else if cont.contents.len() < cap {
                    if cont.kind == ItemKind::RuneBag && h.kind != ItemKind::RuneStone {
                        st.ui.held = Some(h);
                        st.game.msg("Only rune stones go in the rune bag.");
                        return;
                    }
                    cont.contents.push(h);
                } else {
                    st.ui.held = Some(h);
                    st.game.msg("It is full.");
                    return;
                }
                ctx.sounds.push(Sfx::Drop);
            }
            None => {
                if idx < cont.contents.len() {
                    st.ui.held = Some(cont.contents.remove(idx));
                    ctx.sounds.push(Sfx::Pickup);
                }
            }
        }
        return;
    }
    // Plain pack slots.
    let p = &mut st.game.player;
    if right {
        let Some(mut it) = p.pack[i].clone() else { return };
        if st.ui.mode == Mode::Look {
            st.game.examine(&mut it);
            if let Some(slot) = st.game.player.pack[i].as_mut() {
                slot.identified = it.identified;
            }
            return;
        }
        match it.def().class {
            Class::RuneBag => {
                st.ui.panel = Panel::Runes;
                ctx.sounds.push(Sfx::Click);
            }
            Class::Container => {
                st.ui.open = vec![i];
                st.ui.pack_page = 0;
                ctx.sounds.push(Sfx::Click);
            }
            Class::Map => st.overlay = super::Overlay::Map(st.game.cur),
            _ => {
                let it = st.game.player.pack[i].take().unwrap();
                let back = st.game.use_item(it, None);
                if st.game.player.pack[i].is_none() {
                    st.game.player.pack[i] = back;
                } else if let Some(b) = back {
                    if let Some(r) = st.game.player.give(b) {
                        st.game.drop_item(r, None);
                    }
                }
            }
        }
        return;
    }
    match st.ui.held.take() {
        Some(h) => match p.pack[i].as_mut() {
            Some(target) if target.def().capacity > 0 && target.kind != ItemKind::RuneBag && h.def().capacity == 0 => {
                if (target.contents.len() as i32) < target.def().capacity {
                    target.contents.push(h);
                    ctx.sounds.push(Sfx::Drop);
                } else {
                    st.ui.held = Some(h);
                    st.game.msg("It is full.");
                }
            }
            Some(target) if target.kind == ItemKind::RuneBag && h.kind == ItemKind::RuneStone => {
                let back = st.game.use_item(h, None);
                st.ui.held = back;
            }
            Some(target) if target.kind == h.kind && h.def().stack => {
                target.qty += h.qty;
                ctx.sounds.push(Sfx::Drop);
            }
            _ => {
                let old = p.pack[i].replace(h);
                st.ui.held = old;
                ctx.sounds.push(Sfx::Drop);
            }
        },
        None => {
            if let Some(it) = &p.pack[i] {
                if it.kind == ItemKind::RuneBag {
                    st.ui.panel = Panel::Runes;
                    ctx.sounds.push(Sfx::Click);
                    return;
                }
            }
            st.ui.held = p.pack[i].take();
            if st.ui.held.is_some() {
                ctx.sounds.push(Sfx::Pickup);
            }
        }
    }
}

pub fn draw(st: &mut PlayState, ctx: &mut Ctx, fb: &mut Bitmap) {
    match st.ui.panel {
        Panel::Inventory => draw_inventory(st, ctx, fb),
        Panel::Stats => draw_stats(st, ctx, fb),
        Panel::Runes => draw_runes(st, ctx, fb),
    }
}

pub fn draw_item_at(ctx: &mut Ctx, fb: &mut Bitmap, it: &Item, cx: i32, cy: i32) {
    let b = ctx.cache.icon_mut(it).clone();
    fb.blit(&b, cx - b.w / 2, cy - b.h / 2);
    if it.qty > 1 && it.def().stack {
        let s = it.qty.to_string();
        let w = ctx.art.font.text_width(&s);
        ctx.art.font.draw(fb, cx + 7 - w, cy + 3, &s, crate::gfx::palette::WHITE);
    }
}

fn draw_inventory(st: &mut PlayState, ctx: &mut Ctx, fb: &mut Bitmap) {
    let ring = ctx.art.ui.slot.clone();
    for (_, (cx, cy)) in EQ_CIRCLES.iter() {
        fb.blit(&ring, cx - 8, cy - 8);
    }
    for (_, (cx, cy)) in RINGS.iter() {
        fb.ellipse_outline(*cx as f32 + 0.5, *cy as f32 + 0.5, 2.5, 2.5, c(Ramp::Copper, 4));
    }
    for &(cx, cy) in PACK_SLOTS.iter() {
        fb.blit(&ring, cx - 8, cy - 8);
    }
    // Paperdoll.
    let doll = ctx.cache.paperdoll(&st.game.player).clone();
    fb.blit(&doll, PAPERDOLL_C.0 - doll.w / 2, 6);
    // Equipped items in circles.
    let eq = st.game.player.equip.clone();
    for (slot, (cx, cy)) in EQ_CIRCLES.iter() {
        if let Some(it) = eq.get(*slot) {
            draw_item_at(ctx, fb, it, *cx, *cy);
        }
    }
    for (slot, (cx, cy)) in RINGS.iter() {
        if eq.get(*slot).is_some() {
            fb.fill_circle_f(*cx as f32 + 0.5, *cy as f32 + 0.5, 2.0, c(Ramp::Gold, 12));
        }
    }
    // Weight remaining.
    let p = &st.game.player;
    let left = ((p.capacity() - p.load()) / 10).max(0);
    fb.fill_ellipse(WEIGHT_C.0 as f32 + 0.5, WEIGHT_C.1 as f32 + 0.5, 6.5, 5.0, c(Ramp::Slate, 3));
    fb.ellipse_outline(WEIGHT_C.0 as f32 + 0.5, WEIGHT_C.1 as f32 + 0.5, 6.5, 5.0, BLACK);
    let s = left.to_string();
    let w = ctx.art.font.text_width(&s);
    ctx.art.font.draw(fb, WEIGHT_C.0 - w / 2 + 1, WEIGHT_C.1 - 2, &s, c(Ramp::Copper, 13));
    // Pack or open container.
    if !st.ui.open.is_empty() {
        let path = st.ui.open.clone();
        let cont = open_container(&mut st.game.player, &path).cloned();
        if let Some(cont) = cont {
            fb.fill_ellipse(PACK_SLOTS[0].0 as f32 + 0.5, PACK_SLOTS[0].1 as f32 + 0.5, 8.0, 8.0, c(Ramp::Copper, 4));
            draw_item_at(ctx, fb, &cont, PACK_SLOTS[0].0, PACK_SLOTS[0].1);
            for (k, it) in cont.contents.iter().skip(st.ui.pack_page * 7).take(7).enumerate() {
                let (cx, cy) = PACK_SLOTS[k + 1];
                draw_item_at(ctx, fb, it, cx, cy);
            }
        } else {
            st.ui.open.clear();
        }
    } else {
        let pack = st.game.player.pack.clone();
        for (i, slot) in pack.iter().enumerate() {
            if let Some(it) = slot {
                let (cx, cy) = PACK_SLOTS[i];
                draw_item_at(ctx, fb, it, cx, cy);
            }
        }
    }
}

fn draw_stats(st: &mut PlayState, ctx: &mut Ctx, fb: &mut Bitmap) {
    let p = &st.game.player;
    let f = &ctx.art.font;
    let x0 = PANEL.x + 6;
    let x1 = PANEL.x + PANEL.w - 6;
    let mut y = PANEL.y + 5;
    let right = |fb: &mut Bitmap, y: i32, s: &str| {
        let w = f.text_width(s);
        f.draw(fb, x1 - w, y, s, INK);
    };
    f.draw_centered(fb, PANEL.x + PANEL.w / 2, y, &p.name.to_uppercase(), INK);
    y += 7;
    f.draw(fb, x0, y, &p.class.name().to_uppercase(), INK);
    right(fb, y, &crate::game::actions::ordinal(p.level).to_uppercase());
    y += 7;
    for (label, val) in [("STR", p.st.to_string()), ("DEX", p.dex.to_string()), ("INT", p.int.to_string()), ("VIT", format!("{}/{}", p.vit, p.max_vit)), ("MANA", format!("{}/{}", p.mana, p.max_mana)), ("EXP", p.exp.to_string())] {
        f.draw(fb, x0, y, label, INK);
        right(fb, y, &val);
        y += 6;
    }
    // Skills inset.
    let inset = crate::layout::Rect::new(PANEL.x + 3, PANEL.y + 60, PANEL.w - 6, 44);
    fb.fill_rect(inset.x, inset.y, inset.w, inset.h, c(Ramp::Copper, 4));
    fb.rect(inset.x, inset.y, inset.w, inset.h, c(Ramp::Copper, 2));
    let skill_col = c(Ramp::Silver, 12);
    for row in 0..6 {
        let idx = (st.ui.stats_scroll + row) as usize;
        if idx >= ALL_SKILLS.len() {
            break;
        }
        let s = ALL_SKILLS[idx];
        let yy = inset.y + 2 + row * 7;
        f.draw(fb, inset.x + 3, yy, &s.name().to_uppercase(), skill_col);
        let v = p.skills[s as usize].to_string();
        let w = f.text_width(&v);
        f.draw(fb, inset.x + inset.w - 4 - w, yy, &v, skill_col);
    }
    if p.skill_points > 0 {
        let s = format!("{} PTS", p.skill_points);
        f.draw(fb, PANEL.x + 4, PANEL.y + 106, &s, c(Ramp::Gold, 14));
    }
    for (k, x) in [(0, PANEL.x + 33), (1, PANEL.x + 49)] {
        let cy = PANEL.y + 110;
        fb.fill_circle_f(x as f32 + 0.5, cy as f32 + 0.5, 4.5, c(Ramp::Slate, 6));
        fb.ellipse_outline(x as f32 + 0.5, cy as f32 + 0.5, 4.5, 4.5, BLACK);
        let ch = if k == 0 { "-" } else { "+" };
        f.draw(fb, x - 1, cy - 2, ch, c(Ramp::Silver, 14));
    }
}

fn draw_runes(st: &mut PlayState, ctx: &mut Ctx, fb: &mut Bitmap) {
    let f = &ctx.art.font;
    for r in 0..24usize {
        let (x, y) = rune_cell(r);
        if st.game.player.runes[r] {
            draw_rune_tile(fb, ctx, r as u8, x, y, 17, 13);
        } else {
            fb.rect(x, y, 17, 13, c(Ramp::Copper, 5));
        }
    }
    f.draw_centered(fb, PANEL.x + PANEL.w / 2, PANEL.y + 102, "RUNE BAG", INK);
    f.draw_centered(fb, PANEL.x + PANEL.w / 2, PANEL.y + 108, "(click here to close)", c(Ramp::Copper, 3));
}
