---
id: TASK-11
title: Lava never burns, and deep lava traps the player
status: To Do
assignee: []
created_date: '2026-10-27T12:33:00Z'
labels: []
milestone: m-0
dependencies: []
references:
  - src/game/mod.rs
  - src/game/physics.rs
type: bug
ordinal: 5006
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
In Game::update (src/game/mod.rs, the Liquids block), Player::drown is reset to 0 every frame when the player is not swimming, just before the lava check adds dt to it and waits for it to pass 0.5 s. So the lava timer never fills and lava never burns. Lava 3 steps deep (the throne cave on level 5, the ember hall added in TASK-3) is deeper than MAX_STEP (0.27, about 2 steps) climbs out of, so a player who walks in is stuck, unhurt, with no way out. Shallower lava (2 steps, the level 5 antechamber and cave of fire) can be climbed out of. Found by scripted runs in TASK-3: standing in lava for 5 seconds leaves vitality unchanged. The level test no_dead_ends exempts lava because lava is meant to kill. Fixing this makes lava a real hazard (4 damage every 0.5 s, 1 with Resist Fire), which changes difficulty: check it with TASK-6.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Standing in lava burns the player every half second, as the code intends (4 damage, 1 with Resist Fire)
- [ ] #2 Swimming in water still uses its own drowning timer and is unaffected
- [ ] #3 A test or headless script shows lava hurts and water does not
- [ ] #4 HANDBOOK.md says what lava does
<!-- AC:END -->
