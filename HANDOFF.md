# Handoff — where the work stands

Goal: a clone of Ultima Underworld (1992) in Rust whose graphics match the
original's style. Status (2026-10-27): the game is complete and playable from
the title screen to the victory screen, and all 82 tests pass. What remains is
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
- Level pass (2026-10-26, TASK-2): each level's unused space holds new side
  areas with creatures and modest loot, ready for the side quests of
  TASK-3. Gaps to jump: `B::gap` drops the floor, and `B::ramp` builds the way
  back up out of every pit. A running jump clears one tile, or two onto a
  ledge 8 steps lower (`JUMP_UP`, `LEAP_DOWN`, `JUMP_HEADROOM`, pinned to the
  physics by `jump_reach`). On level 1 the gaps guard side areas; from level 2
  on the way down (on level 5 the way to Varian) crosses one. The tests check
  all this: `levels_are_connected` follows the real physics, `no_dead_ends`
  makes sure nothing traps the player, and `gaps_must_be_jumped`. New areas
  go at the end of each level's function, because caves and creatures draw
  from the level's rng and anything added earlier reshapes the older caves.
  Swimmers now climb out onto banks up to `SWIM_CLIMB` high (before, the deep
  water on level 3 was a trap). `MAP_SIZE` stays 64 (decision-1).
- Side quests and puzzles (2026-10-27, TASK-3; the story is in
  `STORY.md`, "Side quests"). Each level has two side quests from its
  people, a puzzle and a page of Maera's Chronicle of the Colony
  (`ItemKind::ChroniclePage`, texts in `dialogue::CHRONICLE`); Elsmore,
  her brother, pays for the pages (flag `chronicle`). New characters:
  Corwin (L1), Brann (L2), Ssethra (L3), Ysolde (L4) and Ilsa (L5);
  Ssethra and Ysolde were unnamed creatures already on the map and are
  renamed after the fact, so the level's rng and caves keep their shape.
  New quest items each have an icon (`art::items::icon`) and a Polish noun.
  New conversations go at the **end** of `assets/dialogue.txt` (and the
  Polish file): saved NPCs keep their conversation's index. Dialogue gained
  `item:Kind>=N`, `flag:NAME<N`, and the actions `flag+ NAME [N]`,
  `move X Z [YAW]` (the speaker goes elsewhere, e.g. home) and `rest X Z R`
  (the unnamed undead nearby crumble). The test
  `at_most_five_visible_choices` counts the options a node can show at
  once (options with exclusive conditions are not counted together, and a
  barter node adds 3); when a node is full, hide an option behind an
  exclusive condition or move it to a sub-node. Puzzle tools in
  `levels.rs`: `B::combo` (a lock of levers, each wanted on or off; the
  door can't be opened by hand), `B::plate` (a pressure plate, which
  prints a message when it opens its door) and `B::bridge_lever` (a lever
  that raises a bridge over lava or water for good; the level tests check
  the levels with every bridge raised). Lava never burns: see TASK-11.
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
cargo test --release                       # 82 tests: level connectivity and jumps, runes, fuzzers, save round-trip, skills, spells, barter, five choices per conversation node
cargo run --release                        # play
target/release/ultima-j --script "..."     # headless runs and screenshots (src/headless.rs)
target/release/ultima-j --script "play; level 1 21.5 47.5; goto 21.5 47.5 180; key W 23; key W+J 1; key W 20"   # a running jump
target/release/ultima-j --maps DIR         # level layouts, with area names and tile coordinates
ULTIMA_J_SELFTEST=out.png target/release/ultima-j --play   # real-window capture
```

`DEVELOPING.md` lists every developer command, the headless script
commands and recipes. Name new level areas with `B::area` so they show on
the `--maps` renders and on the automap after `reveal`.

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

## Roadmap (agreed 2026-10-22)

Replaces the "next steps" of the 2026-10-15 review. For each feature: design
it, play it by hand, then fix and improve.

### Version 1.0, in this order

1. **Check item identification and map notes.** Both are in the code
   already: Lore identifies enchantments on look (`Game::look` in
   `src/game/actions.rs`, `Item::identified`), and the automap takes notes
   (`src/ui/overlay.rs`, `Level::notes`, saved with the game). Play them by
   hand. If either one doesn't work and can't be fixed cheaply, leave it
   out of 1.0 and move it to 2.0.
2. **Bigger levels with gaps to jump.** Maps are fixed at 64×64 tiles
   (`MAP_SIZE` in `src/world/map.rs`, the original's size). First fill the
   unused space in each level. Raising `MAP_SIZE` touches path-finding, the
   automap and the save format. Jumping, falling and Acrobat already work,
   so the gaps are level design only.
3. **NPC backstories, side quests and puzzles.** Per level: 2 side quests
   and 2 puzzles or finding tasks, plus one collection that spans all five
   levels. Write the backstories together with the quests, since the quests
   come from the NPCs. Conversation menus show only 5 options per node.
   Keep `STORY.md`, `STORY.pl.md` and the Polish translation in step.
4. **Art.** Creature bodies are still smooth "clay": the originals have
   chunkier, hand-shaded bodies and more varied poses (weapons held out,
   crouching goblins). The golem has no visible head or face (the
   original's does). The paperdoll's arms are a little stiff and pale next
   to the original's.
5. **Player classes.** Review the eight classes (`PClass` in
   `src/game/state.rs`) so each one plays differently.
6. **Combat balance, magic balance, a bit more difficulty.** Do this last,
   as one playthrough of all five levels: steps 2–5 change experience,
   loot, food and light supply. The scripted runs only show the levels can
   be finished. Barter prices are `Item::value` (Tobin's red potion is 40
   gold, Bodric's chain shirt 100), and gold is scarce, so check that the
   traders are worth visiting. Keep `HANDBOOK.md` in step.

### Version 2.0

- An optional hidden area on a level.
- Depths below the levels: deep water, pits, and so on. The renderer has
  one floor height per tile with no stacked floors, so these have to be
  deep areas in the same map or separate sub-levels. Decide which before
  designing them.
- New side quests.
- Richer intro pictures.
- Item identification and map notes, if they were left out of 1.0.
