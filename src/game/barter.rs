//! Bartering, as in the original: the other person's goods sit in the left
//! trade area and the player's offer in the right one. Goods are priced with
//! `buy_price` on their worth. What the player offers is valued with
//! `sell_price` at half its worth (the trader has to live), and never above
//! what the trader would ask for it, so buying and selling back cannot make
//! money. Coins count at face value, and the difference is paid out in gold.

use super::dialogue::{buy_price, sell_price};
use super::items::{Item, ItemKind};
use super::Game;

/// Slots in each trade area.
pub const SLOTS: usize = 4;

/// What the trader asks for one of their goods.
pub fn ask(g: &Game, it: &Item) -> i32 {
    buy_price(g, it.value().max(1))
}

/// What the trader would give for something the player offers; 0 if they
/// have no use for it.
pub fn worth(g: &Game, it: &Item) -> i32 {
    if it.kind == ItemKind::Coin {
        return it.qty as i32;
    }
    let v = it.value();
    if v <= 0 || !it.contents.is_empty() {
        return 0;
    }
    (sell_price(g, v) / 2).min(ask(g, it))
}

/// "a torch", "a torch and a lantern", "a torch, bread and a lantern".
fn list(items: &[&Item]) -> String {
    let names: Vec<String> = items.iter().map(|i| i.name_with_article()).collect();
    match names.len() {
        0 => String::new(),
        1 => names[0].clone(),
        n => format!("{} and {}", names[..n - 1].join(", "), names[n - 1]),
    }
}

fn capitalise(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().chain(c).collect()).unwrap_or_default()
}

fn goods<'a>(g: &'a Game, npc: u32, picked: &[usize]) -> Vec<&'a Item> {
    let Some(n) = g.level().npc(npc) else { return Vec::new() };
    picked.iter().filter_map(|&i| n.inventory.get(i)).collect()
}

/// An offered item the trader will not take: no value, or a container that
/// still holds things.
fn refused<'a>(g: &Game, offer: &'a [Option<Item>]) -> Option<&'a Item> {
    offer.iter().flatten().find(|it| worth(g, it) == 0)
}

fn refusal(it: &Item) -> String {
    if it.contents.is_empty() {
        format!("I have no use for {}.", it.name_with_article())
    } else {
        format!("Empty the {} first; I'll not buy what I cannot see.", it.display_name())
    }
}

/// The trader's words when the player looks at one of their goods.
pub fn describe_good(g: &Game, it: &Item) -> String {
    format!("{}, for {} gold.", capitalise(&it.name_with_article()), ask(g, it))
}

/// The trader's words when the player looks at something in their offer.
pub fn describe_offer(g: &Game, it: &Item) -> String {
    if it.kind == ItemKind::Coin {
        return format!("{}. Gold is always welcome.", capitalise(&it.name_with_article()));
    }
    match worth(g, it) {
        0 => refusal(it),
        w => format!("For {} I would give thee {} gold.", it.name_with_article(), w),
    }
}

/// "What is mine offer worth?"
pub fn appraise(g: &Game, npc: u32, picked: &[usize], offer: &[Option<Item>]) -> String {
    let chosen = goods(g, npc, picked);
    let ask_total: i32 = chosen.iter().map(|i| ask(g, i)).sum();
    let offered: i32 = offer.iter().flatten().map(|i| worth(g, i)).sum();
    let mut s = if offer.iter().all(|o| o.is_none()) { "Thou hast offered me nothing yet.".to_string() } else { format!("Thine offer is worth {} gold to me.", offered) };
    if !chosen.is_empty() {
        s += &format!(" For {} I ask {} gold.", list(&chosen), ask_total);
    }
    if let Some(it) = refused(g, offer) {
        s += " ";
        s += &refusal(it);
    }
    s
}

/// "Let us trade." Swaps the chosen goods for the offer if it is enough,
/// pays out the difference in gold, and returns the trader's reply.
pub fn trade(g: &mut Game, npc: u32, picked: &mut Vec<usize>, offer: &mut [Option<Item>]) -> String {
    let chosen: Vec<Item> = goods(g, npc, picked).into_iter().cloned().collect();
    let has_offer = offer.iter().any(|o| o.is_some());
    if chosen.is_empty() && !has_offer {
        return "Choose what thou wouldst have from my goods, and lay thine offer beside them.".to_string();
    }
    if let Some(it) = refused(g, offer) {
        return refusal(it);
    }
    let ask_total: i32 = chosen.iter().map(|i| ask(g, i)).sum();
    let offered: i32 = offer.iter().flatten().map(|i| worth(g, i)).sum();
    if !has_offer {
        return format!("For {} I ask {} gold. Lay thine offer on the table.", list(&chosen.iter().collect::<Vec<_>>()), ask_total);
    }
    if offered < ask_total {
        return format!("For {} I ask {} gold, and thine offer is worth but {}. Thou art {} short.", list(&chosen.iter().collect::<Vec<_>>()), ask_total, offered, ask_total - offered);
    }
    // The deal stands: goods change hands.
    let sold: Vec<Item> = offer.iter_mut().filter_map(|o| o.take()).filter(|i| i.kind != ItemKind::Coin).collect();
    let sold_names = list(&sold.iter().collect::<Vec<_>>());
    if let Some(n) = g.level_mut().npc_mut(npc) {
        let mut idx = picked.clone();
        idx.sort_unstable();
        for &i in idx.iter().rev() {
            if i < n.inventory.len() {
                n.inventory.remove(i);
            }
        }
        n.inventory.extend(sold);
    }
    picked.clear();
    for it in chosen.iter() {
        receive(g, it.clone());
    }
    let change = offered - ask_total;
    if change > 0 {
        receive(g, Item::new(ItemKind::Coin).qty(change as u16));
    }
    let bought = list(&chosen.iter().collect::<Vec<_>>());
    match (chosen.is_empty(), change > 0) {
        (true, _) => format!("Done! Here are {} gold coins for {}.", change, sold_names),
        (false, true) => format!("Done! Take {}, and {} gold in change.", bought, change),
        (false, false) => format!("Done! {} is thine.", capitalise(&bought)),
    }
}

