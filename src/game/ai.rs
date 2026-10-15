//! Creature behaviour and combat resolution.

use super::creatures::{AiState, Anim, Attitude, Ranged};
use super::items::{Item, ItemKind, Skill};
use super::magic::Spell;
use super::physics::{self, Body, PLAYER_RADIUS};
use super::{EffectKind, EqSlot, Event, Game, ProjKind, Projectile};
use crate::audio::Sfx;
use crate::util::{wrap_angle, V3};
use crate::world::map::{Map, MAP_SIZE};
use std::collections::VecDeque;

const SIGHT: f32 = 9.0;

/// Breadth-first search over tiles; returns the next tile centre to walk to.
pub fn path_step(map: &Map, from: (i32, i32), to: (i32, i32), max_nodes: usize) -> Option<(f32, f32)> {
    if from == to {
        return None;
    }
    let idx = |x: i32, z: i32| (z * MAP_SIZE + x) as usize;
    let mut prev = vec![u32::MAX; (MAP_SIZE * MAP_SIZE) as usize];
    let mut q = VecDeque::new();
    q.push_back(from);
    prev[idx(from.0, from.1)] = idx(from.0, from.1) as u32;
    let mut found = false;
    let mut visited = 0;
    while let Some((x, z)) = q.pop_front() {
        visited += 1;
        if visited > max_nodes {
            break;
        }
        if (x, z) == to {
            found = true;
            break;
        }
        let here = map.get(x, z);
        for (e, (dx, dz)) in crate::world::map::EDGE_DIR.iter().enumerate() {
            let (nx, nz) = (x + dx, z + dz);
            if !Map::in_bounds(nx, nz) || prev[idx(nx, nz)] != u32::MAX {
                continue;
            }
            let t = map.get(nx, nz);
            if t.is_solid() || !here.edge_open(e) || !t.edge_open((e + 2) % 4) {
                continue;
            }
            let hf = here.corner_floor().iter().cloned().fold(f32::MIN, f32::max);
            let tf = t.corner_floor().iter().cloned().fold(f32::MAX, f32::min);
            if tf - hf > 0.5 {
                continue;
            }
            prev[idx(nx, nz)] = idx(x, z) as u32;
            q.push_back((nx, nz));
        }
    }
    if !found {
        return None;
    }
    // Walk back to find the first step.
    let mut cur = idx(to.0, to.1);
    let start = idx(from.0, from.1);
    loop {
        let p = prev[cur] as usize;
        if p == start {
            let x = (cur as i32) % MAP_SIZE;
            let z = (cur as i32) / MAP_SIZE;
            return Some((x as f32 + 0.5, z as f32 + 0.5));
        }
        cur = p;
    }
}

