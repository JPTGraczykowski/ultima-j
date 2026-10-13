//! Movement and collision for the player and creatures.

use super::state::{DoorAxis, Level};
use crate::util::V3;
use crate::world::map::Liquid;

pub const PLAYER_RADIUS: f32 = 0.19;
pub const PLAYER_HEIGHT: f32 = 0.86;
pub const EYE_HEIGHT: f32 = 0.64;
pub const SWIM_SINK: f32 = 0.22;
pub const MAX_STEP: f32 = 0.27;
pub const GRAVITY: f32 = 11.0;
pub const JUMP_SPEED: f32 = 3.5;

/// A body that wants to move.
#[derive(Clone, Copy)]
pub struct Body {
    pub radius: f32,
    pub height: f32,
    pub max_step: f32,
    /// Id of this body's NPC (to skip self-collision), 0 for the player.
    pub self_id: u32,
    /// Flying bodies ignore floor steps (but not walls).
    pub flying: bool,
    /// Creatures that cannot swim keep out of water; only fliers cross lava.
    pub avoid_water: bool,
    pub avoid_lava: bool,
}

const SAMPLES: [(f32, f32); 9] = [
    (0.0, 0.0),
    (1.0, 0.0),
    (0.7071, 0.7071),
    (0.0, 1.0),
    (-0.7071, 0.7071),
    (-1.0, 0.0),
    (-0.7071, -0.7071),
    (0.0, -1.0),
    (0.7071, -0.7071),
];

/// Does a circle at (x, z) overlap a closed door?
pub fn door_blocks(level: &Level, x: f32, z: f32, r: f32) -> bool {
    for d in &level.doors {
        if !d.blocks() {
            continue;
        }
        let (tx, tz) = (d.tx as f32, d.tz as f32);
        match d.axis {
            DoorAxis::NS => {
                if (z - (tz + 0.5)).abs() < r + 0.04 && x > tx - r && x < tx + 1.0 + r {
                    return true;
                }
            }
            DoorAxis::EW => {
                if (x - (tx + 0.5)).abs() < r + 0.04 && z > tz - r && z < tz + 1.0 + r {
                    return true;
                }
            }
        }
    }
    false
}

/// Would a specific door (if closed) overlap a circle at `p`?
pub fn door_blocks_point(level: &Level, door: u32, p: V3, r: f32) -> bool {
    let Some(d) = level.doors.iter().find(|d| d.id == door) else { return false };
    let (tx, tz) = (d.tx as f32, d.tz as f32);
    match d.axis {
        DoorAxis::NS => (p.z - (tz + 0.5)).abs() < r + 0.05 && p.x > tx - r && p.x < tx + 1.0 + r,
        DoorAxis::EW => (p.x - (tx + 0.5)).abs() < r + 0.05 && p.z > tz - r && p.z < tz + 1.0 + r,
    }
}

/// Can the body stand at (x, z) with its feet at `feet`? Returns the floor height.
pub fn can_stand(level: &Level, b: &Body, x: f32, z: f32, feet: f32, others: &[(u32, V3, f32)]) -> Option<f32> {
    let map = &level.map;
    let mut center_floor = 0.0;
    for (i, (ox, oz)) in SAMPLES.iter().enumerate() {
        let sx = x + ox * b.radius;
        let sz = z + oz * b.radius;
        if !map.point_open(sx, sz) {
            return None;
        }
        let f = map.floor_at(sx, sz);
        let c = map.ceil_at(sx, sz);
        if !b.flying && f > feet + b.max_step {
            return None;
        }
        if c - f.max(feet) < b.height * 0.92 {
            return None;
        }
        if b.flying && feet < f - 0.05 {
            return None;
        }
        if b.avoid_water || b.avoid_lava {
            let l = liquid_at(level, sx, sz);
            if (b.avoid_water && l == Liquid::Water) || (b.avoid_lava && l == Liquid::Lava) {
                return None;
            }
        }
        if i == 0 {
            center_floor = f;
        }
    }
    if door_blocks(level, x, z, b.radius) {
        return None;
    }
    for (id, p, r) in others {
        if *id == b.self_id {
            continue;
        }
        let dx = p.x - x;
        let dz = p.z - z;
        if dx * dx + dz * dz < (r + b.radius) * (r + b.radius) && (p.y - feet).abs() < b.height {
            return None;
        }
    }
    Some(center_floor)
}

/// Move a body by (dx, dz), sliding along walls. Returns true if it moved at all.
pub fn slide_move(level: &Level, b: &Body, pos: &mut V3, dx: f32, dz: f32, others: &[(u32, V3, f32)]) -> bool {
    let dist = (dx * dx + dz * dz).sqrt();
    if dist < 1e-6 {
        return false;
    }
    let steps = (dist / (b.radius * 0.5)).ceil().max(1.0) as i32;
    let sx = dx / steps as f32;
    let sz = dz / steps as f32;
    let mut moved = false;
    for _ in 0..steps {
        if can_stand(level, b, pos.x + sx, pos.z + sz, pos.y, others).is_some() {
            pos.x += sx;
            pos.z += sz;
            moved = true;
        } else if can_stand(level, b, pos.x + sx, pos.z, pos.y, others).is_some() {
            pos.x += sx;
            moved = true;
        } else if can_stand(level, b, pos.x, pos.z + sz, pos.y, others).is_some() {
            pos.z += sz;
            moved = true;
        } else {
            break;
        }
    }
    moved
}

