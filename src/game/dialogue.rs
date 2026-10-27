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
//! `item:Kind>=N` (carrying at least N), `!item:Kind`, `gold>=N`, `afford:N`,
//! `skill:NAME>=N`. Actions: `flag NAME N`, `flag+ NAME [N]` (add N, default 1),
//! `give Kind [data] [qty]`, `take Kind [qty]`, `pay N`, `earn N`, `exp N`,
//! `hostile`, `heal`, `rune N`, `follow`, `mana`, `skill NAME N`, `mend`,
//! `barter`, `end`.
//!
//! Prices: `{buy N}` and `{sell N}` in text show a base price of N gold as
//! adjusted by the player's Charm and Appraise; `afford:N`, `pay N` and
//! `earn N` use the same adjusted prices.
//!
//! Bartering: a node with `! barter` opens the trade areas (the NPC's
//! `inventory` is their stock, see `barter.rs`) and adds the trading options
//! before its own.

use super::creatures::{AiState, Attitude};
use super::items::{Class, Item, ItemKind, Skill, ALL_SKILLS};
use super::Game;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Cond {
    FlagGe(String, i32),
    FlagEq(String, i32),
    NoFlag(String),
    HasItem(ItemKind),
    /// Carrying at least N of a kind.
    ItemGe(ItemKind, i32),
    NoItem(ItemKind),
    GoldGe(i32),
    /// Enough gold for a base price after haggling.
    Afford(i32),
    SkillGe(Skill, i32),
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
        ShardOfWisdom, ShardOfMercy, SilverSapling, Medallion, Hammer, LizardEgg, RelicBones, ChroniclePage, Barrel, Table, Chest, Fountain, BonePile, Plant, Shrine, Lever,
        Campfire, Bed, Brazier, Statue, Bookshelf, Pillar, Gravestone, Cauldron, MoonOrb, Corpse, Blood, Stalagmite, Web, Ward,
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
        if let Some((k, n)) = rest.split_once(">=") {
            return Some(Cond::ItemGe(item_kind_by_name(k.trim())?, n.trim().parse().ok()?));
        }
        return item_kind_by_name(rest).map(Cond::HasItem);
    }
    if let Some(rest) = s.strip_prefix("gold>=") {
        return Some(Cond::GoldGe(rest.trim().parse().ok()?));
    }
    if let Some(rest) = s.strip_prefix("afford:") {
        return Some(Cond::Afford(rest.trim().parse().ok()?));
    }
    if let Some(rest) = s.strip_prefix("skill:") {
        let (n, v) = rest.split_once(">=")?;
        let sk = ALL_SKILLS.iter().find(|k| k.name().eq_ignore_ascii_case(n.trim()))?;
        return Some(Cond::SkillGe(*sk, v.trim().parse().ok()?));
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

    /// The conversations in the current language.
    pub fn load() -> DialogueDb {
        DialogueDb::load_lang(crate::lang::get())
    }

    pub fn load_lang(l: crate::lang::Lang) -> DialogueDb {
        match l {
            crate::lang::Lang::En => DialogueDb::parse(include_str!("../../assets/dialogue.txt")).expect("dialogue.txt is valid"),
            crate::lang::Lang::Pl => DialogueDb::parse(include_str!("../../assets/dialogue.pl.txt")).expect("dialogue.pl.txt is valid"),
        }
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
        Cond::ItemGe(k, n) => g.player.count_item(*k) >= *n,
        Cond::NoItem(k) => !g.player.has_item(*k),
        Cond::GoldGe(n) => gold(g) >= *n,
        Cond::Afford(n) => gold(g) >= buy_price(g, *n),
        Cond::SkillGe(s, n) => g.player.skill(*s) >= *n,
    }
}

/// What the player pays for something with a base price: Charm talks the
/// price down, Appraise knows what it is really worth.
pub fn buy_price(g: &Game, base: i32) -> i32 {
    let off = (g.player.skill(Skill::Charm) * 2 + g.player.skill(Skill::Appraise)).min(35);
    ((base * (100 - off) + 50) / 100).max(1)
}

