//! Item definitions and item instances.

use crate::lang::{self, Noun};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Skill {
    Attack,
    Defense,
    Unarmed,
    Sword,
    Axe,
    Mace,
    Missile,
    Mana,
    Lore,
    Casting,
    Traps,
    Search,
    Track,
    Sneak,
    Repair,
    Charm,
    Picklock,
    Acrobat,
    Appraise,
    Swimming,
}

pub const SKILL_COUNT: usize = 20;
pub const ALL_SKILLS: [Skill; SKILL_COUNT] = [
    Skill::Attack,
    Skill::Defense,
    Skill::Unarmed,
    Skill::Sword,
    Skill::Axe,
    Skill::Mace,
    Skill::Missile,
    Skill::Mana,
    Skill::Lore,
    Skill::Casting,
    Skill::Traps,
    Skill::Search,
    Skill::Track,
    Skill::Sneak,
    Skill::Repair,
    Skill::Charm,
    Skill::Picklock,
    Skill::Acrobat,
    Skill::Appraise,
    Skill::Swimming,
];

impl Skill {
    pub fn name(self) -> &'static str {
        match self {
            Skill::Attack => "Attack",
            Skill::Defense => "Defense",
            Skill::Unarmed => "Unarmed",
            Skill::Sword => "Sword",
            Skill::Axe => "Axe",
            Skill::Mace => "Mace",
            Skill::Missile => "Missile",
            Skill::Mana => "Mana",
            Skill::Lore => "Lore",
            Skill::Casting => "Casting",
            Skill::Traps => "Traps",
            Skill::Search => "Search",
            Skill::Track => "Track",
            Skill::Sneak => "Sneak",
            Skill::Repair => "Repair",
            Skill::Charm => "Charm",
            Skill::Picklock => "Picklock",
            Skill::Acrobat => "Acrobat",
            Skill::Appraise => "Appraise",
            Skill::Swimming => "Swimming",
        }
    }

    /// The name shown to the player (`name` is also the key used in dialogue).
    pub fn label(self) -> &'static str {
        lang::tr(self.name())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Slot {
    Head,
    Chest,
    Legs,
    Feet,
    Hands,
    Shield,
    Ring,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Class {
    Weapon,
    Missile,
    Ammo,
    Armor,
    Food,
    Drink,
    Potion,
    Light,
    Container,
    Key,
    Rune,
    RuneBag,
    Treasure,
    Scroll,
    Book,
    Map,
    Wand,
    Misc,
    Quest,
    /// Furniture and decorations: cannot be picked up.
    Fixture,
}

#[derive(Clone, Copy, Debug)]
pub struct WeaponStats {
    pub skill: Skill,
    /// Damage for bash, slash, thrust.
    pub dmg: [i32; 3],
    /// Attack speed modifier (lower is faster).
    pub delay: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Hash)]
pub enum ItemKind {
    // Weapons
    Dagger,
    Shortsword,
    Longsword,
    Broadsword,
    HandAxe,
    Axe,
    Cudgel,
    Mace,
    JeweledSword,
    Sling,
    Crossbow,
    SlingStone,
    Bolt,
    // Armour
    LeatherCap,
    ChainCowl,
    PlateHelm,
    LeatherVest,
    ChainShirt,
    Breastplate,
    LeatherLeggings,
    ChainLeggings,
    PlateLeggings,
    LeatherBoots,
    PlateBoots,
    LeatherGloves,
    Gauntlets,
    Buckler,
    WoodenShield,
    TowerShield,
    GoldRing,
    SilverRing,
    // Food & drink
    Apple,
    Bread,
    Cheese,
    Meat,
    Fish,
    Mushroom,
    WaterFlask,
    Wine,
    // Potions
    RedPotion,
    BluePotion,
    GreenPotion,
    YellowPotion,
    // Light
    Torch,
    Candle,
    Lantern,
    // Containers
    Bag,
    Pack,
    Box,
    RuneBag,
    // Keys
    IronKey,
    GoldKey,
    SilverKey,
    // Magic
    RuneStone,
    Wand,
    // Treasure
    Coin,
    RedGem,
    BlueGem,
    GreenGem,
    WhiteGem,
    GoldNugget,
    // Writing
    Scroll,
    Book,
    Map,
    // Misc
    Bedroll,
    Rock,
    Bone,
    Skull,
    Lockpick,
    Rope,
    Pole,
    Shovel,
    // Quest items
    ShardOfValor,
    ShardOfWisdom,
    ShardOfMercy,
    SilverSapling,
    Medallion,
    Hammer,
    LizardEgg,
    RelicBones,
    /// A page of the colony's chronicle; `data` is the page, 0-4 (`dialogue::CHRONICLE`).
    ChroniclePage,
    // Fixtures
    Barrel,
    Table,
    Chest,
    Fountain,
    BonePile,
    Plant,
    Shrine,
    Lever,
    Campfire,
    Bed,
    Brazier,
    Statue,
    Bookshelf,
    Pillar,
    Gravestone,
    Cauldron,
    MoonOrb,
    Corpse,
    Blood,
    Stalagmite,
    Web,
    /// A rune of warding laid on the floor by the spell.
    Ward,
}

