---
id: TASK-5
title: Make the eight player classes play differently
status: Done
assignee:
  - '@claude'
created_date: '2026-10-22T11:17:00Z'
updated_date: '2026-10-27T13:07:00Z'
labels: []
milestone: m-0
dependencies: []
references:
  - src/game/state.rs
  - HANDBOOK.md
type: enhancement
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The eight classes (`PClass` in `src/game/state.rs`) need a review so the choice at character creation matters.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Each class has a distinct start (skills, gear or spells) and play style
- [x] #2 HANDBOOK.md describes each class accurately
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Each class gets its own kit and one passive gift (helpers on PClass/Player in state.rs, hooks where the rule applies). Fighter and Paladin, Mage and Druid, Ranger and Shepherd no longer share a start.
1. Fighter: shortsword, wooden shield, leather vest and cap. Gift: full-power swings crit 50% of the time (others 25%).
2. Paladin: skills Attack, Defense, Mace, Casting, Charm; mace, leather vest; runes IN BET SANCT MANI. Gift: +50% damage against the undead.
3. Mage: runes IN LOR ORT JUX BET SANCT, candle, blue potion; +6 mana. Gift: spells cost circle x2 mana instead of x3.
4. Druid: skills Mana, Casting, Mace, Track, Swimming; cudgel, candle; runes IN LOR MANI YLEM BET; +4 mana. Gift: regen every 10 s instead of 14, food nourishes 50% more.
5. Bard: dagger, 40 gold, wine; runes IN LOR SANCT HUR. Gift: 4 skill points per level.
6. Tinker: cudgel, lockpick, leather gloves, torch. Gift: gear wears half as fast, failed repairs never damage it.
7. Ranger: sling, 20 stones, leather boots, dagger. Gift: double damage on a hostile creature that has not noticed you (Idle/Wander).
8. Shepherd: skills Unarmed, Mace, Acrobat, Swimming, Search; cudgel, bread, cheese, water flask. Gift: hunger 1 per 30 s instead of 20, +2 vitality per level.
9. Character creation shows the class line plus its gift; Polish lines in assets/lang/pl.txt.
10. Tests for kits and gifts; update HANDBOOK.md and HANDOFF.md.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented as planned. Gifts are PClass helpers so the numbers live in one place; the Paladin/Ranger bonus is applied in ai::damage_npc so it covers melee, missiles and spells. Character creation wraps the class line and gift into the right half (checked in English and Polish with headless shot1). cargo test --release: 84 passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Every profession now starts differently and has one gift that changes how it plays.

- Kits and skills (src/ui/menus.rs starting_kit, PClass::skills): Fighter gets a shortsword, shield and cap. Paladin swaps Sword for Casting and starts with a mace and the runes for Resist Blows, Lesser Heal and Heal. Mage has more runes and a blue potion. Druid swaps Lore for Mace and starts with a cudgel and the runes for Light, Create Food and Lesser Heal (no Magic Arrow). Bard starts with 40 gold, wine, and the runes for Light and Stealth. Tinker gets gloves and a torch. Ranger gets 20 stones and a dagger. Shepherd swaps Missile and Track for Mace and Acrobat and starts with food and water.
- Gifts (PClass helpers in state.rs, Player::spell_cost): Fighter 50% crits, Mage 2x circle mana cost, Bard 4 skill points per level, Tinker half wear and no repair fumbles, Druid faster regen and +50% food, Paladin +50% against the undead, Ranger double damage on unaware foes, Shepherd slower hunger and +2 vitality per level.
- Character creation shows each gift under the class line (English and Polish).
- HANDBOOK.md section 2 has new profession and gift tables; the hunger, healing, level-up, combat, wear, repair and mana-cost rules mention the class exceptions. HANDOFF.md notes the work.
- Tests: every_profession_starts_differently, profession_traits; cargo test --release passes 84/84.

Risks: the numbers are first guesses. TASK-6 (balance playthrough) should check them, especially the Ranger's sneak attack with a sling and the Paladin's mace at level 1. Existing saves keep their old kit and skills but gain the gifts.
<!-- SECTION:FINAL_SUMMARY:END -->
