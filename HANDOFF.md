# Handoff — where the work stands

Goal: a clone of Ultima Underworld (1992) in Rust whose graphics match the
original's style. Status (2026-10-12): the game is complete and playable from
the title screen to the victory screen; work in progress is visual polish.

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

## How to check things

```sh
source ~/.cargo/env
cargo test --release                       # 48 tests: level connectivity, fuzzers, save round-trip
cargo run --release                        # play
target/release/ultima-j --script "..."     # headless runs and screenshots (src/headless.rs)
target/release/ultima-j --maps DIR         # level layouts
ULTIMA_J_SELFTEST=out.png target/release/ultima-j --play   # real-window capture
```

Compare screenshots against originals side by side (native 320×200 DOS
captures are on oldgames.sk; the reference card is on mocagh.org).

## Next steps (from the last side-by-side review)

1. Command icons: make them smaller and finer (about 24×14) and add the oval
   cartouches the original draws behind each icon; refine the Get and Fight hands.
2. Paperdoll: bigger head with a face (hair, beard), better torso shape; it is
   currently a plain brown figure while the original's is detailed.
3. Creature sprites: more hand-drawn-looking faces and detail; bat wings and the
   troll's body shape need work.
4. Intro vignettes could be richer.
5. A full manual playthrough for balance on levels 2–5.
6. Not implemented: barter screen (trading happens in dialogue), Repair, Charm
   and Appraise skills. Save files are 3–4 MB of JSON (could be compressed).
