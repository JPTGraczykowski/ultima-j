//! Bartering, as in the original: the other person's goods sit in the left
//! trade area and the player's offer in the right one. Goods are priced with
//! `buy_price` on their worth. What the player offers is valued with
//! `sell_price` at half its worth (the trader has to live), and never above
//! what the trader would ask for it, so buying and selling back cannot make
//! money. Coins count at face value, and the difference is paid out in gold.

use super::dialogue::{buy_price, sell_price};
use super::items::{Item, ItemKind};
use super::Game;
use crate::lang::{self, gold, Case};

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
fn list(items: &[&Item], case: Case) -> String {
    lang::list(&items.iter().map(|i| i.noun()).collect::<Vec<_>>(), case)
}

fn capitalise(s: &str) -> String {
    lang::cap(s)
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
        t!("I have no use for {}.", it.noun())
    } else {
        t!("Empty the {} first; I'll not buy what I cannot see.", it.bare())
    }
}

/// The trader's words when the player looks at one of their goods.
pub fn describe_good(g: &Game, it: &Item) -> String {
    let mut n = it.noun();
    n.en = capitalise(&n.en);
    t!("{}, for {}.", n, gold(ask(g, it)))
}

/// The trader's words when the player looks at something in their offer.
pub fn describe_offer(g: &Game, it: &Item) -> String {
    if it.kind == ItemKind::Coin {
        let mut n = it.noun();
        n.en = capitalise(&n.en);
        return t!("{}. Gold is always welcome.", n);
    }
    match worth(g, it) {
        0 => refusal(it),
        w => t!("For {} I would give thee {}.", it.noun(), gold(w)),
    }
}

/// "What is mine offer worth?"
pub fn appraise(g: &Game, npc: u32, picked: &[usize], offer: &[Option<Item>]) -> String {
    let chosen = goods(g, npc, picked);
    let ask_total: i32 = chosen.iter().map(|i| ask(g, i)).sum();
    let offered: i32 = offer.iter().flatten().map(|i| worth(g, i)).sum();
    let mut s = if offer.iter().all(|o| o.is_none()) { t!("Thou hast offered me nothing yet.").to_string() } else { t!("Thine offer is worth {} to me.", gold(offered)) };
    if !chosen.is_empty() {
        s += &t!(" For {} I ask {}.", list(&chosen, Case::Acc), gold(ask_total));
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
        return t!("Choose what thou wouldst have from my goods, and lay thine offer beside them.").to_string();
    }
    if let Some(it) = refused(g, offer) {
        return refusal(it);
    }
    let ask_total: i32 = chosen.iter().map(|i| ask(g, i)).sum();
    let offered: i32 = offer.iter().flatten().map(|i| worth(g, i)).sum();
    if !has_offer {
        return t!("For {} I ask {}. Lay thine offer on the table.", list(&chosen.iter().collect::<Vec<_>>(), Case::Acc), gold(ask_total));
    }
    if offered < ask_total {
        return t!("For {} I ask {}, and thine offer is worth but {}. Thou art {} short.", list(&chosen.iter().collect::<Vec<_>>(), Case::Acc), gold(ask_total), offered, ask_total - offered);
    }
    // The deal stands: goods change hands.
    let sold: Vec<Item> = offer.iter_mut().filter_map(|o| o.take()).filter(|i| i.kind != ItemKind::Coin).collect();
    let sold_names = list(&sold.iter().collect::<Vec<_>>(), Case::Acc);
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
    let chosen: Vec<&Item> = chosen.iter().collect();
    match (chosen.is_empty(), change > 0) {
        (true, _) => t!("Done! Here are {} gold coins for {}.", change, sold_names, gold(change)),
        (false, true) => t!("Done! Take {}, and {} gold in change.", list(&chosen, Case::Acc), change, gold(change)),
        (false, false) => t!("Done! {} is thine.", capitalise(&list(&chosen, Case::Nom))),
    }
}

/// Give an item to the player, or drop it at their feet if they cannot
/// carry it.
fn receive(g: &mut Game, it: Item) {
    let desc = it.noun();
    if let Some(back) = g.player.give(it) {
        g.drop_item(back, None);
        g.msg(t!("You receive {} (it is at your feet).", desc));
    } else {
        g.msg(t!("You receive {}.", desc));
    }
}

/// Hand back whatever is still on the player's side of the table.
pub fn return_offer(g: &mut Game, offer: &mut [Option<Item>]) {
    for it in offer.iter_mut().filter_map(|o| o.take()) {
        if let Some(back) = g.player.give(it) {
            let desc = back.noun();
            g.drop_item(back, None);
            g.msg(t!("You cannot carry {}; it is at your feet.", desc));
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
