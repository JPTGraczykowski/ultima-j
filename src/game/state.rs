//! Core game state types: levels, doors, triggers, the player and the game.

use super::creatures::Npc;
use super::items::{Item, ItemKind, Skill, Slot, SKILL_COUNT};
use super::magic::Spell;
use crate::art::textures::Tex;
use crate::audio::Sfx;
use crate::util::{Rng, V3};
use crate::world::map::Map;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldItem {
    pub id: u32,
    pub item: Item,
    pub pos: V3,
    #[serde(default)]
    pub vel: V3,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum DoorAxis {
    /// Door plane runs east-west (you pass it going north/south).
    NS,
    /// Door plane runs north-south (you pass it going east/west).
    EW,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum DoorKind {
    Wood,
    Iron,
    Portcullis,
    /// Looks like the surrounding wall until found.
    Secret,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Door {
    pub id: u32,
    pub tx: i32,
    pub tz: i32,
    pub axis: DoorAxis,
    pub kind: DoorKind,
    /// 0 = closed, 1 = fully open.
    pub open: f32,
    pub opening: bool,
    /// Key id that opens this door (0 = none).
    pub key: u16,
    pub locked: bool,
    /// Can only be opened by a lever/trigger.
    #[serde(default)]
    pub remote: bool,
    #[serde(default)]
    pub found: bool,
    #[serde(default)]
    pub tex: Option<Tex>,
}

impl Door {
    pub fn is_open(&self) -> bool {
        self.open > 0.85
    }
    pub fn blocks(&self) -> bool {
        self.open < 0.8
    }
    pub fn name(&self) -> &'static str {
        match self.kind {
            DoorKind::Wood => t!("a sturdy door"),
            DoorKind::Iron => t!("a massive iron door"),
            DoorKind::Portcullis => t!("a portcullis"),
            DoorKind::Secret => t!("a secret door"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TriggerKind {
    /// Move to another level (stairs).
    Stairs { level: usize, x: f32, z: f32, yaw: f32 },
    /// Teleport within the level.
    Teleport { x: f32, z: f32 },
    /// Show a message once.
    Message { text: String },
    /// A pressure plate or lever that toggles a door.
    OpenDoor { door: u32 },
    /// Hurt the player (traps).
    Damage { amount: i32 },
    /// Set a quest flag.
    Flag { name: String, value: i32 },
    /// Win the game.
    Victory,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trigger {
    pub tx: i32,
    pub tz: i32,
    pub kind: TriggerKind,
    #[serde(default)]
    pub once: bool,
    #[serde(default)]
    pub fired: bool,
    /// A trap the player has spotted (shown on the map, can be disarmed).
    #[serde(default)]
    pub found: bool,
}

impl Trigger {
    /// An armed trap (not yet sprung or disarmed).
    pub fn is_armed_trap(&self) -> bool {
        matches!(self.kind, TriggerKind::Damage { .. }) && !self.fired
    }
}

/// A lever or switch on a wall (identified by the tile in front of it).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Switch {
    pub tx: i32,
    pub tz: i32,
    pub door: u32,
    pub on: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Level {
    pub name: String,
    pub map: Map,
    pub items: Vec<WorldItem>,
    pub npcs: Vec<Npc>,
    pub doors: Vec<Door>,
    pub triggers: Vec<Trigger>,
    #[serde(default)]
    pub switches: Vec<Switch>,
    /// Writing on walls: tile -> text.
    #[serde(default)]
    pub writings: Vec<(i32, i32, String)>,
    pub ambient: f32,
    pub start: (f32, f32, f32),
    #[serde(default)]
    pub notes: Vec<(i32, i32, String)>,
    pub next_id: u32,
}

impl Level {
    pub fn new(name: &str, map: Map) -> Level {
        Level {
            name: name.to_string(),
            map,
            items: Vec::new(),
            npcs: Vec::new(),
            doors: Vec::new(),
            triggers: Vec::new(),
            switches: Vec::new(),
            writings: Vec::new(),
            ambient: 2.0,
            start: (1.5, 1.5, 0.0),
            notes: Vec::new(),
            next_id: 1,
        }
    }

    pub fn alloc_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn add_item(&mut self, item: Item, x: f32, z: f32) -> u32 {
        let id = self.alloc_id();
        let y = self.map.floor_at(x, z);
        self.items.push(WorldItem { id, item, pos: V3::new(x, y, z), vel: V3::ZERO });
        id
    }

    pub fn add_npc(&mut self, mut npc: Npc) -> u32 {
        let id = self.alloc_id();
        npc.id = id;
        npc.pos.y = self.map.floor_at(npc.pos.x, npc.pos.z) + npc.race.def().hover;
        self.npcs.push(npc);
        id
    }

    pub fn add_door(&mut self, tx: i32, tz: i32, axis: DoorAxis, kind: DoorKind, key: u16) -> u32 {
        let id = self.alloc_id();
        self.doors.push(Door { id, tx, tz, axis, kind, open: 0.0, opening: false, key, locked: key != 0, remote: false, found: kind != DoorKind::Secret, tex: None });
        id
    }

    pub fn npc(&self, id: u32) -> Option<&Npc> {
        self.npcs.iter().find(|n| n.id == id)
    }
    pub fn npc_mut(&mut self, id: u32) -> Option<&mut Npc> {
        self.npcs.iter_mut().find(|n| n.id == id)
    }
    pub fn item_index(&self, id: u32) -> Option<usize> {
        self.items.iter().position(|i| i.id == id)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum PClass {
    Fighter,
    Mage,
    Bard,
    Tinker,
    Druid,
    Paladin,
    Ranger,
    Shepherd,
}

pub const ALL_CLASSES: [PClass; 8] =
    [PClass::Fighter, PClass::Mage, PClass::Bard, PClass::Tinker, PClass::Druid, PClass::Paladin, PClass::Ranger, PClass::Shepherd];

impl PClass {
    pub fn name(self) -> &'static str {
        match self {
            PClass::Fighter => t!("Fighter"),
            PClass::Mage => t!("Mage"),
            PClass::Bard => t!("Bard"),
            PClass::Tinker => t!("Tinker"),
            PClass::Druid => t!("Druid"),
            PClass::Paladin => t!("Paladin"),
            PClass::Ranger => t!("Ranger"),
            PClass::Shepherd => t!("Shepherd"),
        }
    }
    /// Base (str, dex, int) bias.
    pub fn stat_bias(self) -> (i32, i32, i32) {
        match self {
            PClass::Fighter => (6, 2, -2),
            PClass::Mage => (-3, 1, 7),
            PClass::Bard => (0, 3, 3),
            PClass::Tinker => (2, 4, 1),
            PClass::Druid => (0, 1, 5),
            PClass::Paladin => (4, 1, 1),
            PClass::Ranger => (2, 5, 0),
            PClass::Shepherd => (2, 2, 2),
        }
    }
    /// Skills this class starts trained in (and may pick from).
    pub fn skills(self) -> &'static [Skill] {
        use Skill::*;
        match self {
            PClass::Fighter => &[Attack, Defense, Sword, Axe, Mace, Unarmed],
            PClass::Mage => &[Mana, Casting, Lore, Appraise],
            PClass::Bard => &[Lore, Charm, Appraise, Sword, Casting],
            PClass::Tinker => &[Repair, Picklock, Traps, Mace, Appraise],
            PClass::Druid => &[Mana, Casting, Lore, Track, Swimming],
            PClass::Paladin => &[Attack, Defense, Sword, Mace, Charm],
            PClass::Ranger => &[Missile, Track, Sneak, Search, Sword],
            PClass::Shepherd => &[Unarmed, Missile, Track, Swimming, Search],
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Equip {
    pub head: Option<Item>,
    pub chest: Option<Item>,
    pub legs: Option<Item>,
    pub feet: Option<Item>,
    pub hands: Option<Item>,
    /// Right hand (weapon) and left hand (shield).
    pub weapon: Option<Item>,
    pub shield: Option<Item>,
    pub ring_l: Option<Item>,
    pub ring_r: Option<Item>,
    pub shoulder_l: Option<Item>,
    pub shoulder_r: Option<Item>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum EqSlot {
    Head,
    Chest,
    Legs,
    Feet,
    Hands,
    Weapon,
    Shield,
    RingL,
    RingR,
    ShoulderL,
    ShoulderR,
}

pub const ALL_EQ: [EqSlot; 11] = [
    EqSlot::Head,
    EqSlot::Chest,
    EqSlot::Legs,
    EqSlot::Feet,
    EqSlot::Hands,
    EqSlot::Weapon,
    EqSlot::Shield,
    EqSlot::RingL,
    EqSlot::RingR,
    EqSlot::ShoulderL,
    EqSlot::ShoulderR,
];

impl Equip {
    pub fn get(&self, s: EqSlot) -> &Option<Item> {
        match s {
            EqSlot::Head => &self.head,
            EqSlot::Chest => &self.chest,
            EqSlot::Legs => &self.legs,
            EqSlot::Feet => &self.feet,
            EqSlot::Hands => &self.hands,
            EqSlot::Weapon => &self.weapon,
            EqSlot::Shield => &self.shield,
            EqSlot::RingL => &self.ring_l,
            EqSlot::RingR => &self.ring_r,
            EqSlot::ShoulderL => &self.shoulder_l,
            EqSlot::ShoulderR => &self.shoulder_r,
        }
    }
    pub fn get_mut(&mut self, s: EqSlot) -> &mut Option<Item> {
        match s {
            EqSlot::Head => &mut self.head,
            EqSlot::Chest => &mut self.chest,
            EqSlot::Legs => &mut self.legs,
            EqSlot::Feet => &mut self.feet,
            EqSlot::Hands => &mut self.hands,
            EqSlot::Weapon => &mut self.weapon,
            EqSlot::Shield => &mut self.shield,
            EqSlot::RingL => &mut self.ring_l,
            EqSlot::RingR => &mut self.ring_r,
            EqSlot::ShoulderL => &mut self.shoulder_l,
            EqSlot::ShoulderR => &mut self.shoulder_r,
        }
    }
    /// Can this item be placed in that slot?
    pub fn accepts(s: EqSlot, item: &Item) -> bool {
        let d = item.def();
        match s {
            EqSlot::Weapon | EqSlot::Shield | EqSlot::ShoulderL | EqSlot::ShoulderR => true,
            EqSlot::RingL | EqSlot::RingR => matches!(d.armor, Some((Slot::Ring, _))),
            EqSlot::Head => matches!(d.armor, Some((Slot::Head, _))),
            EqSlot::Chest => matches!(d.armor, Some((Slot::Chest, _))),
            EqSlot::Legs => matches!(d.armor, Some((Slot::Legs, _))),
            EqSlot::Feet => matches!(d.armor, Some((Slot::Feet, _))),
            EqSlot::Hands => matches!(d.armor, Some((Slot::Hands, _))),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActiveSpell {
    pub spell: Spell,
    pub left: f32,
}

pub const PACK_SIZE: usize = 8;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub female: bool,
    pub class: PClass,
    pub skin: u8,
    pub hair: u8,
    /// Feet position.
    pub pos: V3,
    pub yaw: f32,
    pub pitch: f32,
    pub vel_y: f32,
    pub on_ground: bool,
    pub swimming: bool,
    pub st: i32,
    pub dex: i32,
    pub int: i32,
    pub vit: i32,
    pub max_vit: i32,
    pub mana: i32,
    pub max_mana: i32,
    pub exp: i32,
    pub level: i32,
    pub skill_points: i32,
    pub skills: [i32; SKILL_COUNT],
    pub equip: Equip,
    pub pack: Vec<Option<Item>>,
    pub runes: [bool; 24],
    pub shelf: Vec<u8>,
    /// 0 = starving .. 100 = stuffed.
    pub hunger: f32,
    /// 0 = rested .. 100 = exhausted.
    pub fatigue: f32,
    pub poison: f32,
    pub spells: Vec<ActiveSpell>,
    pub dead: bool,
    /// Fractional vitality / mana regeneration accumulators.
    #[serde(default)]
    pub regen: f32,
    #[serde(default)]
    pub mana_regen: f32,
    /// Highest point of the current fall (for falling damage).
    #[serde(default)]
    pub fall_from: f32,
    #[serde(default)]
    pub drown: f32,
}

impl Player {
    pub fn new(name: &str, class: PClass, female: bool) -> Player {
        let (sb, db, ib) = class.stat_bias();
        let st = 14 + sb;
        let dex = 14 + db;
        let int = 14 + ib;
        let max_vit = 20 + st / 2;
        let max_mana = (int / 3).max(1) + if matches!(class, PClass::Mage | PClass::Druid) { 6 } else { 0 };
        let mut skills = [0; SKILL_COUNT];
        for s in class.skills() {
            skills[*s as usize] = 4;
        }
        skills[Skill::Attack as usize] += 2;
        skills[Skill::Defense as usize] += 2;
        Player {
            name: name.to_string(),
            female,
            class,
            skin: 0,
            hair: 0,
            pos: V3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            vel_y: 0.0,
            on_ground: true,
            swimming: false,
            st,
            dex,
            int,
            vit: max_vit,
            max_vit,
            mana: max_mana,
            max_mana,
            exp: 0,
            level: 1,
            skill_points: 0,
            skills,
            equip: Equip::default(),
            pack: vec![None; PACK_SIZE],
            runes: [false; 24],
            shelf: Vec::new(),
            hunger: 70.0,
            fatigue: 0.0,
            poison: 0.0,
            spells: Vec::new(),
            dead: false,
            regen: 0.0,
            mana_regen: 0.0,
            fall_from: 0.0,
            drown: 0.0,
        }
    }

    pub fn skill(&self, s: Skill) -> i32 {
        self.skills[s as usize]
    }

    /// Movement speed factor in water (Swimming).
    pub fn swim_speed(&self) -> f32 {
        (0.5 + self.skill(Skill::Swimming) as f32 * 0.03).min(0.85)
    }

    /// How far the eyes sink while swimming (Swimming keeps your head up).
    pub fn swim_sink(&self) -> f32 {
        super::physics::SWIM_SINK * (1.0 - self.skill(Skill::Swimming) as f32 * 0.04).max(0.4)
    }

    /// Load above which you are dragged under while swimming.
    pub fn swim_load_limit(&self) -> i32 {
        self.capacity() * (60 + self.skill(Skill::Swimming) * 4).min(100) / 100
    }

    /// Seconds between drowning blows when swimming overloaded.
    pub fn drown_interval(&self) -> f32 {
        4.0 + self.skill(Skill::Swimming) as f32 * 0.5
    }

    /// Upward speed of a jump (Acrobat jumps higher and so farther).
    pub fn jump_speed(&self) -> f32 {
        super::physics::JUMP_SPEED * (1.0 + (self.skill(Skill::Acrobat) as f32 * 0.015).min(0.15))
    }

    /// Damage from a fall of `drop` height units (Acrobat softens landings).
    pub fn fall_damage(&self, drop: f32) -> i32 {
        let acro = self.skill(Skill::Acrobat) as f32;
        let safe = 1.4 + acro * 0.08;
        if drop <= safe {
            return 0;
        }
        ((drop - safe + 0.2) * 6.0 * (1.0 - acro * 0.04).max(0.4)) as i32
    }

    /// Distance at which creatures that have not yet seen you notice you (Sneak).
    pub fn notice_range(&self, base: f32) -> f32 {
        base * (1.0 - self.skill(Skill::Sneak) as f32 * 0.04).max(0.45)
    }

    pub fn has_spell(&self, s: Spell) -> bool {
        self.spells.iter().any(|a| a.spell == s)
    }

    /// Total carried weight in tenths of stones.
    pub fn load(&self) -> i32 {
        let pack: i32 = self.pack.iter().flatten().map(|i| i.weight()).sum();
        let eq: i32 = ALL_EQ.iter().filter_map(|s| self.equip.get(*s).as_ref()).map(|i| i.weight()).sum();
        pack + eq
    }

    /// Carrying capacity in tenths of stones.
    pub fn capacity(&self) -> i32 {
        self.st * 25
    }

    pub fn armor_value(&self) -> i32 {
        ALL_EQ
            .iter()
            .filter(|s| !matches!(s, EqSlot::ShoulderL | EqSlot::ShoulderR | EqSlot::Weapon))
            .filter_map(|s| self.equip.get(*s).as_ref())
            .map(|i| {
                let base = i.def().armor.map(|a| a.1).unwrap_or(0);
                let ench = if let super::items::Enchant::Protection(n) = i.enchant { n as i32 } else { 0 };
                if i.quality < 20 { base / 2 + ench } else { base + ench }
            })
            .sum()
    }

    /// Experience needed for a given level. Paced so that a thorough player
    /// reaches about level 8 by the end of the Crypts.
    pub fn exp_for_level(level: i32) -> i32 {
        const TABLE: [i32; 16] = [0, 50, 140, 300, 600, 900, 1300, 1800, 2400, 3200, 4200, 5400, 7000, 9000, 11500, 14500];
        TABLE[(level.clamp(1, 16) - 1) as usize]
    }

    /// Highest spell circle the player is experienced enough to cast.
    pub fn max_circle(&self) -> i32 {
        self.level.min(8)
    }

    /// The light the player carries: (radius, brightness).
    pub fn light(&self) -> (f32, f32) {
        let mut best = (3.0f32, 0.55f32);
        for s in [EqSlot::ShoulderL, EqSlot::ShoulderR, EqSlot::Weapon, EqSlot::Shield] {
            if let Some(it) = self.equip.get(s) {
                if it.lit {
                    if let Some((r, b, _)) = it.def().light {
                        if r > best.0 {
                            best = (r, b);
                        }
                    }
                }
            }
        }
        if self.has_spell(Spell::Light) && best.0 < 7.0 {
            best = (7.0, 0.95);
        }
        if self.has_spell(Spell::Daylight) {
            best = (16.0, 1.0);
        }
        if self.has_spell(Spell::NightVision) {
            best.0 = best.0.max(9.0);
            best.1 = best.1.max(0.8);
        }
        best
    }

    pub fn has_item(&self, kind: ItemKind) -> bool {
        fn find(items: &[Item], kind: ItemKind) -> bool {
            items.iter().any(|i| i.kind == kind || find(&i.contents, kind))
        }
        let pack: Vec<Item> = self.pack.iter().flatten().cloned().collect();
        let eq: Vec<Item> = ALL_EQ.iter().filter_map(|s| self.equip.get(*s).clone()).collect();
        find(&pack, kind) || find(&eq, kind)
    }

    /// Put an item into the first free pack slot (or a container). Returns it back if full.
    pub fn give(&mut self, item: Item) -> Option<Item> {
        // Merge stacks first.
        if item.def().stack {
            for it in self.pack.iter_mut().flatten() {
                if it.kind == item.kind {
                    it.qty += item.qty;
                    return None;
                }
            }
        }
        for slot in self.pack.iter_mut() {
            if slot.is_none() {
                *slot = Some(item);
                return None;
            }
        }
        // Try containers in the pack.
        for it in self.pack.iter_mut().flatten() {
            let cap = it.def().capacity;
            if cap > 0 && (it.contents.len() as i32) < cap && it.kind != ItemKind::RuneBag {
                it.contents.push(item);
                return None;
            }
        }
        Some(item)
    }

    /// Remove one item of the given kind anywhere in the inventory.
    pub fn take(&mut self, kind: ItemKind) -> Option<Item> {
        fn take_from(v: &mut Vec<Item>, kind: ItemKind) -> Option<Item> {
            if let Some(i) = v.iter().position(|i| i.kind == kind) {
                return Some(v.remove(i));
            }
            for it in v.iter_mut() {
                if let Some(x) = take_from(&mut it.contents, kind) {
                    return Some(x);
                }
            }
            None
        }
        for slot in self.pack.iter_mut() {
            if let Some(it) = slot {
                if it.kind == kind {
                    return slot.take();
                }
                if let Some(x) = take_from(&mut it.contents, kind) {
                    return Some(x);
                }
            }
        }
        None
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum MsgKind {
    Normal,
    System,
    Speech,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Msg {
    pub text: String,
    pub kind: MsgKind,
}

/// Transient events for the presentation layer (sounds, flashes...).
#[derive(Clone, Debug)]
pub enum Event {
    Sound(Sfx),
    Flash([f32; 3], f32),
    Shake(f32),
    LevelChanged,
    StartConversation(u32),
    ReadText(String),
    Victory,
    PlayerDied,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Projectile {
    pub pos: V3,
    pub vel: V3,
    pub kind: ProjKind,
    pub from_player: bool,
    pub dmg: i32,
    pub life: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ProjKind {
    MagicArrow,
    Fireball,
    Lightning,
    Stone,
    Bolt,
    Poison,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Effect {
    pub pos: V3,
    pub kind: EffectKind,
    pub t: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum EffectKind {
    Blood,
    Sparks,
    Explosion,
    Splash,
    Smoke,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Game {
    pub levels: Vec<Level>,
    pub cur: usize,
    pub player: Player,
    pub time: f64,
    pub rng: Rng,
    pub flags: BTreeMap<String, i32>,
    pub messages: Vec<Msg>,
    pub projectiles: Vec<Projectile>,
    pub effects: Vec<Effect>,
    #[serde(skip)]
    pub events: Vec<Event>,
    /// The NPC currently fought (for the gargoyle's eyes).
    #[serde(default)]
    pub foe: Option<u32>,
    #[serde(default)]
    pub foe_timer: f32,
}

impl Game {
    pub fn level(&self) -> &Level {
        &self.levels[self.cur]
    }
    pub fn level_mut(&mut self) -> &mut Level {
        &mut self.levels[self.cur]
    }

    pub fn msg(&mut self, text: impl Into<String>) {
        self.messages.push(Msg { text: text.into(), kind: MsgKind::Normal });
        self.trim_messages();
    }
    pub fn sys(&mut self, text: impl Into<String>) {
        self.messages.push(Msg { text: text.into(), kind: MsgKind::System });
        self.trim_messages();
    }
    fn trim_messages(&mut self) {
        if self.messages.len() > 200 {
            self.messages.drain(0..50);
        }
    }
    pub fn sound(&mut self, s: Sfx) {
        self.events.push(Event::Sound(s));
    }
    pub fn flag(&self, name: &str) -> i32 {
        *self.flags.get(name).unwrap_or(&0)
    }
    pub fn set_flag(&mut self, name: &str, v: i32) {
        self.flags.insert(name.to_string(), v);
    }

    /// Day number in the abyss (a day lasts 20 minutes of play).
    pub fn day(&self) -> i32 {
        (self.time / 1200.0) as i32 + 1
    }
}
