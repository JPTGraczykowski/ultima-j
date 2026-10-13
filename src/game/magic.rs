//! Rune magic: the 24 runes and the 40 spells of eight circles.

use serde::{Deserialize, Serialize};

pub const RUNE_NAMES: [&str; 24] = [
    "An", "Bet", "Corp", "Des", "Ex", "Flam", "Grav", "Hur", "In", "Jux", "Kal", "Lor", "Mani", "Nox", "Ort", "Por",
    "Quas", "Rel", "Sanct", "Tym", "Uus", "Vas", "Wis", "Ylem",
];

pub const RUNE_MEANINGS: [&str; 24] = [
    "Negate", "Small", "Death", "Down", "Freedom", "Flame", "Energy", "Wind", "Create", "Danger", "Summon", "Light",
    "Life", "Poison", "Magic", "Movement", "Illusion", "Change", "Protect", "Time", "Raise", "Great", "Knowledge",
    "Matter",
];

pub mod rune {
    pub const AN: u8 = 0;
    pub const BET: u8 = 1;
    pub const CORP: u8 = 2;
    pub const DES: u8 = 3;
    pub const EX: u8 = 4;
    pub const FLAM: u8 = 5;
    pub const GRAV: u8 = 6;
    pub const HUR: u8 = 7;
    pub const IN: u8 = 8;
    pub const JUX: u8 = 9;
    pub const KAL: u8 = 10;
    pub const LOR: u8 = 11;
    pub const MANI: u8 = 12;
    pub const NOX: u8 = 13;
    pub const ORT: u8 = 14;
    pub const POR: u8 = 15;
    pub const QUAS: u8 = 16;
    pub const REL: u8 = 17;
    pub const SANCT: u8 = 18;
    pub const TYM: u8 = 19;
    pub const UUS: u8 = 20;
    pub const VAS: u8 = 21;
    pub const WIS: u8 = 22;
    pub const YLEM: u8 = 23;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Hash)]