pub fn liquid_at(level: &Level, x: f32, z: f32) -> Liquid {
    level.map.get(x.floor() as i32, z.floor() as i32).liquid
}

/// Line of sight between two points (eye heights), checking walls and closed doors.
pub fn line_of_sight(level: &Level, a: V3, b: V3) -> bool {
    let d = b - a;
    let len = (d.x * d.x + d.z * d.z).sqrt();
    let steps = (len / 0.1).ceil().max(1.0) as i32;
    for i in 1..steps {
        let t = i as f32 / steps as f32;
        let p = a + d * t;
        if !level.map.point_open(p.x, p.z) {
            return false;
        }
        let f = level.map.floor_at(p.x, p.z);
        let c = level.map.ceil_at(p.x, p.z);
        if p.y < f || p.y > c {
            return false;
        }
        if door_blocks(level, p.x, p.z, 0.01) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::art::textures::Tex;
    use crate::world::map::{Map, Shape, Tile};

    fn corridor() -> Level {
        let mut m = Map::new();
        for x in 1..10 {
            m.set(x, 5, Tile::open(0, 10, Tex::Dirt, Tex::CeilingRock, Tex::RoughStone));
        }
        Level::new("t", m)
    }

    fn body() -> Body {
        Body { radius: PLAYER_RADIUS, height: PLAYER_HEIGHT, max_step: MAX_STEP, self_id: 0, flying: false, avoid_water: false, avoid_lava: false }
    }

    #[test]
    fn cannot_walk_into_walls() {
        let lv = corridor();
        let b = body();
        let mut p = V3::new(2.5, 0.0, 5.5);
        slide_move(&lv, &b, &mut p, 0.0, -1.0, &[]);
        assert!(p.z >= 5.0 + PLAYER_RADIUS - 1e-3, "z = {}", p.z);
        slide_move(&lv, &b, &mut p, 3.0, 0.0, &[]);
        assert!((p.x - 5.5).abs() < 1e-3);
    }

    #[test]
    fn steps_and_ceilings() {
        let mut lv = corridor();
        lv.map.get_mut(4, 5).unwrap().floor = 2; // 0.25 high step: climbable
        lv.map.get_mut(6, 5).unwrap().floor = 5; // 0.625: too high
        let b = body();
        let mut p = V3::new(2.5, 0.0, 5.5);
        slide_move(&lv, &b, &mut p, 2.0, 0.0, &[]);
        assert!(p.x > 4.0, "should climb the low step, x = {}", p.x);
        let mut q = V3::new(5.5, 0.25, 5.5);
        slide_move(&lv, &b, &mut q, 1.0, 0.0, &[]);
        assert!(q.x < 6.0 - PLAYER_RADIUS + 1e-3, "high step blocks, x = {}", q.x);
        lv.map.get_mut(8, 5).unwrap().ceil = 5; // 0.625 high ceiling: too low
        let mut r = V3::new(7.5, 0.0, 5.5);
        slide_move(&lv, &b, &mut r, 1.0, 0.0, &[]);
        assert!(r.x < 8.0 - PLAYER_RADIUS + 1e-3);
    }

    #[test]
    fn diagonal_walls_block() {
        let mut lv = corridor();
        let t = lv.map.get_mut(3, 5).unwrap();
        t.shape = Shape::DiagSE; // open triangle NE, SE, SW
        assert!(can_stand(&lv, &body(), 3.8, 5.8, 0.0, &[]).is_some());
        assert!(can_stand(&lv, &body(), 3.2, 5.2, 0.0, &[]).is_none());
    }

    #[test]
    fn doors_block_until_open() {
        let mut lv = corridor();
        lv.add_door(5, 5, DoorAxis::EW, super::super::state::DoorKind::Wood, 0);
        let b = body();
        let mut p = V3::new(4.5, 0.0, 5.5);
        slide_move(&lv, &b, &mut p, 1.0, 0.0, &[]);
        assert!(p.x < 5.5 - PLAYER_RADIUS);
        lv.doors[0].open = 1.0;
        slide_move(&lv, &b, &mut p, 1.0, 0.0, &[]);
        assert!(p.x > 5.4);
    }

    #[test]
    fn sight_blocked_by_walls() {
        let lv = corridor();
        assert!(line_of_sight(&lv, V3::new(1.5, 0.5, 5.5), V3::new(8.5, 0.5, 5.5)));
        assert!(!line_of_sight(&lv, V3::new(1.5, 0.5, 5.5), V3::new(1.5, 0.5, 8.5)));
    }
}
