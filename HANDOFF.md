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

## Art review (2026-10-14)

- Command icons are pixel art (`ICON_ROWS` in `src/art/ui.rs`): 19-29 px wide
  like the original, black outlines, flat silver tones, a brighter set for the
  active command, and oval recesses with a dark upper-left rim and a warm
  lower-right lip (`oval_rim`). Checked icon by icon against og39/og05.
- Paperdoll: the body is still modelled (so armour sets keep working), the
  head is hand-drawn pixel art (`HEAD_BEARD`/`HEAD_SHAVEN`/`HEAD_FEMALE` and
  `HELM` in `src/art/creatures.rs`); sleeveless woven tunic with a V-neck,
  seam lines where materials meet, bigger feet, proportions matched to og39.
  The Flesh ramp is now the original's redder skin tone.
- Creatures: facial features are painted over the modelled heads as pixel
  art (`src/art/faces.rs`), turning with the head in every pose and hidden by
  anything in front of it. The bat has membrane wings (new triangle
  primitive), the reaper is a bare tree with raised branch arms, bark grooves
  and a carved face, the troll has a bigger head, the lurker a toothy maw,
  the headless the original's orange skin and loincloth.
- Review tools: `--dump-art DIR` now also writes `paperdolls.png` (every
  armour set) and `portraits.png`; the headless script has
  `spawn Race DIST [LOOK]` to put a creature in front of the player.

## Next steps

1. Creature bodies are still smooth "clay": the originals have chunkier,
   hand-shaded bodies and more varied poses (weapons held out, crouching
   goblins). The golem has no visible head/face (the original's does).
2. Paperdoll arms are a little stiff and pale next to the original's.
3. Intro vignettes could be richer.
4. A full manual playthrough for balance on levels 2-5.
5. Not implemented: barter screen (trading happens in dialogue), Repair, Charm
   and Appraise skills. Save files are 3-4 MB of JSON (could be compressed).