pub struct ItemDef {
    pub name: &'static str,
    pub plural: &'static str,
    pub class: Class,
    /// Weight in tenths of a stone.
    pub weight: i32,
    pub value: i32,
    pub weapon: Option<WeaponStats>,
    pub armor: Option<(Slot, i32)>,
    pub nutrition: i32,
    /// Light: (radius, brightness, burn seconds or 0 = forever)
    pub light: Option<(f32, f32, f32)>,
    pub capacity: i32,
    pub stack: bool,
    /// Sprite size in the world (width, height in world units).
    pub size: (f32, f32),
}

const fn d(name: &'static str, plural: &'static str, class: Class, weight: i32, value: i32) -> ItemDef {
    ItemDef {
        name,
        plural,
        class,
        weight,
        value,
        weapon: None,
        armor: None,
        nutrition: 0,
        light: None,
        capacity: 0,
        stack: false,
        size: (0.22, 0.22),
    }
}

const fn wpn(name: &'static str, w: i32, v: i32, skill: Skill, dmg: [i32; 3], delay: f32) -> ItemDef {
    let mut x = d(name, name, Class::Weapon, w, v);
    x.weapon = Some(WeaponStats { skill, dmg, delay });
    x.size = (0.3, 0.3);
    x
}

const fn arm(name: &'static str, plural: &'static str, w: i32, v: i32, slot: Slot, prot: i32) -> ItemDef {
    let mut x = d(name, plural, Class::Armor, w, v);
    x.armor = Some((slot, prot));
    x.size = (0.28, 0.28);
    x
}

const fn food(name: &'static str, plural: &'static str, w: i32, v: i32, n: i32) -> ItemDef {
    let mut x = d(name, plural, Class::Food, w, v);
    x.nutrition = n;
    x.size = (0.18, 0.18);
    x
}

const fn fixture(name: &'static str, size: (f32, f32)) -> ItemDef {
    let mut x = d(name, name, Class::Fixture, 0, 0);
    x.size = size;
    x
}

const fn with_size(mut x: ItemDef, size: (f32, f32)) -> ItemDef {
    x.size = size;
    x
}

const fn stack(mut x: ItemDef) -> ItemDef {
    x.stack = true;
    x
}

const fn light(mut x: ItemDef, r: f32, b: f32, t: f32) -> ItemDef {
    x.light = Some((r, b, t));
    x
}

const fn cont(mut x: ItemDef, cap: i32) -> ItemDef {
    x.capacity = cap;
    x
}

