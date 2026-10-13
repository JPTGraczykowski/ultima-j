//! Conversations. Dialogue trees are written in a small text format
//! (see `assets/dialogue.txt`) and parsed at start-up:
//!
//! ```text
//! === convname
//! -- node
//! NPC text, may span lines. {name} is replaced by the player's name.
//! ! action args        (run when the node is entered)
//! * option text -> next_node ? condition & condition
//! ```
//! Conditions: `flag:NAME>=N`, `flag:NAME=N`, `!flag:NAME`, `item:Kind`,
//! `!item:Kind`, `gold>=N`. Actions: `flag NAME N`, `give Kind [data] [qty]`,
//! `take Kind [qty]`, `exp N`, `hostile`, `heal`, `rune N`, `follow`,
//! `mana`, `skill NAME N`, `end`.

use super::creatures::{AiState, Attitude};
use super::items::{Item, ItemKind, ALL_SKILLS};
use super::Game;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Cond {
    FlagGe(String, i32),
    FlagEq(String, i32),
    NoFlag(String),
    HasItem(ItemKind),
    NoItem(ItemKind),
    GoldGe(i32),
}

#[derive(Clone, Debug)]
pub struct Opt {
    pub text: String,
    pub next: String,
    pub conds: Vec<Cond>,
}

#[derive(Clone, Debug, Default)]
pub struct Node {
    pub text: String,
    pub actions: Vec<Vec<String>>,
    pub options: Vec<Opt>,
}

#[derive(Clone, Debug, Default)]
pub struct Conversation {
    pub name: String,
    pub nodes: HashMap<String, Node>,
}

pub struct DialogueDb {
    pub convs: Vec<Conversation>,
}

pub fn item_kind_by_name(s: &str) -> Option<ItemKind> {
    // Parse a variant name using its Debug representation.
    ALL_ITEM_KINDS.iter().copied().find(|k| format!("{:?}", k) == s)
}

pub const ALL_ITEM_KINDS: &[ItemKind] = {
    use ItemKind::*;
    &[
        Dagger, Shortsword, Longsword, Broadsword, HandAxe, Axe, Cudgel, Mace, JeweledSword, Sling, Crossbow, SlingStone, Bolt,
        LeatherCap, ChainCowl, PlateHelm, LeatherVest, ChainShirt, Breastplate, LeatherLeggings, ChainLeggings, PlateLeggings,
        LeatherBoots, PlateBoots, LeatherGloves, Gauntlets, Buckler, WoodenShield, TowerShield, GoldRing, SilverRing, Apple,
        Bread, Cheese, Meat, Fish, Mushroom, WaterFlask, Wine, RedPotion, BluePotion, GreenPotion, YellowPotion, Torch, Candle,
        Lantern, Bag, Pack, Box, RuneBag, IronKey, GoldKey, SilverKey, RuneStone, Wand, Coin, RedGem, BlueGem, GreenGem,
        WhiteGem, GoldNugget, Scroll, Book, Map, Bedroll, Rock, Bone, Skull, Lockpick, Rope, Pole, Shovel, ShardOfValor,
        ShardOfWisdom, ShardOfMercy, SilverSapling, Medallion, Hammer, LizardEgg, RelicBones, Barrel, Table, Chest, Fountain, BonePile, Plant, Shrine, Lever,
        Campfire, Bed, Brazier, Statue, Bookshelf, Pillar, Gravestone, Cauldron, MoonOrb, Corpse, Blood, Stalagmite, Web,
    ]
};

fn parse_cond(s: &str) -> Option<Cond> {
    let s = s.trim();
    if let Some(rest) = s.strip_prefix("!flag:") {
        return Some(Cond::NoFlag(rest.to_string()));
    }
    if let Some(rest) = s.strip_prefix("flag:") {
        if let Some((n, v)) = rest.split_once(">=") {
            return Some(Cond::FlagGe(n.to_string(), v.trim().parse().ok()?));
        }
        if let Some((n, v)) = rest.split_once('=') {
            return Some(Cond::FlagEq(n.to_string(), v.trim().parse().ok()?));
        }
        return Some(Cond::FlagGe(rest.to_string(), 1));
    }
    if let Some(rest) = s.strip_prefix("!item:") {
        return item_kind_by_name(rest).map(Cond::NoItem);
    }
    if let Some(rest) = s.strip_prefix("item:") {
        return item_kind_by_name(rest).map(Cond::HasItem);
    }
    if let Some(rest) = s.strip_prefix("gold>=") {
        return Some(Cond::GoldGe(rest.trim().parse().ok()?));
    }
    None
}

