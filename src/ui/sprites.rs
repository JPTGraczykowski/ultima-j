//! Lazily generated sprite cache.

use crate::art::creatures::{self, CreatureSprites, Outfit, F_ATTACK, F_CORPSE, F_DIE, F_HIT, F_IDLE, F_WALK};
use crate::art::items;
use crate::art::sprite3d::{CMat, Pattern};
use crate::game::creatures::{Anim, Npc, Race};
use crate::game::items::{Item, ItemKind};
use crate::game::{EqSlot, Player};
use crate::gfx::palette::Ramp;
use crate::gfx::Bitmap;
use crate::util::{wrap_angle, V3};
use std::collections::HashMap;

pub struct SpriteCache {
    creatures: HashMap<(Race, u8), CreatureSprites>,
    icons: HashMap<(ItemKind, u16), Bitmap>,
    fixtures: HashMap<ItemKind, Bitmap>,
    pub effects: Vec<Vec<Bitmap>>,
    portraits: HashMap<(Race, u8), Bitmap>,
    paperdoll: Option<(String, Bitmap)>,
    pub player_portrait: Option<(String, Bitmap)>,
    hands: HashMap<(Option<ItemKind>, u8), Bitmap>,
}

impl Default for SpriteCache {
    fn default() -> Self {
        Self::new()
    }
}

fn icon_variant(it: &Item) -> u16 {
    match it.kind {
        ItemKind::RuneStone => it.data,
        ItemKind::Coin => (it.qty > 1) as u16,
        ItemKind::Torch | ItemKind::Candle | ItemKind::Lantern => it.lit as u16,
        _ => 0,
    }
}

impl SpriteCache {
    pub fn new() -> SpriteCache {
        let effects = (0..10u8).map(|k| (0..4).map(|f| items::effect_sprite(k, f)).collect()).collect();
        SpriteCache {
            creatures: HashMap::new(),
            icons: HashMap::new(),
            fixtures: HashMap::new(),
            effects,
            portraits: HashMap::new(),
            paperdoll: None,
            player_portrait: None,
            hands: HashMap::new(),
        }
    }

    pub fn ensure_creature(&mut self, race: Race, look: u8) {
        self.creatures.entry((race, look)).or_insert_with(|| creatures::render_creature(race, look));
    }

    pub fn creature(&self, race: Race, look: u8) -> Option<&CreatureSprites> {
        self.creatures.get(&(race, look))
    }

    pub fn ensure_item(&mut self, it: &Item) {
        if it.kind.is_fixture() {
            self.fixtures.entry(it.kind).or_insert_with(|| items::fixture(it.kind));
        } else {
            let v = icon_variant(it);
            self.icons.entry((it.kind, v)).or_insert_with(|| items::icon(it.kind, v, v == 1));
        }
    }

    /// Inventory/world icon for a portable item (ensure_item first).
    pub fn icon(&self, it: &Item) -> Option<&Bitmap> {
        if it.kind.is_fixture() {
            self.fixtures.get(&it.kind)
        } else {
            self.icons.get(&(it.kind, icon_variant(it)))
        }
    }

    pub fn icon_mut(&mut self, it: &Item) -> &Bitmap {
        self.ensure_item(it);
        self.icon(it).unwrap()
    }

    /// World sprite and its world size.
    pub fn world_sprite(&self, it: &Item) -> Option<(&Bitmap, (f32, f32))> {
        if it.kind.is_fixture() {
            self.fixtures.get(&it.kind).map(|b| (b, (b.w as f32 / 64.0, b.h as f32 / 64.0)))
        } else {
            let s = it.def().size;
            self.icons.get(&(it.kind, icon_variant(it))).map(|b| (b, (s.0.max(0.2), s.1.max(0.2))))
        }
    }

    pub fn hand_weapon(&mut self, kind: Option<ItemKind>, phase: u8) -> &Bitmap {
        self.hands.entry((kind, phase)).or_insert_with(|| items::hand_weapon(kind, phase))
    }

