//! The conversation screen, laid out like the original: name plates and
//! portraits on top, the other person's words on a large parchment, and
//! your possible replies on the message scroll. Between the portraits are
//! the two trade areas: the other person's goods on the left, your offer on
//! the right.

use super::overlay::{option_lines, pages, parchment, TEXT_AREA};
use super::play::PlayState;
use super::{Ctx, Overlay};
use crate::audio::Sfx;
use crate::game::barter::{self, SLOTS};
use crate::gfx::palette::{c, Ramp, BLACK, INK};
use crate::gfx::Bitmap;
use crate::layout::*;

/// Top-left corners of the trade areas: the NPC's goods and the player's offer.
const GOODS: (i32, i32) = (83, 12);
const OFFER: (i32, i32) = (141, 12);

/// Centre of slot `k` in a trade area.
fn slot_c(area: (i32, i32), k: usize) -> (i32, i32) {
    let k = k as i32;
    (area.0 + 13 + (k % 2) * 26, area.1 + 9 + (k / 2) * 18)
}

fn near(m: (i32, i32), c: (i32, i32)) -> bool {
    (m.0 - c.0).pow(2) + (m.1 - c.1).pow(2) <= 81
}

/// A click in the trade areas: left picks goods and puts things on the
/// table or takes them back, right asks what something is worth. Returns
/// whether the click was used.
pub fn click(st: &mut PlayState, ctx: &mut Ctx, m: (i32, i32), right: bool) -> bool {
    let Overlay::Conversation(cs, page) = &mut st.overlay else { return false };
    let g = &st.game;
    for k in 0..SLOTS {
        if near(m, slot_c(GOODS, k)) {
            let i = cs.goods_page * SLOTS + k;
            let Some(it) = g.level().npc(cs.npc).filter(|_| cs.trading).and_then(|n| n.inventory.get(i)) else { return true };
            if right {
                cs.text = barter::describe_good(g, it);
                *page = 0;
            } else if let Some(p) = cs.picked.iter().position(|&x| x == i) {
                cs.picked.remove(p);
            } else {
                cs.picked.push(i);
            }
            ctx.sounds.push(Sfx::Click);
            return true;
        }
        if near(m, slot_c(OFFER, k)) {
            if !cs.trading {
                return true;
            }
            if right {
                if let Some(it) = &cs.offer[k] {
                    cs.text = barter::describe_offer(g, it);
                    *page = 0;
                }
            } else if st.ui.held.is_some() || cs.offer[k].is_some() {
                std::mem::swap(&mut st.ui.held, &mut cs.offer[k]);
                ctx.sounds.push(if cs.offer[k].is_some() { Sfx::Drop } else { Sfx::Pickup });
            }
            return true;
        }
    }
    false
}

fn plate(fb: &mut Bitmap, ctx: &Ctx, r: Rect, text: &str) {
    fb.fill_rect(r.x, r.y, r.w, r.h, c(Ramp::Copper, 7));
    fb.hline(r.x, r.x + r.w - 1, r.y, c(Ramp::Copper, 10));
    fb.hline(r.x, r.x + r.w - 1, r.y + r.h - 1, c(Ramp::Copper, 3));
    fb.rect(r.x - 1, r.y - 1, r.w + 2, r.h + 2, BLACK);
    ctx.art.font.draw(fb, r.x + 3, r.y + 2, text, INK);
}

fn portrait_frame(fb: &mut Bitmap, x: i32, y: i32, bmp: &Bitmap) {
    fb.fill_rect(x, y, 36, 36, c(Ramp::Blue, 2));
    fb.blit(bmp, x + 1, y + 1);
    fb.rect(x - 1, y - 1, 38, 38, BLACK);
    fb.rect(x - 2, y - 2, 40, 40, c(Ramp::Slate, 8));
}