impl DialogueDb {
    pub fn parse(src: &str) -> Result<DialogueDb, String> {
        let mut convs: Vec<Conversation> = Vec::new();
        let mut cur_node: Option<String> = None;
        for (ln, raw) in src.lines().enumerate() {
            let line = raw.trim_end();
            let t = line.trim();
            if t.starts_with("//") {
                continue;
            }
            if let Some(name) = t.strip_prefix("===") {
                convs.push(Conversation { name: name.trim().to_string(), nodes: HashMap::new() });
                cur_node = None;
                continue;
            }
            let conv = match convs.last_mut() {
                Some(c) => c,
                None => {
                    if t.is_empty() {
                        continue;
                    }
                    return Err(format!("line {}: text outside a conversation", ln + 1));
                }
            };
            if let Some(name) = t.strip_prefix("--") {
                let name = name.trim().to_string();
                conv.nodes.insert(name.clone(), Node::default());
                cur_node = Some(name);
                continue;
            }
            let Some(node_name) = cur_node.clone() else {
                if t.is_empty() {
                    continue;
                }
                return Err(format!("line {}: text outside a node", ln + 1));
            };
            let node = conv.nodes.get_mut(&node_name).unwrap();
            if let Some(a) = t.strip_prefix('!') {
                node.actions.push(a.split_whitespace().map(|s| s.to_string()).collect());
            } else if let Some(o) = t.strip_prefix('*') {
                let (body, conds) = match o.split_once(" ? ") {
                    Some((b, c)) => (b, c),
                    None => (o, ""),
                };
                let (text, next) = body.split_once("->").ok_or(format!("line {}: option without ->", ln + 1))?;
                let mut cs = Vec::new();
                for c in conds.split('&').filter(|c| !c.trim().is_empty()) {
                    cs.push(parse_cond(c).ok_or(format!("line {}: bad condition '{}'", ln + 1, c))?);
                }
                node.options.push(Opt { text: text.trim().to_string(), next: next.trim().to_string(), conds: cs });
            } else if t.is_empty() {
                if !node.text.is_empty() && !node.text.ends_with('\n') {
                    node.text.push('\n');
                }
            } else {
                if !node.text.is_empty() && !node.text.ends_with('\n') {
                    node.text.push(' ');
                }
                node.text.push_str(t);
            }
        }
        // Validate links.
        for c in &convs {
            if !c.nodes.contains_key("start") {
                return Err(format!("conversation {} has no start node", c.name));
            }
            for (nn, n) in &c.nodes {
                for o in &n.options {
                    if o.next != "end" && !c.nodes.contains_key(&o.next) {
                        return Err(format!("conversation {} node {}: unknown target {}", c.name, nn, o.next));
                    }
                }
                for a in &n.actions {
                    if a.first().map(|s| s == "give" || s == "take").unwrap_or(false) && a.get(1).and_then(|k| item_kind_by_name(k)).is_none() {
                        return Err(format!("conversation {} node {}: unknown item in {:?}", c.name, nn, a));
                    }
                }
            }
        }
        Ok(DialogueDb { convs })
    }

    pub fn load() -> DialogueDb {
        DialogueDb::parse(include_str!("../../assets/dialogue.txt")).expect("dialogue.txt is valid")
    }

    pub fn find(&self, name: &str) -> Option<u16> {
        self.convs.iter().position(|c| c.name == name).map(|i| i as u16)
    }
}

fn gold(g: &Game) -> i32 {
    fn count(items: &[Item]) -> i32 {
        items.iter().map(|i| if i.kind == ItemKind::Coin { i.qty as i32 } else { 0 } + count(&i.contents)).sum()
    }
    let pack: Vec<Item> = g.player.pack.iter().flatten().cloned().collect();
    count(&pack)
}

