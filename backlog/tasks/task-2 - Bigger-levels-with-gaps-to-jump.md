---
id: TASK-2
title: Bigger levels with gaps to jump
status: In Progress
assignee:
  - '@claude'
created_date: '2026-10-22T11:17:00Z'
updated_date: '2026-10-26T13:23:00Z'
labels: []
milestone: m-0
dependencies:
  - TASK-1
references:
  - src/world/map.rs
  - src/game/levels.rs
type: feature
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Levels feel small. Maps are fixed at 64×64 tiles (`MAP_SIZE` in `src/world/map.rs`, the original size), and each level leaves space unused. Raising `MAP_SIZE` would touch path-finding, the automap and the save format, so fill the unused space first. Jumping, falling and Acrobat already work, so gaps are level design only.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Unused space in each of the five levels is filled with new areas
- [x] #2 Each level has gaps the player must jump
- [x] #3 Level connectivity tests and the scripted runs still pass
- [x] #4 Decision recorded on whether MAP_SIZE stays at 64
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Findings (measured with the real player physics, game::fuzz_tests::jump_reach):
- A running jump (W) clears a 1-tile gap, landing up to 3 steps higher; a walking jump falls short.
- A running leap clears 2 tiles onto a ledge 8+ steps lower, for everyone; Acrobat adds little, so no jump is a skill gate.
- Saves store whole levels: old saves keep their old maps; only new games get the new layouts.
- AI path-finding steps down any drop, so creatures can follow into pits; every pit needs a way out.

1. Builder helpers in levels.rs: B::gap (drop the floor, keep the ceiling) and B::ramp (slope tiles out of a pit).
2. Tests: levels_are_connected follows the real physics (MAX_STEP, SWIM_CLIMB, jumps); no_dead_ends (every reachable tile leads back to the start, lava exempt); gaps_must_be_jumped (every level has jump-only places; from L2 the way down, on L5 the way to Varian, needs a jump); jump_reach pins JUMP_UP/LEAP_DOWN/JUMP_HEADROOM.
3. Fill unused space with themed areas, creatures and modest loot, appended at the end of each level function so older caves keep their rng-driven shapes.
4. Gaps: L1 optional (guard hall crack, midden crevice); L2 shaft on the way down + diggings leap; L3 broken aqueduct; L4 fallen passage + charnel pit leap; L5 lava crack before Varian.
5. Polish lines, HANDBOOK rules and walkthrough, HANDOFF.
6. Verify: cargo test, --maps renders, old/new tile comparison, headless --script runs with real key input (headless key accepts W+J).
7. Record decision-1: MAP_SIZE stays 64.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Agreed with the user (2026-10-26): L1 gaps are optional and teach the running jump; from L2 on, at least one gap per level is on the way to the stairs down. New areas get level-themed creatures and modest loot (food, light, minor items); TASK-6 rebalances.

Groundwork: corrected measurements with a fixed harness (fresh game per attempt, enough frames). A running jump clears 1 tile (landing up to 3 steps higher) and a running leap clears 2 tiles onto a ledge 8+ steps lower, for every character; Acrobat adds little, so no jump is a skill gate. Pinned by game::fuzz_tests::jump_reach (JUMP_UP=2, LEAP_DOWN=8, JUMP_HEADROOM=14 in levels.rs).
levels_are_connected now follows the real physics (step MAX_STEP, swim climb SWIM_CLIMB, jumps); new no_dead_ends checks every reachable tile leads back to the start (lava exempt: it kills).
Fixed (agreed with the user): swimmers could not climb out of water 3 steps below the bank (L3 great hall, grotto). SWIM_CLIMB=0.45 while swimming, snapping up onto the bank; tests swimmers_climb_out and no_dead_ends cover it.

Level pass done:
- L1: old guard hall north of the library (crack with ramp out west; chest), goblins' midden below Gnarl's den (crevice to a hoard nook), bat roost east of the water cave.
- L2: shaft across the tunnel past the forge (main route; ramp out to the forge), old diggings north of the quarters (leap down to a cache ledge; Thorgrim's sign), rat storeroom off the arrival tunnel, flooded drift south of the spider galleries.
- L3: broken aqueduct span before the stairs (main route; water at the bottom, ramp out), fungus cavern south of the village, ossuary below the temple.
- L4: fallen passage between Aldric's tomb and the guardian's hall (main route; ramp out to the tomb), catacombs (NW), charnel pit with a leap to a reliquary ledge (NE), mass graves (SW), embalmers' rooms (SE).
- L5: lava crack across the throne cave between the gate and Varian (ramp out east, back to the gate; Malchor's fixed spawn at 32.5,21.5 is north of it), cells south of the study, cave of fire with a reaper (SE).
Verification: compared every old level's tiles, items and NPCs (with facing) against commit 47c82ff: only the gap tiles, two diagonal corners where new corridors join, and the L5 lava glow changed. Headless runs with real input: running jumps cross all five main-route gaps (and L2 back north); the L2 leap lands on the ledge from a long enough run-up (1 HP hard landing); a walking jump falls into the L1 crack; climbing out of the L1 crack, the L2 shaft and the L5 lava crack works.
MAP_SIZE decision (decision-1): stays 64. Filling the unused space roughly doubles the explored area per level and every level still has room; raising it would touch path-finding, the automap and the save format. Revisit for the 2.0 depths (TASK-8) if a level needs more.
Note: decision-1 has only its title and status; the CLI cannot fill Context/Decision/Consequences.

Correction to the decision note above: the pass does not double the levels. Measured open tiles, old -> new: L1 567->873 (+54%), L2 677->912 (+35%), L3 1001->1181 (+18%), L4 661->995 (+51%), L5 708->908 (+28%). Every level still has unused space inside 64x64.

decision-1's Context, Decision and Consequences are now written (edited in the file directly, at the user's request, since the CLI cannot set them).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Filled the unused space of all five levels with new side areas and added gaps to jump; MAP_SIZE stays 64 (decision-1).

What changed
- levels.rs: B::gap and B::ramp; 16 new areas with themed creatures and modest loot (no quests or talking NPCs; those come with TASK-3); a main-route gap on L2-L5 (L5: a lava crack before Varian), side-area gaps on L1, leaps down to cache ledges on L2 and L4. New areas sit at the end of each level function so the older caves keep their shapes.
- Physics: swimmers climb out onto banks up to SWIM_CLIMB (0.45). Before, the L3 great hall channel and the grotto trapped a swimmer (agreed with the user).
- Tests: levels_are_connected follows the real physics; new no_dead_ends, gaps_must_be_jumped, jump_reach, swimmers_climb_out. 78 tests pass.
- Headless: key accepts W+J to hold several keys (running jumps in scripts).
- Polish lines for 8 new texts; HANDBOOK (jumping rules, walkthrough), HANDOFF.

Verification: cargo test (78/78); --maps renders; tile/item/NPC comparison against 47c82ff (only intended tiles changed); headless runs with real key input cross every main-route gap, take the L2 leap, fail a walking jump, and climb out of pits.

Risks / follow-ups
- Old saves keep their old maps (levels are saved whole).
- The new loot adds about 200 gold, gems and gold nuggets, and some armour, all before the TASK-6 balance pass.
- Acrobat barely changes jump reach, so no gap is a skill gate (TASK-5/TASK-6 may want that).
- Lava pits are exempt from no_dead_ends (lava kills); the L5 crack's exit was checked by hand.
- Needs the user's play-through before Done, per the roadmap.
<!-- SECTION:FINAL_SUMMARY:END -->
