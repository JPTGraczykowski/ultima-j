# Handoff — where the work stands

Goal: a clone of Ultima Underworld (1992) in Rust whose graphics match the
original's style. Status (2026-10-15): the game is complete and playable from
the title screen to the victory screen, and all 61 tests pass. What remains is
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
  the shards sets flag `ending` = 2 (alive, out through the doors); killing
  him sets `ending` = 1 (the new Lord of Souls). The victory overlay picks its
  text from that flag. Malchor broke the Stone and is a rival for the throne.
  His seven thralls on level 5 carry `creatures::THRALL` in `Npc::flags`; he
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
- Visual pass after the side-by-side review: smaller command icons with oval
  cartouches, reworked creature models (stone golem, bat wings with membranes,
  hunched trolls, humanoid proportions), bigger paperdoll with a face and beard.

## How to check things

```sh
source ~/.cargo/env
cargo test --release                       # 61 tests: level connectivity, runes, fuzzers, save round-trip, skills, spells, barter
cargo run --release                        # play
target/release/ultima-j --script "..."     # headless runs and screenshots (src/headless.rs)
target/release/ultima-j --maps DIR         # level layouts
ULTIMA_J_SELFTEST=out.png target/release/ultima-j --play   # real-window capture
```

Compare screenshots against originals side by side (native 320×200 DOS
captures are on oldgames.sk; the reference card is on mocagh.org).

## Next steps (review of 2026-10-15), in suggested order

1. **Balance playthrough of levels 2–5 by hand.** The scripted runs only show
   the levels can be finished; difficulty, loot, food and light supply are
   untested. Barter prices are now `Item::value` (Tobin's red potion is 40
   gold, Bodric's chain shirt 100) and gold is scarce (about 112 coins
   placed, plus nuggets and whatever the player sells), so check that the
   traders are worth visiting.