/// Give an item to the player, or drop it at their feet if they cannot
/// carry it.
fn receive(g: &mut Game, it: Item) {
    let desc = it.name_with_article();
    if let Some(back) = g.player.give(it) {
        g.drop_item(back, None);
        g.msg(format!("You receive {} (it is at your feet).", desc));
    } else {
        g.msg(format!("You receive {}.", desc));
    }
}

/// Hand back whatever is still on the player's side of the table.
pub fn return_offer(g: &mut Game, offer: &mut [Option<Item>]) {
    for it in offer.iter_mut().filter_map(|o| o.take()) {
        if let Some(back) = g.player.give(it) {
            let desc = back.name_with_article();
            g.drop_item(back, None);
            g.msg(format!("You cannot carry {}; it is at your feet.", desc));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::items::Skill;
    use crate::game::state::{PClass, Player};

    fn tobin(g: &Game) -> u32 {
        g.level().npcs.iter().find(|n| n.name.as_deref() == Some("Tobin")).expect("Tobin is on level 1").id
    }

    fn gold(g: &Game) -> i32 {
        g.player.pack.iter().flatten().filter(|i| i.kind == ItemKind::Coin).map(|i| i.qty as i32).sum()
    }

    #[test]
    fn buy_with_change_and_sell() {
        let mut g = Game::new(Player::new("T", PClass::Fighter, false), 1);
        g.player.pack.iter_mut().for_each(|s| *s = None);
        let id = tobin(&g);
        let stock = g.level().npc(id).unwrap().inventory.clone();
        let torch = stock.iter().position(|i| i.kind == ItemKind::Torch).unwrap();
        let price = ask(&g, &stock[torch]);
        let mut picked = vec![torch];
        // Not enough gold on the table.
        let mut offer: [Option<Item>; SLOTS] = [Some(Item::new(ItemKind::Coin).qty(price as u16 - 1)), None, None, None];
        assert!(trade(&mut g, id, &mut picked, &mut offer).contains("short"));
        assert!(offer[0].is_some());
        // Enough, with 7 gold too many.
        offer[0] = Some(Item::new(ItemKind::Coin).qty(price as u16 + 7));
        let reply = trade(&mut g, id, &mut picked, &mut offer);
        assert!(reply.contains("7 gold in change"), "{}", reply);
        assert!(g.player.has_item(ItemKind::Torch));
        assert_eq!(gold(&g), 7);
        assert!(offer.iter().all(|o| o.is_none()) && picked.is_empty());
        assert_eq!(g.level().npc(id).unwrap().inventory.len(), stock.len() - 1);
        // Sell a gem for gold; it joins the trader's goods.
        let gem = Item::new(ItemKind::RedGem);
        let w = worth(&g, &gem);
        assert!(w > 0 && w < ask(&g, &gem));
        let mut offer: [Option<Item>; SLOTS] = [Some(gem), None, None, None];
        trade(&mut g, id, &mut Vec::new(), &mut offer);
        assert_eq!(gold(&g), 7 + w);
        assert_eq!(g.level().npc(id).unwrap().inventory.last().unwrap().kind, ItemKind::RedGem);
    }

    #[test]
    fn refuses_worthless_things_and_cannot_be_milked() {
        let mut g = Game::new(Player::new("T", PClass::Fighter, false), 1);
        let id = tobin(&g);
        let mut offer: [Option<Item>; SLOTS] = [Some(Item::new(ItemKind::Rock)), None, None, None];
        assert!(trade(&mut g, id, &mut Vec::new(), &mut offer).contains("no use"));
        assert!(offer[0].is_some());
        let mut offer: [Option<Item>; SLOTS] = [Some(Item::new(ItemKind::Bag).with(vec![Item::new(ItemKind::Apple)])), None, None, None];
        assert!(trade(&mut g, id, &mut Vec::new(), &mut offer).contains("Empty the bag"));
        // Even a master haggler never gets more for a thing than it costs.
        g.player.skills[Skill::Charm as usize] = 30;
        g.player.skills[Skill::Appraise as usize] = 30;
        for &k in crate::game::dialogue::ALL_ITEM_KINDS {
            let it = Item::new(k);
            assert!(worth(&g, &it) <= ask(&g, &it), "{:?}", k);
        }
    }
}
