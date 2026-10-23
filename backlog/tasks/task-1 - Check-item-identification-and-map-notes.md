---
id: TASK-1
title: Check item identification and map notes
status: Done
assignee:
  - '@claude'
created_date: '2026-10-22T11:17:00Z'
updated_date: '2026-10-23T14:57:00Z'
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
- [x] #1 Item identification played by hand: enchantments are revealed on look according to Lore
- [x] #2 Map notes played by hand: notes can be added, shown on the automap and survive save/restore
- [x] #3 Any feature that cannot be fixed cheaply is removed from 1.0 and tracked in the v2.0 milestone
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Scripted pre-check (no code changes): headless runs with screenshots - pick up and look at the Level 1 gold ring (Protection +2) with Lore 0 and with high Lore; on the automap write a note, erase one, save to a slot, restore and check the note is still there.
2. Write a short hand-play checklist for the user: which class, where the rings are (L1 gold ring, L3 silver ring), Name Enchantment, map-note steps, save/restore.
3. User plays by hand; record results in implementation notes.
4. Fix cheap bugs found (after user approval), each with a unit test where possible. Anything not cheap: remove from 1.0 and create a v2.0 task.
5. Run cargo test --release, then finalize.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Research findings (before play):
- Identification: Game::examine (actions.rs) checks Lore*6 + INT/2 >= Enchant::difficulty(); state is stored on the item from the world, the pack, containers and the paperdoll. Name Enchantment marks everything carried. Unit test lore_identifies_and_appraise_values covers the logic.
- Only 3 enchanted items exist: Level 1 gold ring (Protection +2, difficulty 26), Level 3 silver ring (Protection +3, difficulty 34), and Malchor's Fireball wand (difficulty depends on circle). Accuracy/Damage enchantments are never placed.
- An identified item keeps its plain name in the inventory; the enchantment shows only in the look message.
- Map notes: click on the map to start a note, type up to 24 chars, Enter saves; clicking within 1 tile of a note erases it with no confirmation; clicks on the note text further right do not hit it; text is not clipped at the map edge. A save round-trip test exists (ui/save.rs).

Scripted pre-check (headless, HOME pointed at a scratch dir so real saves were untouched), 2026-10-22:
- Identification OK: L1 gold ring with Lore 0 -> plain 'gold ring'; Lore 2-3 (default INT 14) -> 'seems to be enchanted'; Lore 5 -> 'protection (+2)', Appraise value rises 100 -> 200. Stays identified after pickup with Lore set back to 0, in the pack, worn on the paperdoll, and after save/restore. Name Enchantment (Ort Wis Ylem) identifies carried items.
- Map notes OK: write, show, erase on click, survive save + restore; typing 'm' in a note does not close the map.
- BUG (cheap): note length limit counts bytes (text.len() < 24 in ui/overlay.rs), so Polish letters cut notes short ('Zażółć gęślą jaźń' -> 'Zażółć gęślą jaź'). Font draws Polish letters fine.
- Cosmetic: a 24-char note near the right edge runs over the compass rose (not clipped to MAP_AREA).
- Headless-only quirks, not game bugs: F-key mode changes did not register after the 'goto' debug command; hovering the options menu overrides keyboard selection.

Hand play (user), 2026-10-23:
- BUG: no mouse cursor on the automap, so a note cannot be placed. Cause: PlayState::draw (ui/play.rs) draws the automap and returns before draw_cursor. Cheap fix.

Fixed (2026-10-23): the automap now draws the cursor (plain cross, also over the view area); the note limit counts letters, not bytes. New test headless::tests::automap_cursor_and_notes (fails without the fixes). cargo test --release: 74 passed.

Hand play (user), 2026-10-23: map notes and item identification both work after the fixes. Nothing had to move to 2.0.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Played both features by hand (user) after a scripted headless pre-check.

Item identification worked as built: Lore x 6 + INT/2 against the enchantment's difficulty, remembered on the item in the world, pack, containers, paperdoll and through save/restore; Name Enchantment identifies carried items. No changes.

Map notes had two bugs, both fixed cheaply (41c83e3): the automap returned before drawing the cursor, so a note could not be placed; and the 24-letter limit counted bytes, cutting Polish notes short. New test headless::tests::automap_cursor_and_notes (fails without the fixes); cargo test --release: 74 passed. Both features stay in 1.0; nothing moved to v2.0.

Possible follow-ups (not created): long notes near the right edge overlap the compass rose; clicking near a note erases it without asking; an identified item's inventory name does not show its enchantment; only 3 enchanted items exist and Accuracy/Damage enchantments are never placed (fits TASK-3/TASK-6 loot work).
<!-- SECTION:FINAL_SUMMARY:END -->