pub fn update_npcs(g: &mut Game, dt: f32) {
    let n = g.level().npcs.len();
    let player_eye = g.eye();
    let stealthy = g.player.has_spell(Spell::Stealth) || g.player.has_spell(Spell::Invisibility);
    let invisible = g.player.has_spell(Spell::Invisibility);
    for i in 0..n {
        let lv = g.cur;
        // Animation clocks.
        {
            let npc = &mut g.levels[lv].npcs[i];
            npc.anim_t += dt;
            npc.cooldown = (npc.cooldown - dt).max(0.0);
            match npc.anim {
                Anim::Die => {
                    if npc.anim_t > 0.6 {
                        npc.anim = Anim::Dead;
                    }
                    continue;
                }
                Anim::Dead => continue,
                Anim::Hit => {
                    if npc.anim_t > 0.3 {
                        npc.anim = Anim::Idle;
                        npc.anim_t = 0.0;
                    }
                }
                _ => {}
            }
            if npc.paralyzed > 0.0 {
                npc.paralyzed -= dt;
                continue;
            }
            npc.feared = (npc.feared - dt).max(0.0);
        }
        // Poison on creatures.
        if g.levels[lv].npcs[i].poisoned > 0.0 {
            let before = g.levels[lv].npcs[i].poisoned;
            g.levels[lv].npcs[i].poisoned -= dt;
            if (before / 3.0).floor() != (g.levels[lv].npcs[i].poisoned / 3.0).floor() {
                let id = g.levels[lv].npcs[i].id;
                damage_npc(g, id, 1, false);
                if !g.levels[lv].npcs[i].alive() {
                    continue;
                }
            }
        }
        let (pos, race, attitude, ai) = {
            let npc = &g.levels[lv].npcs[i];
            (npc.pos, npc.race, npc.attitude, npc.ai)
        };
        let def = race.def();
        let to_player = V3::new(player_eye.x - pos.x, 0.0, player_eye.z - pos.z);
        let dist = (to_player.x * to_player.x + to_player.z * to_player.z).sqrt();
        let npc_eye = V3::new(pos.x, pos.y + def.height * 0.8, pos.z);
        let hostile = attitude == Attitude::Hostile;
        let mut sees = false;
        if hostile && !g.player.dead && !invisible {
            // Creatures already hunting you keep their full sight; others must notice you first.
            let base = if ai == AiState::Attack { SIGHT } else { g.player.notice_range(SIGHT) };
            let range = if stealthy { base * 0.4 } else { base };
            if dist < range && physics::line_of_sight(&g.levels[lv], npc_eye, player_eye) {
                sees = true;
            }
        }
        {
            let npc = &mut g.levels[lv].npcs[i];
            if sees {
                npc.lost_sight = 0.0;
                if npc.ai != AiState::Flee && npc.ai != AiState::Attack {
                    npc.ai = AiState::Attack;
                    if npc.cooldown <= 0.0 && dist > 2.0 {
                        g.events.push(Event::Sound(Sfx::Growl));
                    }
                }
            } else {
                npc.lost_sight += dt;
                if npc.ai == AiState::Attack && npc.lost_sight > 8.0 {
                    npc.ai = AiState::Wander;
                }
            }
            if npc.feared > 0.0 {
                npc.ai = AiState::Flee;
            } else if npc.ai == AiState::Flee && npc.hp > npc.max_hp / 3 {
                npc.ai = if hostile { AiState::Attack } else { AiState::Wander };
            }
        }
        let ai = if g.levels[lv].npcs[i].ai != ai { g.levels[lv].npcs[i].ai } else { ai };
        let speed = def.speed * 1.1;
        let mut want: Option<(f32, f32)> = None; // target point
        let mut face_player = false;
        match ai {
            AiState::Idle => {
                if !hostile && dist < 2.5 {
                    face_player = true;
                }
            }
            AiState::Wander => {
                let npc = &mut g.levels[lv].npcs[i];
                if npc.wander_to.is_none() && g.rng.chance(dt * 0.4) {
                    let a = g.rng.f32() * std::f32::consts::TAU;
                    let r = g.rng.rangef(0.5, 3.0);
                    npc.wander_to = Some((npc.home.0 + a.sin() * r, npc.home.1 + a.cos() * r));
                }
                if let Some(w) = npc.wander_to {
                    if (w.0 - pos.x).abs() + (w.1 - pos.z).abs() < 0.2 {
                        npc.wander_to = None;
                    } else {
                        want = Some(w);
                    }
                }
            }
            AiState::Attack => {
                face_player = true;
                let reach = def.radius + PLAYER_RADIUS + 0.35;
                let ranged = def.ranged != Ranged::None;
                // Melee needs a clear line (no closed door or wall in between).
                let clear = dist <= reach && physics::line_of_sight(&g.levels[lv], npc_eye, player_eye);
                if !clear {
                    let npc = &g.levels[lv].npcs[i];
                    let lost = npc.lost_sight;
                    if ranged && sees && dist > 2.0 && npc.cooldown <= 0.0 && g.rng.chance(dt * 1.2) {
                        npc_shoot(g, i, player_eye);
                    } else if lost < 6.0 {
                        let from = (pos.x.floor() as i32, pos.z.floor() as i32);
                        let to = (player_eye.x.floor() as i32, player_eye.z.floor() as i32);
                        if sees && dist < 1.6 || from == to {
                            want = Some((player_eye.x, player_eye.z));
                        } else if let Some(step) = path_step(&g.levels[lv].map, from, to, 900) {
                            want = Some(step);
                        } else {
                            want = Some((player_eye.x, player_eye.z));
                        }
                    }
                } else {
                    melee(g, i, dt);
                }
            }
            AiState::Flee => {
                let away = V3::new(pos.x - player_eye.x, 0.0, pos.z - player_eye.z).norm();
                want = Some((pos.x + away.x * 2.0, pos.z + away.z * 2.0));
            }
            AiState::Follow => {
                if dist > 1.6 {
                    want = Some((player_eye.x, player_eye.z));
                }
                face_player = dist < 1.6;
            }
        }
        // Movement.
        let others: Vec<(u32, V3, f32)> = {
            let mut o: Vec<(u32, V3, f32)> = g.levels[lv].npcs.iter().filter(|n| n.alive()).map(|n| (n.id, n.pos, n.race.def().radius)).collect();
            o.push((0, g.player.pos, PLAYER_RADIUS));
            o
        };
        let npc_swing = g.levels[lv].npcs[i].swing;
        let npc = &mut g.levels[lv].npcs[i];
        let body = Body { radius: def.radius, height: def.height.min(0.7), max_step: 0.3, self_id: npc.id, flying: def.flies, avoid_water: !def.swims && !def.flies, avoid_lava: !def.flies };
        let mut moving = false;
        if let Some((tx, tz)) = want {
            if npc_swing <= 0.0 {
                let d = V3::new(tx - npc.pos.x, 0.0, tz - npc.pos.z);
                let l = d.len();
                if l > 0.05 {
                    let target_yaw = d.x.atan2(-d.z);
                    let diff = wrap_angle(target_yaw - npc.yaw);
                    npc.yaw = wrap_angle(npc.yaw + diff.clamp(-dt * 5.0, dt * 5.0));
                    let step = (speed * dt).min(l);
                    let dir = d.norm();
                    let mut p = npc.pos;
                    p.y -= def.hover;
                    let lvref = &g.levels[lv];
                    let mut np = p;
                    moving = physics::slide_move(lvref, &body, &mut np, dir.x * step, dir.z * step, &others);
                    if !moving && ai == AiState::Wander {
                        g.levels[lv].npcs[i].wander_to = None;
                    }
                    let floor = g.levels[lv].map.floor_at(np.x, np.z);
                    let npc = &mut g.levels[lv].npcs[i];
                    npc.pos.x = np.x;
                    npc.pos.z = np.z;
                    npc.pos.y = floor + def.hover + if def.flies { (g.time as f32 * 3.0 + npc.id as f32).sin() * 0.05 } else { 0.0 };
                }
            }
        }
        let npc = &mut g.levels[lv].npcs[i];
        if face_player && !moving {
            let target_yaw = to_player.x.atan2(-to_player.z);
            let diff = wrap_angle(target_yaw - npc.yaw);
            npc.yaw = wrap_angle(npc.yaw + diff.clamp(-dt * 4.0, dt * 4.0));
        }
        if npc.anim != Anim::Hit && npc.anim != Anim::Attack {
            let a = if moving { Anim::Walk } else { Anim::Idle };
            if npc.anim != a {
                npc.anim = a;
                npc.anim_t = 0.0;
            }
        }
        if npc.anim == Anim::Attack && npc.anim_t > 0.6 {
            npc.anim = Anim::Idle;
            npc.anim_t = 0.0;
        }
    }
}

