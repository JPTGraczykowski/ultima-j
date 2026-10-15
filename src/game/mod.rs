//! Game logic: the world simulation, independent of rendering and input devices.

pub mod actions;
pub mod ai;
pub mod creatures;
pub mod dialogue;
pub mod items;
pub mod levels;
pub mod magic;
pub mod physics;
pub mod state;

pub use state::*;

use crate::audio::Sfx;
use crate::util::{Rng, V3};
use crate::world::map::Liquid;
use items::{Item, ItemKind};
use magic::Spell;
use physics::*;
use std::collections::BTreeMap;

/// Player movement intent for one frame (already scaled by the input layer).
#[derive(Clone, Copy, Debug, Default)]
pub struct MoveInput {
    /// Forward speed factor: -1 (back) .. 1 (walk) .. 2 (run).
    pub forward: f32,
    /// Sidestep factor -1 (left) .. 1 (right).
    pub strafe: f32,
    /// Turn rate factor -1 (left) .. 1 (right).
    pub turn: f32,
    pub jump: bool,
    pub fly_up: bool,
    pub fly_down: bool,
}

pub const WALK_SPEED: f32 = 1.35;
pub const TURN_SPEED: f32 = 1.9;

impl Game {
    pub fn new(player: Player, seed: u64) -> Game {
        let mut g = Game {
            levels: Vec::new(),
            cur: 0,
            player,
            time: 0.0,
            rng: Rng::new(seed),
            flags: BTreeMap::new(),
            messages: Vec::new(),
            projectiles: Vec::new(),
            effects: Vec::new(),
            events: Vec::new(),
            foe: None,
            foe_timer: 0.0,
        };
        g.levels = levels::build_all();
        let (x, z, yaw) = g.levels[0].start;
        g.player.pos = V3::new(x, g.levels[0].map.floor_at(x, z), z);
        g.player.yaw = yaw;
        g
    }

    pub fn eye(&self) -> V3 {
        let p = &self.player;
        let sink = if p.swimming { p.swim_sink() } else { 0.0 };
        V3::new(p.pos.x, p.pos.y + EYE_HEIGHT - sink, p.pos.z)
    }

    fn player_body(&self) -> Body {
        let flying = self.player.has_spell(Spell::Fly) || self.player.has_spell(Spell::Levitate);
        Body { radius: PLAYER_RADIUS, height: PLAYER_HEIGHT, max_step: MAX_STEP, self_id: 0, flying, avoid_water: false, avoid_lava: false }
    }

    pub fn npc_obstacles(&self) -> Vec<(u32, V3, f32)> {
        self.level().npcs.iter().filter(|n| n.alive()).map(|n| (n.id, n.pos, n.race.def().radius)).collect()
    }

    /// Advance the simulation.
    pub fn update(&mut self, mv: &MoveInput, dt: f32) {
        if self.player.dead {
            return;
        }
        let frozen = self.player.has_spell(Spell::FreezeTime);
        self.time += dt as f64;
        self.update_player(mv, dt);
        if !frozen {
            ai::update_npcs(self, dt);
        }
        self.update_projectiles(dt);
        self.update_doors(dt);
        self.update_timers(dt);
        self.update_effects(dt);
        if self.foe_timer > 0.0 {
            self.foe_timer -= dt;
            if self.foe_timer <= 0.0 {
                self.foe = None;
            }
        }
    }

