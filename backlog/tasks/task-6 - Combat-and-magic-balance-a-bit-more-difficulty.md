---
id: TASK-6
title: Combat and magic balance, a bit more difficulty
status: Done
assignee:
  - '@claude'
created_date: '2026-10-22T11:17:00Z'
updated_date: '2026-11-04T12:26:00Z'
labels: []
milestone: m-0
dependencies:
  - TASK-2
  - TASK-3
  - TASK-5
references:
  - HANDBOOK.md
  - src/game/barter.rs
type: task
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Done last as one playthrough of all five levels, because the earlier 1.0 tasks change experience, loot, food and light supply. The scripted runs only show the levels can be finished. Barter prices are `Item::value` (Tobin red potion 40 gold, Bodric chain shirt 100) and gold is scarce, so check that traders are worth visiting.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 One full hand playthrough of all five levels completed after the other 1.0 tasks
- [x] #2 Combat and magic tuned for somewhat more difficulty
- [x] #3 Traders are worth visiting with the gold a player finds
- [x] #4 HANDBOOK.md numbers match the code
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Measure: a census of each level (hostile exp, coins, gem worth, food, potions, trader goods) and a duel simulator that runs player profiles for each level against that level's creatures through the real game loop.
2. Combat: lower player to-hit (base 50, 2% per Attack/weapon point, DEX/3), count Defense and armour at 2% a point against creature blows, armour soaks armour/4, creatures swing more often, raise creature hits and damage (most from level 3 on).
3. Experience: stretch the table so a thorough player reaches about level 8 in the Crypts again (quests give as much exp as kills).
4. Magic: buff the weak attack spells (Poison, Tremor, Flame Wind).
5. Traders: deeper stock for Tobin and Bodric (potions, breastplate, crossbow) and a third trader, Hissk on level 3, so gold from levels 3-5 has a use.
6. Keep the census and duels as ignored balance tools; update HANDBOOK, DEVELOPING, HANDOFF.
7. The user plays all five levels by hand (AC #1) and the numbers get adjusted from that.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Measured before: an on-level fighter lost ~1 vitality a fight (L3 ghoul 0.9 of 60, L4 troll 3.4 of 72); player to-hit was pinned at the 95% cap; exp from kills + quests put a thorough player at level 10 at the end of the Crypts (handbook said 8); gold after level 2 (~700 coins plus gems) had nothing to buy.
After (balance_duels, 200 fights each, full-power swings): L1 fighter lv2 loses 2-3 to a goblin; L3 fighter lv6 loses 6 to a ghoul, 12 to a lurker; L3 paladin 7 and 19; L4 fighter lv8 loses 18 to a troll, 47 of 72 to the golem; L4 ranger lv8 with a sword loses 40 of 63 to a troll. L5 in plate stays easy one-on-one (2 vs a ghoul); its danger is groups and fire-throwing imps.
Hissk's trade checked headless in English and Polish (peace -> 'Do the People trade?' -> barter screen with his potions and fish). cargo test --release: 84 passed, 2 ignored (the balance tools).
Not changed: sleep (90% heal, food-limited), spell costs, exp per kill, level contents (adding creatures would reshape the levels' rng caves).

2026-11-04: the user played all five levels by hand after the balance pass and asked to close the task.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Combat, magic and traders rebalanced for a bit more difficulty, measured with two new ignored balance tools (balance_census, balance_duels) and played through by the user.

- Combat: player to-hit 50% + 2% per Attack/weapon point + DEX/3 (was capped at 95%); creature blows count Defense and armour at 2% a point, armour soaks armour/4; creatures swing more often and have more hits and damage, most from level 3 on. An on-level fighter now loses 10-25% vitality to standard foes and far more to trolls and the golem (before: ~1 vitality a fight).
- Experience table stretched (level 8 at 2,600) so a thorough player reaches level 8 in the Crypts.
- Magic: Poison 6 damage + 30 s poison, Tremor 20 + Casting, Flame Wind 26 + Casting.
- Traders: Tobin and Bodric stock more (potions, food, breastplate, the only crossbow); Hissk on level 3 trades potions and fish (English and Polish).
- HANDBOOK.md (formulas, exp table, bestiary, spells, traders), DEVELOPING.md and HANDOFF.md updated. cargo test --release: 84 passed, 2 ignored.

Risks: level 5 in plate is still easy one-on-one; old saves keep their creatures' old hits.
<!-- SECTION:FINAL_SUMMARY:END -->
