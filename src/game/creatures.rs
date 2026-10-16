//! Creature (critter and NPC) definitions and instances.

use super::items::Item;
use crate::util::V3;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Hash)]
pub enum Race {
    Rat,
    Bat,
    GreenGoblin,
    GrayGoblin,
    Skeleton,
    Spider,
    Slug,
    Lizardman,
    Dwarf,
    Human,
    Mage,
    Ghoul,
    Troll,
    Ghost,
    Golem,
    Headless,
    Lurker,
    Imp,
    Reaper,
    Demon,
}

pub const ALL_RACES: [Race; 20] = [
    Race::Rat,
    Race::Bat,
    Race::GreenGoblin,
    Race::GrayGoblin,
    Race::Skeleton,
    Race::Spider,
    Race::Slug,
    Race::Lizardman,
    Race::Dwarf,
    Race::Human,
    Race::Mage,
    Race::Ghoul,
    Race::Troll,
    Race::Ghost,
    Race::Golem,
    Race::Headless,
    Race::Lurker,
    Race::Imp,
    Race::Reaper,
    Race::Demon,
];

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Attitude {
    Hostile,
    Upset,
    Mellow,
    Friendly,
}

impl Attitude {
    pub fn word(self) -> &'static str {
        match self {
            Attitude::Hostile => "hostile",
            Attitude::Upset => "upset",
            Attitude::Mellow => "mellow",
            Attitude::Friendly => "friendly",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ranged {
    None,
    /// Casts magic missiles.
    Magic,
    /// Breathes/casts fire.
    Fire,
}

pub struct RaceDef {
    pub name: &'static str,
    pub hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub dmg: (i32, i32),
    pub speed: f32,
    pub exp: i32,
    pub flies: bool,
    pub swims: bool,
    pub hostile: bool,
    pub ranged: Ranged,
    pub poisons: bool,
    pub undead: bool,
    pub radius: f32,
    /// Model height in world units (sprite scale).
    pub height: f32,
    /// Hover height for flying creatures.
    pub hover: f32,
}

impl Race {
    pub fn def(self) -> RaceDef {
        use Race::*;
        let base = RaceDef {
            name: "creature",
            hp: 10,
            attack: 5,
            defense: 3,
            dmg: (1, 4),
            speed: 1.0,
            exp: 10,
            flies: false,
            swims: false,
            hostile: true,
            ranged: Ranged::None,
            poisons: false,
            undead: false,
            radius: 0.2,
            height: 1.00,
            hover: 0.00,
        };
        match self {
            Rat => RaceDef { name: "giant rat", hp: 6, attack: 4, defense: 2, dmg: (1, 3), speed: 1.4, exp: 4, radius: 0.18, height: 0.44, ..base },
            Bat => RaceDef { name: "cave bat", hp: 5, attack: 5, defense: 5, dmg: (1, 3), speed: 1.6, exp: 5, flies: true, radius: 0.15, height: 0.38, hover: 0.56, ..base },
            GreenGoblin => RaceDef { name: "green goblin", hp: 14, attack: 7, defense: 4, dmg: (2, 6), speed: 1.0, exp: 15, height: 0.83, ..base },
            GrayGoblin => RaceDef { name: "gray goblin", hp: 16, attack: 8, defense: 5, dmg: (2, 7), speed: 1.0, exp: 18, height: 0.85, ..base },
            Skeleton => RaceDef { name: "skeleton", hp: 18, attack: 9, defense: 6, dmg: (2, 8), speed: 0.9, exp: 25, undead: true, height: 1.02, ..base },
            Spider => RaceDef { name: "giant spider", hp: 16, attack: 9, defense: 6, dmg: (2, 6), speed: 1.3, exp: 22, poisons: true, radius: 0.3, height: 0.50, ..base },
            Slug => RaceDef { name: "acid slug", hp: 12, attack: 6, defense: 3, dmg: (2, 5), speed: 0.5, exp: 12, radius: 0.25, height: 0.38, ..base },
            Lizardman => RaceDef { name: "lizardman", hp: 22, attack: 10, defense: 7, dmg: (3, 8), speed: 1.1, exp: 30, swims: true, height: 1.00, ..base },
            Dwarf => RaceDef { name: "mountainman", hp: 30, attack: 12, defense: 9, dmg: (3, 9), speed: 0.9, exp: 40, hostile: false, height: 0.83, ..base },
            Human => RaceDef { name: "outcast", hp: 24, attack: 10, defense: 8, dmg: (2, 8), speed: 1.0, exp: 30, hostile: false, height: 1.02, ..base },
            Mage => RaceDef { name: "mage", hp: 20, attack: 8, defense: 6, dmg: (2, 6), speed: 0.9, exp: 45, hostile: false, ranged: Ranged::Magic, height: 1.02, ..base },
            Ghoul => RaceDef { name: "ghoul", hp: 28, attack: 12, defense: 8, dmg: (3, 9), speed: 0.8, exp: 50, undead: true, poisons: true, height: 1.00, ..base },
            Troll => RaceDef { name: "troll", hp: 60, attack: 16, defense: 12, dmg: (5, 14), speed: 0.9, exp: 110, radius: 0.28, height: 1.31, ..base },
            Ghost => RaceDef { name: "ghost", hp: 30, attack: 14, defense: 14, dmg: (3, 10), speed: 1.0, exp: 90, flies: true, undead: true, height: 1.06, hover: 0.12, ..base },
            Golem => RaceDef { name: "earth golem", hp: 90, attack: 18, defense: 18, dmg: (6, 16), speed: 0.7, exp: 160, radius: 0.3, height: 1.38, ..base },
            Headless => RaceDef { name: "headless", hp: 40, attack: 14, defense: 10, dmg: (4, 12), speed: 1.0, exp: 80, height: 1.06, ..base },
            Lurker => RaceDef { name: "lurker", hp: 45, attack: 14, defense: 10, dmg: (4, 12), speed: 0.8, exp: 90, swims: true, radius: 0.35, height: 0.69, ..base },
            Imp => RaceDef { name: "imp", hp: 18, attack: 12, defense: 10, dmg: (2, 6), speed: 1.5, exp: 50, flies: true, ranged: Ranged::Fire, height: 0.62, hover: 0.44, ..base },
            Reaper => RaceDef { name: "reaper", hp: 70, attack: 17, defense: 14, dmg: (5, 15), speed: 0.6, exp: 150, radius: 0.3, height: 1.38, ..base },
            Demon => RaceDef { name: "lord of souls", hp: 220, attack: 24, defense: 20, dmg: (8, 22), speed: 1.0, exp: 1000, ranged: Ranged::Fire, radius: 0.35, height: 1.69, ..base },
        }
    }

}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum AiState {
    /// Stand around near home.
    Idle,
    /// Wander randomly near home.
    Wander,
    /// Chase and attack the player.
    Attack,
    /// Run away from the player.
    Flee,
    /// Follow the player (allies).
    Follow,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Anim {
    Idle,
    Walk,
    Attack,
    Hit,
    Die,
    Dead,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Npc {
    pub id: u32,
    pub race: Race,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub look: u8,
    pub pos: V3,
    pub yaw: f32,
    pub hp: i32,
    pub max_hp: i32,
    pub attitude: Attitude,
    pub ai: AiState,
    pub home: (f32, f32),
    /// Conversation id (index into the dialogue table).
    #[serde(default)]
    pub talk: Option<u16>,
    #[serde(default)]
    pub inventory: Vec<Item>,
    pub anim: Anim,
    pub anim_t: f32,
    #[serde(default)]
    pub cooldown: f32,
    #[serde(default)]
    pub vel_y: f32,
    #[serde(default)]
    pub paralyzed: f32,
    #[serde(default)]
    pub feared: f32,
    #[serde(default)]
    pub poisoned: f32,
    #[serde(default)]
    pub wander_to: Option<(f32, f32)>,
    /// `THRALL` / `MASTER` bits.
    #[serde(default)]
    pub flags: u32,
    /// Seconds since the NPC last saw the player.
    #[serde(default)]
    pub lost_sight: f32,
    /// Attack wind-up progress.
    #[serde(default)]
    pub swing: f32,
}

/// `Npc::flags`: an undead kept alive by its master's power.
pub const THRALL: u32 = 1;
/// `Npc::flags`: when this NPC dies, every thrall on the level crumbles.
pub const MASTER: u32 = 2;

impl Npc {
    pub fn new(id: u32, race: Race, x: f32, z: f32) -> Npc {
        let d = race.def();
        Npc {
            id,
            race,
            name: None,
            look: 0,
            pos: V3::new(x, 0.0, z),
            yaw: 0.0,
            hp: d.hp,
            max_hp: d.hp,
            attitude: if d.hostile { Attitude::Hostile } else { Attitude::Mellow },
            ai: if d.hostile { AiState::Wander } else { AiState::Idle },
            home: (x, z),
            talk: None,
            inventory: Vec::new(),
            anim: Anim::Idle,
            anim_t: 0.0,
            cooldown: 0.0,
            vel_y: 0.0,
            paralyzed: 0.0,
            feared: 0.0,
            poisoned: 0.0,
            wander_to: None,
            flags: 0,
            lost_sight: 99.0,
            swing: 0.0,
        }
    }

    pub fn alive(&self) -> bool {
        !matches!(self.anim, Anim::Die | Anim::Dead)
    }

    /// "a hostile giant rat", "a friendly mountainman named Goldthirst"
    pub fn describe(&self) -> String {
        let d = self.race.def();
        let att = self.attitude.word();
        let first = att.chars().next().unwrap_or('x');
        let art = if "aeiou".contains(first) { "an" } else { "a" };
        match &self.name {
            Some(n) => format!("{} {} {} named {}", art, att, d.name, n),
            None => format!("{} {} {}", art, att, d.name),
        }
    }

    pub fn health_word(&self) -> &'static str {
        let f = self.hp as f32 / self.max_hp.max(1) as f32;
        if f > 0.8 {
            "unhurt"
        } else if f > 0.55 {
            "wounded"
        } else if f > 0.3 {
            "badly wounded"
        } else {
            "near death"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describe_uses_articles() {
        let mut n = Npc::new(1, Race::Rat, 1.0, 1.0);
        assert_eq!(n.describe(), "a hostile giant rat");
        n.attitude = Attitude::Upset;
        assert_eq!(n.describe(), "an upset giant rat");
        n.name = Some("Squeak".into());
        assert_eq!(n.describe(), "an upset giant rat named Squeak");
    }
}