/// A trade area with its items; `picked` slots get a gold ring.
fn barter(fb: &mut Bitmap, ctx: &mut Ctx, area: (i32, i32), items: &[Option<&crate::game::items::Item>], picked: &[bool]) {
    let (x, y) = area;
    fb.fill_rect(x, y, 52, 36, c(Ramp::Copper, 6));
    fb.rect(x - 1, y - 1, 54, 38, BLACK);
    for k in 0..SLOTS {
        let (cx, cy) = slot_c(area, k);
        fb.ellipse_outline(cx as f32, cy as f32, 8.0, 8.0, c(Ramp::Copper, 4));
        fb.ellipse_outline(cx as f32 + 0.5, cy as f32 + 0.5, 7.5, 7.5, c(Ramp::Copper, 9));
        fb.set(cx - 10, cy, BLACK);
        if picked.get(k).copied().unwrap_or(false) {
            fb.fill_circle_f(cx as f32 + 0.5, cy as f32 + 0.5, 7.0, c(Ramp::Gold, 9));
            fb.ellipse_outline(cx as f32 + 0.5, cy as f32 + 0.5, 8.5, 8.5, c(Ramp::Gold, 14));
        }
        if let Some(it) = items.get(k).copied().flatten() {
            super::panel::draw_item_at(ctx, fb, it, cx, cy);
        }
    }
}

pub fn draw(st: &mut PlayState, ctx: &mut Ctx, fb: &mut Bitmap) {
    let Overlay::Conversation(cs, page) = &st.overlay else { return };
    let npc = st.game.level().npc(cs.npc).cloned();
    // Clear the top frame area.
    fb.fill_rect(41, 0, 195, 46, BLACK);
    let name = npc.as_ref().and_then(|n| n.name.as_deref().map(crate::lang::tr_dyn)).unwrap_or_else(|| npc.as_ref().map(|n| crate::lang::cap(&n.race_noun().text())).unwrap_or_default());
    plate(fb, ctx, Rect::new(44, 1, 92, 9), &name);
    let pname = st.game.player.name.clone();
    plate(fb, ctx, Rect::new(140, 1, 92, 9), &pname);
    if let Some(n) = &npc {
        let p = ctx.cache.portrait(n.race, n.look).clone();
        portrait_frame(fb, 44, 12, &p);
    }
    let me = ctx.cache.player_face(&st.game.player).clone();
    portrait_frame(fb, 196, 12, &me);
    // Trade areas.
    let (goods, picked): (Vec<Option<&crate::game::items::Item>>, Vec<bool>) = (0..SLOTS)
        .map(|k| {
            let i = cs.goods_page * SLOTS + k;
            let it = npc.as_ref().filter(|_| cs.trading).and_then(|n| n.inventory.get(i));
            (it, cs.picked.contains(&i))
        })
        .unzip();
    barter(fb, ctx, GOODS, &goods, &picked);
    let offer: Vec<Option<&crate::game::items::Item>> = cs.offer.iter().map(|o| o.as_ref()).collect();
    barter(fb, ctx, OFFER, &offer, &[]);
    if let Some(n) = npc.as_ref().filter(|n| cs.trading && n.inventory.len() > SLOTS) {
        let s = format!("{}/{}", cs.goods_page + 1, n.inventory.len().div_ceil(SLOTS));
        let w = ctx.art.font.text_width(&s);
        ctx.art.font.draw(fb, GOODS.0 + 26 - w / 2, GOODS.1 + 15, &s, c(Ramp::Copper, 13));
    }
    // Speech parchment.
    let area = Rect::new(TEXT_AREA.x - 2, TEXT_AREA.y, TEXT_AREA.w + 4, TEXT_AREA.h + 2);
    parchment(fb, area);
    let ps = pages(ctx, &cs.text, TEXT_AREA);
    let p = (*page).min(ps.len() - 1);
    for (i, l) in ps[p].iter().enumerate() {
        ctx.art.font.draw(fb, TEXT_AREA.x + 4, TEXT_AREA.y + 5 + i as i32 * 6, l, INK);
    }
    // Replies on the message scroll.
    if p + 1 < ps.len() {
        ctx.art.font.draw(fb, TEXT_AREA.x + TEXT_AREA.w - 30, TEXT_AREA.y + TEXT_AREA.h - 7, t!("[MORE]"), c(Ramp::Red, 6));
        ctx.art.font.draw(fb, SCROLL_TEXT_X, SCROLL_TEXT_Y, t!("(click or press a key to continue)"), INK);
    } else if cs.options.is_empty() || cs.ended {
        ctx.art.font.draw(fb, SCROLL_TEXT_X, SCROLL_TEXT_Y, t!("(click or press a key to end the conversation)"), INK);
    } else {
        let lines = option_lines(ctx, &cs.options);
        for (row, (_, l)) in lines.iter().enumerate().take(SCROLL_LINES as usize) {
            ctx.art.font.draw(fb, SCROLL_TEXT_X, SCROLL_TEXT_Y + row as i32 * 6, l, INK);
        }
    }
}
