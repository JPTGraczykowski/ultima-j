# Handoff — where the work stands

Goal: a clone of Ultima Underworld (1992) in Rust whose graphics match the
original's style. Status (2026-10-15): the game is complete and playable from
the title screen to the victory screen, and all 65 tests pass. What remains is
mostly balance.

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
- Five levels (`src/game/levels.rs`), three shard quests and two endings,
  all verified with scripted runs.
- Story (2026-10-16), inspired by Dark Souls: the player rises from a grave as
  one of the undead, learns who they were (Sir Aldric and the Lord of Souls
  tell them), and brings the three shards to Varian, the Lord of Souls
  (`=== lord` in `assets/dialogue.txt`, a peaceful `Race::Demon` drawn as a
  crowned, ashen king; `Humanoid::crown` in `src/art/creatures.rs`). Giving him
  the shards sets flag `ending` = 2 (alive, out through the doors) and shows
  the victory overlay. Killing him sets `ending` = 1: the view fades to black
  and `menus::Story::fall()` tells the rest in five pictures, like the intro
  (the throne gives the player nothing, the great doors open, and the
  goblins, mountain folk and lizardmen fight over the way to the surface).
  Malchor broke the Stone and is the voice that woke the player: the Shard
  Gate burns him, so he posed as Varian (intro, dreams) to get one of
  Varian's knights to bring the shards. He is not on the map at first;
  opening the gate calls `Game::malchor_appears`, which spawns him
  (`levels::malchor`) behind the player, sets flag `malchor_revealed` and
  starts his conversation. The Open spell can't open the gate. His seven
  thralls on level 5 carry `creatures::THRALL` in `Npc::flags`; he
  carries `MASTER`, and `ai::release_thralls` kills them when he dies.
  Conversation menus show only 5 options, so keep each node's visible choices
  at 5 or fewer.
- All 20 skills have an effect. Lore identifies enchantments on look;
  Appraise estimates worth; Charm and Appraise adjust prices in dialogue
  (`{buy N}`/`{sell N}`, `afford:N`, `pay`/`earn`, `skill:NAME>=N`); Repair
  mends gear used from the inventory (Bodric also mends for gold); Traps
  spots, maps and disarms traps (13 across the five levels); Swimming,
  Acrobat and Sneak tune swimming, jumping/falling and creature notice range
  (helpers on `Player` in `src/game/state.rs`).
- Barter screen (`src/game/barter.rs`, `src/ui/convo.rs`): a dialogue node
  with `! barter` opens the two trade areas. The NPC's `inventory` is their
  stock (Tobin and Bodric are stocked in `levels.rs`); pick goods on the left,
  put gold or items on the right, then "Let us trade" / "What is mine offer
  worth?" / "Show me thy other goods". Goods cost `buy_price(value)`; offered
  items are worth `sell_price(value) / 2`, capped at what the trader would
  ask, so there's no buy-low/sell-high loop. Change is paid in gold, sold
  items join the stock, and anything left on the table goes back to the
  player when the conversation ends.
- Magic is complete: all 24 runes are placed (POR in the Level 2 deep mine,
  QUAS in the Level 3 reliquary) and every spell does something (Conceal is
  a stronger Stealth; Rune of Warding leaves an `ItemKind::Ward` on the floor
  that snares the next foe, see `Game::update_wards`). Circle *c* needs
  experience level *c* (`Player::max_circle`), and `exp_for_level` is a
  table paced so a thorough player reaches level 8 in the Crypts.
- `HANDBOOK.md` is the player's handbook. Keep its numbers in step with the
  code when balancing.
- `STORY.md` and `STORY.pl.md` tell the whole story in English and Polish.
  Keep them in step with `assets/dialogue.txt` and the intro and ending text
  in `src/ui/menus.rs`. The README's quest section stays spoiler-free.
- Visual pass after the side-by-side review: smaller command icons with oval
  cartouches, reworked creature models (stone golem, bat wings with membranes,
  hunched trolls, humanoid proportions), bigger paperdoll with a face and beard.

## How to check things

```sh
source ~/.cargo/env
cargo test --release                       # 73 tests: level connectivity, runes, fuzzers, save round-trip, skills, spells, barter
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

## Next steps (review of 2026-10-15), in suggested order

1. **Balance playthrough of levels 2–5 by hand.** The scripted runs only show
   the levels can be finished; difficulty, loot, food and light supply are
   untested. Barter prices are now `Item::value` (Tobin's red potion is 40
   gold, Bodric's chain shirt 100) and gold is scarce (about 112 coins
   placed, plus nuggets and whatever the player sells), so check that the
   traders are worth visiting.
2. Creature bodies are still smooth "clay": the originals have chunkier,
   hand-shaded bodies and more varied poses (weapons held out, crouching
   goblins). The golem has no visible head/face (the original's does).
3. Paperdoll arms are a little stiff and pale next to the original's.
4. Intro vignettes could be richer.