pub enum Spell {
    CreateFood,
    Light,
    MagicArrow,
    ResistBlows,
    Stealth,
    CauseFear,
    DetectMonster,
    LesserHeal,
    RuneOfWarding,
    SlowFall,
    Conceal,
    Lightning,
    NightVision,
    Speed,
    StrengthenDoor,
    Heal,
    Levitate,
    Poison,
    RemoveTrap,
    ResistFire,
    CurePoison,
    Fireball,
    MissileProtection,
    NameEnchantment,
    Open,
    Daylight,
    GateTravel,
    GreaterHeal,
    Paralyze,
    Telekinesis,
    Ally,
    Confusion,
    Fly,
    Invisibility,
    Reveal,
    FlameWind,
    FreezeTime,
    IronFlesh,
    RoamingSight,
    Tremor,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Targeting {
    /// Takes effect immediately on the caster.
    Caster,
    /// A missile or effect aimed at a creature (red circle cursor).
    Combat,
    /// Aimed at an object (blue cross cursor).
    Object,
}

pub struct SpellDef {
    pub name: &'static str,
    pub circle: i32,
    pub runes: &'static [u8],
    pub target: Targeting,
    /// Duration in seconds for lasting effects (0 = instant).
    pub duration: f32,
}

pub const ALL_SPELLS: [Spell; 40] = [
    Spell::CreateFood,
    Spell::Light,
    Spell::MagicArrow,
    Spell::ResistBlows,
    Spell::Stealth,
    Spell::CauseFear,
    Spell::DetectMonster,
    Spell::LesserHeal,
    Spell::RuneOfWarding,
    Spell::SlowFall,
    Spell::Conceal,
    Spell::Lightning,
    Spell::NightVision,
    Spell::Speed,
    Spell::StrengthenDoor,
    Spell::Heal,
    Spell::Levitate,
    Spell::Poison,
    Spell::RemoveTrap,
    Spell::ResistFire,
    Spell::CurePoison,
    Spell::Fireball,
    Spell::MissileProtection,
    Spell::NameEnchantment,
    Spell::Open,
    Spell::Daylight,
    Spell::GateTravel,
    Spell::GreaterHeal,
    Spell::Paralyze,
    Spell::Telekinesis,
    Spell::Ally,
    Spell::Confusion,
    Spell::Fly,
    Spell::Invisibility,
    Spell::Reveal,
    Spell::FlameWind,
    Spell::FreezeTime,
    Spell::IronFlesh,
    Spell::RoamingSight,
    Spell::Tremor,
];

impl Spell {
    pub fn def(self) -> SpellDef {
        use rune::*;
        use Spell::*;
        use Targeting::*;
        let s = |name, circle, runes, target, duration| SpellDef { name, circle, runes, target, duration };
        match self {
            CreateFood => s("Create Food", 1, &[IN, MANI, YLEM], Caster, 0.0),
            Light => s("Light", 1, &[IN, LOR], Caster, 240.0),
            MagicArrow => s("Magic Arrow", 1, &[ORT, JUX], Combat, 0.0),
            ResistBlows => s("Resist Blows", 1, &[BET, IN, SANCT], Caster, 120.0),
            Stealth => s("Stealth", 1, &[SANCT, HUR], Caster, 120.0),
            CauseFear => s("Cause Fear", 2, &[QUAS, CORP], Caster, 0.0),
            DetectMonster => s("Detect Monster", 2, &[WIS, MANI], Caster, 0.0),
            LesserHeal => s("Lesser Heal", 2, &[IN, BET, MANI], Caster, 0.0),
            RuneOfWarding => s("Rune of Warding", 2, &[IN, JUX], Caster, 0.0),
            SlowFall => s("Slow Fall", 2, &[REL, DES, POR], Caster, 60.0),
            Conceal => s("Conceal", 3, &[BET, SANCT, LOR], Caster, 90.0),
            Lightning => s("Lightning", 3, &[ORT, GRAV], Combat, 0.0),
            NightVision => s("Night Vision", 3, &[QUAS, LOR], Caster, 240.0),
            Speed => s("Speed", 3, &[REL, TYM, POR], Caster, 60.0),
            StrengthenDoor => s("Strengthen Door", 3, &[SANCT, JUX], Object, 0.0),
            Heal => s("Heal", 4, &[IN, MANI], Caster, 0.0),
            Levitate => s("Levitate", 4, &[HUR, POR], Caster, 60.0),
            Poison => s("Poison", 4, &[NOX, MANI], Combat, 0.0),
            RemoveTrap => s("Remove Trap", 4, &[AN, JUX], Object, 0.0),
            ResistFire => s("Resist Fire", 4, &[SANCT, FLAM], Caster, 120.0),
            CurePoison => s("Cure Poison", 5, &[AN, NOX], Caster, 0.0),
            Fireball => s("Fireball", 5, &[POR, FLAM], Combat, 0.0),
            MissileProtection => s("Missile Protection", 5, &[GRAV, SANCT, POR], Caster, 120.0),
            NameEnchantment => s("Name Enchantment", 5, &[ORT, WIS, YLEM], Object, 0.0),
            Open => s("Open", 5, &[EX, YLEM], Object, 0.0),
            Daylight => s("Daylight", 6, &[VAS, IN, LOR], Caster, 300.0),
            GateTravel => s("Gate Travel", 6, &[VAS, REL, POR], Caster, 0.0),
            GreaterHeal => s("Greater Heal", 6, &[VAS, IN, MANI], Caster, 0.0),
            Paralyze => s("Paralyze", 6, &[AN, EX, POR], Combat, 0.0),
            Telekinesis => s("Telekinesis", 6, &[ORT, POR, YLEM], Caster, 60.0),
            Ally => s("Ally", 7, &[IN, MANI, REL], Combat, 0.0),
            Confusion => s("Confusion", 7, &[VAS, AN, WIS], Caster, 0.0),
            Fly => s("Fly", 7, &[VAS, HUR, POR], Caster, 60.0),
            Invisibility => s("Invisibility", 7, &[VAS, SANCT, LOR], Caster, 90.0),
            Reveal => s("Reveal", 7, &[ORT, AN, QUAS], Caster, 0.0),
            FlameWind => s("Flame Wind", 8, &[FLAM, HUR], Caster, 0.0),
            FreezeTime => s("Freeze Time", 8, &[AN, TYM], Caster, 15.0),
            IronFlesh => s("Iron Flesh", 8, &[IN, VAS, SANCT], Caster, 120.0),
            RoamingSight => s("Roaming Sight", 8, &[ORT, POR, WIS], Caster, 0.0),
            Tremor => s("Tremor", 8, &[VAS, POR, YLEM], Caster, 0.0),
        }
    }

    pub fn from_runes(runes: &[u8]) -> Option<Spell> {
        ALL_SPELLS.iter().copied().find(|s| s.def().runes == runes)
    }

    pub fn mana_cost(self) -> i32 {
        self.def().circle * 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rune_lookup_roundtrip() {
        for s in ALL_SPELLS {
            assert_eq!(Spell::from_runes(s.def().runes), Some(s), "{:?}", s);
        }
        assert_eq!(Spell::from_runes(&[rune::IN, rune::LOR]), Some(Spell::Light));
        assert_eq!(Spell::from_runes(&[rune::LOR, rune::IN]), None);
    }

    #[test]
    fn circles_have_five_spells() {
        for circle in 1..=8 {
            assert_eq!(ALL_SPELLS.iter().filter(|s| s.def().circle == circle).count(), 5);
        }
    }
}
