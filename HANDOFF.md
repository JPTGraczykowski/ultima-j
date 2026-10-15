# Handoff — where the work stands

Goal: a clone of Ultima Underworld (1992) in Rust whose graphics match the
original's style. Status (2026-10-15): the game is complete and playable from
the title screen to the victory screen, and all 54 tests pass. What remains is
mostly the barter screen and balance.

## Done

- Software renderer: 320×200, 256-colour VGA-style palette with light tables,
  tiles with heights/slopes/diagonals, water and lava, doors, billboards, picking.
- Main screen modelled on the original (layout measured from DOS captures):
  icon column, view frame with gargoyle and dragons, compass, power gem,
  flasks, rune shelf, character panel (inventory, statistics, runes), scroll.
- Procedural art: textures, frame, icons, cursors, 16×16 item icons, 3D-modelled
  fixtures, creature sprites ray-cast from clay models (5 views, 12 frames),
  serif and small pixel fonts, sounds and two music tracks.
- Gameplay: movement (keys and mouse regions), jump, swim, lava, falling;
  look/get/use/talk/fight; paperdoll and containers; melee, missiles, rune magic
  (40 spells); AI with path-finding; conversations (`assets/dialogue.txt`);
  automap with notes; sleep with dream hints; hunger, light fuel, poison;
  save/restore (`~/.ultima-j/`); title, intro, character creation, death, victory.
- Five levels (`src/game/levels.rs`), three shard quests and the final battle,
  all verified with scripted runs.
- All 20 skills have an effect. Lore identifies enchantments on look;
  Appraise estimates worth; Charm and Appraise adjust prices in dialogue
  (`{buy N}`/`{sell N}`, `afford:N`, `pay`/`earn`, `skill:NAME>=N`); Repair
  mends gear used from the inventory (Bodric also mends for gold); Traps
  spots, maps and disarms traps (13 across the five levels); Swimming,
  Acrobat and Sneak tune swimming, jumping/falling and creature notice range
  (helpers on `Player` in `src/game/state.rs`).
- Visual pass after the side-by-side review: smaller command icons with oval
  cartouches, reworked creature models (stone golem, bat wings with membranes,
  hunched trolls, humanoid proportions), bigger paperdoll with a face and beard.

## How to check things

```sh
source ~/.cargo/env
cargo test --release                       # 54 tests: level connectivity, fuzzers, save round-trip, skills
cargo run --release                        # play
target/release/ultima-j --script "..."     # headless runs and screenshots (src/headless.rs)
target/release/ultima-j --maps DIR         # level layouts
ULTIMA_J_SELFTEST=out.png target/release/ultima-j --play   # real-window capture
```

Compare screenshots against originals side by side (native 320×200 DOS
captures are on oldgames.sk; the reference card is on mocagh.org).

## Next steps (review of 2026-10-15), in suggested order

1. **Barter screen.** Trading only happens in dialogue. The two trade areas
   are already drawn in the conversation screen (`barter()` in
   `src/ui/convo.rs`) but do nothing yet. Prices should come from
   `buy_price`/`sell_price` in `src/game/dialogue.rs` (Charm and Appraise)
   and item worth from `Item::value`.
2. **Balance playthrough of levels 2–5 by hand.** The scripted runs only show
   the levels can be finished; difficulty, loot, food and light supply are
   untested.
3. **Smaller items:**
   - Save files are about 3.3 MB of JSON each; compress them or save only
     what changed.
   - Intro vignettes could be richer.
   - More screenshot comparisons against the original for visual polish.
4. **Docs:** once the barter screen is done, remove it from the README's
   "Not yet implemented" list.