fn melee(g: &mut Game, i: usize, dt: f32) {
    let lv = g.cur;
    let def = g.levels[lv].npcs[i].race.def();
    let npc = &mut g.levels[lv].npcs[i];
    if npc.swing <= 0.0 {
        if npc.cooldown > 0.0 {
            return;
        }
        npc.swing = 0.001;
        npc.anim = Anim::Attack;
        npc.anim_t = 0.0;
        return;
    }
    npc.swing += dt;
    if npc.swing < 0.45 {
        return;
    }
    npc.swing = 0.0;
    npc.cooldown = 1.1 / def.speed.max(0.4) + g.rng.rangef(0.0, 0.6);
    // Resolve the blow.
    let p = &g.player;
    let defense = p.skill(Skill::Defense) * 3 + p.armor_value() * 3 + p.dex / 2 + if p.has_spell(Spell::ResistBlows) { 15 } else { 0 };
    let chance = (45 + def.attack * 4 - defense).clamp(8, 92);
    g.sound(Sfx::Swing);
    if g.rng.range(0, 100) < chance {
        let armor = g.player.armor_value();
        let raw = g.rng.range(def.dmg.0, def.dmg.1 + 1);
        let dmg = (raw - armor / 3).max(1);
        let name = def.name;
        g.hurt_player(dmg, &format!("The {} hits you.", name));
        if def.poisons && g.rng.chance(0.25) {
            g.player.poison += 15.0;
            g.msg("You have been poisoned!");
        }
        let e = g.eye();
        let fwd = V3::new(g.player.yaw.sin(), 0.0, -g.player.yaw.cos());
        g.spawn_effect(e + fwd * 0.35 - V3::new(0.0, 0.1, 0.0), EffectKind::Blood);
        // Armour wears.
        if g.rng.chance(0.15) {
            let worn: Vec<EqSlot> = [EqSlot::Head, EqSlot::Chest, EqSlot::Legs, EqSlot::Feet, EqSlot::Hands, EqSlot::Shield]
                .into_iter()
                .filter(|s| g.player.equip.get(*s).as_ref().map(|i| i.def().armor.is_some()).unwrap_or(false))
                .collect();
            if !worn.is_empty() {
                let s = worn[g.rng.range(0, worn.len() as i32) as usize];
                if let Some(it) = g.player.equip.get_mut(s).as_mut() {
                    it.quality = it.quality.saturating_sub(1);
                }
            }
        }
    } else {
        let name = def.name;
        if g.rng.chance(0.5) {
            g.msg(format!("The {} misses you.", name));
        }
    }
}