    fn update_player(&mut self, mv: &MoveInput, dt: f32) {
        let body = self.player_body();
        let others: Vec<(u32, V3, f32)> = self.npc_obstacles();
        let encumbered = self.player.load() > self.player.capacity();
        let mut speed = WALK_SPEED;
        if self.player.has_spell(Spell::Speed) {
            speed *= 1.6;
        }
        if self.player.swimming {
            speed *= self.player.swim_speed();
        }
        if encumbered {
            speed *= 0.45;
        }
        if self.player.fatigue > 90.0 {
            speed *= 0.8;
        }
        let can_fly = self.player.has_spell(Spell::Fly);
        let p = &mut self.player;
        p.yaw = crate::util::wrap_angle(p.yaw + mv.turn * TURN_SPEED * dt);
        let (s, c) = p.yaw.sin_cos();
        let fwd = (s, -c);
        let right = (c, s);
        let fwd_amt = if mv.forward < 0.0 { mv.forward * 0.75 } else { mv.forward };
        let dx = (fwd.0 * fwd_amt + right.0 * mv.strafe * 0.85) * speed * dt;
        let dz = (fwd.1 * fwd_amt + right.1 * mv.strafe * 0.85) * speed * dt;
        let level = &self.levels[self.cur];
        let mut pos = self.player.pos;
        let moved = slide_move(level, &body, &mut pos, dx, dz, &others);
        let old_tile = (self.player.pos.x.floor() as i32, self.player.pos.z.floor() as i32);
        let floor = level.map.floor_at(pos.x, pos.z);
        let ceil = level.map.ceil_at(pos.x, pos.z);
        let liquid = physics::liquid_at(level, pos.x, pos.z);
        let p = &mut self.player;
        p.pos.x = pos.x;
        p.pos.z = pos.z;

        // Vertical motion.
        if body.flying {
            let v = if mv.fly_up { 1.5 } else if mv.fly_down { -1.5 } else { 0.0 };
            p.pos.y = (p.pos.y + v * dt).min(ceil - PLAYER_HEIGHT).max(floor);
            if !can_fly && p.pos.y > floor + 1.2 {
                p.pos.y = floor + 1.2;
            }
            p.vel_y = 0.0;
            p.on_ground = p.pos.y <= floor + 0.01;
            p.fall_from = p.pos.y;
        } else {
            if mv.jump && p.on_ground && !p.swimming {
                p.vel_y = p.jump_speed();
                p.on_ground = false;
                p.fall_from = p.pos.y;
            }
            if p.on_ground {
                if p.pos.y > floor + MAX_STEP {
                    p.on_ground = false;
                    p.vel_y = 0.0;
                    p.fall_from = p.pos.y;
                } else {
                    // Follow slopes and small steps smoothly.
                    let diff = floor - p.pos.y;
                    p.pos.y += diff * (dt * 14.0).min(1.0);
                    if diff.abs() < 0.004 {
                        p.pos.y = floor;
                    }
                }
            }
            if !p.on_ground {
                let slow_fall = self.player.has_spell(Spell::SlowFall);
                let p = &mut self.player;
                p.vel_y -= GRAVITY * dt * if slow_fall { 0.25 } else { 1.0 };
                p.pos.y += p.vel_y * dt;
                p.fall_from = p.fall_from.max(p.pos.y);
                if p.pos.y + PLAYER_HEIGHT > ceil && p.vel_y > 0.0 {
                    p.vel_y = 0.0;
                    p.pos.y = ceil - PLAYER_HEIGHT;
                }
                if p.pos.y <= floor {
                    let drop = p.fall_from - floor;
                    p.pos.y = floor;
                    p.vel_y = 0.0;
                    p.on_ground = true;
                    let dmg = if slow_fall || liquid == Liquid::Water { 0 } else { p.fall_damage(drop) };
                    if dmg > 0 {
                        self.events.push(Event::Sound(Sfx::Thud));
                        self.hurt_player(dmg, "You land hard and hurt yourself.");
                    } else if drop > 1.4 && !slow_fall && liquid != Liquid::Water {
                        self.events.push(Event::Sound(Sfx::Thud));
                        self.msg("You roll as you land and come up unhurt.");
                    } else if drop > 0.4 {
                        self.events.push(Event::Sound(if liquid == Liquid::Water { Sfx::Splash } else { Sfx::Step }));
                    }
                }
            }
        }
        // Liquids.
        let p = &mut self.player;
        let was_swimming = p.swimming;
        p.swimming = liquid == Liquid::Water && p.pos.y <= floor + 0.05 && !body.flying;
        if p.swimming && !was_swimming {
            self.events.push(Event::Sound(Sfx::Splash));
            self.sys("You are swimming.");
        }
        let p = &mut self.player;
        if p.swimming {
            // Heavy loads drag you down.
            let heavy = p.load() > p.swim_load_limit();
            if heavy {
                p.drown += dt;
                if p.drown > p.drown_interval() {
                    p.drown = 0.0;
                    self.events.push(Event::Flash([40.0, 60.0, 220.0], 0.4));
                    self.hurt_player(2, "You are drowning!");
                }
            }
        } else {
            p.drown = 0.0;
        }
        if liquid == Liquid::Lava && self.player.pos.y <= floor + 0.05 && !body.flying {
            self.player.drown += dt;
            if self.player.drown > 0.5 {
                self.player.drown = 0.0;
                let dmg = if self.player.has_spell(Spell::ResistFire) { 1 } else { 4 };
                self.events.push(Event::Flash([255.0, 80.0, 0.0], 0.35));
                self.hurt_player(dmg, "The lava burns you!");
            }
        }
        if moved && self.player.on_ground && !self.player.swimming {
            // occasional footstep sound
            let t = self.time;
            if (t * 2.2).fract() < (dt as f64 * 2.2) {
                self.events.push(Event::Sound(Sfx::Step));
            }
        }
        // Mark explored tiles around the player for the automap.
        self.explore();
        let new_tile = (self.player.pos.x.floor() as i32, self.player.pos.z.floor() as i32);
        if new_tile != old_tile {
            self.spot_traps(new_tile.0, new_tile.1);
            self.check_triggers(new_tile.0, new_tile.1);
        }
    }

