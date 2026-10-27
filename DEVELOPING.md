# Developing Ultima-J

Commands, tools and tricks for working on the game. For the player's view
see `HANDBOOK.md`; for where the work stands see `HANDOFF.md`.

**Contents**

1. [Build, run, test](#1-build-run-test)
2. [Command-line options](#2-command-line-options)
3. [Level maps](#3-level-maps)
4. [Headless scripts](#4-headless-scripts)
5. [Recipes](#5-recipes)
6. [Designing levels](#6-designing-levels)
7. [Saves and settings](#7-saves-and-settings)
8. [Tasks and decisions (Backlog.md)](#8-tasks-and-decisions-backlogmd)

## 1. Build, run, test

```sh
source ~/.cargo/env                 # if cargo is not on the PATH
cargo build --release               # the binary is target/release/ultima-j
cargo run --release                 # title screen
cargo run --release -- --play       # skip the title, start in the dungeon
cargo test --release                # every test (78, about a minute)
cargo test --release levels         # only tests whose name contains "levels"
cargo test --release jump_reach -- --nocapture   # one test, with its printed output
```

Use `--release`: the debug build is too slow for the fuzzers and the
creature renders.

Tests worth knowing by name:

| Test | What it checks |
| --- | --- |
| `levels_are_connected` | Every level can be crossed from its start to its stairs with the real physics (steps, swimming, jumps) |
| `no_dead_ends` | Every reachable tile leads back to the start, so no pit or pool traps the player (lava is exempt) |
| `gaps_must_be_jumped` | Every level has jump-only places; from level 2 on, the way down crosses one |
| `jump_reach` | Pins `JUMP_UP`, `LEAP_DOWN` and `JUMP_HEADROOM` in `levels.rs` to the player physics |
| `every_rune_is_placed` | All 24 runes can be found |
| `random_walks_keep_player_in_bounds`, `ui_fuzz` | Fuzzers: random movement and random clicks never crash or leave the map |
| `every_phrase_is_translated`, `polish_dialogue_matches_english` | Every `t!()` text and every dialogue line has a Polish version |
| `game_state_round_trips_through_a_save` | Saves load back unchanged |

## 2. Command-line options

| Command | What it does |
| --- | --- |
| `ultima-j` | Start the game at the title screen |
| `ultima-j --play` | Start playing at once, with a ready-made character |
| `ultima-j --maps DIR` | Write `level1.png` … `level5.png`: a labelled top-down map of each level (see [3](#3-level-maps)) |
| `ultima-j --script "cmd; cmd; ..."` | Run the game without a window from a script (see [4](#4-headless-scripts)) |
| `ultima-j --dump-art DIR` | Write the generated art: `palette.png`, `textures.png`, `frame.png`, `items.png`, `fixtures.png`, `hands.png`, `paperdolls.png`, `portraits.png` |
| `ultima-j --creatures FILE` | Write a sheet of every creature: views and animation frames |
| `ULTIMA_J_SELFTEST=out.png ultima-j --play` | Open the real window, save what it shows after 3 seconds, and keep running |
| `ultima-j --help` | A short list of these options |

From `cargo`, put the options after `--`: `cargo run --release -- --maps target/maps`.

## 3. Level maps

There are two ways to see a whole level.

**`--maps DIR`** draws every level as built by `src/game/levels.rs`, at 8
pixels a tile:

- the level number and name at the top;
- **area names** in white (the library, the old guard hall, …), named
  characters in lilac under their dot, and where each staircase leads
  ("down to L2");
- **tile coordinates** every 8 tiles along the top and left edges, with a
  brighter grid line. These are the numbers `goto` and `level` take;
- a legend at the bottom: start (white dot), stairs (gold), doors (brown),
  items (green), named characters (lilac), creatures (red), water (blue),
  lava (orange). Brighter floor is higher; a dark dot in a tile is a slope.
  Gaps show as dark tiles, since their floor is far below.

The maps always show fresh levels, never a saved game. They show everything,
including secret doors and Malchor's thralls. Malchor himself is not on
level 5 until the Shard Gate opens.

**`reveal`** in a headless script marks the current level as explored and
writes its area names on the in-game automap, as notes. Open the map with
`tap M` and take a screenshot to see the level as the player would:

```sh
ultima-j --script "play; reveal; tap M; wait 2; shot map1.png"
ultima-j --script "play; level 3 32.5 6.5; reveal; tap M; wait 2; shot map4.png"
```

The automap is smaller, so labels there can overlap; `--maps` is the one to
read.

## 4. Headless scripts

`--script` runs the game without a window, one frame at a time (30 frames a
second), and prints the result to stderr. It starts in play with a fixed
character on level 1, at the start. The commands are listed at the top of
`src/headless.rs`; separate them with `;`.

**Screens**

| Command | What it does |
| --- | --- |
| `play` | A new game on level 1 (the default) |
| `title`, `chargen`, `intro`, `ending` | Go to the title screen, character creation, the intro or the fall ending |
| `lang en` / `lang pl` | Language of the text; put it before `play` for a Polish game |

**Time and input**

| Command | What it does |
| --- | --- |
| `wait N` | Run N frames |
| `key K N` | Hold key K for N frames. `W+J` holds both (a running jump) |
| `tap K` | Press K for one frame |
| `mouse X Y` | Move the mouse, in 320×200 screen pixels |
| `click X Y`, `rclick X Y` | Left or right click |
| `rhold X Y N` | Hold the right button for N frames (walking with the mouse) |
| `type TEXT` | Type characters (names, notes) |

Key names: letters, digits, `F1`–`F10`, `Up`, `Down`, `Left`, `Right`,
`Enter`, `Esc`, `Space`, `Tab`, `Period`, `Semicolon`, `Backspace`. In play,
F2–F6 pick Talk, Get, Look, Fight and Use, `M` opens the automap, `J` or
`Space` jumps.

**Cheats**

| Command | What it does |
| --- | --- |
| `goto X Z [YAW]` | Teleport on the current level. X and Z are in tiles (`.5` is a tile's middle); YAW is degrees, 0 north, 90 east, 180 south |
| `level N X Z` | Go to level N at X, Z. **N counts from 0**: `level 1 …` is level 2 |
| `give Kind` | Put an item in the pack. Kind is the exact `ItemKind` name: `RedPotion`, `Longsword`, `Lantern` |
| `equip Kind Slot` | Equip an item. Slots: `weapon`, `shield`, `head`, `chest`, `legs`, `feet`, `hands`, `shoulder` |
| `set NAME VALUE` | `vit`, `mana`, `level`, `exp` (adds experience), or any skill |
| `skill NAME VALUE` | Set a skill: `traps`, `lore`, `acrobat`, … (`ALL_SKILLS` in `src/game/items.rs`) |
| `runes all` | Learn all 24 runes |
| `shelf A B C` | Put runes on the shelf: `shelf In Lor` |
| `spawn Race DIST [LOOK]` | Put a peaceful creature DIST tiles ahead, facing you (`ALL_RACES` in `src/game/creatures.rs`) |
| `reveal` | Explore the whole level and write its area names on the automap |

**Output**

| Command | What it does |
| --- | --- |
| `shot PATH` | Save a screenshot at 3×, stretched to 4:3 like the DOS screen |
| `shot1 PATH` | Save a screenshot at 1× (320×200) |

When the script ends it prints the last messages (`msg: …`), creatures within
12 tiles with their health and state, any open overlay (conversation, death,
victory) and the player's position, yaw, vitality, level and experience. An
unknown command or key stops the script with an error.

## 5. Recipes

```sh
# A running jump over the crack in level 1's old guard hall: 45 frames of
# run-up from the south wall puts the jump at the edge (50 falls in).
ultima-j --script "play; goto 18.5 19.5 0; key W 45; key W+J 1; key W 20"

# Start on level 4 with a sword, armour and full runes.
ultima-j --script "play; level 3 32.5 6.5; give Longsword; equip Longsword weapon; give ChainShirt; equip ChainShirt chest; runes all; set level 8; shot crypts.png"

# Talk to Elsmore in the outcasts' camp and screenshot the conversation.
ultima-j --script "play; goto 18.5 41.5 270; tap F2; rclick 112 70; wait 5; shot talk.png"

# Look at a creature up close.
ultima-j --script "play; spawn Troll 2; wait 2; shot troll.png"

# The same screen in Polish.
ultima-j --script "lang pl; play; tap M; shot mapa.png"
```

To find coordinates for `goto`, read them off the `--maps` renders. To find
what a script did, read the `player at …` line it prints at the end.

## 6. Designing levels

Levels are built in code, in `src/game/levels.rs`, one function per level
(`level1` … `level5`) using the builder `B`:

- `room`, `corridor`, `cave`, `carve`, `tile` cut open space; `water`,
  `lava`, `bridge`, `slope` shape it; `light` lights it.
- `gap(x, z, w, h, depth)` drops the floor and keeps the ceiling: a gap to
  jump. `ramp` builds the slope back out of it. Every pit needs a way out,
  or `no_dead_ends` fails.
- `door`, `stairs`, `trap`, `lever`, `message` (text when walked over) and
  `writing` (text on a wall).
- `item`, `put`, `coins`, `container`, `rune_stone`; `npc`, `named`,
  `thrall` for creatures and characters.
- `area(name, x, z)` names the area on the developer maps. Give every new
  room or feature one, centred on tile (x, z). The names are not saved and
  not translated; the player never sees them.

Add new areas **at the end** of a level's function. Caves and creatures draw
from the level's random number generator, so anything inserted earlier
reshapes every cave after it.

The checks after a level change:

```sh
cargo test --release -- levels_are_connected no_dead_ends gaps_must_be_jumped
target/release/ultima-j --maps target/maps   # look at the new layout (target/ is not in git)
```

Then walk it with a script (`goto` near the change, `shot`), and finally by
hand. Old saves keep their old maps, since levels are saved whole; test
level changes in a new game.

## 7. Saves and settings

- Saves are `~/.ultima-j/save1.json.gz` … one per slot: gzipped JSON of the
  whole game. To read or edit one: `zcat ~/.ultima-j/save1.json.gz | jq .`.
  Plain `save1.json` files still load, so an edited save can be written back
  without compressing it (remove the `.gz` one).
- Settings (language and modern controls) are `~/.ultima-j/settings.json`.
  Delete it to get the defaults back.
- In the game: Ctrl+S quick-saves, Ctrl+R loads slot 1, Ctrl+Q quits.

## 8. Tasks and decisions (Backlog.md)

Work is tracked with [Backlog.md](https://github.com/MrLesk/Backlog.md) in
`backlog/`. Use the CLI, never edit its files by hand.

```sh
backlog task list --plain                 # the board
backlog task view TASK-3 --plain          # one task
backlog search "jump" --plain             # tasks, docs and decisions
backlog decision list --plain             # decisions, e.g. decision-1 (MAP_SIZE stays 64)
backlog instructions overview             # how the workflow goes
backlog <command> --help                  # options for any command
```