fn npc_shoot(g: &mut Game, i: usize, target: V3) {
    let lv = g.cur;
    let npc = &mut g.levels[lv].npcs[i];
    let def = npc.race.def();
    npc.cooldown = 2.5;
    npc.anim = Anim::Attack;
    npc.anim_t = 0.0;
    let from = V3::new(npc.pos.x, npc.pos.y + def.height * 0.7, npc.pos.z);
    let dir = (target - V3::new(0.0, 0.1, 0.0) - from).norm();
    let (kind, speed, dmg) = match def.ranged {
        Ranged::Fire => (ProjKind::Fireball, 5.0, def.dmg.1),
        _ => (ProjKind::MagicArrow, 6.0, (def.dmg.0 + def.dmg.1) / 2),
    };
    g.projectiles.push(Projectile { pos: from + dir * (def.radius + 0.1), vel: dir * speed, kind, from_player: false, dmg, life: 3.0 });
    g.sound(if kind == ProjKind::Fireball { Sfx::Fire } else { Sfx::Spell });
}

/// Apply damage to a creature; handles death, loot and experience.
pub fn damage_npc(g: &mut Game, id: u32, dmg: i32, by_player: bool) {
    let lv = g.cur;
    let Some(idx) = g.levels[lv].npcs.iter().position(|n| n.id == id) else { return };
    if !g.levels[lv].npcs[idx].alive() {
        return;
    }
    let mut dmg = dmg;
    if g.levels[lv].npcs[idx].race == super::creatures::Race::Demon && !g.has_all_shards() {
        dmg = 0;
        if by_player {
            g.msg("Your blow passes harmlessly through the shadow! Only the Lumen Stone can harm it.");
        }
    }
    {
        let npc = &mut g.levels[lv].npcs[idx];
        npc.hp -= dmg.max(0);
        if by_player {
            npc.attitude = Attitude::Hostile;
            if npc.ai != AiState::Flee {
                npc.ai = AiState::Attack;
            }
            npc.lost_sight = 0.0;
        }
    }
    if by_player {
        g.foe = Some(id);
        g.foe_timer = 8.0;
    }
    let npc = &g.levels[lv].npcs[idx];
    if npc.hp <= 0 {
        let def = npc.race.def();
        let name = match &npc.name {
            Some(n) => n.clone(),
            None => format!("the {}", def.name),
        };
        let pos = npc.pos;
        let inv = npc.inventory.clone();
        let race = npc.race;
        {
            let floor = g.levels[lv].map.floor_at(pos.x, pos.z);
            let npc = &mut g.levels[lv].npcs[idx];
            npc.hp = 0;
            npc.anim = Anim::Die;
            npc.anim_t = 0.0;
            npc.pos.y = floor;
        }
        g.sound(Sfx::MonsterDie);
        let mut s = name.clone();
        if let Some(c) = s.get_mut(0..1) {
            c.make_ascii_uppercase();
        }
        g.msg(format!("{} is dead.", s));
        if by_player {
            g.gain_exp(def.exp);
        }
        // Drop loot.
        let mut k = 0.0f32;
        for it in inv {
            let a = k * 2.1;
            g.levels[lv].add_item(it, pos.x + a.sin() * 0.15, pos.z + a.cos() * 0.15);
            k += 1.0;
        }
        if g.rng.chance(0.3) && !def.undead && !def.flies {
            g.levels[lv].add_item(Item::new(ItemKind::Blood), pos.x, pos.z);
        }
        if def.undead || race == super::creatures::Race::Skeleton {
            g.levels[lv].add_item(Item::new(ItemKind::Bone), pos.x + 0.1, pos.z);
        }
        if g.foe == Some(id) {
            g.foe = None;
        }
        if race == super::creatures::Race::Demon {
            g.set_flag("fiend_slain", 1);
            g.events.push(super::Event::Victory);
        }
    } else {
        let npc = &mut g.levels[lv].npcs[idx];
        npc.anim = Anim::Hit;
        npc.anim_t = 0.0;
        npc.swing = 0.0;
        let fearless = npc.race.def().undead || matches!(npc.race, super::creatures::Race::Demon | super::creatures::Race::Golem);
        if !fearless && npc.hp < npc.max_hp / 5 && g.rng.chance(0.4) {
            npc.ai = AiState::Flee;
        }
        g.sound(Sfx::MonsterHurt);
    }
}