/// What the player is paid for something with a base price.
pub fn sell_price(g: &Game, base: i32) -> i32 {
    let up = (g.player.skill(Skill::Appraise) * 3 + g.player.skill(Skill::Charm)).min(50);
    (base * (100 + up) + 50) / 100
}

/// Replace `{buy N}` and `{sell N}` with adjusted prices.
pub fn fill_prices(g: &Game, s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(k) = rest.find('{') {
        out.push_str(&rest[..k]);
        let tail = &rest[k..];
        let end = tail.find('}').unwrap_or(tail.len() - 1);
        let inner = &tail[1..end];
        let price = match inner.split_once(' ') {
            Some(("buy", n)) => n.trim().parse().ok().map(|n| buy_price(g, n)),
            Some(("sell", n)) => n.trim().parse().ok().map(|n| sell_price(g, n)),
            _ => None,
        };
        match price {
            Some(p) => out.push_str(&p.to_string()),
            None => out.push_str(&tail[..=end]),
        }
        rest = &tail[end + 1..];
    }
    out.push_str(rest);
    out
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
    /// The trade areas are open (a `barter` node has been reached).
    pub trading: bool,
    /// Indices of the NPC's goods the player has chosen.
    pub picked: Vec<usize>,
    /// The player's side of the table.
    pub offer: [Option<Item>; super::barter::SLOTS],
    /// Which four of the NPC's goods are shown.
    pub goods_page: usize,
}

impl ConvState {
    pub fn new(conv: u16, npc: u32) -> ConvState {
        ConvState { conv, npc, node: String::new(), text: String::new(), options: Vec::new(), ended: false, trading: false, picked: Vec::new(), offer: Default::default(), goods_page: 0 }
    }
}

/// Targets of the built-in trading options.
pub const TRADE: &str = "@trade";
pub const APPRAISE: &str = "@appraise";
pub const MORE_GOODS: &str = "@more";

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
    st.text = fill_prices(g, &crate::lang::by_sex(&n.text.replace("{name}", &name), g.player.female));
    for a in &n.actions {
        run_action(g, st.npc, a, &mut st.ended);
    }
    if n.actions.iter().any(|a| a.first().map(|s| s == "barter").unwrap_or(false)) {
        st.trading = true;
    }
    refresh_options(g, db, st);
    // Remember that we've talked.
    let key = format!("talked_{}", conv.name);
    g.set_flag(&key, 1);
}

/// Recompute the options shown for the current node.
pub fn refresh_options(g: &Game, db: &DialogueDb, st: &mut ConvState) {
    let Some(n) = db.convs[st.conv as usize].nodes.get(&st.node) else { return };
    let opt = |text: &str, next: &str| Opt { text: text.to_string(), next: next.to_string(), conds: Vec::new() };
    st.options.clear();
    if n.actions.iter().any(|a| a.first().map(|s| s == "barter").unwrap_or(false)) {
        st.options.push(opt(t!("Let us trade."), TRADE));
        st.options.push(opt(t!("What is mine offer worth to thee?"), APPRAISE));
        let stock = g.level().npc(st.npc).map(|n| n.inventory.len()).unwrap_or(0);
        if stock > super::barter::SLOTS {
            st.options.push(opt(t!("Show me thy other goods."), MORE_GOODS));
        }
    }
    st.options.extend(n.options.iter().filter(|o| o.conds.iter().all(|c| cond_ok(g, c))).map(|o| Opt { text: fill_prices(g, &crate::lang::by_sex(&o.text, g.player.female)), ..o.clone() }));
}

/// Choose one of the built-in trading options; the reply becomes the NPC's
/// text and the node's options stay.
pub fn barter_choice(g: &mut Game, db: &DialogueDb, st: &mut ConvState, what: &str) {
    use super::barter;
    st.text = match what {
        TRADE => barter::trade(g, st.npc, &mut st.picked, &mut st.offer),
        APPRAISE => barter::appraise(g, st.npc, &st.picked, &st.offer),
        _ => {
            let stock = g.level().npc(st.npc).map(|n| n.inventory.len()).unwrap_or(0);
            let pages = stock.div_ceil(barter::SLOTS).max(1);
            st.goods_page = (st.goods_page + 1) % pages;
            t!("Have a look at these, then.").to_string()
        }
    };
    let stock = g.level().npc(st.npc).map(|n| n.inventory.len()).unwrap_or(0);
    if st.goods_page * barter::SLOTS >= stock {
        st.goods_page = 0;
    }
    refresh_options(g, db, st);
}