fn take_gold(g: &mut Game, mut n: i32) {
    fn take(items: &mut Vec<Item>, n: &mut i32) {
        for it in items.iter_mut() {
            if *n <= 0 {
                return;
            }
            if it.kind == ItemKind::Coin {
                let t = (*n).min(it.qty as i32);
                it.qty -= t as u16;
                *n -= t;
            }
            take(&mut it.contents, n);
        }
        items.retain(|i| !(i.kind == ItemKind::Coin && i.qty == 0));
    }
    for slot in g.player.pack.iter_mut() {
        if let Some(it) = slot {
            if it.kind == ItemKind::Coin {
                let t = n.min(it.qty as i32);
                it.qty -= t as u16;
                n -= t;
                if it.qty == 0 {
                    *slot = None;
                }
            } else {
                take(&mut it.contents, &mut n);
            }
        }
    }
}

pub fn cond_ok(g: &Game, c: &Cond) -> bool {
    match c {
        Cond::FlagGe(n, v) => g.flag(n) >= *v,
        Cond::FlagEq(n, v) => g.flag(n) == *v,
        Cond::NoFlag(n) => g.flag(n) == 0,
        Cond::HasItem(k) => g.player.has_item(*k),
        Cond::NoItem(k) => !g.player.has_item(*k),
        Cond::GoldGe(n) => gold(g) >= *n,
    }
}

/// Live state of a conversation in progress.
pub struct ConvState {
    pub conv: u16,
    pub npc: u32,
    pub node: String,
    /// NPC text pages (already wrapped by the UI); `page` is the current one.
    pub text: String,
    pub options: Vec<Opt>,
    pub ended: bool,
}

/// Execute a node's actions and return its visible text and options.
pub fn enter_node(g: &mut Game, db: &DialogueDb, st: &mut ConvState, node: &str) {
    let conv = &db.convs[st.conv as usize];
    if node == "end" {
        st.ended = true;
        return;
    }
    let Some(n) = conv.nodes.get(node) else {
        st.ended = true;
        return;
    };
    st.node = node.to_string();
    let name = g.player.name.clone();
    st.text = n.text.replace("{name}", &name);
    for a in &n.actions {
        run_action(g, st.npc, a, &mut st.ended);
    }
    st.options = n.options.iter().filter(|o| o.conds.iter().all(|c| cond_ok(g, c))).cloned().collect();
    // Remember that we've talked.
    let key = format!("talked_{}", conv.name);
    g.set_flag(&key, 1);
}

fn run_action(g: &mut Game, npc: u32, a: &[String], ended: &mut bool) {
    let arg = |i: usize| a.get(i).map(|s| s.as_str()).unwrap_or("");
    let num = |i: usize, d: i32| a.get(i).and_then(|s| s.parse::<i32>().ok()).unwrap_or(d);
    match arg(0) {
        "flag" => g.set_flag(arg(1), num(2, 1)),
        "give" => {
            if let Some(k) = item_kind_by_name(arg(1)) {
                let mut it = Item::new(k).data(num(2, 0) as u16);
                let qty = num(3, 1);
                if qty > 1 {
                    it.qty = qty as u16;
                }
                if k == ItemKind::Torch {
                    it.fuel = 600.0;
                }
                let desc = it.name_with_article();
                if let Some(back) = g.player.give(it) {
                    g.drop_item(back, None);
                    g.msg(format!("You receive {} (it is at your feet).", desc));
                } else {
                    g.msg(format!("You receive {}.", desc));
                }
            }
        }
        "take" => {
            if let Some(k) = item_kind_by_name(arg(1)) {
                if k == ItemKind::Coin {
                    take_gold(g, num(2, 1));
                } else {
                    for _ in 0..num(2, 1) {
                        g.player.take(k);
                    }
                }
            }
        }
        "exp" => g.gain_exp(num(1, 10)),
        "heal" => {
            let m = g.player.max_vit;
            g.heal_player(m);
            g.player.poison = 0.0;
        }
        "mana" => g.player.mana = g.player.max_mana,
        "rune" => {
            let r = num(1, 0) as usize % 24;
            g.player.runes[r] = true;
            g.msg(format!("You learn the {} rune.", super::magic::RUNE_NAMES[r]));
        }
        "skill" => {
            if let Some(s) = ALL_SKILLS.iter().find(|s| s.name().eq_ignore_ascii_case(arg(1))) {
                g.player.skills[*s as usize] += num(2, 1);
                g.msg(format!("Your {} skill improves.", s.name()));
            }
        }
        "hostile" => {
            if let Some(n) = g.level_mut().npc_mut(npc) {
                n.attitude = Attitude::Hostile;
                n.ai = AiState::Attack;
            }
            *ended = true;
        }
        "follow" => {
            if let Some(n) = g.level_mut().npc_mut(npc) {
                n.attitude = Attitude::Friendly;
                n.ai = AiState::Follow;
            }
        }
        "friendly" => {
            if let Some(n) = g.level_mut().npc_mut(npc) {
                n.attitude = Attitude::Friendly;
            }
        }
        "stay" => {
            if let Some(n) = g.level_mut().npc_mut(npc) {
                n.ai = AiState::Idle;
            }
        }
        "door" => {
            let id = num(1, 0) as u32;
            if let Some(d) = g.level_mut().doors.iter_mut().find(|d| d.id == id || d.key == id as u16) {
                d.locked = false;
                d.opening = true;
            }
        }
        "victory" => g.events.push(super::Event::Victory),
        "end" => *ended = true,
        _ => {}
    }
}