/// The player swings their weapon. `aim` is the world-space direction of the attack,
/// `kind` 0 = bash, 1 = slash, 2 = thrust; `power` 0..1 from the power gem.
pub fn player_attack(g: &mut Game, aim: V3, kind: usize, power: f32) {
    let weapon = g.player.equip.weapon.clone();
    // Missile weapons fire ammunition from the pack.
    if let Some(w) = weapon.as_ref().filter(|w| w.def().class == super::items::Class::Missile) {
        let ammo = if w.kind == ItemKind::Sling { ItemKind::SlingStone } else { ItemKind::Bolt };
        let found = take_ammo(g, ammo);
        if !found {
            g.msg("You have no ammunition for that weapon.");
            return;
        }
        let ws = w.def().weapon.unwrap();
        let skill = g.player.skill(Skill::Missile);
        let dmg = ws.dmg[2] + skill / 2 + (power * 4.0) as i32 + g.rng.range(0, 3);
        let eye = g.eye();
        let dir = aim.norm();
        let kind = if ammo == ItemKind::SlingStone { ProjKind::Stone } else { ProjKind::Bolt };
        g.projectiles.push(Projectile { pos: eye + dir * 0.3 - V3::new(0.0, 0.05, 0.0), vel: dir * (8.0 + power * 4.0), kind, from_player: true, dmg, life: 2.5 });
        g.sound(Sfx::Swing);
        return;
    }
    let (skill, dmg_table, reach) = match weapon.as_ref().and_then(|w| w.def().weapon.map(|ws| (w, ws))) {
        Some((w, ws)) if w.def().class == super::items::Class::Weapon => (ws.skill, ws.dmg, 1.25),
        _ => (Skill::Unarmed, [2, 1, 2], 1.0),
    };
    g.sound(Sfx::Swing);
    let eye = g.eye();
    // Find the best target in front.
    let lv = g.cur;
    let mut best: Option<(u32, f32)> = None;
    for n in &g.levels[lv].npcs {
        if !n.alive() {
            continue;
        }
        let def = n.race.def();
        let c = V3::new(n.pos.x, n.pos.y + def.height * 0.5, n.pos.z);
        let to = c - eye;
        let flat = (to.x * to.x + to.z * to.z).sqrt();
        if flat - def.radius > reach {
            continue;
        }
        let a = V3::new(aim.x, 0.0, aim.z).norm();
        let cosang = (to.x * a.x + to.z * a.z) / flat.max(1e-3);
        if cosang < 0.55 {
            continue;
        }
        let score = flat - cosang;
        if best.map(|b| score < b.1).unwrap_or(true) {
            best = Some((n.id, score));
        }
    }
    let Some((id, _)) = best else {
        return;
    };
    let (npc_pos, def) = {
        let npc = g.levels[lv].npc(id).unwrap();
        (npc.pos, npc.race.def())
    };
    let p = &g.player;
    let accuracy = match weapon.as_ref().map(|w| w.enchant) {
        Some(super::items::Enchant::Accuracy(n)) => n as i32 * 5,
        _ => 0,
    };
    let chance = (55 + (p.skill(Skill::Attack) + p.skill(skill)) * 3 + p.dex / 2 - def.defense * 3 + (power * 10.0) as i32 + accuracy).clamp(10, 95);
    if g.rng.range(0, 100) < chance {
        let base = dmg_table[kind.min(2)];
        let strength_bonus = (p.st - 10) / 5;
        let mut dmg = ((base as f32 + strength_bonus as f32) * (0.4 + power * 0.9)).round() as i32 + g.rng.range(0, 3);
        if let Some(w) = &weapon {
            if let super::items::Enchant::Damage(n) = w.enchant {
                dmg += n as i32;
            }
            if w.quality < 30 {
                dmg = dmg * 2 / 3;
            }
        }
        if power >= 0.98 && g.rng.chance(0.25) {
            dmg = dmg * 3 / 2;
            g.msg("A critical hit!");
        }
        let dmg = dmg.max(1);
        let c = V3::new(npc_pos.x, npc_pos.y + def.height * 0.6, npc_pos.z);
        let blood = if def.undead { EffectKind::Sparks } else { EffectKind::Blood };
        g.spawn_effect(c - (c - eye).norm() * def.radius, blood);
        g.sound(Sfx::Hit);
        damage_npc(g, id, dmg, true);
        // Weapons wear slowly.
        if g.rng.chance(0.06) {
            if let Some(w) = g.player.equip.weapon.as_mut() {
                w.quality = w.quality.saturating_sub(2);
            }
        }
    } else {
        g.sound(Sfx::Miss);
        // Provoke even on a miss.
        if let Some(n) = g.levels[lv].npc_mut(id) {
            n.attitude = Attitude::Hostile;
            n.ai = AiState::Attack;
        }
        g.foe = Some(id);
        g.foe_timer = 8.0;
    }
    let _ = physics::PLAYER_HEIGHT;
}