    /// Mark tiles visible from the player as explored.
    pub fn explore(&mut self) {
        let eye = self.eye();
        let cx = eye.x.floor() as i32;
        let cz = eye.z.floor() as i32;
        let r = 4;
        let lv = self.cur;
        for dz in -r..=r {
            for dx in -r..=r {
                let tx = cx + dx;
                let tz = cz + dz;
                if !crate::world::map::Map::in_bounds(tx, tz) {
                    continue;
                }
                if self.levels[lv].map.get(tx, tz).explored {
                    continue;
                }
                let target = V3::new(tx as f32 + 0.5, eye.y, tz as f32 + 0.5);
                let near = dx.abs() <= 1 && dz.abs() <= 1;
                let solid = self.levels[lv].map.get(tx, tz).is_solid();
                if near || (!solid && physics::line_of_sight(&self.levels[lv], eye, target)) {
                    if let Some(t) = self.levels[lv].map.get_mut(tx, tz) {
                        t.explored = true;
                    }
                }
            }
        }
    }

    pub fn check_triggers(&mut self, tx: i32, tz: i32) {
        let mut fire: Vec<usize> = Vec::new();
        for (i, t) in self.level().triggers.iter().enumerate() {
            if t.tx == tx && t.tz == tz && !(t.once && t.fired) {
                fire.push(i);
            }
        }
        for i in fire {
            let kind = {
                let t = &mut self.level_mut().triggers[i];
                t.fired = true;
                t.kind.clone()
            };
            match kind {
                TriggerKind::Stairs { level, x, z, yaw } => {
                    self.change_level(level, x, z, yaw);
                    return;
                }
                TriggerKind::Teleport { x, z } => {
                    let y = self.level().map.floor_at(x, z);
                    self.player.pos = V3::new(x, y, z);
                    self.sound(Sfx::Spell);
                }
                TriggerKind::Message { text } => self.msg(text),
                TriggerKind::OpenDoor { door } => {
                    if let Some(d) = self.level_mut().doors.iter_mut().find(|d| d.id == door) {
                        d.opening = true;
                        d.locked = false;
                    }
                    self.sound(Sfx::DoorOpen);
                }
                TriggerKind::Damage { amount } => {
                    self.spring_trap(tx, tz, amount);
                }
                TriggerKind::Flag { name, value } => self.set_flag(&name, value),
                TriggerKind::Victory => self.events.push(Event::Victory),
            }
        }
    }

    pub fn change_level(&mut self, level: usize, x: f32, z: f32, yaw: f32) {
        if level >= self.levels.len() {
            return;
        }
        let going_down = level > self.cur;
        self.cur = level;
        let y = self.levels[level].map.floor_at(x, z);
        self.player.pos = V3::new(x, y, z);
        self.player.yaw = yaw;
        self.player.vel_y = 0.0;
        self.player.on_ground = true;
        self.projectiles.clear();
        self.effects.clear();
        self.foe = None;
        self.events.push(Event::LevelChanged);
        let name = self.levels[level].name.clone();
        self.sys(format!("You {} to {}.", if going_down { "descend" } else { "climb" }, name));
        self.explore();
    }

