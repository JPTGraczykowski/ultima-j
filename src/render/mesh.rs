//! Converts the tile map into world-space polygons, grouped per tile.

use crate::art::textures::Tex;
use crate::util::V3;
use crate::world::map::{Map, Tile, CORNER_OFS, EDGE_DIR, MAP_SIZE};

#[derive(Clone, Copy, Debug, Default)]
pub struct WVert {
    pub p: V3,
    pub u: f32,
    pub v: f32,
    /// Ambient light at this vertex (0..31).
    pub l: f32,
}

#[derive(Clone, Debug)]
pub struct Poly {
    pub v: [WVert; 4],
    pub n: u8,
    pub tex: Tex,
    /// Polygons drawn from both sides (doors, thin objects).
    pub two_sided: bool,
    /// Ignore lighting (glowing things).
    pub fullbright: bool,
    /// Picking id (see render::id_*).
    pub id: u32,
}

impl Poly {
    pub fn verts(&self) -> &[WVert] {
        &self.v[..self.n as usize]
    }
}

pub struct LevelMesh {
    pub polys: Vec<Poly>,
    /// (start, count) into `polys` for each tile.
    pub tile_ranges: Vec<(u32, u32)>,
}

/// Average ambient light of the (up to) four open tiles touching a corner.
fn corner_light(map: &Map, cx: i32, cz: i32) -> f32 {
    let mut sum = 0.0;
    let mut n = 0.0;
    for (dx, dz) in [(-1, -1), (0, -1), (-1, 0), (0, 0)] {
        let t = map.get(cx + dx, cz + dz);
        if !t.is_solid() {
            sum += t.light as f32;
            n += 1.0;
        }
    }
    if n > 0.0 { sum / n } else { 0.0 }
}

fn corner_pos(x: i32, z: i32, corner: usize) -> (f32, f32) {
    let (ox, oz) = CORNER_OFS[corner];
    (x as f32 + ox, z as f32 + oz)
}

/// Build a wall quad between two points (left, right as seen by the viewer),
/// spanning heights [bot_l, top_l] at the left and [bot_r, top_r] at the right.
#[allow(clippy::too_many_arguments)]
pub fn wall_poly(
    l: (f32, f32),
    r: (f32, f32),
    bot_l: f32,
    top_l: f32,
    bot_r: f32,
    top_r: f32,
    tex: Tex,
    light_l: f32,
    light_r: f32,
) -> Option<Poly> {
    let dx = r.0 - l.0;
    let dz = r.1 - l.1;
    let len = (dx * dx + dz * dz).sqrt();
    if len < 1e-6 {
        return None;
    }
    let (ux, uz) = (dx / len, dz / len);
    let u_l = l.0 * ux + l.1 * uz;
    let u_r = r.0 * ux + r.1 * uz;
    let hl = top_l - bot_l;
    let hr = top_r - bot_r;
    let mk = |x: f32, z: f32, y: f32, u: f32, li: f32| WVert { p: V3::new(x, y, z), u, v: -y, l: li };
    let tl = mk(l.0, l.1, top_l, u_l, light_l);
    let tr = mk(r.0, r.1, top_r, u_r, light_r);
    let br = mk(r.0, r.1, bot_r, u_r, light_r);
    let bl = mk(l.0, l.1, bot_l, u_l, light_l);
    const EPS: f32 = 1e-4;
    if hl > EPS && hr > EPS {
        Some(Poly { v: [tl, tr, br, bl], n: 4, tex, two_sided: false, fullbright: false, id: 0 })
    } else if hl > EPS {
        Some(Poly { v: [tl, tr, bl, bl], n: 3, tex, two_sided: false, fullbright: false, id: 0 })
    } else if hr > EPS {
        Some(Poly { v: [tl, tr, br, br], n: 3, tex, two_sided: false, fullbright: false, id: 0 })
    } else {
        None
    }
}