/// Texts of scrolls, books and plaques, indexed by item data.
pub fn scroll_text(id: u16) -> &'static str {
    SCROLLS.get(id as usize).copied().unwrap_or("The writing is too faded to read.")
}

pub const SCROLLS: &[&str] = &[
    // 0
    "The writing is too faded to read.",
    // 1 - the player's map scroll
    "Enscribed upon the scroll is your map.",
    // 2
    "To whoever finds this: the green goblins and the gray have warred since the Baron's colony fell. Neither will forgive the other. Trust neither, but the gray chief Rhugg can be bargained with.",
    // 3
    "Runes of power: IN means create, LOR means light. Speak them together upon the rune shelf, and darkness shall be driven back. ORT is magic, JUX is danger; together they make a bolt of force.",
    // 4
    "The Lumen Stone was broken into three shards: Valor, Wisdom and Mercy. Only when they are joined at the altar of the deep can the Shadow be bound again. - from the journal of Lord Varian",
    // 5
    "Mountain folk mining log, day 312: The spiders have overrun the lower galleries. King Thorgrim has sealed the eastern shaft. The hammer of our fathers lies beyond it still.",
    // 6
    "Ssss... the Sage keeps the green shard in the pool of thinking. Answer true and ye shall have it. - lizard scratchings, translated by a patient scholar",
    // 7
    "Here lies Sir Aldric, knight of the Lumen. He swore to guard the shard of Mercy until his bones were laid to rest in consecrated ground.",
    // 8
    "Malchor's notes: The girl's blood is noble; it will serve. Once the three shards are kept apart, the Fiend's prison weakens. In the deep sanctum I shall finish what Varian could not.",
    // 9
    "IN MANI heals the body. AN NOX cures poison. POR FLAM hurls fire. VAS means great; with it, many spells grow mighty.",
    // 10
    "A child's drawing of the sun, labelled 'what the sky looks like'. Someone was born down here.",
    // 11
    "Recipe: mushrooms from the lower caves, boiled thrice, are safe to eat. Boiled once, you will see colours that do not exist.",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_dialogue_parses() {
        let db = DialogueDb::load();
        assert!(!db.convs.is_empty());
    }

    #[test]
    fn parse_basic() {
        let src = "=== test\n-- start\nHello {name}.\nSecond line.\n! flag met 1\n* Bye -> end\n* More -> more ? flag:met>=1 & gold>=5\n-- more\nMore text.\n* Back -> start\n";
        let db = DialogueDb::parse(src).unwrap();
        let c = &db.convs[0];
        assert_eq!(c.nodes["start"].text, "Hello {name}. Second line.");
        assert_eq!(c.nodes["start"].options.len(), 2);
        assert_eq!(c.nodes["start"].options[1].conds.len(), 2);
    }

    #[test]
    fn bad_target_is_an_error() {
        let src = "=== t\n-- start\nHi\n* x -> nowhere\n";
        assert!(DialogueDb::parse(src).is_err());
    }
}