    pub fn portrait(&mut self, race: Race, look: u8) -> &Bitmap {
        self.portraits.entry((race, look)).or_insert_with(|| creatures::portrait(race, look, 34))
    }

    /// Paperdoll body image for the current equipment (cached).
    pub fn paperdoll(&mut self, p: &Player) -> &Bitmap {
        let key = paperdoll_key(p);
        if self.paperdoll.as_ref().map(|(k, _)| k != &key).unwrap_or(true) {
            let bmp = creatures::paperdoll(&outfit(p), 46, 72, 80.0);
            self.paperdoll = Some((key, bmp));
        }
        &self.paperdoll.as_ref().unwrap().1
    }

    pub fn player_face(&mut self, p: &Player) -> &Bitmap {
        let key = format!("{}{}{}{:?}", p.female, p.skin, p.hair, p.equip.head.as_ref().map(|i| i.kind));
        if self.player_portrait.as_ref().map(|(k, _)| k != &key).unwrap_or(true) {
            let v = crate::art::sprite3d::View3 {
                yaw: -0.25,
                pitch: 0.05,
                scale: 34.0 / (0.82 * 0.3),
                w: 34,
                h: 34,
                center: V3::new(0.0, 0.82 * 0.86, 0.0),
                center_row: 17.0,
            };
            let bmp = creatures::player_portrait(&outfit(p), &v);
            self.player_portrait = Some((key, bmp));
        }
        &self.player_portrait.as_ref().unwrap().1
    }
}

fn paperdoll_key(p: &Player) -> String {
    let k = |s: EqSlot| p.equip.get(s).as_ref().map(|i| format!("{:?}", i.kind)).unwrap_or_default();
    format!("{}{}{}|{}|{}|{}|{}|{}", p.female, p.skin, p.hair, k(EqSlot::Head), k(EqSlot::Chest), k(EqSlot::Legs), k(EqSlot::Feet), k(EqSlot::Hands))
}

pub fn armor_cmat(kind: ItemKind) -> CMat {
    use ItemKind::*;
    match kind {
        LeatherCap | LeatherVest | LeatherLeggings | LeatherBoots | LeatherGloves => CMat::new(Ramp::Earth, 1, 8, Pattern::Noise(0.2)),
        ChainCowl | ChainShirt | ChainLeggings => CMat::new(Ramp::Grey, 3, 12, Pattern::Chain),
        _ => CMat::new(Ramp::Silver, 3, 14, Pattern::Metal),
    }
}

pub fn outfit(p: &Player) -> Outfit {
    let m = |s: EqSlot| p.equip.get(s).as_ref().map(|i| armor_cmat(i.kind));
    Outfit { female: p.female, beard: p.hair != 1, skin: p.skin, hair: p.hair, head: m(EqSlot::Head), chest: m(EqSlot::Chest), legs: m(EqSlot::Legs), feet: m(EqSlot::Feet), hands: m(EqSlot::Hands) }
}

/// Pick the animation frame, view index and mirroring for a creature seen from `viewer`.
pub fn creature_frame(n: &Npc, viewer: V3) -> (usize, usize, bool) {
    let frame = match n.anim {
        Anim::Idle => F_IDLE,
        Anim::Walk => F_WALK + ((n.anim_t * 6.0) as usize % 4),
        Anim::Attack => F_ATTACK + ((n.anim_t / 0.2) as usize).min(2),
        Anim::Hit => F_HIT,
        Anim::Die => F_DIE + ((n.anim_t / 0.2) as usize).min(2),
        Anim::Dead => F_CORPSE,
    };
    if frame >= F_DIE {
        return (frame, 0, false);
    }
    let dx = viewer.x - n.pos.x;
    let dz = viewer.z - n.pos.z;
    let phi = dx.atan2(-dz);
    let rel = wrap_angle(phi - n.yaw);
    let k = ((rel / (std::f32::consts::PI / 4.0)).round() as i32).rem_euclid(8) as usize;
    if k <= 4 { (frame, k, false) } else { (frame, 8 - k, true) }
}