    pub fn hurt_player(&mut self, dmg: i32, why: &str) {
        if self.player.dead || dmg <= 0 {
            return;
        }
        let mut dmg = dmg;
        if self.player.has_spell(Spell::IronFlesh) {
            dmg = (dmg + 1) / 2;
        }
        self.player.vit -= dmg;
        if !why.is_empty() {
            self.msg(why.to_string());
        }
        self.events.push(Event::Shake(0.25));
        self.events.push(Event::Sound(Sfx::PlayerHurt));
        if self.player.vit <= self.player.max_vit / 5 {
            self.events.push(Event::Flash([220.0, 0.0, 0.0], 0.35));
        }
        if self.player.vit <= 0 {
            self.player.vit = 0;
            self.player.dead = true;
            self.events.push(Event::PlayerDied);
        }
    }

    pub fn heal_player(&mut self, amount: i32) {
        self.player.vit = (self.player.vit + amount).min(self.player.max_vit);
    }

    pub fn gain_exp(&mut self, amount: i32) {
        self.player.exp += amount;
        while self.player.level < 16 && self.player.exp >= Player::exp_for_level(self.player.level + 1) {
            self.player.level += 1;
            let p = &mut self.player;
            let vit_gain = 3 + p.st / 6;
            let mana_gain = 1 + p.int / 6 + p.skills[items::Skill::Mana as usize] / 4;
            p.max_vit += vit_gain;
            p.vit += vit_gain;
            p.max_mana += mana_gain;
            p.mana += mana_gain;
            p.skill_points += 3;
            let lvl = p.level;
            self.sound(Sfx::LevelUp);
            self.sys(format!("You have attained experience level {}! You may improve your skills.", lvl));
        }
    }

    fn update_doors(&mut self, dt: f32) {
        for d in self.levels[self.cur].doors.iter_mut() {
            let target = if d.opening { 1.0 } else { 0.0 };
            if d.open < target {
                d.open = (d.open + dt * 1.6).min(1.0);
            } else if d.open > target {
                d.open = (d.open - dt * 1.6).max(0.0);
            }
        }
    }

    fn update_timers(&mut self, dt: f32) {
        let p = &mut self.player;
        // Hunger and fatigue.
        let before = p.hunger;
        p.hunger = (p.hunger - dt / 20.0).max(0.0);
        p.fatigue = (p.fatigue + dt / 36.0).min(100.0);
        let hunger_msg = if before >= 25.0 && p.hunger < 25.0 {
            Some("You are getting hungry.")
        } else if before >= 8.0 && p.hunger < 8.0 {
            Some("You are starving!")
        } else {
            None
        };
        // Regeneration.
        let p = &mut self.player;
        if p.hunger > 0.0 {
            p.regen += dt / if p.hunger > 25.0 { 14.0 } else { 30.0 };
        } else {
            p.regen -= dt / 15.0;
        }
        let mana_rate = 7.0 - (p.skills[items::Skill::Mana as usize] as f32 * 0.15).min(4.0);
        p.mana_regen += dt / mana_rate;
        let mut vit_delta = 0;
        while p.regen >= 1.0 {
            p.regen -= 1.0;
            vit_delta += 1;
        }
        while p.regen <= -1.0 {
            p.regen += 1.0;
            vit_delta -= 1;
        }
        while p.mana_regen >= 1.0 {
            p.mana_regen -= 1.0;
            if p.mana < p.max_mana {
                p.mana += 1;
            }
        }
        if vit_delta > 0 && p.poison <= 0.0 {
            p.vit = (p.vit + vit_delta).min(p.max_vit);
        }
        if let Some(m) = hunger_msg {
            self.msg(m);
        }
        if vit_delta < 0 {
            self.hurt_player(-vit_delta, "You are starving!");
        }
        // Poison.
        if self.player.poison > 0.0 {
            let before = self.player.poison;
            self.player.poison -= dt;
            if (before / 4.0).floor() != (self.player.poison / 4.0).floor() {
                self.hurt_player(1, "");
            }
            if self.player.poison <= 0.0 {
                self.player.poison = 0.0;
                self.msg("The poison has worn off.");
            }
        }
        // Light sources burn.
        let mut burnt: Vec<String> = Vec::new();
        for s in [EqSlot::ShoulderL, EqSlot::ShoulderR, EqSlot::Weapon, EqSlot::Shield] {
            if let Some(it) = self.player.equip.get_mut(s) {
                if it.lit && it.def().light.map(|l| l.2 > 0.0).unwrap_or(false) {
                    it.fuel -= dt;
                    if it.fuel <= 0.0 {
                        it.lit = false;
                        it.fuel = 0.0;
                        burnt.push(it.def().name.to_string());
                    }
                }
            }
        }
        for name in burnt {
            self.msg(format!("Your {} has burned out.", name));
        }
        // Active spells.
        let mut expired: Vec<Spell> = Vec::new();
        for a in self.player.spells.iter_mut() {
            a.left -= dt;
            if a.left <= 0.0 {
                expired.push(a.spell);
            }
        }
        self.player.spells.retain(|a| a.left > 0.0);
        for s in expired {
            self.msg(format!("The {} spell has worn off.", s.def().name));
        }
    }

