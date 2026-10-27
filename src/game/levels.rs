//! Level construction: a small toolkit for carving rooms, corridors, caves,
//! water and lava, and the designs of the five levels of the abyss.

use super::actions::SHARD_GATE;
use super::creatures::{AiState, Attitude, Npc, Race, MASTER, THRALL};
use super::dialogue::DialogueDb;
use super::items::{Enchant, Item, ItemKind};
use super::magic::rune;
use super::state::{DoorAxis, DoorKind, Level, Switch, Trigger, TriggerKind};
use crate::art::textures::Tex;
use crate::util::Rng;
use crate::world::map::{Liquid, Map, Shape, Tile, MAP_SIZE};

/// Builder wrapping a level with convenient carving operations.
pub struct B<'a> {
    pub lv: Level,
    pub floor_tex: Tex,
    pub wall_tex: Tex,
    pub ceil_tex: Tex,
    pub rng: Rng,
    pub db: &'a DialogueDb,
}

impl<'a> B<'a> {
    pub fn new(name: &str, seed: u64, db: &'a DialogueDb) -> B<'a> {
        B { lv: Level::new(name, Map::new()), floor_tex: Tex::Dirt, wall_tex: Tex::RoughStone, ceil_tex: Tex::CeilingRock, rng: Rng::new(seed), db }
    }

    pub fn style(&mut self, floor: Tex, wall: Tex, ceil: Tex) -> &mut Self {
        self.floor_tex = floor;
        self.wall_tex = wall;
        self.ceil_tex = ceil;
        self
    }

    fn open_tile(&self, floor: i16, ceil: i16) -> Tile {
        // Two extra steps of headroom so full-size creatures fit, as in the original.
        Tile::open(floor, ceil + 2, self.floor_tex, self.ceil_tex, self.wall_tex)
    }

    /// Carve a rectangular room (x, z = top-left tile; w, h in tiles).
    pub fn room(&mut self, x: i32, z: i32, w: i32, h: i32, floor: i16, height: i16) -> &mut Self {
        for zz in z..z + h {
            for xx in x..x + w {
                let t = self.open_tile(floor, floor + height);
                self.lv.map.set(xx, zz, t);
            }
        }
        self.paint_walls(x - 1, z - 1, w + 2, h + 2);
        self
    }

    /// Give solid tiles around an area the current wall texture.
    pub fn paint_walls(&mut self, x: i32, z: i32, w: i32, h: i32) {
        for zz in z..z + h {
            for xx in x..x + w {
                if let Some(t) = self.lv.map.get_mut(xx, zz) {
                    if t.is_solid() && t.wall_tex != Tex::Plaque && t.wall_tex != Tex::WallLever {
                        t.wall_tex = self.wall_tex;
                    }
                }
            }
        }
    }

    /// Corridor: horizontal from x0 to x1 along z0, then vertical along x1 to z1.
    pub fn corridor(&mut self, x0: i32, z0: i32, x1: i32, z1: i32, floor: i16, height: i16) -> &mut Self {
        let (ax, bx) = (x0.min(x1), x0.max(x1));
        for x in ax..=bx {
            self.carve(x, z0, floor, height);
        }
        let (az, bz) = (z0.min(z1), z0.max(z1));
        for z in az..=bz {
            self.carve(x1, z, floor, height);
        }
        self
    }

    pub fn carve(&mut self, x: i32, z: i32, floor: i16, height: i16) {
        if self.lv.map.get(x, z).is_solid() && Map::in_bounds(x, z) {
            let t = self.open_tile(floor, floor + height);
            self.lv.map.set(x, z, t);
            self.paint_walls(x - 1, z - 1, 3, 3);
        } else if self.lv.map.get(x, z).is_diag() {
            // A corridor running into a rounded cave corner straightens it.
            if let Some(t) = self.lv.map.get_mut(x, z) {
                t.shape = Shape::Open;
            }
        }
    }

    /// Force a tile open with the given heights (even if already open).
    pub fn tile(&mut self, x: i32, z: i32, floor: i16, height: i16) {
        let t = self.open_tile(floor, floor + height);
        self.lv.map.set(x, z, t);
        self.paint_walls(x - 1, z - 1, 3, 3);
    }

    /// Organic cave area around a centre.
    pub fn cave(&mut self, cx: i32, cz: i32, rx: i32, rz: i32, floor: i16, height: i16) -> &mut Self {
        let seed = self.rng.next_u32();
        for z in cz - rz..=cz + rz {
            for x in cx - rx..=cx + rx {
                let dx = (x - cx) as f32 / rx as f32;
                let dz = (z - cz) as f32 / rz as f32;
                let d = dx * dx + dz * dz;
                let n = crate::util::vnoise2(x as f32 * 0.45, z as f32 * 0.45, 64, seed);
                if d + (n - 0.5) * 0.7 < 0.85 && x > 0 && z > 0 && x < MAP_SIZE - 1 && z < MAP_SIZE - 1 && self.lv.map.get(x, z).is_solid() {
                    let fl = floor + if n > 0.7 { 1 } else { 0 };
                    let t = self.open_tile(fl, floor + height + ((n * 4.0) as i16));
                    self.lv.map.set(x, z, t);
                }
            }
        }
        // Always keep the centre open.
        if self.lv.map.get(cx, cz).is_solid() {
            self.tile(cx, cz, floor, height);
        }
        self.paint_walls(cx - rx - 1, cz - rz - 1, rx * 2 + 3, rz * 2 + 3);
        self.smooth_diagonals(cx - rx - 1, cz - rz - 1, rx * 2 + 3, rz * 2 + 3);
        self
    }

    /// Turn convex corners of open areas into diagonals for a rounder look.
    pub fn smooth_diagonals(&mut self, x: i32, z: i32, w: i32, h: i32) {
        let mut changes = Vec::new();
        for zz in z..z + h {
            for xx in x..x + w {
                let t = self.lv.map.get(xx, zz);
                if t.is_solid() || t.shape != Shape::Open || t.liquid != Liquid::None {
                    continue;
                }
                let s = |dx: i32, dz: i32| self.lv.map.get(xx + dx, zz + dz).is_solid();
                let (n, e, so, wv) = (s(0, -1), s(1, 0), s(0, 1), s(-1, 0));
                let open_count = [n, e, so, wv].iter().filter(|b| !**b).count();
                if open_count != 2 {
                    continue;
                }
                let shape = if n && wv && !e && !so {
                    Some(Shape::DiagSE)
                } else if n && e && !so && !wv {
                    Some(Shape::DiagSW)
                } else if so && wv && !n && !e {
                    Some(Shape::DiagNE)
                } else if so && e && !n && !wv {
                    Some(Shape::DiagNW)
                } else {
                    None
                };
                if let Some(sh) = shape {
                    changes.push((xx, zz, sh));
                }
            }
        }
        for (xx, zz, sh) in changes {
            if let Some(t) = self.lv.map.get_mut(xx, zz) {
                t.shape = sh;
            }
        }
    }

    pub fn floor_tex(&mut self, x: i32, z: i32, w: i32, h: i32, tex: Tex) {
        for zz in z..z + h {
            for xx in x..x + w {
                if let Some(t) = self.lv.map.get_mut(xx, zz) {
                    if !t.is_solid() && t.liquid == Liquid::None {
                        t.floor_tex = tex;
                    }
                }
            }
        }
    }

    pub fn water(&mut self, x: i32, z: i32, w: i32, h: i32, level: i16) {
        self.liquid(x, z, w, h, level, Liquid::Water);
    }

    pub fn lava(&mut self, x: i32, z: i32, w: i32, h: i32, level: i16) {
        self.liquid(x, z, w, h, level, Liquid::Lava);
        for zz in z - 2..z + h + 2 {
            for xx in x - 2..x + w + 2 {
                if let Some(t) = self.lv.map.get_mut(xx, zz) {
                    if !t.is_solid() && t.light < 12 {
                        t.light = 12;
                    }
                }
            }
        }
    }

    fn liquid(&mut self, x: i32, z: i32, w: i32, h: i32, level: i16, l: Liquid) {
        for zz in z..z + h {
            for xx in x..x + w {
                if let Some(t) = self.lv.map.get_mut(xx, zz) {
                    if !t.is_solid() {
                        t.floor = level;
                        t.floor_tex = if l == Liquid::Water { Tex::Water } else { Tex::Lava };
                        t.liquid = l;
                        if l == Liquid::Lava {
                            t.light = 22;
                        }
                        if !t.is_diag() {
                            t.shape = Shape::Open;
                        }
                    }
                }
            }
        }
    }

    /// A wooden bridge across water or lava.
    pub fn bridge(&mut self, x: i32, z: i32, w: i32, h: i32, floor: i16) {
        for zz in z..z + h {
            for xx in x..x + w {
                if let Some(t) = self.lv.map.get_mut(xx, zz) {
                    if !t.is_solid() {
                        t.floor = floor;
                        t.floor_tex = Tex::Bridge;
                        t.liquid = Liquid::None;
                        t.shape = Shape::Open;
                    }
                }
            }
        }
    }

    pub fn slope(&mut self, x: i32, z: i32, shape: Shape, floor: i16, rise: i16) {
        if let Some(t) = self.lv.map.get_mut(x, z) {
            t.shape = shape;
            t.floor = floor;
            t.slope = rise;
        }
    }

    /// A gap to jump: drop the floor of open tiles `depth` steps, keeping the
    /// ceiling. A running jump clears one tile (see `JUMP_UP`),
    /// or two onto a ledge `LEAP_DOWN` lower. Every gap needs a way out of the
    /// bottom back to the side it is jumped from, so a missed jump costs a
    /// fall, not the game.
    pub fn gap(&mut self, x: i32, z: i32, w: i32, h: i32, depth: i16) {
        for zz in z..z + h {
            for xx in x..x + w {
                if let Some(t) = self.lv.map.get_mut(xx, zz) {
                    if !t.is_solid() {
                        t.floor -= depth;
                        t.shape = Shape::Open;
                        t.slope = 0;
                    }
                }
            }
        }
    }

    /// A ramp of `n` slope tiles from (x, z) in direction (dx, dz), rising
    /// from floor `lo` to `hi` with a ceiling `height` steps above `hi`.
    pub fn ramp(&mut self, x: i32, z: i32, (dx, dz): (i32, i32), n: i32, lo: i16, hi: i16, height: i16) {
        let shape = match (dx, dz) {
            (0, -1) => Shape::SlopeN,
            (0, 1) => Shape::SlopeS,
            (1, 0) => Shape::SlopeE,
            _ => Shape::SlopeW,
        };
        let rise = (hi - lo) / n as i16;
        for i in 0..n {
            let (tx, tz) = (x + dx * i, z + dz * i);
            let fl = lo + rise * i as i16;
            self.tile(tx, tz, fl, hi - fl + height);
            self.slope(tx, tz, shape, fl, rise);
        }
    }

    pub fn light(&mut self, x: i32, z: i32, w: i32, h: i32, l: u8) {
        for zz in z..z + h {
            for xx in x..x + w {
                if let Some(t) = self.lv.map.get_mut(xx, zz) {
                    t.light = t.light.max(l);
                }
            }
        }
    }

    pub fn wall(&mut self, x: i32, z: i32, tex: Tex) {
        if let Some(t) = self.lv.map.get_mut(x, z) {
            t.wall_tex = tex;
        }
    }

    /// Nearest open, dry spot to (x, z): objects placed in rock are moved out.
    fn spot(&self, x: f32, z: f32) -> (f32, f32) {
        let ok = |px: f32, pz: f32| {
            let t = self.lv.map.get(px.floor() as i32, pz.floor() as i32);
            self.lv.map.point_open(px, pz) && t.liquid == Liquid::None
        };
        if ok(x, z) {
            return (x, z);
        }
        let (tx, tz) = (x.floor() as i32, z.floor() as i32);
        for r in 1..8 {
            for dz in -r..=r {
                for dx in -r..=r {
                    let t = self.lv.map.get(tx + dx, tz + dz);
                    if t.shape == Shape::Open && t.liquid == Liquid::None {
                        return ((tx + dx) as f32 + 0.5, (tz + dz) as f32 + 0.5);
                    }
                }
            }
        }
        (x, z)
    }

    pub fn item(&mut self, kind: ItemKind, x: f32, z: f32) -> u32 {
        let mut it = Item::new(kind);
        if kind == ItemKind::Torch {
            it.fuel = 600.0;
        }
        self.put(it, x, z)
    }

    pub fn put(&mut self, it: Item, x: f32, z: f32) -> u32 {
        let (x, z) = self.spot(x, z);
        self.lv.add_item(it, x, z)
    }

    pub fn rune_stone(&mut self, r: u8, x: f32, z: f32) {
        self.put(Item::new(ItemKind::RuneStone).data(r as u16), x, z);
    }

    pub fn coins(&mut self, n: u16, x: f32, z: f32) {
        self.put(Item::new(ItemKind::Coin).qty(n).data(n), x, z);
    }

    pub fn container(&mut self, kind: ItemKind, x: f32, z: f32, contents: Vec<Item>) {
        let it = Item::new(kind).with(contents);
        self.put(it, x, z);
    }

    pub fn npc(&mut self, race: Race, x: f32, z: f32) -> u32 {
        let (x, z) = self.spot(x, z);
        let mut n = Npc::new(0, race, x, z);
        n.yaw = self.rng.rangef(-3.1, 3.1);
        self.lv.add_npc(n)
    }

    /// An undead bound to its master's power (see `creatures::THRALL`).
    pub fn thrall(&mut self, race: Race, x: f32, z: f32) -> u32 {
        let id = self.npc(race, x, z);
        self.npc_mut(id).flags |= THRALL;
        id
    }

    pub fn named(&mut self, race: Race, look: u8, name: &str, conv: &str, x: f32, z: f32, yaw_deg: f32) -> u32 {
        let (x, z) = self.spot(x, z);
        let mut n = Npc::new(0, race, x, z);
        n.name = Some(name.to_string());
        n.look = look;
        n.talk = if conv.is_empty() { None } else { Some(self.db.find(conv).unwrap_or_else(|| panic!("no conversation {}", conv))) };
        n.yaw = yaw_deg.to_radians();
        n.ai = AiState::Idle;
        if !race.def().hostile {
            n.attitude = Attitude::Friendly;
        } else {
            n.attitude = Attitude::Mellow;
        }
        self.lv.add_npc(n)
    }

    pub fn npc_mut(&mut self, id: u32) -> &mut Npc {
        self.lv.npc_mut(id).unwrap()
    }

    pub fn mellow(&mut self, id: u32) {
        let n = self.npc_mut(id);
        n.attitude = Attitude::Mellow;
        n.ai = AiState::Idle;
    }

    pub fn door(&mut self, x: i32, z: i32, axis: DoorAxis, kind: DoorKind, key: u16) -> u32 {
        self.lv.add_door(x, z, axis, kind, key)
    }

    pub fn stairs(&mut self, x: i32, z: i32, level: usize, dx: f32, dz: f32, yaw_deg: f32) {
        self.lv.triggers.push(Trigger { tx: x, tz: z, kind: TriggerKind::Stairs { level, x: dx, z: dz, yaw: yaw_deg.to_radians() }, once: false, fired: false, found: false });
    }

    /// Name the area around tile (x, z) on the developer maps.
    pub fn area(&mut self, name: &str, x: i32, z: i32) {
        self.lv.areas.push((x, z, name.to_string()));
    }

    pub fn message(&mut self, x: i32, z: i32, text: &str) {
        self.lv.triggers.push(Trigger { tx: x, tz: z, kind: TriggerKind::Message { text: text.to_string() }, once: true, fired: false, found: false });
    }

    pub fn trap(&mut self, x: i32, z: i32, dmg: i32) {
        self.lv.triggers.push(Trigger { tx: x, tz: z, kind: TriggerKind::Damage { amount: dmg }, once: true, fired: false, found: false });
    }

    pub fn writing(&mut self, wall_x: i32, wall_z: i32, text: &str) {
        self.wall(wall_x, wall_z, Tex::Plaque);
        self.lv.writings.push((wall_x, wall_z, text.to_string()));
    }

    pub fn lever(&mut self, wall_x: i32, wall_z: i32, door: u32) {
        self.wall(wall_x, wall_z, Tex::WallLever);
        self.lv.switches.push(Switch { tx: wall_x, tz: wall_z, door, on: false, want: None });
    }

    /// A combination lock: levers on the walls at (x, z), all starting off,
    /// and the door opens only while each one is in its wanted position.
    /// Keep the levers two tiles apart so each is pulled on its own. The door
    /// cannot be opened by hand.
    pub fn combo(&mut self, levers: &[(i32, i32, bool)], door: u32) {
        assert!(levers.iter().any(|l| l.2), "a combination lock that starts open");
        for &(x, z, want) in levers {
            self.wall(x, z, Tex::WallLever);
            self.lv.switches.push(Switch { tx: x, tz: z, door, on: false, want: Some(want) });
        }
        self.lv.doors.iter_mut().find(|d| d.id == door).unwrap().remote = true;
    }

    /// A pressure plate on tile (x, z) that opens a door when stepped on.
    pub fn plate(&mut self, x: i32, z: i32, door: u32) {
        self.lv.triggers.push(Trigger { tx: x, tz: z, kind: TriggerKind::OpenDoor { door }, once: true, fired: false, found: false });
    }

    pub fn finish(self) -> Level {
        self.lv
    }
}

/// Height steps a running jump over one tile may land above its take-off
/// floor (pinned to the physics by `game::fuzz_tests::jump_reach`).
#[cfg(test)]
pub const JUMP_UP: i16 = 2;
/// How far below its take-off floor a running leap over two tiles must land.
#[cfg(test)]
pub const LEAP_DOWN: i16 = 8;
/// Headroom a jump needs above its take-off floor, in height steps.
#[cfg(test)]
pub const JUMP_HEADROOM: i16 = 14;

/// The labels for the developer maps of level `li`: its area names, and
/// where its stairs lead.
pub fn map_labels(lv: &Level, li: usize) -> Vec<(i32, i32, String)> {
    let mut labels = lv.areas.clone();
    for t in &lv.triggers {
        if let TriggerKind::Stairs { level, .. } = t.kind {
            let dir = if level > li { "down" } else { "up" };
            labels.push((t.tx, t.tz + 1, format!("{} to L{}", dir, level + 1)));
        }
    }
    labels
}

/// Build every level of the abyss.
pub fn build_all() -> Vec<Level> {
    let db = DialogueDb::load();
    vec![level1(&db), level2(&db), level3(&db), level4(&db), level5(&db)]
}

// ------------------------------------------------------------------ level 1

fn level1(db: &DialogueDb) -> Level {
    let mut b = B::new("the Upper Halls", 101, db);
    b.lv.ambient = 3.0;
    // Entrance hall, sealed doors to the north.
    b.style(Tex::Flagstones, Tex::RoughStone, Tex::CeilingRock);
    b.room(28, 52, 5, 6, 0, 10);
    b.area("Entrance hall", 30, 54);
    b.carve(30, 51, 0, 10);
    let gate = b.door(30, 51, DoorAxis::NS, DoorKind::Iron, 999);
    b.lv.doors.iter_mut().find(|d| d.id == gate).unwrap().remote = true;
    b.light(28, 52, 5, 6, 9);
    b.lv.start = (30.5, 56.5, 0.0);
    b.writing(28, 51, "We attacked the doors with every tool we had, but they gave not a hair. Hence we have resigned ourselves to live in this hellish pit. - Corwin");
    b.item(ItemKind::Torch, 29.4, 55.2);
    b.put(Item::new(ItemKind::Map).data(1), 31.6, 55.6);
    b.item(ItemKind::Bread, 29.2, 57.3);
    b.put(Item::new(ItemKind::Scroll).data(3), 31.7, 53.2);
    b.item(ItemKind::BonePile, 32.4, 57.4);
    b.item(ItemKind::Gravestone, 28.6, 57.4);
    b.message(29, 56, "Beside you gapes a shallow, empty grave. The earth inside still holds the shape of a body - your body.");

    // West corridor to the outcasts' camp.
    b.style(Tex::Dirt, Tex::StoneBlocks, Tex::CeilingRock);
    b.corridor(27, 55, 20, 46, 0, 8);
    b.room(14, 40, 9, 6, 0, 12);
    b.area("Outcasts' camp", 18, 44);
    b.light(14, 40, 9, 6, 10);
    b.item(ItemKind::Campfire, 18.5, 43.0);
    b.named(Race::Human, 1, "Elsmore", "elsmore", 16.5, 42.0, 90.0);
    let tobin = b.named(Race::Human, 0, "Tobin", "tobin", 21.0, 41.0, 200.0);
    b.npc_mut(tobin).inventory = [ItemKind::Torch, ItemKind::Torch, ItemKind::Bread, ItemKind::Bread, ItemKind::RedPotion, ItemKind::Lantern, ItemKind::Lockpick, ItemKind::Candle].into_iter().map(Item::new).collect();
    b.item(ItemKind::Barrel, 14.6, 40.6);
    b.item(ItemKind::Barrel, 15.5, 40.5);
    b.item(ItemKind::Bed, 21.4, 44.6);
    b.item(ItemKind::Bedroll, 15.0, 45.3);
    b.message(20, 47, "You smell wood smoke. Someone has made a camp nearby.");

    // Shrine to the south of the corridor.
    b.corridor(24, 56, 24, 58, 0, 8);
    b.style(Tex::MarbleTiles, Tex::Marble, Tex::CeilingRock);
    b.room(22, 59, 5, 4, 0, 10);
    b.area("Shrine", 24, 60);
    b.light(22, 59, 5, 4, 9);
    b.item(ItemKind::Shrine, 24.5, 61.4);
    b.writing(24, 63, "Rest here, weary soul. The Virtues watch over thee. Kneel, and thy skills may grow.");
    b.item(ItemKind::Candle, 22.6, 59.5);

    // Green goblin den to the west, behind a door.
    b.style(Tex::Dirt, Tex::IrregularStone, Tex::CeilingRock);
    b.corridor(13, 42, 8, 42, 0, 8);
    b.door(11, 42, DoorAxis::EW, DoorKind::Wood, 0);
    b.room(2, 38, 6, 8, 0, 9);
    b.area("Goblin den", 4, 41);
    b.npc(Race::GreenGoblin, 4.5, 40.5);
    b.npc(Race::GreenGoblin, 5.5, 44.0);
    let gnarl = b.named(Race::GreenGoblin, 0, "Gnarl", "", 3.5, 42.5, 90.0);
    {
        let n = b.npc_mut(gnarl);
        n.attitude = Attitude::Hostile;
        n.ai = AiState::Wander;
        n.hp = 24;
        n.max_hp = 24;
        n.inventory = vec![Item::new(ItemKind::Medallion), Item::new(ItemKind::IronKey).data(7)];
    }
    b.item(ItemKind::Cudgel, 3.0, 44.6);
    b.coins(12, 6.5, 39.0);
    b.item(ItemKind::LeatherVest, 2.6, 39.0);
    b.item(ItemKind::Bone, 6.0, 44.8);
    b.item(ItemKind::BonePile, 2.8, 41.0);
    b.put(Item::new(ItemKind::Scroll).data(2), 7.4, 45.4);

    // Locked library north of the camp.
    b.style(Tex::Dirt, Tex::StoneBlocks, Tex::CeilingRock);
    b.corridor(18, 39, 18, 35, 0, 8);
    b.door(18, 37, DoorAxis::NS, DoorKind::Wood, 7);
    b.trap(18, 36, 3);
    b.style(Tex::WoodFloor, Tex::DarkPanels, Tex::CeilingBeams);
    b.room(15, 28, 7, 6, 2, 9);
    b.area("Library", 18, 29);
    b.tile(18, 34, 0, 10);
    b.slope(18, 34, Shape::SlopeN, 0, 2);
    b.light(15, 28, 7, 6, 4);
    b.item(ItemKind::Longsword, 16.0, 29.0);
    b.item(ItemKind::RedPotion, 20.5, 29.0);
    b.container(
        ItemKind::Chest,
        18.5,
        28.7,
        vec![Item::new(ItemKind::Coin).qty(40).data(40), Item::new(ItemKind::RuneStone).data(rune::ORT as u16), Item::new(ItemKind::RuneStone).data(rune::JUX as u16), Item::new(ItemKind::Scroll).data(9)],
    );
    b.item(ItemKind::Bookshelf, 15.7, 31.0);
    b.put(Item::new(ItemKind::Book).data(4), 20.6, 32.5);
    b.item(ItemKind::Candle, 16.0, 33.0);
    // Secret room east of the library.
    b.tile(22, 31, 2, 9);
    b.door(22, 31, DoorAxis::EW, DoorKind::Secret, 0);
    b.room(23, 29, 4, 4, 2, 9);
    b.area("Secret room", 25, 32);
    b.put(Item::new(ItemKind::GoldRing).enchanted(Enchant::Protection(2), 0), 24.5, 30.0);
    b.item(ItemKind::BluePotion, 25.6, 31.8);
    b.rune_stone(rune::BET, 23.6, 32.0);

    // East: rat cave.
    b.style(Tex::Dirt, Tex::StoneBlocks, Tex::CeilingRock);
    b.corridor(33, 55, 40, 55, 0, 8);
    b.style(Tex::Gravel, Tex::CaveRock, Tex::CeilingRock);
    b.cave(44, 54, 4, 4, 0, 9);
    b.area("Rat cave", 44, 53);
    b.corridor(40, 55, 44, 55, 0, 9);
    b.npc(Race::Rat, 44.5, 53.5);
    b.npc(Race::Rat, 45.5, 55.5);
    b.npc(Race::Rat, 43.0, 56.0);
    b.item(ItemKind::Dagger, 46.5, 56.0);
    b.item(ItemKind::Apple, 43.0, 52.0);
    // Slug tunnel to the south-east.
    b.cave(57, 58, 3, 3, 0, 8);
    b.area("Slug cave", 57, 58);
    b.corridor(44, 57, 57, 57, 0, 8);
    b.trap(52, 57, 3);
    b.npc(Race::Slug, 57.5, 58.5);
    b.container(ItemKind::Box, 58.5, 59.5, vec![Item::new(ItemKind::RuneStone).data(rune::MANI as u16), Item::new(ItemKind::Candle)]);
    b.item(ItemKind::Mushroom, 56.0, 57.0);
    b.item(ItemKind::Mushroom, 58.0, 56.5);

    // Water cave to the north.
    b.cave(45, 35, 6, 5, 0, 12);
    b.area("Water cave", 46, 37);
    b.corridor(44, 54, 44, 38, 0, 8);
    b.water(43, 33, 5, 4, -2);
    b.npc(Race::Bat, 46.5, 34.5);
    b.npc(Race::Bat, 41.0, 37.0);
    b.rune_stone(rune::IN, 49.0, 37.0);
    b.rune_stone(rune::LOR, 41.5, 31.5);
    b.item(ItemKind::Fish, 47.5, 38.5);
    b.item(ItemKind::Plant, 40.5, 38.5);

    // Gray goblin hall to the north.
    b.style(Tex::Dirt, Tex::IrregularStone, Tex::CeilingRock);
    b.room(38, 14, 15, 9, 0, 12);
    b.area("Gray goblin hall", 45, 18);
    b.corridor(49, 35, 49, 22, 0, 9);
    b.light(38, 14, 15, 9, 9);
    b.named(Race::GrayGoblin, 0, "Rhugg", "rhugg", 45.5, 15.5, 180.0);
    for (x, z) in [(40.5, 17.5), (49.5, 17.5), (42.5, 20.5)] {
        let id = b.npc(Race::GrayGoblin, x, z);
        b.mellow(id);
    }
    b.item(ItemKind::Brazier, 39.0, 14.8);
    b.item(ItemKind::Brazier, 51.5, 14.8);
    b.item(ItemKind::Barrel, 52.0, 21.5);
    b.item(ItemKind::Table, 47.5, 15.0);
    b.item(ItemKind::Meat, 47.2, 15.0);

    // Stairs down, behind the gray goblins' portcullis.
    b.style(Tex::Flagstones, Tex::RoughStone, Tex::CeilingRock);
    b.corridor(53, 18, 58, 18, 0, 9);
    b.door(55, 18, DoorAxis::EW, DoorKind::Portcullis, 55);
    b.lv.doors.last_mut().unwrap().remote = true;
    b.tile(57, 18, -2, 11);
    b.slope(57, 18, Shape::SlopeW, -2, 2);
    b.tile(58, 18, -4, 13);
    b.slope(58, 18, Shape::SlopeW, -4, 2);
    b.tile(59, 18, -4, 12);
    b.tile(60, 18, -4, 12);
    b.message(56, 18, "Steps lead down into the depths. You hear the distant ring of hammers.");
    b.stairs(60, 18, 1, 6.5, 30.5, 90.0);

    // The old guard hall north of the library, split by a crack: the
    // first jump, with a way back up from the bottom on the west.
    b.style(Tex::Flagstones, Tex::StoneBlocks, Tex::CeilingRock);
    b.corridor(18, 27, 18, 20, 2, 9);
    b.room(11, 10, 16, 10, 2, 12);
    b.area("Old guard hall", 18, 11);
    b.light(11, 10, 16, 10, 5);
    b.style(Tex::Dirt, Tex::RoughStone, Tex::CeilingRock);
    b.gap(11, 14, 16, 1, 12);
    b.area("crack", 18, 15);
    b.ramp(10, 14, (-1, 0), 3, -10, 2, 9);
    b.corridor(7, 14, 7, 17, 2, 9);
    b.corridor(7, 17, 10, 17, 2, 9);
    b.message(18, 19, "A crack splits the hall from wall to wall. With a running start you could leap it.");
    b.writing(27, 16, "The floor gave way when the mountain shook. Who will not leap may climb out by the west.");
    b.item(ItemKind::Bedroll, 12.0, 18.6);
    b.item(ItemKind::Barrel, 25.6, 18.6);
    b.item(ItemKind::Torch, 24.5, 17.0);
    b.item(ItemKind::Rock, 15.0, 14.5);
    b.npc(Race::Rat, 14.5, 11.5);
    b.npc(Race::Rat, 22.5, 12.0);
    b.item(ItemKind::BonePile, 12.0, 10.8);
    b.item(ItemKind::Table, 19.0, 11.0);
    b.item(ItemKind::Candle, 19.0, 11.0);
    b.container(ItemKind::Chest, 25.5, 10.8, vec![Item::new(ItemKind::Buckler), Item::new(ItemKind::Coin).qty(18).data(18), Item::new(ItemKind::Apple)]);

    // The goblins' midden below the den, and past a crevice their hoard.
    b.style(Tex::Mud, Tex::IrregularStone, Tex::CeilingRock);
    b.cave(8, 54, 4, 3, 0, 9);
    b.area("Midden", 8, 54);
    b.corridor(4, 46, 4, 52, 0, 8);
    b.corridor(4, 52, 8, 52, 0, 8);
    b.npc(Race::Rat, 6.5, 53.5);
    b.npc(Race::Rat, 10.5, 55.0);
    b.npc(Race::Slug, 8.5, 56.0);
    b.item(ItemKind::BonePile, 5.5, 55.5);
    b.item(ItemKind::Bone, 9.5, 52.5);
    b.item(ItemKind::Mushroom, 11.0, 53.0);
    b.corridor(8, 57, 8, 60, 0, 12);
    b.gap(8, 59, 1, 1, 12);
    b.ramp(9, 59, (1, 0), 3, -12, 0, 9);
    b.corridor(12, 59, 12, 56, 0, 9);
    b.corridor(12, 56, 10, 56, 0, 9);
    b.room(5, 61, 7, 2, 0, 9);
    b.area("Hoard nook", 8, 61);
    b.container(ItemKind::Box, 10.5, 61.6, vec![Item::new(ItemKind::Coin).qty(15).data(15), Item::new(ItemKind::GreenPotion)]);
    b.item(ItemKind::Sling, 6.0, 61.5);
    b.item(ItemKind::SlingStone, 6.6, 62.2);
    b.item(ItemKind::Meat, 8.5, 62.3);

    // A bat roost east of the water cave, where an explorer met their end.
    b.style(Tex::Gravel, Tex::CaveRock, Tex::CeilingRock);
    b.cave(58, 43, 4, 5, 0, 13);
    b.area("Bat roost", 58, 43);
    b.corridor(49, 37, 56, 42, 0, 8);
    b.npc(Race::Bat, 57.5, 41.5);
    b.npc(Race::Bat, 59.5, 45.5);
    b.npc(Race::Bat, 56.5, 46.0);
    // Wenna, Tobin's partner (his side quest).
    b.container(ItemKind::Corpse, 59.5, 43.0, vec![Item::new(ItemKind::LeatherCap), Item::new(ItemKind::Rope), Item::new(ItemKind::Coin).qty(8).data(8), Item::new(ItemKind::EngravedRing)]);
    b.item(ItemKind::WaterFlask, 57.0, 44.5);
    b.item(ItemKind::Stalagmite, 60.5, 41.0);

    // Side quests. Corwin, the old guard captain, drinks in the south half
    // of the guard hall; the green goblins hid his sword in their hoard.
    b.named(Race::Human, 2, "Corwin", "corwin", 13.5, 18.0, 90.0);
    b.item(ItemKind::Barrel, 12.0, 16.0);
    b.item(ItemKind::CaptainsSword, 7.0, 61.4);

    // His armoury, off the north half of the hall: a lock of three levers
    // on the north wall, west and east pulled, the middle left alone.
    b.style(Tex::Flagstones, Tex::StoneBlocks, Tex::CeilingRock);
    b.tile(27, 11, 2, 9);
    let armoury = b.door(27, 11, DoorAxis::EW, DoorKind::Iron, 0);
    b.room(28, 10, 4, 3, 2, 9);
    b.area("Armoury", 29, 11);
    b.light(28, 10, 4, 3, 4);
    b.combo(&[(14, 9, true), (18, 9, false), (22, 9, true)], armoury);
    b.writing(27, 12, "ARMOURY. The lock opens to the watch as it ought to stand: the west post and the east post at their levers, the centre post asleep. - Capt. Corwin");
    b.item(ItemKind::ChainCowl, 29.0, 10.6);
    b.item(ItemKind::WoodenShield, 30.6, 10.6);
    b.coins(25, 30.5, 12.4);

    // Maera's study, behind a second secret door in the secret room: the
    // first page of her chronicle.
    b.style(Tex::WoodFloor, Tex::DarkPanels, Tex::CeilingBeams);
    b.tile(27, 31, 2, 9);
    b.door(27, 31, DoorAxis::EW, DoorKind::Secret, 0);
    b.room(28, 29, 3, 4, 2, 9);
    b.area("Maera's study", 29, 30);
    b.light(28, 29, 3, 4, 3);
    b.writing(26, 28, "Here I write, where no one comes to ask what I am writing. - M.");
    b.item(ItemKind::Table, 29.5, 30.0);
    b.put(Item::new(ItemKind::ChroniclePage).data(0), 29.5, 30.0);
    b.item(ItemKind::Bookshelf, 30.4, 32.4);
    b.item(ItemKind::Candle, 28.6, 29.5);
    b.finish()
}

// ------------------------------------------------------------------ level 2

fn level2(db: &DialogueDb) -> Level {
    let mut b = B::new("the Mines of the Mountain Folk", 202, db);
    b.lv.ambient = 2.5;
    // Arrival tunnel (stairs up at the west end).
    b.style(Tex::Gravel, Tex::CaveRock, Tex::CeilingRock);
    b.corridor(5, 30, 19, 30, 0, 8);
    b.area("Arrival tunnel", 11, 31);
    b.tile(2, 30, 4, 8);
    b.tile(3, 30, 2, 10);
    b.slope(3, 30, Shape::SlopeW, 2, 2);
    b.tile(4, 30, 0, 10);
    b.slope(4, 30, Shape::SlopeW, 0, 2);
    b.stairs(2, 30, 0, 56.5, 18.5, 270.0);
    b.lv.start = (6.5, 30.5, 90.0);
    b.message(9, 30, "The tunnel widens. Torchlight flickers ahead.");

    // King Thorgrim's hall.
    b.style(Tex::Flagstones, Tex::GreyBlocks, Tex::CeilingRock);
    b.room(20, 24, 15, 13, 0, 14);
    b.area("Thorgrim's hall", 27, 31);
    b.light(20, 24, 15, 13, 11);
    b.room(28, 25, 5, 3, 2, 12);
    b.slope(30, 28, Shape::SlopeN, 0, 2);
    b.floor_tex(28, 25, 5, 3, Tex::MarbleTiles);
    for (x, z) in [(23.0, 27.5), (23.0, 33.0), (31.5, 33.0), (26.0, 30.5)] {
        b.item(ItemKind::Pillar, x, z);
    }
    for (x, z) in [(20.8, 24.8), (34.2, 24.8), (20.8, 36.2), (34.2, 36.2)] {
        b.item(ItemKind::Brazier, x, z);
    }
    b.named(Race::Dwarf, 0, "Thorgrim", "thorgrim", 30.5, 25.6, 180.0);
    for (x, z) in [(27.5, 28.5), (33.5, 28.5)] {
        let id = b.npc(Race::Dwarf, x, z);
        b.mellow(id);
        b.npc_mut(id).look = 2;
        b.npc_mut(id).attitude = Attitude::Friendly;
    }
    b.item(ItemKind::Statue, 25.0, 25.0);
    b.writing(27, 23, "Hall of the Mountain King. Strangers, show respect, or show the door your back.");

    // Living quarters to the west.
    b.style(Tex::WoodFloor, Tex::WoodPlanks, Tex::CeilingBeams);
    b.corridor(19, 26, 17, 26, 0, 8);
    b.room(8, 20, 9, 8, 0, 9);
    b.area("Quarters", 12, 24);
    b.light(8, 20, 9, 8, 7);
    b.named(Race::Dwarf, 1, "Hilda", "hilda", 11.5, 23.5, 90.0);
    b.item(ItemKind::Bed, 9.6, 20.7);
    b.item(ItemKind::Bed, 12.8, 20.7);
    b.item(ItemKind::Barrel, 16.2, 27.2);
    b.item(ItemKind::Table, 14.0, 24.5);
    b.item(ItemKind::Cheese, 14.0, 24.4);
    b.item(ItemKind::WaterFlask, 9.0, 27.0);
    b.item(ItemKind::Bread, 13.5, 26.8);

    // The forge, to the south.
    b.style(Tex::Flagstones, Tex::Bricks, Tex::CeilingRock);
    b.corridor(27, 37, 27, 41, 0, 9);
    b.room(22, 42, 11, 7, 0, 11);
    b.area("Forge", 27, 46);
    b.lava(24, 47, 7, 1, -1);
    b.light(22, 42, 11, 7, 10);
    let bodric = b.named(Race::Dwarf, 3, "Bodric", "bodric", 26.5, 44.5, 0.0);
    b.npc_mut(bodric).inventory = [ItemKind::ChainShirt, ItemKind::Gauntlets, ItemKind::Broadsword, ItemKind::WoodenShield, ItemKind::Torch, ItemKind::Torch].into_iter().map(Item::new).collect();
    b.item(ItemKind::Cauldron, 31.3, 43.0);
    b.item(ItemKind::Table, 29.0, 43.3);
    b.rune_stone(rune::FLAM, 29.0, 43.2);
    b.item(ItemKind::Mace, 23.0, 43.0);
    b.item(ItemKind::ChainCowl, 22.8, 45.5);

    // The spider-infested eastern galleries, sealed by the king.
    b.style(Tex::Gravel, Tex::CaveRock, Tex::CeilingRock);
    b.tile(35, 30, 0, 9);
    b.door(35, 30, DoorAxis::EW, DoorKind::Iron, 21);
    b.cave(46, 28, 5, 5, 0, 10);
    b.area("Spider galleries", 50, 37);
    b.cave(50, 44, 6, 5, 0, 11);
    b.cave(57, 32, 5, 4, 0, 10);
    b.corridor(36, 30, 46, 30, 0, 9);
    b.corridor(46, 30, 46, 44, 0, 9);
    b.corridor(46, 44, 50, 44, 0, 9);
    b.corridor(50, 44, 58, 32, 0, 9);
    b.trap(40, 30, 5);
    b.trap(46, 38, 5);
    for (x, z) in [(45.5, 27.5), (48.5, 29.5), (49.5, 44.5), (52.5, 46.5)] {
        b.npc(Race::Spider, x, z);
    }
    let queen = b.npc(Race::Spider, 57.5, 31.5);
    {
        let q = b.npc_mut(queen);
        q.name = Some("the spider queen".into());
        q.hp = 34;
        q.max_hp = 34;
    }
    for (x, z) in [(44.0, 26.0), (51.0, 42.5), (59.0, 33.0), (47.0, 36.0)] {
        b.item(ItemKind::Web, x, z);
    }
    b.item(ItemKind::BonePile, 48.0, 46.0);
    b.item(ItemKind::Skull, 50.5, 47.5);
    b.item(ItemKind::Hammer, 58.5, 31.0);
    b.rune_stone(rune::HUR, 53.5, 47.0);
    b.rune_stone(rune::SANCT, 44.5, 30.5);
    b.item(ItemKind::GoldNugget, 56.0, 34.0);
    b.item(ItemKind::GoldNugget, 41.0, 30.4);
    b.container(ItemKind::Corpse, 54.0, 31.0, vec![Item::new(ItemKind::ChainShirt), Item::new(ItemKind::Lantern), Item::new(ItemKind::GreenPotion)]);

    // The deep mine and the way down, south-west of the hall.
    b.cave(16, 54, 6, 4, 0, 9);
    b.area("Deep mine", 16, 55);
    b.corridor(21, 36, 21, 54, 0, 8);
    b.trap(21, 45, 5);
    // The tunnel floor fell into an old shaft: jump it, or climb out of the
    // shaft by the ramp to the forge.
    for z in 49..=51 {
        b.tile(21, z, 0, 12);
    }
    b.gap(21, 50, 1, 1, 12);
    b.area("shaft", 21, 50);
    b.ramp(22, 50, (1, 0), 3, -12, 0, 9);
    b.corridor(25, 50, 25, 49, 0, 9);
    b.message(21, 49, "The tunnel floor has fallen into an old shaft. A good run and a leap would carry you over.");
    b.corridor(21, 54, 16, 54, 0, 8);
    b.corridor(13, 54, 6, 54, 0, 8);
    b.lava(14, 50, 2, 9, -3);
    b.bridge(14, 54, 2, 1, 0);
    b.npc(Race::Slug, 18.5, 55.5);
    b.npc(Race::Slug, 11.5, 52.5);
    b.item(ItemKind::GoldNugget, 19.0, 57.0);
    b.item(ItemKind::Shovel, 18.0, 52.0);
    b.rune_stone(rune::POR, 18.6, 53.4);
    b.writing(16, 49, "DANGER - the old fissure runs hot. Use the bridge.");
    b.tile(5, 54, -2, 10);
    b.slope(5, 54, Shape::SlopeE, -2, 2);
    b.tile(4, 54, -4, 12);
    b.slope(4, 54, Shape::SlopeE, -4, 2);
    b.tile(3, 54, -4, 12);
    b.tile(2, 54, -4, 12);
    b.stairs(2, 54, 2, 8.5, 6.5, 180.0);

    // The old diggings north of the quarters. The east face fell into a
    // shaft; the ledge beyond is reached only by a leap, and from the
    // bottom of the shaft a ramp climbs back to the diggings.
    b.style(Tex::Gravel, Tex::CaveRock, Tex::CeilingRock);
    b.cave(10, 10, 5, 4, 0, 10);
    b.area("Old diggings", 11, 10);
    b.corridor(12, 19, 12, 12, 0, 8);
    b.room(15, 8, 4, 5, 0, 12);
    b.corridor(10, 10, 15, 10, 0, 9);
    b.room(19, 8, 2, 5, 0, 12);
    b.gap(19, 8, 2, 5, 16);
    b.room(21, 8, 3, 5, -8, 20);
    b.area("ledge", 22, 13);
    b.ramp(19, 13, (0, 1), 4, -16, 0, 9);
    b.corridor(19, 17, 12, 17, 0, 8);
    b.writing(17, 7, "DIGGING CLOSED. The east face fell into the old shaft. Gold glints on the ledge beyond, but no fool is to jump for it. - Thorgrim");
    b.npc(Race::Slug, 8.5, 9.5);
    b.npc(Race::Slug, 12.5, 12.0);
    b.item(ItemKind::Rock, 9.0, 12.5);
    b.item(ItemKind::Rock, 16.5, 11.5);
    b.item(ItemKind::BonePile, 7.0, 8.0);
    b.item(ItemKind::Torch, 16.0, 8.6);
    b.container(ItemKind::Box, 23.4, 9.6, vec![Item::new(ItemKind::GoldNugget), Item::new(ItemKind::GoldNugget), Item::new(ItemKind::RedGem)]);
    b.item(ItemKind::Lantern, 22.0, 12.0);
    // The storeroom off the arrival tunnel, overrun by rats.
    b.style(Tex::WoodFloor, Tex::WoodPlanks, Tex::CeilingBeams);
    b.corridor(10, 31, 10, 35, 0, 8);
    b.door(10, 33, DoorAxis::NS, DoorKind::Wood, 0);
    b.room(5, 36, 9, 6, 0, 9);
    b.area("Storeroom", 9, 38);
    b.light(5, 36, 9, 6, 3);
    for (x, z) in [(5.6, 36.6), (6.6, 36.5), (12.4, 41.4)] {
        b.item(ItemKind::Barrel, x, z);
    }
    for (x, z) in [(7.5, 39.0), (11.0, 37.5), (9.5, 40.5)] {
        b.npc(Race::Rat, x, z);
    }
    b.item(ItemKind::Bread, 8.0, 36.6);
    b.item(ItemKind::Cheese, 12.6, 36.8);
    b.item(ItemKind::Wine, 5.6, 40.0);
    b.item(ItemKind::WaterFlask, 6.0, 41.3);
    b.item(ItemKind::Candle, 10.5, 41.2);

    // A flooded drift south of the spider galleries.
    b.style(Tex::Gravel, Tex::CaveRock, Tex::CeilingRock);
    b.cave(50, 57, 6, 4, 0, 10);
    b.area("Flooded drift", 50, 57);
    b.corridor(50, 47, 50, 54, 0, 9);
    b.water(47, 58, 4, 3, -2);
    b.npc(Race::Spider, 46.5, 56.0);
    b.npc(Race::Spider, 54.5, 58.5);
    b.item(ItemKind::Web, 52.0, 55.0);
    b.item(ItemKind::Web, 45.5, 58.0);
    b.item(ItemKind::GoldNugget, 55.0, 56.5);
    b.item(ItemKind::BonePile, 52.5, 60.0);
    b.container(ItemKind::Corpse, 53.5, 59.5, vec![Item::new(ItemKind::GreenPotion), Item::new(ItemKind::Lockpick), Item::new(ItemKind::Candle)]);
    b.finish()
}

// ------------------------------------------------------------------ level 3

fn level3(db: &DialogueDb) -> Level {
    let mut b = B::new("the Flooded Warrens", 303, db);
    b.lv.ambient = 2.2;
    b.style(Tex::Mud, Tex::MossyStone, Tex::CeilingRock);
    // Stairs up in the north-west.
    b.tile(8, 2, 4, 8);
    b.tile(8, 3, 2, 10);
    b.slope(8, 3, Shape::SlopeN, 2, 2);
    b.tile(8, 4, 0, 10);
    b.slope(8, 4, Shape::SlopeN, 0, 2);
    b.stairs(8, 2, 1, 7.5, 54.5, 90.0);
    b.corridor(8, 5, 8, 12, 0, 8);
    b.lv.start = (8.5, 6.5, 180.0);
    b.message(8, 9, "The air is damp and smells of rot. Water drips from above.");

    // The great flooded hall with channels and bridges.
    b.room(4, 13, 26, 14, 0, 12);
    b.area("Great flooded hall", 17, 19);
    b.water(4, 18, 26, 3, -3);
    b.bridge(11, 18, 2, 3, 0);
    b.bridge(24, 18, 2, 3, 0);
    b.npc(Race::Lurker, 18.5, 19.5);
    b.item(ItemKind::Plant, 5.0, 14.0);
    b.item(ItemKind::Plant, 28.5, 25.5);
    b.item(ItemKind::Stalagmite, 16.0, 24.0);

    // Lizardman village: huts east of the hall.
    b.style(Tex::Grass, Tex::Cobbles, Tex::CeilingRock);
    b.corridor(30, 15, 33, 15, 0, 9);
    b.room(34, 10, 14, 12, 0, 11);
    b.area("Lizardman village", 41, 19);
    b.light(34, 10, 14, 12, 8);
    b.water(39, 14, 4, 4, -2);
    b.named(Race::Lizardman, 0, "Hissk", "hissk", 35.5, 15.5, 270.0);
    for (x, z) in [(45.5, 12.5), (37.0, 20.0), (46.0, 19.5)] {
        let id = b.npc(Race::Lizardman, x, z);
        b.mellow(id);
        b.npc_mut(id).look = 1;
    }
    b.item(ItemKind::Campfire, 44.0, 16.0);
    b.item(ItemKind::Fish, 43.0, 17.5);
    b.item(ItemKind::Fish, 36.0, 11.0);
    // The Sage's pool of thinking, north of the village.
    b.style(Tex::MarbleTiles, Tex::Sandstone, Tex::CeilingRock);
    b.corridor(41, 9, 41, 6, 0, 9);
    b.room(37, 1, 9, 5, 0, 12);
    b.area("Sage's pool", 41, 2);
    b.water(39, 2, 5, 2, -1);
    b.light(37, 1, 9, 5, 10);
    b.named(Race::Lizardman, 2, "Ssrath", "ssrath", 41.5, 4.5, 180.0);
    b.item(ItemKind::MoonOrb, 37.6, 1.6);

    // The lurkers' grotto (deep water) to the east: the stolen eggs.
    b.style(Tex::Mud, Tex::CaveRock, Tex::CeilingRock);
    b.corridor(48, 16, 52, 16, 0, 9);
    b.cave(57, 18, 5, 6, 0, 12);
    b.area("Lurkers' grotto", 57, 18);
    b.water(53, 13, 6, 11, -3);
    b.npc(Race::Lurker, 56.5, 16.5);
    b.npc(Race::Lurker, 58.5, 21.5);
    b.item(ItemKind::LizardEgg, 60.5, 20.5);
    b.rune_stone(rune::WIS, 61.0, 13.0);
    b.item(ItemKind::BonePile, 60.0, 22.0);

    // The drowned temple to the south-west (ghouls).
    b.style(Tex::Flagstones, Tex::Sandstone, Tex::CeilingRock);
    b.corridor(10, 27, 10, 33, 0, 8);
    b.trap(10, 30, 5);
    b.room(4, 34, 14, 12, 0, 12);
    b.area("Drowned temple", 11, 39);
    b.water(4, 40, 14, 2, -1);
    b.bridge(10, 40, 2, 2, 0);
    for (x, z) in [(6.0, 36.5), (15.5, 36.5), (6.0, 44.0), (15.5, 44.0)] {
        b.item(ItemKind::Pillar, x, z);
    }
    b.npc(Race::Ghoul, 8.5, 37.5);
    b.npc(Race::Ghoul, 13.5, 43.5);
    b.item(ItemKind::Shrine, 10.5, 45.3);
    b.rune_stone(rune::NOX, 5.0, 45.0);
    b.rune_stone(rune::AN, 16.5, 34.8);
    b.item(ItemKind::YellowPotion, 15.8, 45.2);
    b.put(Item::new(ItemKind::Scroll).data(6), 7.0, 34.6);
    b.writing(10, 33, "Temple of the Deep Waters. The faithful drowned here rather than flee. They do not rest.");
    // A hidden reliquary east of the temple, opened by a lever on the west wall.
    b.tile(18, 37, 0, 10);
    let grate = b.door(18, 37, DoorAxis::EW, DoorKind::Portcullis, 0);
    b.lv.doors.iter_mut().find(|d| d.id == grate).unwrap().remote = true;
    b.room(19, 36, 2, 3, 0, 10);
    b.area("Reliquary", 21, 35);
    b.lever(3, 38, grate);
    b.put(Item::new(ItemKind::SilverRing).enchanted(Enchant::Protection(3), 0), 20.0, 37.5);
    b.item(ItemKind::BluePotion, 19.6, 36.4);
    b.rune_stone(rune::QUAS, 20.6, 36.7);

    // Southern caves and the old aqueduct to the stairs.
    b.style(Tex::Gravel, Tex::MossyStone, Tex::CeilingRock);
    b.corridor(18, 40, 30, 40, 0, 8);
    b.cave(34, 42, 5, 5, 0, 10);
    b.area("Southern caves", 34, 42);
    b.npc(Race::Bat, 34.5, 41.5);
    b.npc(Race::Ghoul, 36.5, 45.5);
    b.rune_stone(rune::GRAV, 37.0, 39.0);
    b.rune_stone(rune::REL, 31.0, 45.0);
    b.corridor(36, 45, 50, 55, 0, 8);
    b.trap(44, 45, 5);
    // A span of the old aqueduct has fallen: jump the gap, or swim the
    // water below to the ramp that climbs back.
    for z in 49..=51 {
        b.tile(50, z, 0, 12);
    }
    b.gap(50, 50, 1, 1, 12);
    b.area("Broken aqueduct", 51, 48);
    b.water(50, 50, 1, 1, -12);
    b.ramp(51, 50, (1, 0), 3, -12, 0, 9);
    b.corridor(51, 45, 54, 50, 0, 8);
    b.message(50, 48, "The aqueduct has broken away ahead. Far below, black water rushes through the gap.");
    b.cave(53, 57, 4, 3, 0, 10);
    b.corridor(50, 57, 53, 57, 0, 8);
    b.rune_stone(rune::TYM, 55.0, 56.0);
    b.npc(Race::Lizardman, 52.5, 58.5);
    b.item(ItemKind::RedPotion, 51.0, 59.0);
    // Stairs down at the far south.
    b.corridor(54, 60, 54, 60, 0, 8);
    b.tile(55, 60, -2, 10);
    b.slope(55, 60, Shape::SlopeW, -2, 2);
    b.tile(56, 60, -4, 12);
    b.slope(56, 60, Shape::SlopeW, -4, 2);
    b.tile(57, 60, -4, 12);
    b.tile(58, 60, -4, 12);
    b.stairs(58, 60, 3, 32.5, 6.5, 180.0);

    // A fungus cavern below the village, with a pool and a lurker in it.
    b.style(Tex::Mud, Tex::CaveRock, Tex::CeilingRock);
    b.cave(47, 31, 6, 5, 0, 11);
    b.area("Fungus cavern", 47, 31);
    b.corridor(44, 22, 44, 29, 0, 9);
    b.water(45, 32, 4, 3, -2);
    b.npc(Race::Lurker, 46.5, 33.5);
    b.npc(Race::Bat, 50.5, 29.5);
    b.npc(Race::Bat, 43.5, 33.0);
    b.item(ItemKind::Mushroom, 51.0, 33.0);
    b.item(ItemKind::Mushroom, 42.5, 30.0);
    b.item(ItemKind::Plant, 49.5, 27.5);
    b.container(ItemKind::Corpse, 51.5, 31.0, vec![Item::new(ItemKind::LeatherLeggings), Item::new(ItemKind::Torch), Item::new(ItemKind::Coin).qty(12).data(12)]);

    // The ossuary below the temple.
    b.style(Tex::Flagstones, Tex::Sandstone, Tex::CeilingRock);
    b.corridor(13, 46, 13, 50, 0, 8);
    b.room(6, 51, 12, 6, 0, 10);
    b.area("Ossuary", 12, 53);
    b.light(6, 51, 12, 6, 2);
    b.npc(Race::Ghoul, 9.5, 54.5);
    b.npc(Race::Bat, 15.5, 53.0);
    for x in [7.0, 10.0, 13.0, 16.0] {
        b.item(ItemKind::Gravestone, x, 51.3);
    }
    b.item(ItemKind::BonePile, 7.0, 55.5);
    b.item(ItemKind::BonePile, 16.5, 56.0);
    b.item(ItemKind::Skull, 12.0, 55.0);
    b.item(ItemKind::Candle, 6.6, 52.4);
    b.container(ItemKind::Chest, 16.5, 54.5, vec![Item::new(ItemKind::BluePotion), Item::new(ItemKind::Coin).qty(25).data(25)]);
    b.finish()
}

// ------------------------------------------------------------------ level 4

fn level4(db: &DialogueDb) -> Level {
    let mut b = B::new("the Crypts", 404, db);
    b.lv.ambient = 1.6;
    b.style(Tex::Flagstones, Tex::GreyBlocks, Tex::CeilingRock);
    // Stairs up in the north.
    b.tile(32, 2, 4, 8);
    b.tile(32, 3, 2, 10);
    b.slope(32, 3, Shape::SlopeN, 2, 2);
    b.tile(32, 4, 0, 10);
    b.slope(32, 4, Shape::SlopeN, 0, 2);
    b.stairs(32, 2, 2, 56.5, 60.5, 270.0);
    b.corridor(32, 5, 32, 9, 0, 9);
    b.lv.start = (32.5, 6.5, 180.0);
    b.message(32, 7, "A chill runs down your spine. This is a place of the dead.");

    // The long gallery of tombs.
    b.room(24, 10, 17, 5, 0, 12);
    b.area("Gallery of tombs", 32, 12);
    for k in 0..6 {
        let x = 25 + k * 3;
        b.tile(x, 9, 0, 6);
        b.tile(x, 15, 0, 6);
        b.item(ItemKind::Gravestone, x as f32 + 0.5, 9.3);
        b.item(ItemKind::Gravestone, x as f32 + 0.5, 15.7);
    }
    b.npc(Race::Skeleton, 28.5, 12.5);
    b.npc(Race::Skeleton, 37.5, 12.0);
    b.item(ItemKind::Bone, 30.0, 11.0);
    b.item(ItemKind::Skull, 34.2, 13.6);
    b.rune_stone(rune::CORP, 27.5, 15.6);

    // Sir Aldric's tomb chamber (centre).
    b.style(Tex::MarbleTiles, Tex::CarvedFrieze, Tex::CeilingRock);
    b.corridor(32, 15, 32, 21, 0, 9);
    b.trap(32, 18, 7);
    b.room(26, 22, 13, 9, 0, 13);
    b.area("Aldric's tomb", 32, 24);
    b.light(26, 22, 13, 9, 5);
    b.item(ItemKind::Statue, 27.0, 23.0);
    b.item(ItemKind::Statue, 38.0, 23.0);
    b.item(ItemKind::Chest, 32.5, 22.6);
    let aldric = b.named(Race::Ghost, 0, "Sir Aldric", "aldric", 32.5, 26.5, 180.0);
    b.npc_mut(aldric).attitude = Attitude::Friendly;
    b.writing(32, 31, "Here lies Sir Aldric, Knight of the Lumen, Keeper of Mercy. May he rest in peace.");

    // West wing: ghosts and headless.
    b.style(Tex::Flagstones, Tex::GreyBlocks, Tex::CeilingRock);
    b.corridor(25, 26, 14, 26, 0, 9);
    b.trap(19, 26, 6);
    b.room(4, 20, 10, 12, 0, 11);
    b.area("West wing", 9, 25);
    b.npc(Race::Ghost, 8.5, 24.5);
    b.npc(Race::Headless, 6.5, 29.5);
    b.npc(Race::Headless, 11.0, 30.0);
    b.item(ItemKind::Gravestone, 5.0, 21.0);
    b.item(ItemKind::Gravestone, 12.5, 21.0);
    b.rune_stone(rune::DES, 5.0, 31.0);
    b.rune_stone(rune::EX, 12.8, 21.5);
    b.container(ItemKind::Chest, 4.6, 26.0, vec![Item::new(ItemKind::PlateHelm), Item::new(ItemKind::Coin).qty(60).data(60), Item::new(ItemKind::BluePotion)]);
    b.item(ItemKind::SilverSapling, 9.0, 31.2);

    // East wing: the troll den, where Aldric's bones were dragged.
    b.style(Tex::Gravel, Tex::CaveRock, Tex::CeilingRock);
    b.corridor(39, 26, 46, 26, 0, 9);
    b.trap(43, 26, 6);
    b.cave(52, 26, 6, 6, 0, 12);
    b.area("Troll den", 52, 26);
    b.npc(Race::Troll, 52.5, 25.5);
    b.npc(Race::Troll, 55.5, 29.5);
    b.item(ItemKind::RelicBones, 57.0, 24.0);
    b.item(ItemKind::BonePile, 50.0, 30.0);
    b.item(ItemKind::Meat, 49.5, 22.0);
    b.rune_stone(rune::VAS, 56.0, 30.5);
    b.item(ItemKind::TowerShield, 48.5, 24.0);

    // South: the guardian's hall and the way down.
    b.style(Tex::Flagstones, Tex::GreyBlocks, Tex::CeilingRock);
    b.corridor(32, 31, 32, 40, 0, 9);
    b.trap(32, 35, 6);
    // The floor has fallen in ahead of the guardian's hall; the way back up
    // from the bottom comes out in Aldric's chamber.
    for z in 37..=39 {
        b.tile(32, z, 0, 12);
    }
    b.gap(32, 38, 1, 1, 12);
    b.area("Fallen passage", 32, 36);
    b.ramp(33, 38, (1, 0), 3, -12, 0, 9);
    b.corridor(36, 38, 36, 31, 0, 9);
    b.message(32, 37, "The floor of the passage has fallen into the crypts below. You will have to leap.");
    b.room(24, 41, 17, 10, 0, 14);
    b.area("Guardian's hall", 32, 45);
    b.light(24, 41, 17, 10, 4);
    let golem = b.npc(Race::Golem, 32.5, 46.5);
    b.npc_mut(golem).ai = AiState::Idle;
    for (x, z) in [(26.0, 43.0), (39.0, 43.0), (26.0, 49.0), (39.0, 49.0)] {
        b.item(ItemKind::Pillar, x, z);
    }
    b.rune_stone(rune::YLEM, 25.0, 50.0);
    b.rune_stone(rune::KAL, 40.0, 50.0);
    b.rune_stone(rune::UUS, 33.0, 41.5);
    b.item(ItemKind::PlateLeggings, 38.0, 41.6);
    b.corridor(32, 51, 32, 56, 0, 9);
    b.tile(32, 57, -2, 11);
    b.slope(32, 57, Shape::SlopeN, -2, 2);
    b.tile(32, 58, -4, 13);
    b.slope(32, 58, Shape::SlopeN, -4, 2);
    b.tile(32, 59, -4, 12);
    b.tile(32, 60, -4, 12);
    b.stairs(32, 60, 4, 32.5, 6.5, 180.0);

    // The catacombs west of the gallery.
    b.style(Tex::Flagstones, Tex::GreyBlocks, Tex::CeilingRock);
    b.corridor(23, 12, 15, 12, 0, 9);
    b.room(5, 5, 10, 8, 0, 10);
    b.area("Catacombs", 10, 8);
    b.light(5, 5, 10, 8, 2);
    for z in [5.4, 8.0, 10.6] {
        b.item(ItemKind::Gravestone, 5.4, z);
        b.item(ItemKind::Gravestone, 14.6, z + 1.0);
    }
    b.npc(Race::Skeleton, 8.5, 8.5);
    b.npc(Race::Skeleton, 11.5, 11.0);
    b.item(ItemKind::Skull, 10.0, 6.0);
    b.item(ItemKind::Bone, 7.0, 11.5);
    b.container(ItemKind::Chest, 9.5, 5.6, vec![Item::new(ItemKind::Coin).qty(30).data(30), Item::new(ItemKind::RedPotion)]);

    // The charnel pit east of the gallery: a leap down to the reliquary
    // ledge, and from the bottom a ramp back up.
    b.corridor(41, 12, 48, 12, 0, 9);
    b.room(49, 6, 4, 8, 0, 12);
    b.area("Charnel pit", 52, 5);
    b.room(53, 6, 2, 8, 0, 12);
    b.gap(53, 6, 2, 8, 16);
    b.room(55, 8, 4, 4, -8, 20);
    b.area("ledge", 57, 10);
    b.ramp(53, 14, (0, 1), 4, -16, 0, 9);
    b.corridor(49, 18, 53, 18, 0, 9);
    b.corridor(49, 14, 49, 18, 0, 9);
    b.light(49, 6, 10, 8, 3);
    b.npc(Race::Skeleton, 50.5, 8.5);
    b.npc(Race::Ghost, 51.0, 12.5);
    b.item(ItemKind::BonePile, 53.5, 10.0);
    b.item(ItemKind::BonePile, 54.5, 12.5);
    b.item(ItemKind::Skull, 50.0, 6.6);
    b.item(ItemKind::Candle, 57.0, 8.6);
    b.container(ItemKind::Chest, 57.5, 10.5, vec![Item::new(ItemKind::WhiteGem), Item::new(ItemKind::Coin).qty(40).data(40), Item::new(ItemKind::YellowPotion)]);
    b.writing(52, 5, "The unworthy dead were thrown into the pit. The worthy were laid on the far side, where none could reach to rob them.");

    // The mass graves south of the west wing.
    b.style(Tex::Gravel, Tex::CaveRock, Tex::CeilingRock);
    b.cave(10, 42, 6, 5, 0, 10);
    b.area("Mass graves", 10, 42);
    b.corridor(8, 32, 8, 40, 0, 9);
    b.npc(Race::Skeleton, 12.5, 41.5);
    b.npc(Race::Skeleton, 7.5, 45.0);
    b.npc(Race::Headless, 13.0, 45.5);
    for (x, z) in [(6.0, 40.0), (11.0, 46.0), (14.5, 40.5)] {
        b.item(ItemKind::BonePile, x, z);
    }
    b.item(ItemKind::Skull, 9.5, 43.5);
    b.container(ItemKind::Corpse, 10.5, 39.5, vec![Item::new(ItemKind::PlateBoots), Item::new(ItemKind::Coin).qty(20).data(20)]);

    // The embalmers' rooms east of the guardian's hall.
    b.style(Tex::MarbleTiles, Tex::CarvedFrieze, Tex::CeilingRock);
    b.corridor(41, 45, 46, 45, 0, 9);
    b.room(47, 42, 9, 7, 0, 10);
    b.area("Embalmers' rooms", 51, 45);
    b.light(47, 42, 9, 7, 4);
    b.npc(Race::Ghost, 52.5, 45.5);
    b.item(ItemKind::Table, 49.0, 43.0);
    b.item(ItemKind::Table, 53.0, 43.0);
    b.item(ItemKind::Cauldron, 54.8, 47.6);
    b.item(ItemKind::Bookshelf, 47.6, 47.6);
    b.item(ItemKind::Candle, 49.0, 43.0);
    b.item(ItemKind::Candle, 53.0, 43.0);
    b.item(ItemKind::Wine, 48.0, 45.5);
    b.item(ItemKind::BluePotion, 55.0, 42.6);
    b.finish()
}

// ------------------------------------------------------------------ level 5

/// Malchor, master of the thralls. He is not on the map at the start: the
/// gate's light burns him, so he hides until the player opens it with the
/// shards, then steps out behind them (`Game::malchor_appears`).
pub fn malchor(db: &DialogueDb, x: f32, z: f32) -> Npc {
    let mut n = Npc::new(0, Race::Mage, x, z);
    n.name = Some("Malchor".into());
    n.talk = db.find("malchor");
    n.attitude = Attitude::Hostile;
    n.ai = AiState::Idle;
    n.flags |= MASTER;
    n.hp = 70;
    n.max_hp = 70;
    n.inventory = vec![Item::new(ItemKind::SilverKey).data(501), Item::new(ItemKind::Wand).enchanted(Enchant::Spell(super::magic::Spell::Fireball), 4), Item::new(ItemKind::BluePotion)];
    n
}

fn level5(db: &DialogueDb) -> Level {
    let mut b = B::new("the Deep Sanctum", 505, db);
    b.lv.ambient = 2.0;
    b.style(Tex::Flagstones, Tex::RedBricks, Tex::CeilingRock);
    // Stairs up in the north.
    b.tile(32, 2, 4, 8);
    b.tile(32, 3, 2, 10);
    b.slope(32, 3, Shape::SlopeN, 2, 2);
    b.tile(32, 4, 0, 10);
    b.slope(32, 4, Shape::SlopeN, 0, 2);
    b.stairs(32, 2, 3, 32.5, 56.5, 0.0);
    b.corridor(32, 5, 32, 10, 0, 9);
    b.lv.start = (32.5, 6.5, 180.0);
    b.message(32, 8, "Heat rises from below. A cold voice drifts up out of the darkness: \"Come to me, my knight.\"");

    // Antechamber with fire and Malchor's thralls.
    b.room(24, 11, 17, 9, 0, 13);
    b.area("Antechamber", 32, 13);
    b.lava(24, 15, 4, 2, -2);
    b.lava(37, 15, 4, 2, -2);
    b.thrall(Race::Skeleton, 28.5, 13.5);
    b.thrall(Race::Skeleton, 36.5, 17.5);
    b.thrall(Race::Ghoul, 32.5, 18.5);
    b.message(32, 11, "Hollow soldiers turn toward you, a violet light burning in their empty eyes. Some will other than their own holds them here.");
    b.item(ItemKind::Brazier, 25.0, 11.8);
    b.item(ItemKind::Brazier, 40.0, 11.8);
    b.item(ItemKind::RedPotion, 39.5, 19.0);

    // West: Malchor's study.
    b.style(Tex::WoodFloor, Tex::DarkPanels, Tex::CeilingBeams);
    b.corridor(23, 15, 18, 15, 0, 9);
    b.trap(20, 15, 8);
    b.thrall(Race::Headless, 22.5, 15.5);
    b.room(8, 10, 10, 10, 0, 11);
    b.area("Malchor's study", 12, 14);
    b.light(8, 10, 10, 10, 8);
    // Malchor himself is not here: he waits unseen by the Shard Gate (see `malchor`).
    b.item(ItemKind::Bookshelf, 8.8, 10.6);
    b.item(ItemKind::Bookshelf, 10.4, 10.6);
    b.item(ItemKind::Cauldron, 16.0, 18.5);
    b.item(ItemKind::Table, 14.0, 12.0);
    b.put(Item::new(ItemKind::Book).data(8), 14.0, 12.0);
    b.thrall(Race::Skeleton, 9.5, 18.5);
    b.thrall(Race::Ghost, 16.5, 11.5);

    // East: Malchor's soul vault.
    b.style(Tex::Flagstones, Tex::GreyBlocks, Tex::CeilingRock);
    b.corridor(41, 15, 47, 15, 0, 9);
    b.trap(43, 15, 8);
    b.thrall(Race::Ghoul, 44.5, 15.5);
    b.door(46, 15, DoorAxis::EW, DoorKind::Iron, 501);
    b.room(48, 12, 6, 6, 0, 10);
    b.area("Soul vault", 50, 14);
    b.light(48, 12, 6, 6, 6);
    b.item(ItemKind::BonePile, 52.5, 16.8);
    b.item(ItemKind::Skull, 49.0, 13.0);
    b.container(ItemKind::Chest, 52.6, 12.8, vec![Item::new(ItemKind::RedPotion), Item::new(ItemKind::RedPotion), Item::new(ItemKind::Coin).qty(50).data(50)]);
    b.put(Item::new(ItemKind::Scroll).data(12), 49.5, 16.5);

    // South: the Shard Gate and the Throne of Souls.
    b.style(Tex::Flagstones, Tex::RedBricks, Tex::CeilingRock);
    b.corridor(32, 20, 32, 27, 0, 10);
    b.door(32, 25, DoorAxis::NS, DoorKind::Iron, SHARD_GATE);
    b.area("Shard Gate", 32, 24);
    b.writing(31, 24, "None shall pass but the bearer of Valor, Wisdom and Mercy.");
    b.style(Tex::Gravel, Tex::CaveRock, Tex::CeilingRock);
    b.cave(32, 40, 13, 11, 0, 16);
    b.area("Throne of Souls", 32, 44);
    b.corridor(32, 27, 32, 36, 0, 10);
    b.lava(20, 44, 9, 5, -3);
    b.lava(36, 34, 8, 4, -3);
    b.lava(26, 48, 14, 3, -3);
    b.light(18, 28, 30, 24, 8);
    let lord = b.named(Race::Demon, 0, "Varian", "lord", 32.5, 41.5, 0.0);
    b.npc_mut(lord).attitude = Attitude::Mellow;
    b.item(ItemKind::BonePile, 30.0, 43.0);
    b.item(ItemKind::BonePile, 35.0, 42.5);
    b.item(ItemKind::Shrine, 32.5, 30.0);
    b.item(ItemKind::Stalagmite, 22.0, 33.0);
    b.item(ItemKind::Stalagmite, 42.0, 40.0);

    // A crack across the throne cave, lava glowing at the bottom. The way
    // out of it climbs east and comes back to the gate.
    b.gap(30, 31, 7, 1, 12);
    b.area("lava crack", 33, 32);
    b.lava(30, 31, 7, 1, -12);
    b.ramp(37, 31, (1, 0), 4, -12, 0, 9);
    b.corridor(33, 27, 41, 31, 0, 9);
    b.message(32, 29, "The floor before the throne is split by a crack. Far below it, lava glows.");

    // The cells south of Malchor's study.
    b.style(Tex::Flagstones, Tex::RedBricks, Tex::CeilingRock);
    b.corridor(12, 20, 12, 24, 0, 9);
    b.room(6, 25, 12, 7, 0, 10);
    b.area("Cells", 11, 28);
    b.light(6, 25, 12, 7, 3);
    b.npc(Race::Imp, 9.5, 28.5);
    b.npc(Race::Imp, 15.5, 27.0);
    b.item(ItemKind::BonePile, 6.6, 30.5);
    b.item(ItemKind::Skull, 16.5, 30.8);
    b.container(ItemKind::Corpse, 7.5, 26.0, vec![Item::new(ItemKind::LeatherGloves), Item::new(ItemKind::Candle)]);
    b.container(ItemKind::Chest, 16.4, 25.6, vec![Item::new(ItemKind::Coin).qty(40).data(40), Item::new(ItemKind::YellowPotion)]);

    // A cave of fire south-east of the antechamber, where a reaper waits.
    b.style(Tex::Gravel, Tex::CaveRock, Tex::CeilingRock);
    b.cave(52, 30, 6, 5, 0, 12);
    b.area("Cave of fire", 52, 30);
    b.corridor(40, 20, 48, 26, 0, 9);
    b.lava(54, 31, 3, 3, -2);
    b.npc(Race::Reaper, 51.5, 31.5);
    b.npc(Race::Imp, 48.5, 28.5);
    b.item(ItemKind::Stalagmite, 49.0, 33.5);
    b.item(ItemKind::GoldNugget, 56.0, 28.0);
    b.item(ItemKind::BlueGem, 50.5, 26.5);
    b.item(ItemKind::RedPotion, 47.5, 31.5);
    b.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::physics::{MAX_STEP, SWIM_CLIMB};
    use crate::world::map::{EDGE_DIR, HSTEP};
    use std::collections::VecDeque;

    /// Tiles the player can reach from (x, z) in one move: a step up to
    /// `MAX_STEP` (`SWIM_CLIMB` out of water) and, with `jumps`, a running
    /// jump over one tile or a leap over two onto a ledge well below.
    fn moves(m: &Map, x: i32, z: i32, jumps: bool) -> Vec<(i32, i32)> {
        let lo = |t: &Tile| t.corner_floor().iter().cloned().fold(f32::MAX, f32::min);
        let hi = |t: &Tile| t.corner_floor().iter().cloned().fold(f32::MIN, f32::max);
        let here = m.get(x, z);
        let mut out = Vec::new();
        for (e, (dx, dz)) in EDGE_DIR.iter().enumerate() {
            let next = |i: i32| (x + dx * i, z + dz * i);
            let passable = |i: i32| {
                let (a, b) = (m.get(next(i - 1).0, next(i - 1).1), m.get(next(i).0, next(i).1));
                Map::in_bounds(next(i).0, next(i).1) && !b.is_solid() && a.edge_open(e) && b.edge_open((e + 2) % 4)
            };
            if !passable(1) {
                continue;
            }
            let t = m.get(next(1).0, next(1).1);
            let climb = match here.liquid {
                Liquid::Water => SWIM_CLIMB,
                _ => MAX_STEP,
            };
            if lo(t) - hi(here) <= climb + 0.01 {
                out.push(next(1));
            }
            // Jumps take off from dry, level floor with room overhead.
            if !jumps || here.liquid != Liquid::None || here.shape != Shape::Open {
                continue;
            }
            let floor = here.floor;
            let roomy = |i: i32| m.get(next(i).0, next(i).1).ceil >= floor + JUMP_HEADROOM;
            for span in 1..=2 {
                let path_ok = (0..=span + 1).all(roomy) && (1..=span + 1).all(passable) && (1..=span).all(|i| hi(m.get(next(i).0, next(i).1)) <= floor as f32 * HSTEP);
                let land = m.get(next(span + 1).0, next(span + 1).1);
                let land_ok = if span == 1 { hi(land) <= (floor + JUMP_UP) as f32 * HSTEP } else { hi(land) <= (floor - LEAP_DOWN) as f32 * HSTEP };
                if path_ok && land_ok {
                    out.push(next(span + 1));
                }
            }
        }
        out
    }

    /// Every tile reachable from `from` (all `false` if it is in rock).
    fn reachable(m: &Map, from: (i32, i32), jumps: bool) -> Vec<bool> {
        let idx = |x: i32, z: i32| (z * MAP_SIZE + x) as usize;
        let mut seen = vec![false; (MAP_SIZE * MAP_SIZE) as usize];
        if m.get(from.0, from.1).is_solid() {
            return seen;
        }
        let mut q = VecDeque::from([from]);
        seen[idx(from.0, from.1)] = true;
        while let Some((x, z)) = q.pop_front() {
            for (nx, nz) in moves(m, x, z, jumps) {
                if !seen[idx(nx, nz)] {
                    seen[idx(nx, nz)] = true;
                    q.push_back((nx, nz));
                }
            }
        }
        seen
    }

    /// Every tile from which `to` can be reached.
    fn leads_to(m: &Map, to: (i32, i32)) -> Vec<bool> {
        let idx = |x: i32, z: i32| (z * MAP_SIZE + x) as usize;
        let mut back: Vec<Vec<(i32, i32)>> = vec![Vec::new(); (MAP_SIZE * MAP_SIZE) as usize];
        for z in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                if !m.get(x, z).is_solid() {
                    for (nx, nz) in moves(m, x, z, true) {
                        back[idx(nx, nz)].push((x, z));
                    }
                }
            }
        }
        let mut seen = vec![false; (MAP_SIZE * MAP_SIZE) as usize];
        let mut q = VecDeque::from([to]);
        seen[idx(to.0, to.1)] = true;
        while let Some((x, z)) = q.pop_front() {
            for &(px, pz) in &back[idx(x, z)] {
                if !seen[idx(px, pz)] {
                    seen[idx(px, pz)] = true;
                    q.push_back((px, pz));
                }
            }
        }
        seen
    }

    /// Every level's NPCs, items and stairs must be reachable from its start
    /// (treating doors as passable), walking, swimming and jumping.
    #[test]
    fn levels_are_connected() {
        let levels = build_all();
        for (li, lv) in levels.iter().enumerate() {
            let m = &lv.map;
            let idx = |x: i32, z: i32| (z * MAP_SIZE + x) as usize;
            let start = (lv.start.0.floor() as i32, lv.start.1.floor() as i32);
            assert!(!m.get(start.0, start.1).is_solid(), "level {} start is in rock", li);
            let seen = reachable(m, start, true);
            let reach = |x: f32, z: f32| seen[idx(x.floor() as i32, z.floor() as i32)];
            for n in &lv.npcs {
                assert!(reach(n.pos.x, n.pos.z), "level {} npc {:?} {:?} at ({:.1},{:.1}) unreachable", li, n.race, n.name, n.pos.x, n.pos.z);
                assert!(m.point_open(n.pos.x, n.pos.z), "level {} npc {:?} inside rock", li, n.race);
            }
            for it in &lv.items {
                assert!(reach(it.pos.x, it.pos.z), "level {} item {:?} at ({:.1},{:.1}) unreachable", li, it.item.kind, it.pos.x, it.pos.z);
            }
            for t in &lv.triggers {
                if let TriggerKind::Damage { .. } = t.kind {
                    assert!(seen[idx(t.tx, t.tz)], "level {} trap at ({},{}) unreachable", li, t.tx, t.tz);
                    assert_eq!(m.get(t.tx, t.tz).liquid, Liquid::None, "level {} trap at ({},{}) in liquid", li, t.tx, t.tz);
                    let shared = lv.triggers.iter().filter(|o| (o.tx, o.tz) == (t.tx, t.tz)).count();
                    assert_eq!(shared, 1, "level {} trap at ({},{}) shares its tile", li, t.tx, t.tz);
                }
                if let TriggerKind::Stairs { level, x, z, .. } = t.kind {
                    assert!(seen[idx(t.tx, t.tz)], "level {} stairs at ({},{}) unreachable", li, t.tx, t.tz);
                    let dest = &levels[level];
                    assert!(dest.map.point_open(x, z), "level {} stairs lead into rock on level {}", li, level);
                    let ds = (dest.start.0.floor() as i32, dest.start.1.floor() as i32);
                    let arrive = (x.floor() as i32, z.floor() as i32);
                    assert!(reachable(&dest.map, ds, true)[idx(arrive.0, arrive.1)], "level {} stairs arrive at an unreachable spot on level {}", li, level);
                }
            }
        }
    }

    /// Every level has places only a jump reaches. On the first level they
    /// are side areas; below it the way on (the stairs down, and at the
    /// bottom Varian) lies across a gap.
    #[test]
    fn gaps_must_be_jumped() {
        for (li, lv) in build_all().iter().enumerate() {
            let m = &lv.map;
            let idx = |x: i32, z: i32| (z * MAP_SIZE + x) as usize;
            let start = (lv.start.0.floor() as i32, lv.start.1.floor() as i32);
            let walk = reachable(m, start, false);
            let jump = reachable(m, start, true);
            assert!(jump.iter().zip(&walk).any(|(j, w)| *j && !*w), "level {} has no gap to jump", li);
            if li == 0 {
                continue;
            }
            let way_on: Vec<(i32, i32)> = match lv.triggers.iter().find_map(|t| match t.kind {
                TriggerKind::Stairs { level, .. } if level == li + 1 => Some((t.tx, t.tz)),
                _ => None,
            }) {
                Some(stairs) => vec![stairs],
                None => lv.npcs.iter().filter(|n| n.name.as_deref() == Some("Varian")).map(|n| (n.pos.x.floor() as i32, n.pos.z.floor() as i32)).collect(),
            };
            assert!(!way_on.is_empty(), "level {} has no way on", li);
            for (x, z) in way_on {
                assert!(!walk[idx(x, z)], "level {}: the way on at ({},{}) needs no jump", li, x, z);
            }
        }
    }

    /// No pit, ledge or pool is a trap: from every tile the player can reach,
    /// they can get back to the start (except from lava, which kills).
    #[test]
    fn no_dead_ends() {
        for (li, lv) in build_all().iter().enumerate() {
            let m = &lv.map;
            let start = (lv.start.0.floor() as i32, lv.start.1.floor() as i32);
            let there = reachable(m, start, true);
            let back = leads_to(m, start);
            for z in 0..MAP_SIZE {
                for x in 0..MAP_SIZE {
                    let i = (z * MAP_SIZE + x) as usize;
                    if there[i] && !back[i] && m.get(x, z).liquid != Liquid::Lava {
                        panic!("level {}: no way back to the start from ({},{})", li, x, z);
                    }
                }
            }
        }
    }

    /// Every one of the 24 runes must be lying somewhere in the Abyss.
    #[test]
    fn every_rune_is_placed() {
        fn collect(items: &[Item], found: &mut [bool; 24]) {
            for it in items {
                if it.kind == ItemKind::RuneStone {
                    found[it.data as usize % 24] = true;
                }
                collect(&it.contents, found);
            }
        }
        let mut found = [false; 24];
        for lv in build_all() {
            let items: Vec<Item> = lv.items.iter().map(|w| w.item.clone()).collect();
            collect(&items, &mut found);
        }
        for (r, f) in found.iter().enumerate() {
            assert!(f, "the {} rune is nowhere in the Abyss", super::super::magic::RUNE_NAMES[r]);
        }
    }
}