impl ItemKind {
    pub fn def(self) -> ItemDef {
        use ItemKind::*;
        use Skill as K;
        match self {
            Dagger => wpn("dagger", 5, 10, K::Sword, [2, 3, 4], 0.7),
            Shortsword => wpn("shortsword", 9, 25, K::Sword, [3, 5, 5], 0.85),
            Longsword => wpn("longsword", 12, 50, K::Sword, [4, 7, 6], 1.0),
            Broadsword => wpn("broadsword", 15, 80, K::Sword, [5, 8, 6], 1.1),
            HandAxe => wpn("hand axe", 9, 20, K::Axe, [4, 5, 2], 0.9),
            Axe => wpn("axe", 16, 45, K::Axe, [6, 8, 2], 1.15),
            Cudgel => wpn("cudgel", 8, 5, K::Mace, [4, 2, 2], 0.85),
            Mace => wpn("mace", 14, 40, K::Mace, [7, 4, 3], 1.1),
            JeweledSword => wpn("jeweled sword", 12, 300, K::Sword, [7, 10, 9], 0.9),
            Sling => {
                let mut x = wpn("sling", 2, 8, K::Missile, [0, 0, 4], 1.0);
                x.class = Class::Missile;
                x
            }
            Crossbow => {
                let mut x = wpn("crossbow", 18, 80, K::Missile, [0, 0, 8], 1.3);
                x.class = Class::Missile;
                x
            }
            SlingStone => stack(d("sling stone", "sling stones", Class::Ammo, 1, 1)),
            Bolt => stack(d("crossbow bolt", "crossbow bolts", Class::Ammo, 1, 2)),
            LeatherCap => arm("leather cap", "leather caps", 5, 10, Slot::Head, 1),
            ChainCowl => arm("chain cowl", "chain cowls", 10, 40, Slot::Head, 2),
            PlateHelm => arm("plate helm", "plate helms", 15, 90, Slot::Head, 3),
            LeatherVest => arm("leather vest", "leather vests", 10, 20, Slot::Chest, 2),
            ChainShirt => arm("chain shirt", "chain shirts", 30, 100, Slot::Chest, 4),
            Breastplate => arm("breastplate", "breastplates", 45, 250, Slot::Chest, 6),
            LeatherLeggings => arm("pair of leather leggings", "pairs of leather leggings", 8, 15, Slot::Legs, 1),
            ChainLeggings => arm("pair of chain leggings", "pairs of chain leggings", 20, 60, Slot::Legs, 2),
            PlateLeggings => arm("pair of plate leggings", "pairs of plate leggings", 30, 150, Slot::Legs, 3),
            LeatherBoots => arm("pair of leather boots", "pairs of leather boots", 6, 15, Slot::Feet, 1),
            PlateBoots => arm("pair of plate boots", "pairs of plate boots", 15, 80, Slot::Feet, 2),
            LeatherGloves => arm("pair of leather gloves", "pairs of leather gloves", 3, 10, Slot::Hands, 1),
            Gauntlets => arm("pair of gauntlets", "pairs of gauntlets", 8, 60, Slot::Hands, 2),
            Buckler => arm("buckler", "bucklers", 8, 20, Slot::Shield, 2),
            WoodenShield => arm("wooden shield", "wooden shields", 14, 35, Slot::Shield, 3),
            TowerShield => arm("tower shield", "tower shields", 30, 120, Slot::Shield, 5),
            GoldRing => arm("gold ring", "gold rings", 1, 100, Slot::Ring, 0),
            SilverRing => arm("silver ring", "silver rings", 1, 60, Slot::Ring, 0),
            Apple => food("apple", "apples", 2, 2, 20),
            Bread => food("loaf of bread", "loaves of bread", 4, 3, 40),
            Cheese => food("piece of cheese", "pieces of cheese", 3, 4, 35),
            Meat => food("piece of meat", "pieces of meat", 5, 5, 55),
            Fish => food("fish", "fish", 4, 4, 45),
            Mushroom => food("mushroom", "mushrooms", 1, 1, 10),
            WaterFlask => {
                let mut x = food("flask of water", "flasks of water", 6, 2, 8);
                x.class = Class::Drink;
                x
            }
            Wine => {
                let mut x = food("bottle of wine", "bottles of wine", 8, 10, 10);
                x.class = Class::Drink;
                x
            }
            RedPotion => d("red potion", "red potions", Class::Potion, 4, 40),
            BluePotion => d("blue potion", "blue potions", Class::Potion, 4, 40),
            GreenPotion => d("green potion", "green potions", Class::Potion, 4, 30),
            YellowPotion => d("yellow potion", "yellow potions", Class::Potion, 4, 60),
            Torch => light(d("torch", "torches", Class::Light, 5, 3), 5.0, 0.85, 600.0),
            Candle => light(d("candle", "candles", Class::Light, 1, 2), 3.5, 0.7, 450.0),
            Lantern => light(d("lantern", "lanterns", Class::Light, 8, 40), 6.5, 0.95, 1500.0),
            Bag => cont(d("bag", "bags", Class::Container, 2, 5), 8),
            Pack => cont(d("pack", "packs", Class::Container, 4, 15), 8),
            Box => cont(with_size(d("small box", "small boxes", Class::Container, 10, 5), (0.3, 0.25)), 6),
            RuneBag => cont(d("rune bag", "rune bags", Class::RuneBag, 1, 0), 24),
            IronKey => d("iron key", "iron keys", Class::Key, 1, 1),
            GoldKey => d("gold key", "gold keys", Class::Key, 1, 10),
            SilverKey => d("silver key", "silver keys", Class::Key, 1, 5),
            RuneStone => d("rune stone", "rune stones", Class::Rune, 1, 50),
            Wand => d("wand", "wands", Class::Wand, 2, 100),
            Coin => stack(d("gold coin", "gold coins", Class::Treasure, 0, 1)),
            RedGem => d("ruby", "rubies", Class::Treasure, 1, 120),
            BlueGem => d("sapphire", "sapphires", Class::Treasure, 1, 100),
            GreenGem => d("emerald", "emeralds", Class::Treasure, 1, 110),
            WhiteGem => d("diamond", "diamonds", Class::Treasure, 1, 200),
            GoldNugget => d("gold nugget", "gold nuggets", Class::Treasure, 3, 40),
            Scroll => d("scroll", "scrolls", Class::Scroll, 1, 5),
            Book => d("book", "books", Class::Book, 5, 15),
            Map => d("map", "maps", Class::Map, 1, 0),
            Bedroll => d("bedroll", "bedrolls", Class::Misc, 10, 8),
            Rock => d("rock", "rocks", Class::Misc, 10, 0),
            Bone => d("bone", "bones", Class::Misc, 2, 0),
            Skull => d("skull", "skulls", Class::Misc, 3, 0),
            Lockpick => d("lockpick", "lockpicks", Class::Misc, 1, 10),
            Rope => d("coil of rope", "coils of rope", Class::Misc, 6, 10),
            Pole => with_size(d("pole", "poles", Class::Misc, 12, 4), (0.5, 0.5)),
            Shovel => d("shovel", "shovels", Class::Misc, 12, 10),
            ShardOfValor => d("shard of Valor", "shards of Valor", Class::Quest, 2, 0),
            ShardOfWisdom => d("shard of Wisdom", "shards of Wisdom", Class::Quest, 2, 0),
            ShardOfMercy => d("shard of Mercy", "shards of Mercy", Class::Quest, 2, 0),
            SilverSapling => d("silver sapling", "silver saplings", Class::Quest, 3, 0),
            Medallion => d("medallion", "medallions", Class::Quest, 1, 0),
            Hammer => {
                let mut x = wpn("Hammer of the Mountain", 20, 500, K::Mace, [9, 5, 3], 1.2);
                x.class = Class::Weapon;
                x
            }
            LizardEgg => d("clutch of lizard eggs", "clutches of lizard eggs", Class::Quest, 6, 0),
            RelicBones => d("bundle of ancient bones", "bundles of ancient bones", Class::Quest, 8, 0),
            ChroniclePage => d("chronicle page", "chronicle pages", Class::Scroll, 1, 0),
            Barrel => fixture("barrel", (0.42, 0.55)),
            Table => fixture("table", (0.8, 0.45)),
            Chest => cont(fixture("chest", (0.55, 0.42)), 12),
            Fountain => fixture("fountain", (0.8, 0.7)),
            BonePile => fixture("pile of bones", (0.5, 0.25)),
            Plant => fixture("plant", (0.35, 0.4)),
            Shrine => fixture("shrine", (0.6, 0.95)),
            Lever => fixture("lever", (0.25, 0.35)),
            Campfire => fixture("campfire", (0.45, 0.4)),
            Bed => fixture("bed", (0.9, 0.35)),
            Brazier => fixture("brazier", (0.4, 0.6)),
            Statue => fixture("statue", (0.5, 1.1)),
            Bookshelf => fixture("bookshelf", (0.7, 0.9)),
            Pillar => fixture("pillar", (0.35, 1.2)),
            Gravestone => fixture("gravestone", (0.4, 0.5)),
            Cauldron => fixture("cauldron", (0.45, 0.4)),
            MoonOrb => fixture("strange orb", (0.3, 0.55)),
            Corpse => cont(fixture("corpse", (0.7, 0.25)), 8),
            Blood => fixture("bloodstain", (0.5, 0.08)),
            Stalagmite => fixture("stalagmite", (0.35, 0.6)),
            Web => fixture("spider web", (0.7, 0.7)),
            Ward => fixture("rune of warding", (0.5, 0.08)),
        }
    }