    fn update_effects(&mut self, dt: f32) {
        for e in self.effects.iter_mut() {
            e.t += dt;
        }
        self.effects.retain(|e| e.t < 0.6);
    }

    pub fn spawn_effect(&mut self, pos: V3, kind: EffectKind) {
        self.effects.push(Effect { pos, kind, t: 0.0 });
    }

    fn update_projectiles(&mut self, dt: f32) {
        let mut i = 0;
        while i < self.projectiles.len() {
            let mut pr = self.projectiles[i].clone();
            pr.life -= dt;
            let steps = 4;
            let mut hit: Option<Option<u32>> = None; // Some(None) = wall, Some(Some(id)) = npc, player = id 0
            for _ in 0..steps {
                pr.pos = pr.pos + pr.vel * (dt / steps as f32);
                let lv = &self.levels[self.cur];
                let f = lv.map.floor_at(pr.pos.x, pr.pos.z);
                let c = lv.map.ceil_at(pr.pos.x, pr.pos.z);
                if !lv.map.point_open(pr.pos.x, pr.pos.z) || pr.pos.y < f || pr.pos.y > c || physics::door_blocks(lv, pr.pos.x, pr.pos.z, 0.02) {
                    hit = Some(None);
                    break;
                }
                if pr.from_player {
                    if let Some(n) = lv.npcs.iter().find(|n| {
                        let d = n.race.def();
                        n.alive() && (n.pos.x - pr.pos.x).powi(2) + (n.pos.z - pr.pos.z).powi(2) < (d.radius + 0.1).powi(2) && pr.pos.y > n.pos.y - 0.1 && pr.pos.y < n.pos.y + d.height + 0.1
                    }) {
                        hit = Some(Some(n.id));
                        break;
                    }
                } else {
                    let p = &self.player.pos;
                    if (p.x - pr.pos.x).powi(2) + (p.z - pr.pos.z).powi(2) < (PLAYER_RADIUS + 0.1).powi(2) && pr.pos.y > p.y - 0.1 && pr.pos.y < p.y + PLAYER_HEIGHT + 0.1 {
                        hit = Some(Some(0));
                        break;
                    }
                }
            }
            if pr.life <= 0.0 && hit.is_none() {
                hit = Some(None);
            }
            match hit {
                None => {
                    self.projectiles[i] = pr;
                    i += 1;
                }
                Some(target) => {
                    self.projectiles.remove(i);
                    let kind = match pr.kind {
                        ProjKind::Fireball => EffectKind::Explosion,
                        ProjKind::Lightning | ProjKind::MagicArrow => EffectKind::Sparks,
                        _ => EffectKind::Smoke,
                    };
                    self.spawn_effect(pr.pos, kind);
                    if pr.kind == ProjKind::Fireball {
                        self.sound(Sfx::Fire);
                        // Splash damage.
                        let center = pr.pos;
                        let ids: Vec<u32> = self.level().npcs.iter().filter(|n| n.alive() && (n.pos - center).len() < 1.3).map(|n| n.id).collect();
                        for id in ids {
                            if Some(id) != target {
                                ai::damage_npc(self, id, pr.dmg / 2, true);
                            }
                        }
                        if !pr.from_player || (self.player.pos - center).len() < 1.0 {
                            if (self.player.pos - center).len() < 1.0 && target != Some(0) {
                                self.hurt_player(pr.dmg / 3, "You are caught in the blast!");
                            }
                        }
                    }
                    match target {
                        Some(0) if !pr.from_player => {
                            let mut dmg = pr.dmg;
                            if self.player.has_spell(Spell::MissileProtection) {
                                dmg /= 3;
                            }
                            let why = match pr.kind {
                                ProjKind::Fireball => "You are burned by fire!",
                                ProjKind::Lightning => "You are struck by lightning!",
                                ProjKind::Poison => "You are hit by poison!",
                                _ => "You are hit!",
                            };
                            if pr.kind == ProjKind::Poison {
                                self.player.poison += 12.0;
                            }
                            self.hurt_player(dmg, why);
                        }
                        Some(id) if pr.from_player => {
                            ai::damage_npc(self, id, pr.dmg, true);
                            if pr.kind == ProjKind::Poison {
                                if let Some(n) = self.level_mut().npc_mut(id) {
                                    n.poisoned += 15.0;
                                }
                            }
                        }
                        _ => {
                            if matches!(pr.kind, ProjKind::Stone | ProjKind::Bolt) {
                                // Ammunition falls to the floor.
                                let kind = if pr.kind == ProjKind::Stone { ItemKind::SlingStone } else { ItemKind::Bolt };
                                let back = pr.pos - pr.vel.norm() * 0.2;
                                self.level_mut().add_item(Item::new(kind), back.x, back.z);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Message history lines for the scroll (newest last).
    pub fn recent_messages(&self, n: usize) -> &[Msg] {
        let len = self.messages.len();
        &self.messages[len.saturating_sub(n)..]
    }
}

#[cfg(test)]
mod fuzz_tests {
    use super::*;

    /// Random movement and fighting on every level must never put the player
    /// inside rock or below the floor, and must never panic.
    #[test]
    fn random_walks_keep_player_in_bounds() {
        let mut rng = Rng::new(99);
        let mut p = Player::new("Fuzz", PClass::Fighter, false);
        p.max_vit = 10_000;
        p.vit = 10_000;
        let mut g = Game::new(p, 5);
        for lvl in 0..g.levels.len() {
            let (x, z, yaw) = g.levels[lvl].start;
            g.change_level(lvl, x, z, yaw);
            for frame in 0..2400 {
                let mv = MoveInput {
                    forward: rng.rangef(-1.0, 2.0),
                    strafe: if rng.chance(0.2) { rng.rangef(-1.0, 1.0) } else { 0.0 },
                    turn: if frame % 40 < 10 { rng.rangef(-1.0, 1.0) } else { 0.0 },
                    jump: rng.chance(0.01),
                    fly_up: false,
                    fly_down: false,
                };
                if rng.chance(0.02) {
                    let aim = V3::new(g.player.yaw.sin(), 0.0, -g.player.yaw.cos());
                    ai::player_attack(&mut g, aim, rng.range(0, 3) as usize, rng.f32());
                }
                g.update(&mv, 1.0 / 30.0);
                g.events.clear();
                g.player.vit = g.player.max_vit;
                g.player.dead = false;
                g.player.hunger = 80.0;
                let pp = g.player.pos;
                let lv = g.level();
                assert!(lv.map.point_open(pp.x, pp.z), "level {} frame {}: player in rock at {:?}", lvl, frame, pp);
                let f = lv.map.floor_at(pp.x, pp.z);
                assert!(pp.y >= f - 0.05, "level {} frame {}: below floor {:?} (floor {})", lvl, frame, pp, f);
                if g.cur != lvl {
                    // Took the stairs: come back.
                    let (x, z, yaw) = g.levels[lvl].start;
                    g.change_level(lvl, x, z, yaw);
                }
            }
        }
    }
}
