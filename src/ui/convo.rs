//! The conversation screen, laid out like the original: name plates and
//! portraits on top, the other person's words on a large parchment, and
//! your possible replies on the message scroll.

use super::overlay::{option_lines, pages, parchment, TEXT_AREA};
use super::play::PlayState;
use super::{Ctx, Overlay};
use crate::gfx::palette::{c, Ramp, BLACK, INK};
use crate::gfx::Bitmap;
use crate::layout::*;

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

fn barter(fb: &mut Bitmap, x: i32, y: i32) {
    fb.fill_rect(x, y, 52, 36, c(Ramp::Copper, 6));
    fb.rect(x - 1, y - 1, 54, 38, BLACK);
    for k in 0..4 {
        let cx = x + 13 + (k % 2) * 26;
        let cy = y + 9 + (k / 2) * 18;
        fb.ellipse_outline(cx as f32, cy as f32, 8.0, 8.0, c(Ramp::Copper, 4));
        fb.ellipse_outline(cx as f32 + 0.5, cy as f32 + 0.5, 7.5, 7.5, c(Ramp::Copper, 9));
        fb.set(cx - 10, cy, BLACK);
    }
}

pub fn draw(st: &mut PlayState, ctx: &mut Ctx, fb: &mut Bitmap) {
    let Overlay::Conversation(cs, page) = &st.overlay else { return };
    let npc = st.game.level().npc(cs.npc).cloned();
    // Clear the top frame area.
    fb.fill_rect(41, 0, 195, 46, BLACK);
    let name = npc.as_ref().and_then(|n| n.name.clone()).unwrap_or_else(|| npc.as_ref().map(|n| n.race.def().name.to_string()).unwrap_or_default());
    plate(fb, ctx, Rect::new(44, 1, 92, 9), &name);
    let pname = st.game.player.name.clone();
    plate(fb, ctx, Rect::new(140, 1, 92, 9), &pname);
    if let Some(n) = &npc {
        let p = ctx.cache.portrait(n.race, n.look).clone();
        portrait_frame(fb, 44, 12, &p);
    }
    let me = ctx.cache.player_face(&st.game.player).clone();
    portrait_frame(fb, 196, 12, &me);
    barter(fb, 83, 12);
    barter(fb, 141, 12);
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
        ctx.art.font.draw(fb, TEXT_AREA.x + TEXT_AREA.w - 30, TEXT_AREA.y + TEXT_AREA.h - 7, "[MORE]", c(Ramp::Red, 6));
        ctx.art.font.draw(fb, SCROLL_TEXT_X, SCROLL_TEXT_Y, "(click or press a key to continue)", INK);
    } else if cs.options.is_empty() || cs.ended {
        ctx.art.font.draw(fb, SCROLL_TEXT_X, SCROLL_TEXT_Y, "(click or press a key to end the conversation)", INK);
    } else {
        let lines = option_lines(ctx, &cs.options);
        for (row, (_, l)) in lines.iter().enumerate().take(SCROLL_LINES as usize) {
            ctx.art.font.draw(fb, SCROLL_TEXT_X, SCROLL_TEXT_Y + row as i32 * 6, l, INK);
        }
    }
}
