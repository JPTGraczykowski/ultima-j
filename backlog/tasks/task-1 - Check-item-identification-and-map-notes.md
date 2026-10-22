---
id: TASK-1
title: Check item identification and map notes
status: To Do
assignee: []
created_date: '2026-10-22T11:17:00Z'
labels: []
milestone: m-0
dependencies: []
references:
  - HANDOFF.md
priority: high
type: task
ordinal: 1
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Both features are already in the code but have never been played by hand. Lore identifies enchantments on look (`Game::look` in `src/game/actions.rs`, `Item::identified`); the automap takes notes (`src/ui/overlay.rs`, `Level::notes`, saved with the game). If either one does not work and cannot be fixed cheaply, it moves to 2.0.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Item identification played by hand: enchantments are revealed on look according to Lore
- [ ] #2 Map notes played by hand: notes can be added, shown on the automap and survive save/restore
- [ ] #3 Any feature that cannot be fixed cheaply is removed from 1.0 and tracked in the v2.0 milestone
<!-- AC:END -->