/// Add the part of a vertical face where `upper` exceeds `lower` along the
/// edge (heights given at left and right endpoints). Handles crossings.
#[allow(clippy::too_many_arguments)]
fn add_step(
    out: &mut Vec<Poly>,
    l: (f32, f32),
    r: (f32, f32),
    lower_l: f32,
    lower_r: f32,
    upper_l: f32,
    upper_r: f32,
    tex: Tex,
    li_l: f32,
    li_r: f32,
) {
    let dl = upper_l - lower_l;
    let dr = upper_r - lower_r;
    if dl <= 1e-4 && dr <= 1e-4 {
        return;
    }
    if dl >= -1e-4 && dr >= -1e-4 {
        if let Some(p) = wall_poly(l, r, lower_l, upper_l.max(lower_l), lower_r, upper_r.max(lower_r), tex, li_l, li_r) {
            out.push(p);
        }
        return;
    }
    // Crossing: split where the difference is zero.
    let t = dl / (dl - dr);
    let m = (l.0 + (r.0 - l.0) * t, l.1 + (r.1 - l.1) * t);
    let hm = lower_l + (lower_r - lower_l) * t;
    let li_m = li_l + (li_r - li_l) * t;
    if dl > 0.0 {
        if let Some(p) = wall_poly(l, m, lower_l, upper_l, hm, hm, tex, li_l, li_m) {
            out.push(p);
        }
    } else if let Some(p) = wall_poly(m, r, hm, hm, lower_r, upper_r, tex, li_m, li_r) {
        out.push(p);
    }
}

pub fn build_tile(map: &Map, x: i32, z: i32, out: &mut Vec<Poly>) {
    let t: &Tile = map.get(x, z);
    if t.is_solid() {
        return;
    }
    let cf = t.corner_floor();
    let ch = t.ceil_h();
    let lights: [f32; 4] = [
        corner_light(map, x, z),
        corner_light(map, x + 1, z),
        corner_light(map, x + 1, z + 1),
        corner_light(map, x, z + 1),
    ];
    // Floor and ceiling.
    let fc = t.floor_corners();
    let mut fv = [WVert::default(); 4];
    let mut cv = [WVert::default(); 4];
    for (i, &corner) in fc.iter().enumerate() {
        let (px, pz) = corner_pos(x, z, corner);
        fv[i] = WVert { p: V3::new(px, cf[corner], pz), u: px, v: pz, l: lights[corner] };
        cv[fc.len() - 1 - i] = WVert { p: V3::new(px, ch, pz), u: px, v: pz, l: lights[corner] };
    }
    let n = fc.len() as u8;
    let fullbright_floor = t.floor_tex.emissive();
    out.push(Poly { v: fv, n, tex: t.floor_tex, two_sided: false, fullbright: fullbright_floor, id: crate::render::surf_id(crate::render::ID_FLOOR, x, z, t.floor_tex) });
    out.push(Poly { v: cv, n, tex: t.ceil_tex, two_sided: false, fullbright: false, id: crate::render::surf_id(crate::render::ID_CEIL, x, z, t.ceil_tex) });

    // Edge walls.
    for edge in 0..4 {
        if !t.edge_open(edge) {
            continue;
        }
        let a = edge;
        let b = (edge + 1) % 4;
        let l = corner_pos(x, z, a);
        let r = corner_pos(x, z, b);
        let (dx, dz) = EDGE_DIR[edge];
        let nb = map.get(x + dx, z + dz);
        let wall_id = crate::render::surf_id(crate::render::ID_WALL, x + dx, z + dz, nb.wall_tex);
        let first = out.len();
        let opposite = (edge + 2) % 4;
        if nb.is_solid() || !nb.edge_open(opposite) {
            if let Some(mut p) = wall_poly(l, r, cf[a], ch, cf[b], ch, nb.wall_tex, lights[a], lights[b]) {
                p.id = wall_id;
                out.push(p);
            }
            continue;
        }
        // Neighbour's floor at the same two points: my corner a == their corner (opposite+1)%4 etc.
        let ncf = nb.corner_floor();
        // Map my corner index to the neighbour's coincident corner.
        let coincide = |my: usize| -> usize {
            match (edge, my) {
                (0, 0) => 3, // my NW = their SW
                (0, 1) => 2, // my NE = their SE
                (1, 1) => 0, // my NE = their NW
                (1, 2) => 3, // my SE = their SW
                (2, 2) => 1, // my SE = their NE
                (2, 3) => 0, // my SW = their NW
                (3, 3) => 2, // my SW = their SE
                (3, 0) => 1, // my NW = their NE
                _ => unreachable!(),
            }
        };
        let nfa = ncf[coincide(a)];
        let nfb = ncf[coincide(b)];
        let nch = nb.ceil_h();
        // Floor step up to the neighbour (clamped to my ceiling).
        add_step(out, l, r, cf[a], cf[b], nfa.min(ch), nfb.min(ch), nb.wall_tex, lights[a], lights[b]);
        // Ceiling step down to the neighbour's lower ceiling.
        if nch < ch {
            let lo_a = nch.max(cf[a]);
            let lo_b = nch.max(cf[b]);
            add_step(out, l, r, lo_a, lo_b, ch, ch, nb.wall_tex, lights[a], lights[b]);
        }
        for p in out[first..].iter_mut() {
            p.id = wall_id;
        }
    }
    // Diagonal wall.
    if let Some((a, b)) = t.diag_wall() {
        let l = corner_pos(x, z, a);
        let r = corner_pos(x, z, b);
        if let Some(mut p) = wall_poly(l, r, cf[a], ch, cf[b], ch, t.wall_tex, lights[a], lights[b]) {
            p.id = crate::render::surf_id(crate::render::ID_WALL, x, z, t.wall_tex);
            out.push(p);
        }
    }
}

