//! Player actions: interacting with the world, using items, casting spells.

use super::ai;
use super::creatures::{AiState, Attitude};
use super::items::{Class, Enchant, Item, ItemKind, Skill};
use super::magic::{Spell, Targeting};
use super::physics;
use super::{ActiveSpell, DoorKind, EffectKind, EqSlot, Event, Game, ProjKind, Projectile};
use crate::art::textures::Tex;
use crate::audio::Sfx;
use crate::util::V3;

/// What the cursor points at in the 3D view.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Target {
    None,
    Floor { tx: i32, tz: i32, tex: Tex },
    Ceiling { tx: i32, tz: i32, tex: Tex },
    Wall { tx: i32, tz: i32, tex: Tex },
    Item(u32),
    Npc(u32),
    Door(u32),
}

pub const REACH: f32 = 1.7;
/// Key id of the gate that only opens for the bearer of all three shards.
pub const SHARD_GATE: u16 = 900;

impl Game {
    pub fn has_all_shards(&self) -> bool {
        self.player.has_item(ItemKind::ShardOfValor) && self.player.has_item(ItemKind::ShardOfWisdom) && self.player.has_item(ItemKind::ShardOfMercy)
    }
}

fn cap(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

impl Game {
    pub fn look(&mut self, t: Target) {
        match t {
            Target::None => self.msg("You see nothing of interest."),
            Target::Floor { tex, .. } | Target::Ceiling { tex, .. } => {
                self.msg(format!("You see {}.", tex.describe()));
            }
            Target::Wall { tx, tz, tex } => {
                if tex == Tex::Plaque {
                    if let Some((_, _, text)) = self.level().writings.iter().find(|w| (w.0 - tx).abs() <= 1 && (w.1 - tz).abs() <= 1) {
                        let text = text.clone();
                        self.msg(format!("The writing reads: {}", text));
                        return;
                    }
                }
                self.msg(format!("You see {}.", tex.describe()));
            }
            Target::Item(id) => {
                if let Some(wi) = self.level().items.iter().find(|i| i.id == id) {
                    let it = &wi.item;
                    let s = format!("You see {}{}.", if it.qty > 1 { "" } else { "" }, describe_item(it));
                    self.msg(s);
                }
            }
            Target::Npc(id) => {
                if let Some(n) = self.level().npc(id) {
                    let s = if n.alive() {
                        let mut d = format!("You see {}.", n.describe());
                        if n.hp < n.max_hp {
                            d = format!("You see {} ({}).", n.describe(), n.health_word());
                        }
                        d
                    } else {
                        format!("You see the corpse of a {}.", n.race.def().name)
                    };
                    self.msg(s);
                }
            }
            Target::Door(id) => {
                if let Some(d) = self.level().doors.iter().find(|d| d.id == id) {
                    let s = if d.kind == DoorKind::Secret && !d.found {
                        "You see a stone wall.".to_string()
                    } else if d.is_open() {
                        format!("You see {} (open).", d.name())
                    } else {
                        format!("You see {}.", d.name())
                    };
                    self.msg(s);
                }
            }
        }
    }

    /// Try to pick up a target. Returns the item to put in the hand.
    pub fn get(&mut self, t: Target, dist: f32) -> Option<Item> {
        match t {
            Target::Item(id) => {
                if dist > REACH && !self.player.has_spell(Spell::Telekinesis) {
                    self.msg("That is too far away to take.");
                    return None;
                }
                let idx = self.level().item_index(id)?;
                let it = &self.level().items[idx].item;
                if it.kind.is_fixture() {
                    self.msg("You cannot pick that up.");
                    return None;
                }
                let wi = self.level_mut().items.remove(idx);
                self.sound(Sfx::Pickup);
                Some(wi.item)
            }
            Target::None => None,
            _ => {
                self.msg("You cannot pick that up.");
                None
            }
        }
    }

    /// Drop an item into the world near a point (clamped to the player's reach).
    pub fn drop_item(&mut self, item: Item, at: Option<V3>) {
        let eye = self.eye();
        let fwd = V3::new(self.player.yaw.sin(), 0.0, -self.player.yaw.cos());
        let mut p = at.unwrap_or(eye + fwd * 0.6);
        let d = V3::new(p.x - eye.x, 0.0, p.z - eye.z);
        if d.len() > REACH {
            let n = d.norm();
            p = V3::new(eye.x + n.x * REACH, p.y, eye.z + n.z * REACH);
        }
        // Walk the drop point back towards the player until it is in open space.
        let mut q = p;
        for _ in 0..12 {
            if self.level().map.point_open(q.x, q.z) && !physics::door_blocks(self.level(), q.x, q.z, 0.05) && physics::line_of_sight(self.level(), eye, V3::new(q.x, eye.y, q.z)) {
                break;
            }
            q = q + (V3::new(eye.x, q.y, eye.z) - q) * 0.2;
        }
        if item.kind == ItemKind::SilverSapling {
            self.set_flag("sapling_planted", 1);
            self.set_flag("sapling_level", self.cur as i32);
            self.set_flag("sapling_x", (q.x * 100.0) as i32);
            self.set_flag("sapling_z", (q.z * 100.0) as i32);
            self.msg("You plant the silver sapling. Should you fall, it will call you back.");
        }
        self.level_mut().add_item(item, q.x, q.z);
        self.sound(Sfx::Drop);
    }

    pub fn talk(&mut self, t: Target, dist: f32) {
        match t {
            Target::Npc(id) => {
                let Some(n) = self.level().npc(id) else { return };
                if !n.alive() {
                    self.msg("The dead tell no tales.");
                    return;
                }
                if dist > 3.0 {
                    self.msg("You are too far away to talk.");
                    return;
                }
                if n.talk.is_some() && n.attitude != Attitude::Hostile {
                    self.events.push(Event::StartConversation(id));
                } else if n.talk.is_some() && n.attitude == Attitude::Hostile {
                    let name = n.race.def().name;
                    self.msg(format!("The {} is in no mood to talk.", name));
                } else {
                    self.msg("You get no response.");
                }
            }
            Target::Door(_) => self.msg("The door does not reply."),
            Target::None => self.msg("There is no one there to talk to."),
            _ => self.msg("You cannot talk to that."),
        }
    }

    /// Use something in the world. `held` is the item in hand, if any (e.g. a key).
    pub fn use_world(&mut self, t: Target, dist: f32, held: Option<&Item>) {
        match t {
            Target::Door(id) => {
                if dist > REACH + 0.4 {
                    self.msg("That is too far away.");
                    return;
                }
                self.use_door(id, held);
            }
            Target::Wall { tx, tz, tex } => {
                if dist > REACH + 0.4 {
                    self.msg("That is too far away.");
                    return;
                }
                if tex == Tex::WallLever {
                    self.pull_lever(tx, tz);
                } else if tex == Tex::Plaque {
                    self.look(t);
                } else {
                    self.msg("You cannot use that.");
                }
            }
            Target::Item(id) => {
                if dist > REACH {
                    self.msg("That is too far away.");
                    return;
                }
                let Some(idx) = self.level().item_index(id) else { return };
                let kind = self.level().items[idx].item.kind;
                match kind {
                    ItemKind::Lever => {
                        let p = self.level().items[idx].pos;
                        self.pull_lever(p.x.floor() as i32, p.z.floor() as i32);
                    }
                    ItemKind::Fountain => {
                        self.heal_player(5);
                        self.player.hunger = (self.player.hunger + 3.0).min(100.0);
                        self.sound(Sfx::Drink);
                        self.msg("You drink from the fountain. The cool water refreshes you.");
                    }
                    ItemKind::Bed => self.sleep(),
                    ItemKind::Shrine => {
                        self.sound(Sfx::Spell);
                        self.heal_player(10);
                        if self.player.skill_points > 0 {
                            self.msg("You kneel at the shrine. You feel able to improve your skills (see the statistics panel).");
                        } else {
                            self.msg("You kneel at the shrine and feel at peace.");
                        }
                    }
                    ItemKind::Chest | ItemKind::Corpse | ItemKind::Box => {
                        let wi = &mut self.level_mut().items[idx];
                        let contents = std::mem::take(&mut wi.item.contents);
                        let pos = wi.pos;
                        if contents.is_empty() {
                            self.msg("It is empty.");
                        } else {
                            self.msg("You open it. Its contents spill out.");
                            for (k, it) in contents.into_iter().enumerate() {
                                let a = k as f32 * 1.7;
                                self.level_mut().add_item(it, pos.x + a.sin() * 0.25, pos.z + a.cos() * 0.25);
                            }
                            self.sound(Sfx::DoorOpen);
                        }
                    }
                    ItemKind::MoonOrb => {
                        self.events.push(Event::ReadText(
                            "Peering into the orb, you glimpse the deepest hall of the abyss: a pit of fire, and something vast stirring within it. A voice whispers: 'Bring the three shards together, and the way shall open.'".into(),
                        ));
                    }
                    ItemKind::Campfire | ItemKind::Brazier => self.msg("The fire crackles warmly."),
                    _ => {
                        if kind.is_fixture() {
                            self.msg("You cannot use that.");
                        } else {
                            self.msg("You must pick that up to use it.");
                        }
                    }
                }
            }
            Target::Npc(_) => self.msg("You cannot use that."),
            _ => self.msg("You cannot use that."),
        }
    }

    fn use_door(&mut self, id: u32, held: Option<&Item>) {
        let has_key = |g: &Game, key: u16| -> bool {
            fn find(items: &[Item], key: u16) -> bool {
                items.iter().any(|i| (matches!(i.kind, ItemKind::IronKey | ItemKind::GoldKey | ItemKind::SilverKey) && i.data == key) || find(&i.contents, key))
            }
            let pack: Vec<Item> = g.player.pack.iter().flatten().cloned().collect();
            find(&pack, key) || held.map(|h| find(std::slice::from_ref(h), key)).unwrap_or(false)
        };
        let Some(d) = self.level().doors.iter().find(|d| d.id == id).cloned() else { return };
        if d.key == SHARD_GATE && d.locked {
            if self.has_all_shards() {
                let dm = self.level_mut().doors.iter_mut().find(|x| x.id == id).unwrap();
                dm.locked = false;
                dm.opening = true;
                self.sound(Sfx::Spell);
                self.events.push(Event::Flash([255.0, 240.0, 160.0], 0.5));
                self.msg("The three shards blaze with light! The great gate grinds open.");
            } else {
                self.sound(Sfx::Locked);
                self.msg("The gate is sealed by a powerful magic. Three hollows in its surface glow faintly: red, blue and green.");
            }
            return;
        }
        if d.kind == DoorKind::Secret && !d.found {
            let chance = 30 + self.player.skill(Skill::Search) * 6 + self.player.int;
            if self.rng.range(0, 100) < chance {
                self.level_mut().doors.iter_mut().find(|x| x.id == id).unwrap().found = true;
                self.msg("You found a secret door!");
            } else {
                self.msg("You see nothing unusual about the wall.");
                return;
            }
        }
        if d.remote && !d.opening {
            self.msg("The door will not budge. Perhaps it is opened by some mechanism.");
            return;
        }
        if d.locked {
            if d.key != 0 && has_key(self, d.key) {
                let dm = self.level_mut().doors.iter_mut().find(|x| x.id == id).unwrap();
                dm.locked = false;
                dm.opening = true;
                self.sound(Sfx::DoorOpen);
                self.msg("You unlock the door with your key.");
            } else if held.map(|h| h.kind == ItemKind::Lockpick).unwrap_or(false) {
                let chance = 20 + self.player.skill(Skill::Picklock) * 6 + self.player.dex;
                if self.rng.range(0, 100) < chance {
                    self.level_mut().doors.iter_mut().find(|x| x.id == id).unwrap().locked = false;
                    self.msg("You succeed in picking the lock.");
                    self.sound(Sfx::Click);
                } else {
                    self.msg("You fail to pick the lock.");
                    self.sound(Sfx::Locked);
                }
            } else {
                self.sound(Sfx::Locked);
                self.msg("The door is locked.");
            }
            return;
        }
        let dm = self.level_mut().doors.iter_mut().find(|x| x.id == id).unwrap();
        dm.opening = !dm.opening;
        let opening = dm.opening;
        // Do not close the door on the player.
        if !opening && physics::door_blocks_point(self.level(), id, self.player.pos, physics::PLAYER_RADIUS) {
            self.level_mut().doors.iter_mut().find(|x| x.id == id).unwrap().opening = true;
            self.msg("You are in the way.");
            return;
        }
        self.sound(if opening { Sfx::DoorOpen } else { Sfx::DoorClose });
    }

    fn pull_lever(&mut self, tx: i32, tz: i32) {
        let lvl = self.cur;
        let sw = self.levels[lvl].switches.iter_mut().find(|s| (s.tx - tx).abs() <= 1 && (s.tz - tz).abs() <= 1);
        let Some(sw) = sw else {
            self.sound(Sfx::Lever);
            self.msg("You pull the lever, but nothing seems to happen.");
            return;
        };
        sw.on = !sw.on;
        let door = sw.door;
        let on = sw.on;
        self.sound(Sfx::Lever);
        if let Some(d) = self.levels[lvl].doors.iter_mut().find(|d| d.id == door) {
            d.opening = on;
            d.locked = false;
            self.sound(if on { Sfx::DoorOpen } else { Sfx::DoorClose });
            self.msg("You pull the lever. You hear a grinding noise.");
        }
    }

    /// Use an item from the inventory. Returns the item if it still exists.
    pub fn use_item(&mut self, mut item: Item, equipped_slot: Option<EqSlot>) -> Option<Item> {
        let def = item.def();
        match def.class {
            Class::Food => {
                if self.player.hunger > 92.0 {
                    self.msg("You are too full to eat that now.");
                    return Some(item);
                }
                self.player.hunger = (self.player.hunger + def.nutrition as f32).min(100.0);
                self.sound(Sfx::Eat);
                let taste = match item.kind {
                    ItemKind::Mushroom => "That mushroom tasted rather odd.",
                    ItemKind::Meat => "That meat was tough, but filling.",
                    _ => "That was delicious.",
                };
                self.msg(taste);
                if item.kind == ItemKind::Mushroom && self.rng.chance(0.3) {
                    self.msg("You feel a bit light-headed.");
                }
                consume(item)
            }
            Class::Drink => {
                self.player.hunger = (self.player.hunger + def.nutrition as f32).min(100.0);
                self.sound(Sfx::Drink);
                if item.kind == ItemKind::Wine {
                    self.msg("The wine warms you. You feel a little drunk.");
                    self.heal_player(2);
                } else {
                    self.msg("You drink the water. It is refreshing.");
                    self.player.fatigue = (self.player.fatigue - 5.0).max(0.0);
                }
                consume(item)
            }
            Class::Potion => {
                self.sound(Sfx::Drink);
                match item.kind {
                    ItemKind::RedPotion => {
                        let amt = self.rng.range(12, 24);
                        self.heal_player(amt);
                        self.msg("You feel much better.");
                    }
                    ItemKind::BluePotion => {
                        self.player.mana = (self.player.mana + 12).min(self.player.max_mana);
                        self.msg("Your mind feels clear and powerful.");
                    }
                    ItemKind::GreenPotion => {
                        self.player.poison = 0.0;
                        self.msg("You feel the poison leave your body.");
                    }
                    _ => {
                        self.add_spell(Spell::NightVision);
                        self.msg("Your eyes tingle. The darkness seems to recede.");
                    }
                }
                consume(item)
            }
            Class::Light => {
                if item.fuel <= 0.0 && def.light.map(|l| l.2 > 0.0).unwrap_or(false) {
                    self.msg(format!("The {} is burnt out.", def.name));
                    return Some(item);
                }
                item.lit = !item.lit;
                if item.lit {
                    self.msg(format!("You light the {}.", def.name));
                    if equipped_slot.is_none() {
                        // Lights only shine when carried in a hand or on a shoulder.
                        for s in [EqSlot::ShoulderL, EqSlot::ShoulderR] {
                            if self.player.equip.get(s).is_none() {
                                *self.player.equip.get_mut(s) = Some(item);
                                self.msg("You place it on your shoulder.");
                                return None;
                            }
                        }
                        self.msg("Place it in a hand or on a shoulder for it to light your way.");
                    }
                } else {
                    self.msg(format!("You put out the {}.", def.name));
                }
                Some(item)
            }
            Class::Scroll | Class::Book => {
                let text = super::dialogue::scroll_text(item.data);
                self.events.push(Event::ReadText(text.to_string()));
                Some(item)
            }
            Class::Rune => {
                let r = item.data as usize % 24;
                if self.player.runes[r] {
                    self.msg(format!("You already have the {} rune.", super::magic::RUNE_NAMES[r]));
                    Some(item)
                } else {
                    self.player.runes[r] = true;
                    self.sound(Sfx::Spell);
                    self.msg(format!("You place the {} rune in your rune bag.", super::magic::RUNE_NAMES[r]));
                    None
                }
            }
            Class::Wand => {
                if let Enchant::Spell(sp) = item.enchant {
                    if item.charges == 0 {
                        self.msg("The wand is out of charges.");
                        return Some(item);
                    }
                    item.charges -= 1;
                    let aim = V3::new(self.player.yaw.sin(), 0.0, -self.player.yaw.cos());
                    self.apply_spell(sp, aim);
                } else {
                    self.msg("Nothing happens.");
                }
                Some(item)
            }
            Class::Misc if item.kind == ItemKind::Bedroll => {
                self.sleep();
                Some(item)
            }
            Class::Treasure => {
                let v = def.value * item.qty as i32;
                self.msg(format!("{} worth about {} gold.", cap(&item.name_with_article()), v.max(1)));
                Some(item)
            }
            _ => {
                if let Enchant::Spell(sp) = item.enchant {
                    if item.identified {
                        self.msg(format!("This item holds the power of {}.", sp.def().name));
                    }
                }
                self.msg("Nothing happens.");
                Some(item)
            }
        }
    }

    pub fn sleep(&mut self) {
        let eye = self.eye();
        let danger = self.level().npcs.iter().any(|n| {
            n.alive() && n.attitude == Attitude::Hostile && (n.pos - eye).len() < 7.0 && physics::line_of_sight(self.level(), V3::new(n.pos.x, n.pos.y + 0.4, n.pos.z), eye)
        });
        if danger {
            self.msg("You cannot rest with enemies nearby!");
            return;
        }
        if self.player.fatigue < 15.0 && self.player.vit == self.player.max_vit {
            self.msg("You are not tired.");
            return;
        }
        let hungry = self.player.hunger < 25.0;
        let frac = if hungry { 0.4 } else { 0.9 };
        let vit_gain = ((self.player.max_vit - self.player.vit) as f32 * frac) as i32;
        self.heal_player(vit_gain);
        self.player.mana = self.player.mana + ((self.player.max_mana - self.player.mana) as f32 * frac) as i32;
        self.player.fatigue = 0.0;
        self.player.hunger = (self.player.hunger - 15.0).max(0.0);
        self.time += 480.0;
        self.events.push(Event::ReadText(String::from("@sleep")));
        let dream = self.dream();
        self.events.push(Event::ReadText(dream));
        self.msg(if hungry { "You wake, but your hunger kept you from resting well." } else { "You wake feeling refreshed." });
    }

    pub fn add_spell(&mut self, s: Spell) {
        let dur = s.def().duration.max(30.0);
        if let Some(a) = self.player.spells.iter_mut().find(|a| a.spell == s) {
            a.left = dur;
        } else {
            self.player.spells.push(ActiveSpell { spell: s, left: dur });
        }
    }

    /// Cast the spell on the rune shelf toward `aim`.
    pub fn cast_shelf(&mut self, aim: V3) {
        if self.player.shelf.is_empty() {
            self.msg("You have no runes on your rune shelf.");
            return;
        }
        let shelf = self.player.shelf.clone();
        let Some(spell) = Spell::from_runes(&shelf) else {
            self.sound(Sfx::Fizzle);
            self.msg("Those runes do not form a spell.");
            return;
        };
        let def = spell.def();
        let cost = spell.mana_cost();
        if (self.player.level + 1) / 2 < def.circle {
            self.msg("You are not experienced enough to cast that spell.");
            return;
        }
        if self.player.mana < cost {
            self.sound(Sfx::Fizzle);
            self.msg("You do not have enough mana to cast that spell.");
            return;
        }
        let chance = (45 + self.player.skill(Skill::Casting) * 7 + self.player.int - def.circle * 10).clamp(10, 98);
        if self.rng.range(0, 100) >= chance {
            self.player.mana -= cost / 2;
            self.sound(Sfx::Fizzle);
            self.msg("The spell fizzles.");
            return;
        }
        self.player.mana -= cost;
        self.apply_spell(spell, aim);
    }

    pub fn apply_spell(&mut self, spell: Spell, aim: V3) {
        let def = spell.def();
        let eye = self.eye();
        self.sound(Sfx::Spell);
        let aim = aim.norm();
        let shoot = |g: &mut Game, kind: ProjKind, speed: f32, dmg: i32| {
            g.projectiles.push(Projectile { pos: eye + aim * 0.3 - V3::new(0.0, 0.08, 0.0), vel: aim * speed, kind, from_player: true, dmg, life: 3.0 });
        };
        let casting = self.player.skill(Skill::Casting);
        match spell {
            Spell::MagicArrow => shoot(self, ProjKind::MagicArrow, 7.0, 4 + casting / 2),
            Spell::Lightning => {
                self.sound(Sfx::Zap);
                shoot(self, ProjKind::Lightning, 10.0, 8 + casting / 2)
            }
            Spell::Fireball => {
                self.sound(Sfx::Fire);
                shoot(self, ProjKind::Fireball, 6.0, 14 + casting)
            }
            Spell::Poison => shoot(self, ProjKind::Poison, 6.0, 3),
            Spell::Paralyze | Spell::Ally => {
                if let Some(id) = self.creature_in_aim(aim, 6.0) {
                    let n = self.level_mut().npc_mut(id).unwrap();
                    if spell == Spell::Paralyze {
                        n.paralyzed = 10.0;
                        self.msg("The creature is frozen in place!");
                    } else {
                        n.attitude = Attitude::Friendly;
                        n.ai = AiState::Follow;
                        self.msg("The creature becomes your ally.");
                    }
                } else {
                    self.msg("The spell finds no target.");
                }
            }
            Spell::LesserHeal => {
                let a = self.rng.range(6, 12);
                self.heal_player(a);
                self.msg("You feel better.");
            }
            Spell::Heal => {
                let a = self.rng.range(14, 24);
                self.heal_player(a);
                self.msg("You feel much better.");
            }
            Spell::GreaterHeal => {
                let a = self.player.max_vit;
                self.heal_player(a);
                self.msg("You are completely healed.");
            }
            Spell::CurePoison => {
                self.player.poison = 0.0;
                self.msg("You are cured of poison.");
            }
            Spell::CreateFood => {
                let it = Item::new(if self.rng.chance(0.5) { ItemKind::Bread } else { ItemKind::Apple });
                self.drop_item(it, None);
                self.msg("Food appears before you.");
            }
            Spell::DetectMonster => {
                let n = self.level().npcs.iter().filter(|n| n.alive() && n.attitude == Attitude::Hostile && (n.pos - eye).len() < 12.0).count();
                self.msg(match n {
                    0 => "You sense no hostile creatures nearby.".to_string(),
                    1 => "You sense one hostile creature nearby.".to_string(),
                    _ => format!("You sense {} hostile creatures nearby.", n),
                });
            }
            Spell::CauseFear => {
                let ids: Vec<u32> = self.level().npcs.iter().filter(|n| n.alive() && (n.pos - eye).len() < 6.0 && n.attitude == Attitude::Hostile).map(|n| n.id).collect();
                for id in &ids {
                    if let Some(n) = self.level_mut().npc_mut(*id) {
                        n.feared = 12.0;
                        n.ai = AiState::Flee;
                    }
                }
                self.msg(if ids.is_empty() { "Nothing seems affected." } else { "Your foes flee in terror!" });
            }
            Spell::Confusion => {
                let ids: Vec<u32> = self.level().npcs.iter().filter(|n| n.alive() && (n.pos - eye).len() < 6.0 && n.attitude == Attitude::Hostile).map(|n| n.id).collect();
                for id in &ids {
                    if let Some(n) = self.level_mut().npc_mut(*id) {
                        n.attitude = Attitude::Mellow;
                        n.ai = AiState::Wander;
                    }
                }
                self.msg("Your foes forget their quarrel with you.");
            }
            Spell::Reveal => {
                let (cx, cz) = (eye.x, eye.z);
                let mut found = 0;
                for d in self.level_mut().doors.iter_mut() {
                    if d.kind == DoorKind::Secret && !d.found && ((d.tx as f32 + 0.5 - cx).powi(2) + (d.tz as f32 + 0.5 - cz).powi(2)).sqrt() < 8.0 {
                        d.found = true;
                        found += 1;
                    }
                }
                self.msg(if found > 0 { "Hidden passages are revealed to you!" } else { "Nothing hidden is revealed." });
            }
            Spell::RoamingSight => {
                for t in self.level_mut().map.tiles.iter_mut() {
                    if !t.is_solid() {
                        t.explored = true;
                    }
                }
                self.msg("Your spirit soars through the level. Your map is complete.");
            }
            Spell::GateTravel => {
                let (x, z, yaw) = self.level().start;
                let y = self.level().map.floor_at(x, z);
                self.player.pos = V3::new(x, y, z);
                self.player.yaw = yaw;
                self.msg("You are whisked away through a moongate!");
            }
            Spell::Tremor | Spell::FlameWind => {
                let ids: Vec<u32> = self.level().npcs.iter().filter(|n| n.alive() && (n.pos - eye).len() < 5.0 && n.attitude != Attitude::Friendly).map(|n| n.id).collect();
                let dmg = if spell == Spell::Tremor { 12 } else { 18 } + casting;
                self.events.push(Event::Shake(0.6));
                for id in ids {
                    ai::damage_npc(self, id, dmg, true);
                }
                if spell == Spell::FlameWind {
                    self.spawn_effect(eye + aim, EffectKind::Explosion);
                }
                self.msg(if spell == Spell::Tremor { "The ground heaves violently!" } else { "A wind of flame roars about you!" });
            }
            Spell::Open | Spell::StrengthenDoor | Spell::RemoveTrap => {
                let door = self.level().doors.iter().filter(|d| ((d.tx as f32 + 0.5 - eye.x).powi(2) + (d.tz as f32 + 0.5 - eye.z).powi(2)).sqrt() < 3.0).map(|d| d.id).next();
                match (spell, door) {
                    (Spell::Open, Some(id)) => {
                        let d = self.level_mut().doors.iter_mut().find(|d| d.id == id).unwrap();
                        d.locked = false;
                        d.opening = true;
                        self.msg("The door swings open.");
                    }
                    (Spell::StrengthenDoor, Some(id)) => {
                        let d = self.level_mut().doors.iter_mut().find(|d| d.id == id).unwrap();
                        d.opening = false;
                        self.msg("The door is magically sealed.");
                    }
                    (Spell::RemoveTrap, _) => {
                        let (tx, tz) = (eye.x.floor() as i32, eye.z.floor() as i32);
                        let mut n = 0;
                        for t in self.level_mut().triggers.iter_mut() {
                            if matches!(t.kind, super::TriggerKind::Damage { .. }) && (t.tx - tx).abs() <= 2 && (t.tz - tz).abs() <= 2 {
                                t.fired = true;
                                t.once = true;
                                n += 1;
                            }
                        }
                        self.msg(if n > 0 { "You disarm a trap." } else { "You find no traps." });
                    }
                    _ => self.msg("There is no door nearby."),
                }
            }
            Spell::NameEnchantment => self.msg("Hold an item in your hand and look at it to learn its nature."),
            Spell::RuneOfWarding => self.msg("A protective rune glows briefly upon the floor."),
            _ => {
                if def.target == Targeting::Caster && def.duration > 0.0 {
                    self.add_spell(spell);
                    self.msg(format!("You cast {}.", def.name));
                } else {
                    self.msg(format!("You cast {}.", def.name));
                }
            }
        }
    }

    /// The nearest living creature roughly along the aim direction.
    pub fn creature_in_aim(&self, aim: V3, range: f32) -> Option<u32> {
        let eye = self.eye();
        let mut best: Option<(u32, f32)> = None;
        for n in &self.level().npcs {
            if !n.alive() {
                continue;
            }
            let c = V3::new(n.pos.x, n.pos.y + n.race.def().height * 0.5, n.pos.z);
            let to = c - eye;
            let d = to.len();
            if d > range {
                continue;
            }
            let cos = to.dot(aim) / d.max(1e-3);
            if cos > 0.9 && physics::line_of_sight(self.level(), eye, c) && best.map(|b| d < b.1).unwrap_or(true) {
                best = Some((n.id, d));
            }
        }
        best.map(|b| b.0)
    }

    /// A dream vision hinting at the next step of the quest.
    pub fn dream(&mut self) -> String {
        let has = |k| self.player.has_item(k);
        let hint = if !has(ItemKind::ShardOfValor) && self.flag("thorgrim_done") == 0 {
            "Seek the mountain folk below. Their king keeps the shard of Valor, and he will test thy courage."
        } else if !has(ItemKind::ShardOfWisdom) && self.flag("ssrath_done") == 0 {
            "The lizard folk of the flooded warrens grieve for their stolen young. Their Sage holds the shard of Wisdom."
        } else if !has(ItemKind::ShardOfMercy) && self.flag("aldric_done") == 0 {
            "In the crypts a knight cannot rest. Return to him what was taken, and Mercy shall be thine."
        } else if self.flag("fiend_slain") == 0 {
            "The three shards are thine. Descend to the Deep Sanctum, open the sealed gate, and face the Shadow before it wakes."
        } else {
            "Thy task is done. Rest, hero."
        };
        format!("You dream. A purple-cloaked spirit appears before you and speaks: \"{}\"", hint)
    }

    /// Use the Track skill: report the nearest creature's direction.
    pub fn track(&mut self) {
        let eye = self.eye();
        let skill = self.player.skill(Skill::Track);
        let range = 6.0 + skill as f32 * 1.5;
        let best = self
            .level()
            .npcs
            .iter()
            .filter(|n| n.alive() && n.attitude != Attitude::Friendly)
            .map(|n| (n, (n.pos - eye).len()))
            .filter(|(_, d)| *d < range)
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(n, d)| (n.race.def().name, n.pos, d));
        if self.rng.range(0, 100) > 30 + skill * 8 {
            self.msg("You search for tracks, but find nothing you can read.");
            return;
        }
        match best {
            None => self.msg("You find no fresh tracks nearby."),
            Some((name, pos, d)) => {
                let a = (pos.x - eye.x).atan2(-(pos.z - eye.z));
                let dirs = ["north", "north-east", "east", "south-east", "south", "south-west", "west", "north-west"];
                let k = ((a / std::f32::consts::TAU * 8.0).round() as i32).rem_euclid(8) as usize;
                let how = if d < 4.0 { "very close" } else if d < 8.0 { "nearby" } else { "some distance away" };
                self.msg(format!("You find the tracks of a {} to the {}, {}.", name, dirs[k], how));
            }
        }
    }

    /// A short status report (clicking the compass).
    pub fn status_report(&mut self) {
        let p = &self.player;
        let hunger = match p.hunger as i32 {
            0..=7 => "starving",
            8..=24 => "hungry",
            25..=59 => "a little hungry",
            60..=89 => "fed",
            _ => "well fed",
        };
        let fatigue = match p.fatigue as i32 {
            0..=30 => "rested",
            31..=70 => "somewhat tired",
            _ => "exhausted",
        };
        let health = if p.poison > 0.0 { "poisoned" } else if p.vit < p.max_vit / 3 { "badly wounded" } else if p.vit < p.max_vit { "wounded" } else { "in good health" };
        let day = self.day();
        let lvl = self.cur + 1;
        let enc = if p.load() > p.capacity() { " You are overburdened." } else { "" };
        self.msg(format!("You are {}, {} and {}.{}", health, hunger, fatigue, enc));
        self.msg(format!("It is the {} day of your imprisonment. You are on level {} of the abyss.", ordinal(day), lvl));
    }
}

