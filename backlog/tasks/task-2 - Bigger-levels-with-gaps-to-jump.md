---
id: TASK-2
title: Bigger levels with gaps to jump
status: In Progress
assignee:
  - '@claude'
created_date: '2026-10-22T11:17:00Z'
updated_date: '2026-10-26T12:49:00Z'
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
- [ ] #1 Unused space in each of the five levels is filled with new areas
- [ ] #2 Each level has gaps the player must jump
- [ ] #3 Level connectivity tests and the scripted runs still pass
- [ ] #4 Decision recorded on whether MAP_SIZE stays at 64
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Findings (measured with the real player physics):
- A running jump (W, forward 2.0) clears a 1-tile gap; the far side may be up to 2 height steps higher. A walking jump falls short.
- A 2-tile gap is only cleared with Acrobat ~10 onto a ledge 8 steps lower, or with the Speed spell.
- Saves store whole levels, so old saves keep their old maps; only new games get the new layouts.
- AI path-finding steps down any drop, so creatures can follow into pits; pits need a way out.

1. Builder helpers in levels.rs: B::gap (a pit across a corridor/room, floor well below, high ceiling for the jump arc) and a ramp out of each pit back to the near side, so a missed jump costs HP and time, never a soft-lock.
2. Tests: extend levels_are_connected with jump edges (1 tile anywhere, 2 tiles onto a ledge 8+ steps lower); add a no-soft-lock check (from every reachable tile the start is reachable again); add a physics test that pins the jump distances the level test assumes.
3. Fill unused space, level by level, in each level's style, with creatures and loot but no new quests or talking NPCs (TASK-3 puts those here):
   L1 north-west and south-west; L2 north and south-east; L3 south-west and middle east; L4 four corners; L5 corners and south.
4. At least one jump per level; first gap on L1 is easy and signposted with a writing.
5. Check with cargo test, --maps renders, and headless scripted runs that jump each gap.
6. Record decision: MAP_SIZE stays 64 (path-finding, automap, save format untouched; filled levels still have room).
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Agreed with the user (2026-10-26): L1 gaps are optional and teach the running jump; from L2 on, at least one gap per level is on the way to the stairs down. New areas get level-themed creatures and modest loot (food, light, minor items); TASK-6 rebalances.

Groundwork: corrected measurements with a fixed harness (fresh game per attempt, enough frames). A running jump clears 1 tile (landing up to 3 steps higher) and a running leap clears 2 tiles onto a ledge 8+ steps lower, for every character; Acrobat adds little, so no jump is a skill gate. Pinned by game::fuzz_tests::jump_reach (JUMP_UP=2, LEAP_DOWN=8, JUMP_HEADROOM=14 in levels.rs).
levels_are_connected now follows the real physics (step MAX_STEP, swim climb SWIM_CLIMB, jumps); new no_dead_ends checks every reachable tile leads back to the start (lava exempt: it kills).
Fixed (agreed with the user): swimmers could not climb out of water 3 steps below the bank (L3 great hall, grotto). SWIM_CLIMB=0.45 while swimming, snapping up onto the bank; tests swimmers_climb_out and no_dead_ends cover it.
<!-- SECTION:NOTES:END -->
