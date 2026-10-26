---
id: decision-1
title: >-
  MAP_SIZE stays 64x64 for 1.0: levels grow into their unused space, not past
  the edge
date: '2026-10-26T13:18:00Z'
status: accepted
---
## Context

Levels felt small (TASK-2). Each level is a 64×64 tile grid (`MAP_SIZE` in
`src/world/map.rs`, the original game's size), and each one left large
corners unused. Raising `MAP_SIZE` would touch path-finding (`ai::path_step`
works on the whole grid), the automap, the `--maps` renders and the save
format, which stores whole levels.

## Decision

Keep `MAP_SIZE` at 64 for 1.0. Levels grow into their unused space instead.
The TASK-2 pass added 16 side areas and gaps to jump. Open tiles per level,
before -> after: L1 567->873, L2 677->912, L3 1001->1181, L4 661->995,
L5 708->908. Every level still has unused space inside 64×64.

## Consequences

- Path-finding, the automap and saves are unchanged. Old saves load as
  before but keep their old maps; new games get the new layouts.
- New areas must fit inside the grid. Build them at the end of each level
  function, because caves and creatures draw from the level's rng and
  anything added earlier reshapes the older caves.
- Revisit for 2.0 if the depths below the levels (TASK-8) or the hidden area
  (TASK-7) need more room than a level has left. A bigger grid would then need
  path-finding, the automap and save loading checked again.