impl LevelMesh {
    pub fn build(map: &Map) -> LevelMesh {
        let mut polys = Vec::new();
        let mut tile_ranges = Vec::with_capacity((MAP_SIZE * MAP_SIZE) as usize);
        for z in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                let start = polys.len() as u32;
                build_tile(map, x, z, &mut polys);
                tile_ranges.push((start, polys.len() as u32 - start));
            }
        }
        LevelMesh { polys, tile_ranges }
    }

    pub fn tile_polys(&self, x: i32, z: i32) -> &[Poly] {
        let (s, n) = self.tile_ranges[(z * MAP_SIZE + x) as usize];
        &self.polys[s as usize..(s + n) as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::map::Shape;

    fn room() -> Map {
        let mut m = Map::new();
        for z in 1..4 {
            for x in 1..4 {
                m.set(x, z, Tile::open(0, 8, Tex::Dirt, Tex::CeilingRock, Tex::RoughStone));
            }
        }
        m
    }

    #[test]
    fn open_tile_in_room_corner_has_two_walls() {
        let m = room();
        let mut out = Vec::new();
        build_tile(&m, 1, 1, &mut out);
        // floor + ceiling + north wall + west wall
        assert_eq!(out.len(), 4);
        build_tile(&m, 2, 2, &mut out);
        assert_eq!(out.len(), 6);
    }

    #[test]
    fn step_up_creates_face() {
        let mut m = room();
        m.get_mut(2, 1).unwrap().floor = 2;
        let mut out = Vec::new();
        build_tile(&m, 2, 2, &mut out);
        // floor, ceiling, plus a step face toward the north neighbour
        assert_eq!(out.len(), 3);
        let step = &out[2];
        assert!((step.v[0].p.y - 0.25).abs() < 1e-5);
    }

    #[test]
    fn diagonal_tile_has_triangle_floor_and_diag_wall() {
        let mut m = room();
        m.get_mut(1, 1).unwrap().shape = Shape::DiagSE;
        let mut out = Vec::new();
        build_tile(&m, 1, 1, &mut out);
        assert_eq!(out[0].n, 3);
        // floor, ceiling, diagonal wall (N and W edges are closed for DiagSE)
        assert_eq!(out.len(), 3);
    }
}