fn run_action(g: &mut Game, npc: u32, a: &[String], ended: &mut bool) {
    let arg = |i: usize| a.get(i).map(|s| s.as_str()).unwrap_or("");
    let num = |i: usize, d: i32| a.get(i).and_then(|s| s.parse::<i32>().ok()).unwrap_or(d);
    match arg(0) {
        "flag" => g.set_flag(arg(1), num(2, 1)),
        "flag+" => g.set_flag(arg(1), g.flag(arg(1)) + num(2, 1)),
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
                let desc = it.noun();
                if let Some(back) = g.player.give(it) {
                    g.drop_item(back, None);
                    g.msg(t!("You receive {} (it is at your feet).", desc));
                } else {
                    g.msg(t!("You receive {}.", desc));
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
        "pay" => take_gold(g, buy_price(g, num(1, 0))),
        "earn" => {
            let n = sell_price(g, num(1, 0));
            let coins = Item::new(ItemKind::Coin).qty(n as u16);
            if let Some(back) = g.player.give(coins) {
                g.drop_item(back, None);
            }
            g.msg(t!("You receive {}.", crate::lang::gold(n)));
        }
        "mend" => {
            for s in super::ALL_EQ {
                if let Some(it) = g.player.equip.get_mut(s).as_mut() {
                    if matches!(it.def().class, Class::Weapon | Class::Missile | Class::Armor) {
                        it.quality = 100;
                    }
                }
            }
            for it in g.player.pack.iter_mut().flatten() {
                if matches!(it.def().class, Class::Weapon | Class::Missile | Class::Armor) {
                    it.quality = 100;
                }
            }
            g.msg(t!("Your weapons and armour are as good as new."));
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
            g.msg(t!("You learn the {} rune.", super::magic::RUNE_NAMES[r]));
        }
        "skill" => {
            if let Some(s) = ALL_SKILLS.iter().find(|s| s.name().eq_ignore_ascii_case(arg(1))) {
                g.player.skills[*s as usize] += num(2, 1);
                g.msg(t!("Your {} skill improves.", s.label()));
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
    crate::lang::tr(SCROLLS.get(id as usize).copied().unwrap_or(SCROLLS[0]))
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
    "The Lumen Stone was broken into three shards: Valor, Wisdom and Mercy. I cannot make it whole. I go down to the Throne of Souls, to keep the dead of the Abyss until another can bear the light. - from the journal of Lord Varian",
    // 5
    "Mountain folk mining log, day 312: The spiders have overrun the lower galleries. King Thorgrim has sealed the eastern shaft. The hammer of our fathers lies beyond it still.",
    // 6
    "Ssss... the Sage keeps the green shard in the pool of thinking. Answer true and ye shall have it. - lizard scratchings, translated by a patient scholar",
    // 7
    "Here lies Sir Aldric, knight of the Lumen. He swore to guard the shard of Mercy until his bones were laid to rest in consecrated ground.",
    // 8
    "Malchor's notes: Varian weakens. The gate to his Throne burns me, but it will open to the three shards in the hands of one of his own knights - and the peoples would give their shards to a knight, never to me. The one who took my blast lies in the old graves by the doors. Wake it. Speak to it in Varian's voice. Wait by the gate.",
    // 9
    "IN MANI heals the body. AN NOX cures poison. POR FLAM hurls fire. VAS means great; with it, many spells grow mighty.",
    // 10
    "A child's drawing of the sun, labelled 'what the sky looks like'. Someone was born down here.",
    // 11
    "Recipe: mushrooms from the lower caves, boiled thrice, are safe to eat. Boiled once, you will see colours that do not exist.",
    // 12
    "A list in Malchor's hand, every line crossed out but the last: 'Aldric - the bones taken, the ghost bound. Corwin - drunk. Elsmore - too thin to bother. The knight who took the blast - risen, and walking. Save for last.'",
];

/// The pages of the Chronicle of the Colony, one on each level, indexed by
/// the page's data.
pub fn chronicle_text(page: u16) -> &'static str {
    crate::lang::tr(CHRONICLE[page as usize % CHRONICLE.len()])
}

pub const CHRONICLE: &[&str] = &[
    "The Chronicle of the Colony, page the first. In the fortieth year of his lordship, Lord Varian led three hundred souls through the great doors and down into the Abyss, the Lumen Stone carried before them. Where its light fell the rock grew warm and the mushrooms sweet. Here, he said, no one shall be cold, or hungry, or alone. - Maera, chronicler",
    "The Chronicle of the Colony, page the second. The mountain folk dug the halls, the lizard-folk brought fish up from the black waters, the goblins herded the cave beasts, and men kept the peace. Four peoples, one light. In the tenth year a goblin and a dwarf were wed in the great hall, and Lord Varian himself gave away the bride. - Maera",
    "The Chronicle of the Colony, page the third. The court mage Malchor asks again to study the Stone, and again Lord Varian refuses him. Tonight at table Malchor said that a light kept in one hand is a light kept from every other, and the hall fell silent. Sir Aldric and the young knight at our lord's left hand watched him until he was gone. - Maera",
    "The Chronicle of the Colony, page the fourth. The Stone is broken. Malchor struck at it in the night, and the knight at Lord Varian's left threw themselves into the blast. We buried them by the great doors, and the doors closed behind us all. Each people took a shard and turned on the others. Our lord has gone down below the crypts. He did not say if he would return. - Maera",
    "The Chronicle of the Colony, the last page. I followed my lord down to write the end of his story, and found Malchor's hollows instead. They do not sleep. Whoever reads this: the dead you see walking were our people. Remember them by their names, if you can. Mine was Maera.",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_dialogue_parses() {
        let db = DialogueDb::load();
        assert!(!db.convs.is_empty());
    }

    /// The Polish conversations are the English ones with other words: the
    /// same conversations in the same order (saves keep their index), the
    /// same nodes, actions, options, targets and conditions.
    #[test]
    fn polish_dialogue_matches_english() {
        let en = DialogueDb::load_lang(crate::lang::Lang::En);
        let pl = DialogueDb::load_lang(crate::lang::Lang::Pl);
        assert_eq!(en.convs.iter().map(|c| &c.name).collect::<Vec<_>>(), pl.convs.iter().map(|c| &c.name).collect::<Vec<_>>());
        for (a, b) in en.convs.iter().zip(&pl.convs) {
            let mut names: Vec<&String> = a.nodes.keys().collect();
            names.sort();
            let mut pl_names: Vec<&String> = b.nodes.keys().collect();
            pl_names.sort();
            assert_eq!(names, pl_names, "conversation {}", a.name);
            for n in names {
                let (x, y) = (&a.nodes[n], &b.nodes[n]);
                let at = format!("{} / {}", a.name, n);
                assert_eq!(x.actions, y.actions, "{}", at);
                assert_eq!(x.options.len(), y.options.len(), "{}", at);
                for (o, p) in x.options.iter().zip(&y.options) {
                    assert_eq!(o.next, p.next, "{}", at);
                    assert_eq!(format!("{:?}", o.conds), format!("{:?}", p.conds), "{}", at);
                }
                assert!(!y.text.is_empty() || x.text.is_empty(), "{} has no Polish text", at);
                for (key, _) in [("{name}", 0), ("{buy", 0), ("{sell", 0)] {
                    assert_eq!(x.text.matches(key).count(), y.text.matches(key).count(), "{} uses {} differently", at, key);
                }
            }
        }
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
    fn charm_and_appraise_haggle() {
        use super::super::state::{PClass, Player};
        let mut g = Game::new(Player::new("T", PClass::Fighter, false), 1);
        assert_eq!(buy_price(&g, 20), 20);
        assert_eq!(sell_price(&g, 30), 30);
        assert_eq!(fill_prices(&g, "A potion, {buy 20} gold. {name}"), "A potion, 20 gold. {name}");
        g.player.skills[Skill::Charm as usize] = 5;
        g.player.skills[Skill::Appraise as usize] = 5;
        assert_eq!(buy_price(&g, 20), 17);
        assert_eq!(sell_price(&g, 30), 36);
        assert!(cond_ok(&g, &parse_cond("skill:Charm>=5").unwrap()));
        assert!(!cond_ok(&g, &parse_cond("skill:charm>=6").unwrap()));
        assert!(!cond_ok(&g, &parse_cond("afford:1").unwrap()));
    }

    /// Whether two conditions can never hold together.
    fn exclusive(a: &Cond, b: &Cond) -> bool {
        use Cond::*;
        let one_way = |a: &Cond, b: &Cond| match (a, b) {
            (NoFlag(n), FlagGe(m, v)) => n == m && *v >= 1,
            (NoFlag(n), FlagEq(m, v)) => n == m && *v != 0,
            (FlagEq(n, x), FlagEq(m, y)) => n == m && x != y,
            (FlagEq(n, x), FlagGe(m, y)) => n == m && x < y,
            (NoItem(k), HasItem(j)) | (NoItem(k), ItemGe(j, _)) => k == j,
            _ => false,
        };
        one_way(a, b) || one_way(b, a)
    }

    /// The conversation menu shows 5 choices. No node may offer more at
    /// once, counting the trading options a barter node adds: options whose
    /// conditions exclude each other are never shown together.
    #[test]
    fn at_most_five_visible_choices() {
        for l in [crate::lang::Lang::En, crate::lang::Lang::Pl] {
            let db = DialogueDb::load_lang(l);
            for c in &db.convs {
                for (name, n) in &c.nodes {
                    let opts = &n.options;
                    let together = |i: usize, j: usize| !opts[i].conds.iter().any(|a| opts[j].conds.iter().any(|b| exclusive(a, b)));
                    let mut most = 0;
                    for set in 0u32..1 << opts.len() {
                        let members: Vec<usize> = (0..opts.len()).filter(|i| set & 1 << i != 0).collect();
                        if members.len() > most && members.iter().all(|&i| members.iter().all(|&j| i == j || together(i, j))) {
                            most = members.len();
                        }
                    }
                    // Let us trade, what is it worth, and other goods.
                    let trading = if n.actions.iter().any(|a| a[0] == "barter") { 3 } else { 0 };
                    assert!(most + trading <= 5, "{} / {} can show {} choices", c.name, name, most + trading);
                }
            }
        }
    }

    #[test]
    fn counted_items_and_added_flags() {
        use super::super::state::{PClass, Player};
        let mut g = Game::new(Player::new("T", PClass::Fighter, false), 1);
        let two = parse_cond("item:Mushroom>=2").unwrap();
        g.player.give(Item::new(ItemKind::Mushroom));
        assert!(cond_ok(&g, &parse_cond("item:Mushroom").unwrap()));
        assert!(!cond_ok(&g, &two));
        g.player.give(Item::new(ItemKind::Mushroom));
        assert!(cond_ok(&g, &two));
        assert!(cond_ok(&g, &parse_cond("item:Coin>=0").unwrap()));
        let mut ended = false;
        let act = |s: &str| s.split_whitespace().map(String::from).collect::<Vec<_>>();
        run_action(&mut g, 0, &act("take Mushroom 2"), &mut ended);
        assert_eq!(g.player.count_item(ItemKind::Mushroom), 0);
        run_action(&mut g, 0, &act("flag+ pages"), &mut ended);
        run_action(&mut g, 0, &act("flag+ pages 2"), &mut ended);
        assert_eq!(g.flag("pages"), 3);
        assert!(parse_cond("item:Nothing>=2").is_none());
    }

    #[test]
    fn bad_target_is_an_error() {
        let src = "=== t\n-- start\nHi\n* x -> nowhere\n";
        assert!(DialogueDb::parse(src).is_err());
    }
}
