---
id: TASK-3
title: NPC backstories, side quests and puzzles
status: Done
assignee:
  - '@claude'
created_date: '2026-10-22T11:17:00Z'
updated_date: '2026-10-27T12:44:00Z'
labels: []
milestone: m-0
dependencies:
  - TASK-2
references:
  - assets/dialogue.txt
  - STORY.md
  - STORY.pl.md
type: feature
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The levels need more to do between the main quests. Quests come from the NPCs, so backstories and quests are written together. Conversation menus show only 5 options per node.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Each level has 2 side quests and 2 puzzles or finding tasks
- [x] #2 One collection quest spans all five levels
- [x] #3 No conversation node shows more than 5 visible choices
- [x] #4 STORY.md, STORY.pl.md and the Polish translation are in step with assets/dialogue.txt
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Design (draft for review), built on the side areas TASK-2 left for this.

Mechanics to add:
- Condition item:Kind>=N (count quest items) and action 'flag+ NAME' (count deliveries).
- Lever-combination locks: B::combo(levers, pattern, door); the door opens only when every lever matches. Clues on plaques.
- Pressure plates: B::plate(x, z, door) on the existing TriggerKind::OpenDoor.
- New quest items with icons and Polish nouns: ChroniclePage (readable, data 0-4), plus one token per side quest where no existing item fits.

Collection across all five levels: the Chronicle of the Colony. Five pages, one per level, each a chapter of the colony's history (founding, the four peoples, the breaking, Varian's descent, Malchor's rise). Elsmore pays per page and gives a ring for the full set.

L1 Upper Halls
- SQ Corwin (new NPC, old guard hall): the colony's last guard captain, who wrote the plaque at the doors (Malchor's list: 'Corwin - drunk'). The green goblins took his sword to their hoard nook; bring it back and he teaches a weapon skill and tells of the night the doors shut.
- SQ Tobin: the dead explorer in the bat roost was Wenna, his partner. Bring back her ring; he gives a lantern and talks about who he was.
- P Old guard hall armoury: two-lever combination, clue on a plaque.
- F Chronicle page 1, in the library, behind the secret room.

L2 Mines
- SQ Hilda: her son Brann jumped for the gold on the diggings ledge and broke his leg. Bring him a healing potion; Hilda rewards you.
- SQ Bodric: the corpse in the flooded drift is his apprentice Dain. Bring back Dain's token; Bodric forges you a better weapon.
- P Storeroom treasury: three-lever combination, clue in a dwarven saying.
- F Chronicle page 2, found by following the miners' marks (plaques) to a secret door in the deep mine.

L3 Warrens
- SQ Hissk: bring 3 mushrooms from the fungus cavern for the clutch-mother.
- SQ Ssethra (the lone lizardman by the aqueduct): an exile who spoke for the warm-bloods. Carry his words to Ssrath and bring back the Sage's forgiveness (dialogue and flags).
- P Southern caves: pressure plates open a sunken vault.
- F Chronicle page 3, in the ossuary.