    pub fn is_fixture(self) -> bool {
        self.def().class == Class::Fixture
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Enchant {
    None,
    Accuracy(u8),
    Damage(u8),
    Protection(u8),
    Spell(crate::game::magic::Spell),
}

impl Enchant {
    /// How hard the enchantment is to identify with Lore.
    pub fn difficulty(self) -> i32 {
        match self {
            Enchant::None => 0,
            Enchant::Accuracy(n) | Enchant::Damage(n) | Enchant::Protection(n) => 10 + n as i32 * 8,
            Enchant::Spell(sp) => 10 + sp.def().circle * 7,
        }
    }

    /// Extra worth in gold.
    pub fn value(self) -> i32 {
        match self {
            Enchant::None => 0,
            Enchant::Accuracy(n) | Enchant::Damage(n) | Enchant::Protection(n) => n as i32 * 50,
            Enchant::Spell(sp) => sp.def().circle * 30,
        }
    }

    /// "an enchantment of protection (+2)", "the Fireball spell".
    pub fn describe(self) -> String {
        match self {
            Enchant::None => String::new(),
            Enchant::Accuracy(n) => t!("an enchantment of accuracy (+{})", n),
            Enchant::Damage(n) => t!("an enchantment of damage (+{})", n),
            Enchant::Protection(n) => t!("an enchantment of protection (+{})", n),
            Enchant::Spell(sp) => t!("the {} spell", sp.label()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub kind: ItemKind,
    #[serde(default = "one")]
    pub qty: u16,
    /// Condition 0..=100.
    #[serde(default = "full")]
    pub quality: u8,
    #[serde(default)]
    pub lit: bool,
    /// Seconds of light left.
    #[serde(default)]
    pub fuel: f32,
    #[serde(default)]
    pub contents: Vec<Item>,
    /// Key id for keys; text id for scrolls/books; rune index for rune stones.
    #[serde(default)]
    pub data: u16,
    #[serde(default = "no_enchant")]
    pub enchant: Enchant,
    #[serde(default)]
    pub charges: u8,
    #[serde(default)]
    pub identified: bool,
}

fn one() -> u16 {
    1
}
fn full() -> u8 {
    100
}
fn no_enchant() -> Enchant {
    Enchant::None
}

impl Item {
    pub fn new(kind: ItemKind) -> Item {
        let def = kind.def();
        Item {
            kind,
            qty: 1,
            quality: 100,
            lit: false,
            fuel: def.light.map(|l| l.2).unwrap_or(0.0),
            contents: Vec::new(),
            data: 0,
            enchant: Enchant::None,
            charges: 0,
            identified: false,
        }
    }
    pub fn qty(mut self, q: u16) -> Item {
        self.qty = q;
        self
    }
    pub fn data(mut self, d: u16) -> Item {
        self.data = d;
        self
    }
    pub fn with(mut self, items: Vec<Item>) -> Item {
        self.contents = items;
        self
    }
    pub fn enchanted(mut self, e: Enchant, charges: u8) -> Item {
        self.enchant = e;
        self.charges = charges;
        self
    }

    pub fn def(&self) -> ItemDef {
        self.kind.def()
    }

    /// Total weight in tenths of a stone, including contents.
    pub fn weight(&self) -> i32 {
        let d = self.def();
        let own = if d.stack { (d.weight * self.qty as i32 + 9) / 10 } else { d.weight };
        own + self.contents.iter().map(|i| i.weight()).sum::<i32>()
    }

    /// "a sword", "an apple", "5 gold coins".
    pub fn name_with_article(&self) -> String {
        let d = self.def();
        if self.qty > 1 {
            return format!("{} {}", self.qty, d.plural);
        }
        let n = self.display_name();
        if n.starts_with("pair") || n.starts_with("piece") || n.starts_with("loaf") || n.starts_with("flask") || n.starts_with("bottle") || n.starts_with("coil") {
            return format!("a {}", n);
        }
        let first = n.chars().next().unwrap_or('x').to_ascii_lowercase();
        let art = if "aeiou".contains(first) { "an" } else { "a" };
        format!("{} {}", art, n)
    }

    /// The item as a noun phrase, with its article in English: "a sword",
    /// "an apple", "5 gold coins".
    pub fn noun(&self) -> Noun {
        self.polish(Noun::new(self.name_with_article(), self.def().name).qty(self.qty as u32))
    }

    /// The item without an article, for "the ...": "lit torch".
    pub fn bare(&self) -> Noun {
        self.polish(Noun::new(self.display_name(), self.def().name))
    }

    fn polish(&self, n: Noun) -> Noun {
        match self.kind {
            ItemKind::RuneStone => n.suffix(crate::game::magic::RUNE_NAMES[self.data as usize % 24]),
            ItemKind::Torch | ItemKind::Candle | ItemKind::Lantern if self.lit => n.adj("lit"),
            _ => n,
        }
    }

    pub fn display_name(&self) -> String {
        let d = self.def();
        match self.kind {
            ItemKind::RuneStone => format!("{} rune stone", crate::game::magic::RUNE_NAMES[self.data as usize % 24]),
            ItemKind::Torch | ItemKind::Candle | ItemKind::Lantern if self.lit => format!("lit {}", d.name),
            _ => d.name.to_string(),
        }
    }

    /// What the item is worth in gold: worn gear is worth less, enchantments more.
    pub fn value(&self) -> i32 {
        let d = self.def();
        let mut v = d.value * self.qty as i32;
        if matches!(d.class, Class::Weapon | Class::Missile | Class::Armor) {
            v = v * (20 + self.quality as i32 * 4 / 5) / 100;
        }
        v + self.enchant.value() + self.charges as i32 * 10
    }

    pub fn condition_word(&self) -> &'static str {
        match self.quality {
            0..=15 => "badly worn ",
            16..=40 => "worn ",
            41..=75 => "somewhat used ",
            _ => "",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn articles() {
        assert_eq!(Item::new(ItemKind::Apple).name_with_article(), "an apple");
        assert_eq!(Item::new(ItemKind::Longsword).name_with_article(), "a longsword");
        assert_eq!(Item::new(ItemKind::Coin).qty(5).name_with_article(), "5 gold coins");
        assert_eq!(Item::new(ItemKind::LeatherBoots).name_with_article(), "a pair of leather boots");
    }

    #[test]
    fn every_item_and_skill_has_a_polish_name() {
        lang::set(lang::Lang::Pl);
        for k in crate::game::dialogue::ALL_ITEM_KINDS {
            let n = k.def().name;
            assert!(lang::has_noun(n), "no Polish noun for '{}'", n);
        }
        for s in ALL_SKILLS {
            assert!(lang::has_phrase(s.name()), "no Polish name for the {} skill", s.name());
        }
        let mut lit = Item::new(ItemKind::Torch);
        lit.lit = true;
        assert_eq!(t!("You see {}.", lit.noun()), "Widzisz zapaloną pochodnię.");
        lang::set(lang::Lang::En);
    }

    #[test]
    fn weights_include_contents() {
        let bag = Item::new(ItemKind::Bag).with(vec![Item::new(ItemKind::Rock), Item::new(ItemKind::Apple)]);
        assert_eq!(bag.weight(), 2 + 10 + 2);
        assert_eq!(Item::new(ItemKind::Coin).qty(25).weight(), 0);
        assert_eq!(Item::new(ItemKind::SlingStone).qty(20).weight(), 2);
    }
}
