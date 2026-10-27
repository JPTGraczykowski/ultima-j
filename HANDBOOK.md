# The Abyss Below — Player's Handbook

*A guide for those who wake in the pit.*

Part One explains how the game works and has no spoilers. Part Two is a
walkthrough of the quest, with all the spoilers. Every number in this handbook
comes from the game's code. If the two ever disagree, the code is right.

**Contents**

- Part One — How to survive
  1. [The tale so far](#1-the-tale-so-far)
  2. [Making a character](#2-making-a-character)
  3. [The screen](#3-the-screen)
  4. [Controls](#4-controls)
  5. [Staying alive](#5-staying-alive)
  6. [Experience and levels](#6-experience-and-levels)
  7. [Skills](#7-skills)
  8. [Combat](#8-combat)
  9. [Magic](#9-magic)
  10. [Items](#10-items)
  11. [Talking and trading](#11-talking-and-trading)
  12. [Bestiary](#12-bestiary)
- Part Two — The quest (spoilers)
  13. [Walkthrough](#13-walkthrough)
  14. [Where the runes are](#14-where-the-runes-are)

---

# Part One — How to survive

## 1. The tale so far

Long ago Lord Varian founded a colony deep in the great Abyss, where men,
mountain folk, goblins and lizardmen lived as one, bound together by the light
of the **Lumen Stone**. Then the Stone was broken into three shards (Valor,
Wisdom and Mercy), and the colony fell into ruin and war. Varian went down to
the **Throne of Souls** to keep the dead of the Abyss at rest, and became the
**Lord of Souls**.

Now his strength is failing, and the dead rise from their graves, hollow and
mindless. You are one of them: you claw your way out of a shallow grave by the
sealed great doors with no memory of who you were. Unlike the others you can
still think, and a voice from far below is calling you. Find out who you were,
how to live again, and how to leave the Abyss.

## 2. Making a character

From the title screen choose **Create Character** (or **Introduction** to see
the story first; **Journey Onward** loads a saved game). You pick your sex,
profession, appearance and name. Each choice re-rolls your
attributes, and you can say *No* at the end to start over.

### Attributes

| Attribute | What it does |
| --- | --- |
| **Strength (ST)** | Melee damage (+1 for every 5 points above 10), carrying capacity (ST × 2.5 stones), vitality at creation and on each level-up |
| **Dexterity (DEX)** | Hitting and dodging (+1% per 2 points), picking locks, repairing gear, disarming traps |
| **Intelligence (INT)** | Starting mana, mana on each level-up, casting success, Lore, Search, spotting traps |
| **Vitality** | Your hit points. At 0 you die. |
| **Mana** | Spent to cast spells |

Each attribute starts at 14, plus the profession bonus below, plus a random
−2 to +3. Vitality starts at 20 + ST/2 + 0–5. Mana starts at INT/3; Mages and
Druids get 6 more.

### Professions

Each profession starts with 4 points in each of its skills. Everyone also gets
+2 Attack and +2 Defense on top of that.

| Profession | ST / DEX / INT bonus | Skills | Starts with |
| --- | --- | --- | --- |
| **Fighter** | +6 / +2 / −2 | Attack, Defense, Sword, Axe, Mace, Unarmed | Dagger, leather vest |
| **Mage** | −3 / +1 / +7 | Mana, Casting, Lore, Appraise | Runes IN, LOR, ORT, JUX; a candle |
| **Bard** | 0 / +3 / +3 | Lore, Charm, Appraise, Sword, Casting | Dagger, 20 gold |
| **Tinker** | +2 / +4 / +1 | Repair, Picklock, Traps, Mace, Appraise | Cudgel, lockpick |
| **Druid** | 0 / +1 / +5 | Mana, Casting, Lore, Track, Swimming | Runes IN, LOR, ORT, JUX; a candle |
| **Paladin** | +4 / +1 / +1 | Attack, Defense, Sword, Mace, Charm | Dagger, leather vest |
| **Ranger** | +2 / +5 / 0 | Missile, Track, Sneak, Search, Sword | Sling, 12 sling stones, leather boots |
| **Shepherd** | +2 / +2 / +2 | Unarmed, Missile, Track, Swimming, Search | Cudgel, a loaf of bread |

Everyone also starts with an empty **rune bag** and an apple.

**Choosing a profession.** Fighters and Paladins have the easiest start. Mages
and Druids can cast *Light* and *Magic Arrow* from the first minute but are
fragile. Tinkers get past locked doors and traps. Bards talk their way past
Rhugg and pay less at the traders. Rangers fight from a distance.

## 3. The screen

The screen copies the original game's layout:

- **The view** (centre): the dungeon in 3D. The **gargoyle** above it watches
  your current foe, and its eyes show how hurt that foe is.
- **Command icons** (left): Options, Talk, Get, Look, Fight, Use.
- **Compass** (bottom centre): the direction you face. Click it for a status
  report: health, hunger, fatigue, load, the day, and which level you are on.
- **Power gem** (in the compass): fills while you wind up an attack.
- **Flasks**: red is vitality, blue is mana. Click one to read the exact value.
- **Rune shelf** (bottom right): the runes you have lined up to cast.
- **Character panel** (right): your paperdoll and pack. The **pull chain**
  turns it into the statistics page. Click the rune bag to open your runes.
- **Message scroll** (bottom): what you see, hear and do.

## 4. Controls

| Action | Keys / mouse |
| --- | --- |
| Run / walk forward | `W` / `S` (or `Up`) |
| Back, turn, sidestep | `X`, `A`/`D` (or `Left`/`Right`), `Z`/`C` |
| Mouse movement | Hold the left button in the view. The arrow shows the direction; the further from the centre, the faster you go. |
| Jump | `J` or `Space` (or the right button while moving) |
| Look down / centre / up | `1` / `2` / `3` |
| Fly up / down (while flying) | `E` / `Q` |
| Talk, Get, Look, Fight, Use | Click an icon (or `F2`–`F6`), then right-click in the view |
| Default mode | Right-click to look. Hold or drag the right button to get, use or talk. |
| Attack | In Fight mode, hold the right button, watch the power gem, then release. Aim high to bash, at the middle to slash, low to thrust (or press `P`, `;`, `.`). |
| Inventory | Left-click to pick up or put down. Right-click to use: eat, drink, read, light, open, or repair worn gear. |
| Statistics | Click the pull chain (or `F7`) |
| Runes and spells | Left-click the rune bag, click runes onto the shelf, then click the shelf (or `F8`) to cast. Right-click the shelf to clear it. |
| Map | `M` or `Tab` |
| Track / sleep | `F9` / `F10` |
| Options, save, restore | `F1` / `Esc`; `Ctrl+S` to quick-save, `Ctrl+R` to restore |
| Help | `H` |
| Fullscreen / aspect | `F11` / `F12` |

**Modern controls.** Choose *Controls* in the options menu (`F1` / `Esc`) to
switch to a modern layout (the game remembers your choice). Anything not
listed here works as above, and the `F` keys still work.

| Action | Keys / mouse |
| --- | --- |
| Forward / back, turn | `W`/`S` or `Up`/`Down` (hold `Shift` to walk), `A`/`D` or `Left`/`Right` |
| Sidestep | `Q` / `E` |
| Look up / centre / down | `Z` / `X` / `C` |
| Fly up / down (while flying) | `R` / `T` |
| Options, Talk, Get, Look, Fight, Use | `1`–`6` (or click an icon), then right-click in the view. |
| Statistics / cast | `7` / `8` |
| Attack | `V`, `B`, `N` (bash, slash, thrust), or hold the right button in Fight mode |
| Track / sleep | `9` / `0` |

**Doors, levers and chests.** Choose *Use* and right-click them. To unlock a
door, carry its key in your pack. To pick a lock, take a lockpick on the
cursor and use it on the door.

**The map** fills in as you explore. You can write your own notes on it.
Traps you have spotted are marked on it.

## 5. Staying alive

### Vitality and healing

- **Natural healing**: 1 vitality every 14 seconds while you are fed, and
  every 30 seconds while you are hungry. You don't heal at all while poisoned.
- **Sleep** (`F10`, a bedroll from your pack, or a bed) restores 90% of your
  missing vitality and mana. Only 40% if you go to sleep hungry. Eight minutes
  pass while you sleep and you get hungrier. You can't sleep with an enemy in
  sight nearby, or when you are neither tired nor hurt.
- **Dreams**: when you sleep, a spirit gives you a hint about what to do next.
- **Shrines** heal 10 and remind you to spend unused skill points.
- **Fountains** heal 5 and take the edge off your hunger.
- **Red potions** heal 12–23. **Healing spells** are covered under Magic.

### Hunger

Hunger drops by 1 point every 20 seconds, so a full stomach lasts about half
an hour. The game tells you when you get *hungry* (below 25) and *starving*
(below 8). At 0 you lose 1 vitality every 15 seconds. You can't eat above 92.

### Fatigue

You tire by 1 point every 36 seconds. Past 90 you move 20% slower. Sleep
clears it, and a drink of water helps a little.

### Poison

Spiders, ghouls and poison bolts can poison you. You lose 1 vitality every 4
seconds until it wears off, and you don't heal meanwhile. A **green potion** or
**Cure Poison** (AN NOX) ends it at once.

### Light

The Abyss is dark. Without a light you see about 3 tiles. A light only shines
while it is in a hand or on a shoulder. Lighting one from your pack puts it on
a free shoulder for you.

| Light | Radius | Burns for |
| --- | --- | --- |
| None | 3 | — |
| Candle | 3.5 | 7½ minutes |
| Torch | 5 | 10 minutes |
| Lantern | 6.5 | 25 minutes |
| *Light* spell (IN LOR) | 7 | 4 minutes |
| *Night Vision* spell or yellow potion | 9 | 4 minutes |
| *Daylight* spell (VAS IN LOR) | 16 | 5 minutes |

Put out a light you don't need (use it again) to save fuel.

### Carrying

You can carry ST × 2.5 stones. Above that you move at less than half speed.
Coins weigh nothing. Bags and packs hold 8 items each and count towards your
load. Your pack has 8 slots, and anything that doesn't fit goes into a
container in your pack.

### Water, lava and falling

- **Swimming** is slower than walking. If your load is too heavy, you start to
  drown and lose 2 vitality every few seconds. The Swimming skill helps. You
  can pull yourself out onto a bank a little higher than a step.
- **Jumping**: run at a gap and jump at its edge. A running jump clears a gap
  one tile wide, and two tiles if you land lower down. A jump from a walk
  falls short. Every pit has a way back up, so a missed jump costs a fall.
- **Lava** burns for 4 every half-second. *Resist Fire* cuts that to 1.
  *Levitate* or *Fly* lets you cross it safely. Some lava is too deep to
  climb out of, so never step into a pool you can't see the bottom of.
- **Falling** more than about a body's height hurts. You can land safely in
  water, and with the *Slow Fall* spell.

### Traps

Pressure plates are hidden in the floor, 13 of them across the five levels.
When you come close you may spot one; Traps skill and INT make that more
likely. You can also look at a floor tile for a better chance. Once you've
spotted a trap, it is marked on the map, and you can *Use* its tile to try to
disarm it. Sometimes a disarm attempt sets the trap off.

### Death and the silver sapling

When you die, you can restore your last saved game. Somewhere in the Abyss a
**silver sapling** grows. Drop it on the ground to plant it, and if you die
you come back to life at that spot with full vitality.

**Save often.** `Ctrl+S` quick-saves, and the options menu (`F1`) has more
save slots. Saves are kept in `~/.ultima-j/`.

## 6. Experience and levels

You gain experience by killing creatures, finishing quests, disarming traps
and learning things in conversation.

| Level | Experience | | Level | Experience |
| --- | --- | --- | --- | --- |
| 1 | 0 | | 9 | 2,400 |
| 2 | 50 | | 10 | 3,200 |
| 3 | 140 | | 11 | 4,200 |
| 4 | 300 | | 12 | 5,400 |
| 5 | 600 | | 13 | 7,000 |
| 6 | 900 | | 14 | 9,000 |
| 7 | 1,300 | | 15 | 11,500 |
| 8 | 1,800 | | 16 | 14,500 |

An adventurer who explores everything usually reaches level 3 in the Upper
Halls, level 5 or 6 in the Flooded Warrens, and level 8 in the Crypts.

Each level gives you:

- **+3 + ST/6** maximum vitality
- **+1 + INT/6 + Mana skill/4** maximum mana
- **3 skill points**

To spend skill points, open the statistics page (pull chain or `F7`) and
click a skill. The page shows how many points you have left. A skill can go
up to 30.

Your level also limits which spells you can cast: you can cast a spell of any
circle up to your level. See [Magic](#9-magic).

## 7. Skills

| Skill | Effect |
| --- | --- |
| **Attack** | +3% to hit with every weapon |
| **Defense** | −3% chance for creatures to hit you |
| **Unarmed** | +3% to hit with your fists |
| **Sword** | +3% to hit with daggers and swords |
| **Axe** | +3% to hit with axes |
| **Mace** | +3% to hit with cudgels and maces |
| **Missile** | +1 damage per 2 points with a sling or crossbow |
| **Mana** | Faster mana regeneration. Each point spent also gives +1 maximum mana, and you gain more mana on each level-up. |
| **Lore** | Names an item's enchantment when you look at it, if Lore × 6 + INT/2 is high enough. Stronger enchantments need more. |
| **Casting** | +7% chance to cast a spell, and more damage from attack spells |
| **Traps** | Better chance to spot traps (5% + 7% per point + INT/4) and to disarm them (20% + 8% per point + DEX/2) |
| **Search** | Finds secret doors when you *Use* a suspicious wall: 30% + 6% per point + INT |
| **Track** | `F9` finds the nearest unfriendly creature and tells you its direction and distance. Range is 6 + 1.5 tiles per point; success is 30% + 8% per point. |
| **Sneak** | Creatures that haven't seen you yet notice you 4% closer per point (down to 45% of the normal range) |
| **Repair** | Mends worn weapons and armour when you *use* them from the inventory. Success is 30% + 7% per point + DEX/2. The skill limits how good you can make an item: up to 30% + 7% per point. Bad slips can damage the item. |
| **Charm** | Lowers what you pay (2% per point), raises what you're paid, and opens friendlier lines in conversation |
| **Picklock** | Opens locked doors with a lockpick: 20% + 6% per point + DEX |
| **Acrobat** | Higher jumps (up to +15%), longer safe falls, and less damage when you land badly |
| **Appraise** | Tells you roughly what an item is worth (closer with more skill). Lowers buying prices 1% per point and raises selling prices 3% per point. |
| **Swimming** | Faster swimming (up to 85% of walking speed), your head stays higher, and you can carry more before you start to drown |

Charm and Appraise together can lower prices by up to 35% and raise selling
prices by up to 50%.

## 8. Combat

Choose **Fight** (`F5`), aim the cursor at your foe, and hold the right mouse
button. The **power gem** fills. The fuller it is when you let go, the harder
you hit. A swing at full power has a 25% chance to be a **critical hit** that
does half again as much damage.

Where you aim decides how you attack:

| Aim | Attack | Key |
| --- | --- | --- |
| High | Bash | `P` |
| Middle | Slash | `;` |
| Low | Thrust | `.` |

Each weapon does different damage with each attack. Use its best one.

**Chance to hit** = 55% + 3% × (Attack + weapon skill) + DEX/2 − 3% × the foe's
defense + up to 10% for power, kept between 10% and 95%.

**Damage** = (weapon damage + (ST − 10)/5) × (0.4 + 0.9 × power) + 0–2, plus
any damage enchantment. Badly worn weapons (below 30% condition) do a third
less damage.

**Being hit.** A creature hits you with a chance of 45% + 4% × its attack −
(3% × Defense + 3% × armour + DEX/2), kept between 8% and 92%. Your armour
also takes armour/3 off every blow.

**Wear.** Weapons and armour wear down slowly as you fight. Armour below 20%
condition protects only half as well. Repair it yourself, or pay Bodric the
smith to do it.

**Missile weapons.** Slings fire sling stones and crossbows fire bolts, both
from your pack. Shots that miss fall to the floor, so you can pick them up
again.

**Running away.** Most creatures flee when nearly dead. The undead, golems
and the Lord of Souls never do.

### Weapons

Damage is listed as bash / slash / thrust.

| Weapon | Skill | Damage | Speed | Weight (stones) | Value |
| --- | --- | --- | --- | --- | --- |
| Dagger | Sword | 2 / 3 / 4 | fast | 0.5 | 10 |
| Shortsword | Sword | 3 / 5 / 5 | fast | 0.9 | 25 |
| Longsword | Sword | 4 / 7 / 6 | normal | 1.2 | 50 |
| Broadsword | Sword | 5 / 8 / 6 | slow | 1.5 | 80 |
| Jeweled sword | Sword | 7 / 10 / 9 | fast | 1.2 | 300 |
| Hand axe | Axe | 4 / 5 / 2 | fast | 0.9 | 20 |
| Axe | Axe | 6 / 8 / 2 | slow | 1.6 | 45 |
| Cudgel | Mace | 4 / 2 / 2 | fast | 0.8 | 5 |
| Mace | Mace | 7 / 4 / 3 | slow | 1.4 | 40 |
| Hammer of the Mountain | Mace | 9 / 5 / 3 | slow | 2.0 | 500 |
| Fists | Unarmed | 2 / 1 / 2 | — | — | — |
| Sling | Missile | 4 | normal | 0.2 | 8 |
| Crossbow | Missile | 8 | slow | 1.8 | 80 |

### Armour

| Slot | Item | Protection | Weight | Value |
| --- | --- | --- | --- | --- |
| Head | Leather cap / chain cowl / plate helm | 1 / 2 / 3 | 0.5 / 1.0 / 1.5 | 10 / 40 / 90 |
| Body | Leather vest / chain shirt / breastplate | 2 / 4 / 6 | 1.0 / 3.0 / 4.5 | 20 / 100 / 250 |
| Legs | Leather / chain / plate leggings | 1 / 2 / 3 | 0.8 / 2.0 / 3.0 | 15 / 60 / 150 |
| Feet | Leather boots / plate boots | 1 / 2 | 0.6 / 1.5 | 15 / 80 |
| Hands | Leather gloves / gauntlets | 1 / 2 | 0.3 / 0.8 | 10 / 60 |
| Shield | Buckler / wooden shield / tower shield | 2 / 3 / 5 | 0.8 / 1.4 / 3.0 | 20 / 35 / 120 |
| Rings | Gold ring / silver ring | 0 (+ enchantment) | 0.1 | 100 / 60 |

Rings protect only if they are enchanted with Protection.

## 9. Magic

### How to cast

1. **Find rune stones.** Use one from your pack and it goes into your rune
   bag for good. There are 24 runes.
2. **Open the rune bag** (left-click it in the panel) and click up to three
   runes to put them on the **rune shelf**. The order matters.
3. **Cast** by clicking the shelf or pressing `F8`. Attack spells fly towards
   the cursor. Right-click the shelf to clear it.

The runes stay on the shelf, so you can cast the same spell again and again.

### Rules

- **Mana cost** = 3 × the spell's circle.
- **Level needed**: you can cast a spell whose circle is no higher than your
  experience level (circle 1 at level 1, circle 2 at level 2, … circle 8 at
  level 8).
- **Chance to succeed** = 45% + 7% × Casting + INT − 10% × circle, kept between
  10% and 98%. A spell that fizzles still costs half its mana.
- Mana comes back by itself: 1 point every 3 to 7 seconds, faster with a
  higher Mana skill. Sleep and blue potions (+12) restore more.

### The 24 runes

| Rune | Meaning | | Rune | Meaning | | Rune | Meaning |
| --- | --- | --- | --- | --- | --- | --- | --- |
| An | Negate | | In | Create | | Quas | Illusion |
| Bet | Small | | Jux | Danger | | Rel | Change |
| Corp | Death | | Kal | Summon | | Sanct | Protect |
| Des | Down | | Lor | Light | | Tym | Time |
| Ex | Freedom | | Mani | Life | | Uus | Raise |
| Flam | Flame | | Nox | Poison | | Vas | Great |
| Grav | Energy | | Ort | Magic | | Wis | Knowledge |
| Hur | Wind | | Por | Movement | | Ylem | Matter |

### The 40 spells

Durations are in minutes. "Area" means every creature within a few paces.

**First circle** (3 mana, level 1)

| Spell | Runes | Effect |
| --- | --- | --- |
| Create Food | In Mani Ylem | A loaf of bread or an apple appears |
| Light | In Lor | Light radius 7 for 4 min |
| Magic Arrow | Ort Jux | A bolt of force: 4 + Casting/2 damage |
| Resist Blows | Bet In Sanct | Creatures are 15% less likely to hit you, for 2 min |
| Stealth | Sanct Hur | Creatures notice you at 40% of the usual range, for 2 min |

**Second circle** (6 mana, level 2)

| Spell | Runes | Effect |
| --- | --- | --- |
| Cause Fear | Quas Corp | Hostile creatures nearby flee |
| Detect Monster | Wis Mani | Counts the hostile creatures within 12 tiles |
| Lesser Heal | In Bet Mani | Heals 6–11 |
| Rune of Warding | In Jux | Lays a glowing rune on the floor where you stand. The next hostile creature to cross it takes 3 + Casting/2 damage and can't move for 5 + Casting/3 seconds. |
| Slow Fall | Rel Des Por | Fall slowly and take no falling damage, for 1 min |

**Third circle** (9 mana, level 3)

| Spell | Runes | Effect |
| --- | --- | --- |
| Conceal | Bet Sanct Lor | A stronger Stealth: creatures see you only from 20% of the usual range, for 1½ min |
| Lightning | Ort Grav | A fast bolt: 8 + Casting/2 damage |
| Night Vision | Quas Lor | See in the dark (radius 9) for 4 min |
| Speed | Rel Tym Por | Move 60% faster for 1 min |
| Strengthen Door | Sanct Jux | Seals the nearest door shut |

**Fourth circle** (12 mana, level 4)

| Spell | Runes | Effect |
| --- | --- | --- |
| Heal | In Mani | Heals 14–23 |
| Levitate | Hur Por | Float up to a little above the floor for 1 min, which is enough to cross lava and water |
| Poison | Nox Mani | A poison bolt: 3 damage, then the target stays poisoned |
| Remove Trap | An Jux | Disarms traps within 3 tiles |
| Resist Fire | Sanct Flam | Lava hurts much less, for 2 min |

**Fifth circle** (15 mana, level 5)

| Spell | Runes | Effect |
| --- | --- | --- |
| Cure Poison | An Nox | Ends poisoning |
| Fireball | Por Flam | An exploding ball: 14 + Casting damage, and half that to others nearby. Don't stand too close! |
| Missile Protection | Grav Sanct Por | Missiles and magic bolts do a third of their damage to you, for 2 min |
| Name Enchantment | Ort Wis Ylem | Identifies every enchanted item you carry |
| Open | Ex Ylem | Opens the nearest door, even a locked one |

**Sixth circle** (18 mana, level 6)

| Spell | Runes | Effect |
| --- | --- | --- |
| Daylight | Vas In Lor | Light radius 16 for 5 min |
| Gate Travel | Vas Rel Por | Takes you back to where you arrived on this level |
| Greater Heal | Vas In Mani | Heals you fully |
| Paralyze | An Ex Por | The creature you aim at can't move for 10 seconds |
| Telekinesis | Ort Por Ylem | Pick things up from a distance, for 1 min |

**Seventh circle** (21 mana, level 7)

| Spell | Runes | Effect |
| --- | --- | --- |
| Ally | In Mani Rel | The creature you aim at becomes your friend and follows you |
| Confusion | Vas An Wis | Hostile creatures nearby stop fighting you |
| Fly | Vas Hur Por | Fly freely (`E` up, `Q` down) for 1 min |
| Invisibility | Vas Sanct Lor | Creatures can't see you, for 1½ min |
| Reveal | Ort An Quas | Shows every secret door on the level |

**Eighth circle** (24 mana, level 8)

| Spell | Runes | Effect |
| --- | --- | --- |
| Flame Wind | Flam Hur | 18 + Casting damage to everything around you |
| Freeze Time | An Tym | Every creature stops for 15 seconds |
| Iron Flesh | In Vas Sanct | Halves all damage you take, for 2 min |
| Roaming Sight | Ort Por Wis | Shows the whole level on your map |
| Tremor | Vas Por Ylem | 12 + Casting damage to everything around you |

**Wands** hold a spell and a number of charges. *Use* one to cast its spell
without spending mana or needing a level.

## 10. Items

### Food and drink

| Food | Nourishment | | Food | Nourishment |
| --- | --- | --- | --- | --- |
| Mushroom | 10 | | Cheese | 35 |
| Apple | 20 | | Bread | 40 |
| Flask of water | 8 (and eases fatigue) | | Fish | 45 |
| Bottle of wine | 10 (and heals 2) | | Meat | 55 |

### Potions

| Potion | Effect |
| --- | --- |
| Red | Heals 12–23 |
| Blue | Restores 12 mana |
| Green | Cures poison |
| Yellow | Night vision for 4 minutes |

### Containers

Bags and packs hold 8 items; small boxes hold 6. Chests and corpses can be
opened where they lie: *Use* them and their contents spill onto the floor.

### Treasure

Gems and gold nuggets are worth money to traders. Rubies are worth 120,
sapphires 100, emeralds 110, diamonds 200 and gold nuggets 40. Gold coins
stack in your pack and weigh nothing.

### Enchantments

Some items are enchanted. They can carry **Accuracy** (+5% to hit per
point), **Damage** (+1 per point), or **Protection** (+1 armour per point),
or hold a spell. An enchantment stays hidden until Lore or *Name Enchantment*
identifies it. You still benefit from it before then.

## 11. Talking and trading

Choose **Talk** (`F2`) and right-click a person. Pick your answers from the
list. Some answers only show up if you carry the right item, have learned
something, or are skilled enough in Charm.

When you *Look* at a creature you see its mood:

- **Friendly** creatures won't harm you.
- **Mellow** ones leave you alone.
- **Upset** ones may turn on you.
- **Hostile** ones attack on sight.

Striking anyone makes them hostile, even if you miss.

### Bartering

Traders show their goods in the top left of the barter screen. To trade:

1. Click the goods you want.
2. Put gold or items from your inventory into your trade area (top right).
3. Choose **"Let us trade"**.

Right-click anything on either side to ask what it is worth. Change comes back
in gold, and anything you leave on the table returns to you when you finish
talking. Traders pay half price for what you sell them, and never more than
they would ask for it themselves.

## 12. Bestiary

"Hits" is vitality. "Exp" is the experience you get for a kill.

| Creature | Hits | Damage | Exp | Notes |
| --- | --- | --- | --- | --- |
| Giant rat | 6 | 1–3 | 4 | Quick |
| Cave bat | 5 | 1–3 | 5 | Flies, very quick |
| Acid slug | 12 | 2–5 | 12 | Very slow |
| Green goblin | 14 | 2–6 | 15 | |
| Gray goblin | 16 | 2–7 | 18 | |
| Giant spider | 16 | 2–6 | 22 | Quick, **poisonous** |
| Skeleton | 18 | 2–8 | 25 | Undead |
| Imp | 18 | 2–6 | 50 | Flies, **throws fire** |
| Mage | 20 | 2–6 | 45 | **Casts magic bolts** |
| Lizardman | 22 | 3–8 | 30 | Swims |
| Outcast (human) | 24 | 2–8 | 30 | Usually peaceful |
| Ghoul | 28 | 3–9 | 50 | Undead, **poisonous** |
| Mountainman (dwarf) | 30 | 3–9 | 40 | Usually peaceful |
| Ghost | 30 | 3–10 | 90 | Undead, flies, hard to hit |
| Headless | 40 | 4–12 | 80 | |
| Lurker | 45 | 4–12 | 90 | Lives in deep water |
| Troll | 60 | 5–14 | 110 | |
| Reaper | 70 | 5–15 | 150 | Slow |
| Earth golem | 90 | 6–16 | 160 | Slow, very hard to hit, never flees |
| Lord of souls | 220 | 8–22 | 1000 | **Throws fire** once angered. See Part Two. |

The undead feel no fear and leave bones when destroyed.

---

# Part Two — The quest (spoilers)

> Everything below gives the game away. Stop here if you'd rather find your
> own way.

## 13. Walkthrough

To win, you need the three shards of the Lumen Stone, one from each of the
three peoples. Carry them through the Shard Gate on the last level to the Lord
of Souls and choose an ending: give him the shards and he gives you back your
life and opens the great doors, or strike him down and try to take his throne.

The shards are what open the way: the gate to the Throne of Souls answers
only to the light of the Lumen Stone, and the three shards are all that is
left of it. That is also why someone woke you. The voice in your grave and in
your dreams says it is the Lord of Souls, but it belongs to **Malchor**, who
broke the Stone. The gate burns him and the peoples would never give him
their shards, so he raised one of Varian's own knights to fetch them. Read
his notes on level 5 and you can see it coming.

### Level 1 — The Upper Halls

You begin in the entrance hall. The great doors behind you won't open. On the
floor are a **torch**, a **map**, a loaf of **bread**, and a scroll that
explains the IN LOR and ORT JUX spells. Light the torch.

- **The outcasts' camp** (west): **Elsmore** tells you about the colony, the
  shards and the Lord of Souls, and how to go down. Ask him for food and he
  gives you bread. **Tobin** sells torches, bread, a red potion, a lantern, a lockpick and
  a candle. With Charm 4 or more he gives you a candle and lower prices. There
  is a **bedroll** here.
- **The shrine** (south of the corridor): heals you and reminds you about
  skill points.
- **The green goblin den** (behind the door west of the camp): two goblins and
  their chief **Gnarl**. Gnarl carries **Gnarl's medallion** and the **iron key
  to the library**. There's also a cudgel, a leather vest and 12 gold.
- **The goblins' midden** (south of the den): two rats and a slug. A crevice
  to the south guards the goblins' hoard: a box with 15 gold and a green
  potion, a sling with stones, and some meat. If you fall in, a ramp climbs
  back to the midden.
- **The library** (locked door north of the camp; the key comes from Gnarl).
  Watch out: there is a trap just inside. Inside are a longsword, a red potion,
  and a chest with 40 gold, the **ORT** and **JUX** runes, and a scroll. The
  **east wall is a secret door**. Use it until you find it. Behind it are an
  enchanted **gold ring of Protection +2**, a blue potion and the **BET** rune.
- **The old guard hall** (north of the library): a crack splits the hall, so
  take a running jump. A ramp at the west end climbs out if you fall. Two
  rats; across the crack, a chest with a buckler, 18 gold and an apple.
- **The rat cave** (east): three rats and a dagger. The slug tunnel to the
  south-east has a trap, an acid slug, and a box with the **MANI** rune and
  a candle.
- **The water cave** (north): bats, a fish, and the **IN** and **LOR** runes.
  East of it is a bat roost with three bats and a dead explorer carrying a
  leather cap, a rope and 8 gold.
- **The gray goblin hall** (far north-east): chief **Rhugg** guards the
  portcullis to the stairs. He opens it if you:
  - bring him **Gnarl's medallion**. He also gives you 30 gold and 60 exp.
    This is the best deal.
  - pay **50 gold**, or **25** with Charm 4 or more.

### Level 2 — The Mines of the Mountain Folk

- **King Thorgrim** (the great hall): he won't give you the Shard of Valor
  until you prove yourself. Offer your help and he gives you the **key to the
  eastern galleries**.
- **Hilda** (living quarters, west) gives you cheese and bread.
- **The storeroom** (behind a door south of the arrival tunnel): three rats
  among the barrels, and food and drink.
- **The old diggings** (north of the quarters): two slugs. Thorgrim closed
  them when the east face fell into a shaft. Leap from the dig face onto the
  lower ledge beyond for a box with two gold nuggets and a red gem, and a
  lantern. To get back, drop into the shaft and climb its ramp.
- **Bodric the smith** (forge, south): sells a chain shirt, gauntlets, a
  broadsword, a wooden shield and torches. He buys gold nuggets for 30 each,
  and repairs all your gear for 15 gold. With Charm 3 or more he repairs it
  once for free. The **FLAM** rune lies on his table, and there's a mace and a
  chain cowl nearby.
- **The eastern galleries** (iron door east of the hall): two traps, four
  giant spiders, and **the spider queen** (34 hits). Bring a green potion or
  plenty of vitality, because their bite is poisonous. Deep inside are the
  **Hammer of the Mountain**, the **HUR** and **SANCT** runes, two gold
  nuggets, and a dead adventurer carrying a **chain shirt, a lantern and a
  green potion**. South of them, a flooded drift holds two more spiders, a
  gold nugget, and a dead miner with a green potion, a lockpick and a candle.
- Bring the Hammer back to Thorgrim and he gives you the **Shard of Valor**
  and 200 exp. (You must give the Hammer back, so use it while you have it.)
- **The way down** is through the deep mine south-west of the hall. There's a
  trap at its entrance, and past the forge the tunnel floor has fallen into a
  shaft: take a running jump (if you fall, a ramp climbs to the forge). Then
  comes a lava fissure to cross by the bridge, two slugs, a gold
  nugget, a shovel, and the **POR** rune, which you need for Fireball, Levitate
  and Fly.

### Level 3 — The Flooded Warrens

- **Hissk** (the lizardman village, east of the great flooded hall) sends you
  to the Sage.
- **Ssrath the Sage** (the pool of thinking, north of the village): ask him
  to teach you magic and he gives you the **WIS** rune. Ask about the shard
  and he tells you that lurkers have stolen the clutch of eggs. Ask about the
  Lord of Souls to learn what Malchor wants, and about your past to be sent
  to Sir Aldric.
- **The lurkers' grotto** (deep water, far east): two lurkers guard the
  **clutch of lizard eggs**. The **WIS** rune is also here, in case you didn't
  get it from Ssrath. Don't swim with a heavy pack.
- **The fungus cavern** (south of the village): a lurker in the pool, two
  bats, mushrooms, and a corpse with leather leggings, a torch and 12 gold.
- Give the eggs to Ssrath for the **Shard of Wisdom** and 250 exp.
- **The drowned temple** (south-west): there is a trap at the entrance, and
  two ghouls inside. You'll find a shrine, the **NOX** and **AN** runes, a
  yellow potion and a scroll. A **lever on the west wall** opens a hidden
  reliquary to the east, which holds a **silver ring of Protection +3**, a
  blue potion and the **QUAS** rune. Below the temple, the ossuary holds a
  ghoul, a bat, and a chest with a blue potion and 25 gold.
- **The southern caves**: the **GRAV**, **REL** and **TYM** runes, a trap, a
  ghoul, a bat, a lone lizardman and a red potion. A span of the old
  aqueduct to the stairs has fallen: jump the gap, or swim the water below
  to the ramp that climbs back. The stairs down are in the far south.

### Level 4 — The Crypts

- **The gallery of tombs**: two skeletons and the **CORP** rune. West of it
  are the catacombs (two skeletons, a chest with 30 gold and a red potion).
  East of it is the charnel pit (a skeleton and a ghost): leap the pit onto
  the ledge beyond for a chest with a white gem, 40 gold and a yellow potion.
  A ramp climbs out of the pit.
- **Sir Aldric's tomb** (centre; there is a trap on the way in): the
  ghost of **Sir Aldric** is friendly. Trolls have stolen his bones. Ask him
  for advice and he tells you about the silver sapling. Ask whether he knows
  you and he tells you who you were (100 exp): a Knight of the Lumen, killed
  shielding Lord Varian when Malchor broke the Stone. Tell him a voice woke
  you and he warns you about Malchor's silver tongue.
- **The west wing** (a trap at the entrance): a ghost and two headless. Here
  you'll find the **DES** and **EX** runes, a chest with a **plate helm, 60
  gold and a blue potion**, and the **silver sapling**. Plant it somewhere safe
  before the hard fights. South of the wing are the mass graves: two
  skeletons, a headless, and a corpse with plate boots and 20 gold.
- **The troll den** (east wing, a trap at the entrance): two trolls guard
  the **bundle of ancient bones**. There's also a **tower shield**, some meat,
  and the **VAS** rune.
- Give Aldric his bones for the **Shard of Mercy**, 300 exp and a full heal.
- **The guardian's hall** (south, a trap on the way): the floor of the
  passage has fallen in, so take a running jump (a ramp from the bottom leads
  back to Aldric's tomb). A slow **earth golem** (90 hits) waits among the
  pillars. Here are the **YLEM**, **KAL** and **UUS** runes, **plate
  leggings**, and the stairs down. East of the hall are the embalmers' rooms:
  a ghost, wine and a blue potion.

### Level 5 — The Deep Sanctum

Malchor's **thralls**, hollow undead that walk on his power, guard the level.
There are seven: two skeletons and a ghoul in the antechamber, a headless in
the corridor to the study, a skeleton and a ghost in the study, and a ghoul
outside the soul vault. **When Malchor dies, every thrall still standing
crumbles to dust.** Malchor himself is nowhere to be seen until you open the
Shard Gate.

- **The antechamber**: three thralls, lava pools, and a red potion.
- **Malchor's study** (west, a trap and a headless on the way): two thralls,
  and Malchor's notes on the table, which give his plan away. South of it are
  the cells: two imps, a dead prisoner with leather gloves, and a chest with
  40 gold and a yellow potion.
- **The cave of fire** (south-east of the antechamber): a **reaper**, an imp,
  a blue gem, a gold nugget and a red potion.
- **Malchor's soul vault** (east, a trap and a ghoul on the way, then an iron
  door that only Malchor's silver key opens): a chest with two red potions and
  50 gold, and Malchor's list of the souls he meant to take.
- **The Shard Gate** (south) opens only if you carry all three shards; the
  Open spell won't work on it. As it opens, **Malchor** steps out behind you.
  He admits that the voice was his and demands the shards. No matter what
  you say, he fights you (70 hits, magic bolts). He carries the **silver
  key** to his soul vault, a **wand of Fireball** with 4 charges, and a blue
  potion. Heal at the shrine behind the gate before you face him or Varian.
- **The Throne of Souls**: just past the shrine, a crack with lava at the
  bottom splits the cave. Take a running jump; the way out of the crack climbs
  east, back to the gate. **Varian, the Lord of Souls**, waits among the
  lava pools. He is peaceful and tells you who you were, and that he never
  called you. Ask him to make you
  live again to choose your ending:
  - **Live again.** Give him the shards and he makes you alive again and
    opens the great doors. You climb out into daylight.
  - **Take his throne.** He fights you (220 hits, throws fire). **Your blows
    pass through him unless you carry all three shards.** Use red potions and
    the Fireball wand, and fight him away from the lava. When he dies, the
    game ends and the rest of the story is told in pictures: the throne gives
    you nothing, the great doors open, and the peoples of the Abyss go to war
    over the way out.
  If you attack him before choosing, you have chosen the throne.

## 14. Where the runes are

| Rune | Where |
| --- | --- |
| An | Level 3, the drowned temple |
| Bet | Level 1, the secret room behind the library |
| Corp | Level 4, the gallery of tombs |
| Des | Level 4, the west wing |
| Ex | Level 4, the west wing |
| Flam | Level 2, on Bodric's table at the forge |
| Grav | Level 3, the southern caves |
| Hur | Level 2, the spider galleries |
| In | Level 1, the water cave (Mages and Druids start with it) |
| Jux | Level 1, the library chest (Mages and Druids start with it) |
| Kal | Level 4, the guardian's hall |
| Lor | Level 1, the water cave (Mages and Druids start with it) |
| Mani | Level 1, the box in the slug tunnel |
| Nox | Level 3, the drowned temple |
| Ort | Level 1, the library chest (Mages and Druids start with it) |
| Por | Level 2, the deep mine, near the shovel |
| Quas | Level 3, the hidden reliquary behind the drowned temple |
| Rel | Level 3, the southern caves |
| Sanct | Level 2, the spider galleries |
| Tym | Level 3, the southern caves |
| Uus | Level 4, the guardian's hall |
| Vas | Level 4, the troll den |
| Wis | Level 3, from Ssrath, or in the lurkers' grotto |
| Ylem | Level 4, the guardian's hall |