pub fn ordinal(n: i32) -> String {
    let suffix = match (n % 10, n % 100) {
        (1, 11) | (2, 12) | (3, 13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{}{}", n, suffix)
}

fn consume(mut item: Item) -> Option<Item> {
    if item.qty > 1 {
        item.qty -= 1;
        Some(item)
    } else {
        None
    }
}

pub fn describe_item(it: &Item) -> String {
    let d = it.def();
    let base = it.name_with_article();
    if d.class == Class::Weapon || d.class == Class::Armor {
        let cond = it.condition_word();
        if !cond.is_empty() {
            let mut s = base.clone();
            let sp = s.find(' ').unwrap_or(0);
            s.insert_str(sp + 1, cond);
            return s;
        }
    }
    if d.class == Class::Light && it.fuel <= 0.0 && d.light.map(|l| l.2 > 0.0).unwrap_or(false) {
        return format!("{} (burnt out)", base);
    }
    base
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinals() {
        assert_eq!(ordinal(1), "1st");
        assert_eq!(ordinal(2), "2nd");
        assert_eq!(ordinal(11), "11th");
        assert_eq!(ordinal(23), "23rd");
    }

    #[test]
    fn describe_worn_weapon() {
        let mut it = Item::new(ItemKind::Longsword);
        it.quality = 30;
        assert_eq!(describe_item(&it), "a worn longsword");
    }
}