/// Remove one piece of ammunition from the pack (or containers in it).
fn take_ammo(g: &mut Game, kind: ItemKind) -> bool {
    fn take(items: &mut Vec<Item>, kind: ItemKind) -> bool {
        if let Some(i) = items.iter().position(|i| i.kind == kind) {
            if items[i].qty > 1 {
                items[i].qty -= 1;
            } else {
                items.remove(i);
            }
            return true;
        }
        items.iter_mut().any(|i| take(&mut i.contents, kind))
    }
    for slot in g.player.pack.iter_mut() {
        if let Some(it) = slot {
            if it.kind == kind {
                if it.qty > 1 {
                    it.qty -= 1;
                } else {
                    *slot = None;
                }
                return true;
            }
            if take(&mut it.contents, kind) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::art::textures::Tex;
    use crate::world::map::Tile;

    #[test]
    fn bfs_finds_way_around_wall() {
        let mut m = Map::new();
        // U-shaped corridor: (1,1)->(1,5)->(5,5)->(5,1)
        for z in 1..6 {
            m.set(1, z, Tile::open(0, 8, Tex::Dirt, Tex::CeilingRock, Tex::RoughStone));
            m.set(5, z, Tile::open(0, 8, Tex::Dirt, Tex::CeilingRock, Tex::RoughStone));
        }
        for x in 1..6 {
            m.set(x, 5, Tile::open(0, 8, Tex::Dirt, Tex::CeilingRock, Tex::RoughStone));
        }
        let step = path_step(&m, (1, 1), (5, 1), 1000).unwrap();
        assert_eq!(step, (1.5, 2.5));
        assert!(path_step(&m, (1, 1), (3, 1), 1000).is_none());
    }
}