L4 Crypts
- SQ Ysolde (the ghost in the embalmers' rooms, made friendly): bring her book of rites from the catacombs so she can lay the mass graves to rest. Reward: a rune or potions.
- SQ Aldric: find the tomb of his squire Pell in the catacombs and bring back Pell's shield.
- P Tomb of the three kings: lever combination.
- F Chronicle page 4, behind a secret door named in Ysolde's book.

L5 Sanctum
- SQ Ilsa (new NPC in the cells): Malchor's former apprentice. Bring her notebook from his study; she tells his story and gives you the soul-vault key.
- SQ Free the souls in the soul vault (break the jars). Varian thanks you at the end.
- P A hidden bridge over the lava, opened by a lever.
- F Chronicle page 5, in the cave of fire.

Steps
1. Mechanics, with tests.
2. Per level: NPCs, items, areas and dialogue in English, then Polish. Check after each level: levels_are_connected, no_dead_ends, max 5 visible choices per node, plus a scripted run.
3. A test that no node has more than 5 options visible at once.
4. Update STORY.md, STORY.pl.md, HANDOFF.md, HANDBOOK.md if needed.
5. Play by hand.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Step 1, mechanics, done (2026-10-27): item:Kind>=N condition (Player::count_item), flag+ action, combination locks (B::combo, Switch::want, Game::check_combo; the lever nearest the wall tile used is pulled), pressure plates (B::plate), ChroniclePage item (icon, Polish noun, five page texts in dialogue::CHRONICLE, read like a scroll). Tests: at_most_five_visible_choices (options with exclusive conditions are not counted together; barter adds 3), counted_items_and_added_flags, combination_lock_opens_only_when_every_lever_is_right. 81 tests pass. B::combo and B::plate are unused until the levels need them.

Level 1 done (2026-10-27): Corwin (new, old guard hall, look 2) and his captain's sword in the hoard nook (CaptainsSword; reward +1 Defense, +1 Attack); Tobin's partner Wenna, the corpse in the bat roost (EngravedRing; reward a lantern); the armoury off the north half of the guard hall (three-lever lock on the north wall, west and east pulled, plaque on the east wall); Maera's study behind a second secret door in the secret room, with chronicle page 1. Elsmore is Maera's brother and takes the pages (flag chronicle; 20 gold each, the fifth 80 gold and +2 Lore). Added flag:NAME<N. New conversations are appended at the end of dialogue.txt so saved conversation indices hold. Checked by script: Corwin's sword, Tobin's ring both ways, five pages to Elsmore, the lever lock (wrong order clicks, right opens), Polish conversation.

Level 2 done (2026-10-27): Brann (new, Hilda's son, on the diggings ledge with a broken leg; a red potion heals him and the new 'move X Z [YAW]' action sends him home to the quarters; Hilda then gives his father's chain leggings). Dain, Bodric's apprentice, is the corpse in the flooded drift (ApprenticeHammer; Bodric forges a broadsword, axe or mace). Bodric's flattery option moved into his mend node to keep start at 5 choices. Treasury under the storeroom: three levers on the west wall (north off, middle and south pulled), plaque by the door. Old vein: a secret door south of the deep mine with a miner's mark (Hilda describes the mark), Maera's camp and chronicle page 2. Checked by script: Brann and Hilda, Bodric, the treasury lock, the secret door.

Level 3 done (2026-10-27): Hissk asks for three mushrooms for the sick clutch-mother Ssiva (item:Mushroom>=3; a third mushroom added to the fungus cavern; reward +2 Swimming). Ssethra is the lizardman by the stairs down, made a named exile (kept as an npc call so the rng holds): he asks the player to speak to Ssrath, who forgives him; he then gives two green potions and +1 Sneak and moves home to the village. Ssrath's 'Who was I?' moved under his Lord of Souls node to make room. Sunken vault north of the southern caves: a portcullis opened by plates at the bottom of a flooded sinkhole east of the caves (ramp out to the aqueduct passage), riddle plaque by the door. Plates now print a message when they open a door. Ossuary: three grave plaques on the north wall; Maera's empty grave has a box with chronicle page 3. Checked by script: Hissk, Ssethra and Ssrath, the vault shut and opened.

Level 4 done (2026-10-27): Ysolde is the ghost of the embalmers' rooms, made a named friendly embalmer (kept as an npc call). Her book of rites (BookOfRites, readable, SCROLLS 13) lies in the catacombs; with it she performs the rites: the new 'rest X Z R' action crumbles the unnamed undead of the mass graves (the headless there are not undead and stay), and she gives two red potions. The book's margin names the chronicler's cell, a secret door in the north wall of the west wing, with chronicle page 4. Aldric's squire Pell lies in the mass graves corpse with his shield (SquiresShield, wearable); Aldric gives +2 Attack. Aldric's advice option hides while the shield is carried, to keep 5 choices. Tomb of the three kings south of the embalmers' rooms: a lever beside each king's plaque, raise the merciful (Orrin, Bren), leave Haldor. Checked by script: Ysolde and the mass graves, Aldric and the shield, the kings' lock.

Level 5 done (2026-10-27): Ilsa (new, Human look 4, Malchor's apprentice in the cells): her notebook (IlsaNotebook, readable, SCROLLS 14) lies in Malchor's study; for it she gives a copy of the soul-vault key (SilverKey 501) and tells Malchor's story without spoiling the reveal. The jar of souls (SoulJar) in the vault goes to Varian (new 'souls' option under his 'Who art thou?' node; heal and exp). New bridge levers: Switch::bridge, B::bridge_lever, Level::raise_bridge; the level tests check the levels with every bridge raised. Malchor's private way: the ember hall south of the cells, a river of lava with his hoard beyond (jewelled sword), the bridge raised by a lever by his cauldron (the notebook says so). Maera's body lies in the cave of fire with the last chronicle page. Found a bug that predates this task: lava never burns, because Player::drown is reset every frame when not swimming, before the lava check adds to it; lava 3 steps deep cannot be climbed out of, so it is a pit that cannot kill. Raised with the user.

Docs (2026-10-27): STORY.md and STORY.pl.md have a 'Side quests' section (the chronicle and each level's quests and puzzles); HANDOFF.md describes the new quest mechanics, the dialogue rules (new conversations at the end, five choices per node) and the test count (81). The lava bug is TASK-11. Still to do: play the side quests by hand.

Closed (2026-10-27) after TASK-11 made lava burn. Checked the level 5 side areas against the fix: Maera's body in the cave of fire lies beside the lava, not in it; the ember hall hoard needs the bridge lever (or Levitate/Fly), as designed. Hand play signed off by the user.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Each of the five levels now has two side quests and two puzzles or finding tasks, all built from the NPCs' backstories (Corwin, Tobin, Hilda and Brann, Bodric, Hissk, Ssethra, Ysolde, Aldric, Ilsa, Varian). The Chronicle of the Colony runs across all five levels: Maera's five pages go to her brother Elsmore.

New mechanics: item:Kind>=N and flag:NAME<N conditions, the flag+ action, 'move' and 'rest' dialogue actions, lever combination locks, pressure plates and bridge levers. Each has a test, and the level tests check the levels with every bridge raised.

Dialogue: new conversations are appended at the end of dialogue.txt so saved games still work. at_most_five_visible_choices keeps every node to five choices. STORY.md, STORY.pl.md and the Polish translation match the dialogue, and HANDOFF.md describes the mechanics.

Verified: all 82 tests pass, each level was checked with scripted runs, and the user signed off on playing it by hand. Found and filed the lava bug (TASK-11, now fixed). Follow-up: TASK-6 should weigh the harder lava together with the new rewards.
<!-- SECTION:FINAL_SUMMARY:END -->
