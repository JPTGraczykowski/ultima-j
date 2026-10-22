---
id: TASK-2
title: Bigger levels with gaps to jump
status: To Do
assignee: []
created_date: '2026-10-22T11:17:00Z'
labels: []
milestone: m-0
dependencies:
  - TASK-1
references:
  - src/world/map.rs
  - src/game/levels.rs
type: feature
ordinal: 2
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
