//! The tile map. Like the original, each level is a 64x64 grid of tiles; a
//! tile is solid, open, half-open along a diagonal, or a slope, and has its
//! own floor and ceiling heights (in 1/8 tile steps) and textures.

use crate::art::textures::Tex;
use serde::{Deserialize, Serialize};

pub const MAP_SIZE: i32 = 64;
/// World units per height step.
pub const HSTEP: f32 = 1.0 / 8.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Shape {
    Solid,
    Open,
    /// Diagonals are named after their open half.
    DiagSE,
    DiagSW,
    DiagNE,
    DiagNW,
    /// Slopes are named after the direction that rises.
    SlopeN,
    SlopeS,
    SlopeE,
    SlopeW,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Liquid {
    None,
    Water,
    Lava,
}

/// Corner indices (clockwise from north-west, seen from above).
pub const NW: usize = 0;
pub const NE: usize = 1;
pub const SE: usize = 2;
pub const SW: usize = 3;

/// Edge e runs from corner e to corner (e+1)%4: N, E, S, W.
pub const EDGE_N: usize = 0;
pub const EDGE_E: usize = 1;
pub const EDGE_S: usize = 2;
pub const EDGE_W: usize = 3;
pub const EDGE_DIR: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub shape: Shape,
    pub floor: i16,
    pub ceil: i16,
    pub slope: i16,
    pub floor_tex: Tex,
    pub ceil_tex: Tex,
    pub wall_tex: Tex,
    pub liquid: Liquid,
    /// Ambient light 0..31.
    pub light: u8,
    /// Seen by the player (automap).
    #[serde(default)]
    pub explored: bool,
}

impl Tile {
    pub const SOLID: Tile = Tile {
        shape: Shape::Solid,
        floor: 0,
        ceil: 0,
        slope: 0,
        floor_tex: Tex::Dirt,
        ceil_tex: Tex::CeilingRock,
        wall_tex: Tex::RoughStone,
        liquid: Liquid::None,
        light: 0,
        explored: false,
    };

    pub fn open(floor: i16, ceil: i16, floor_tex: Tex, ceil_tex: Tex, wall_tex: Tex) -> Tile {
        Tile { shape: Shape::Open, floor, ceil, slope: 0, floor_tex, ceil_tex, wall_tex, liquid: Liquid::None, light: 0, explored: false }
    }

    #[inline]
    pub fn is_solid(&self) -> bool {
        self.shape == Shape::Solid
    }

    pub fn is_diag(&self) -> bool {
        matches!(self.shape, Shape::DiagSE | Shape::DiagSW | Shape::DiagNE | Shape::DiagNW)
    }

    /// Floor heights at the four corners (world units), NW, NE, SE, SW.
    pub fn corner_floor(&self) -> [f32; 4] {
        let lo = self.floor as f32 * HSTEP;
        let hi = (self.floor + self.slope) as f32 * HSTEP;
        match self.shape {
            Shape::SlopeN => [hi, hi, lo, lo],
            Shape::SlopeS => [lo, lo, hi, hi],
            Shape::SlopeE => [lo, hi, hi, lo],
            Shape::SlopeW => [hi, lo, lo, hi],
            _ => [lo; 4],
        }
    }

    pub fn ceil_h(&self) -> f32 {
        self.ceil as f32 * HSTEP
    }

    /// Is the given edge of this tile fully open (passable)?
    pub fn edge_open(&self, edge: usize) -> bool {
        match self.shape {
            Shape::Solid => false,
            Shape::DiagSE => edge == EDGE_E || edge == EDGE_S,
            Shape::DiagSW => edge == EDGE_S || edge == EDGE_W,
            Shape::DiagNE => edge == EDGE_N || edge == EDGE_E,
            Shape::DiagNW => edge == EDGE_N || edge == EDGE_W,
            _ => true,
        }
    }

    /// For a point inside the tile (fractions 0..1), is it in the open part?
    pub fn point_open(&self, fx: f32, fz: f32) -> bool {
        match self.shape {
            Shape::Solid => false,
            Shape::DiagSE => fx + fz >= 1.0,
            Shape::DiagSW => fz >= fx,
            Shape::DiagNE => fx >= fz,
            Shape::DiagNW => fx + fz <= 1.0,
            _ => true,
        }
    }

    /// The diagonal wall as (left, right) corner indices seen from the open side.
    pub fn diag_wall(&self) -> Option<(usize, usize)> {
        match self.shape {
            Shape::DiagSE => Some((SW, NE)),
            Shape::DiagSW => Some((NW, SE)),
            Shape::DiagNE => Some((SE, NW)),
            Shape::DiagNW => Some((NE, SW)),
            _ => None,
        }
    }

    /// Corners of the open floor polygon, clockwise seen from above.
    pub fn floor_corners(&self) -> &'static [usize] {
        match self.shape {
            Shape::DiagSE => &[NE, SE, SW],
            Shape::DiagSW => &[SE, SW, NW],
            Shape::DiagNE => &[NW, NE, SE],
            Shape::DiagNW => &[NW, NE, SW],
            Shape::Solid => &[],
            _ => &[NW, NE, SE, SW],
        }
    }
}

/// Corner offsets within a tile (x, z).
pub const CORNER_OFS: [(f32, f32); 4] = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Map {
    pub tiles: Vec<Tile>,
}

impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}

impl Map {
    pub fn new() -> Map {
        Map { tiles: vec![Tile::SOLID; (MAP_SIZE * MAP_SIZE) as usize] }
    }

    #[inline]
    pub fn in_bounds(x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < MAP_SIZE && y < MAP_SIZE
    }

    #[inline]
    pub fn get(&self, x: i32, y: i32) -> &Tile {
        if Self::in_bounds(x, y) { &self.tiles[(y * MAP_SIZE + x) as usize] } else { &Tile::SOLID }
    }

    #[inline]
    pub fn get_mut(&mut self, x: i32, y: i32) -> Option<&mut Tile> {
        if Self::in_bounds(x, y) { Some(&mut self.tiles[(y * MAP_SIZE + x) as usize]) } else { None }
    }

    pub fn set(&mut self, x: i32, y: i32, t: Tile) {
        if let Some(d) = self.get_mut(x, y) {
            *d = t;
        }
    }

    /// Floor height at a world position (interpolating slopes).
    pub fn floor_at(&self, x: f32, z: f32) -> f32 {
        let tx = x.floor() as i32;
        let tz = z.floor() as i32;
        let t = self.get(tx, tz);
        let c = t.corner_floor();
        let fx = x - tx as f32;
        let fz = z - tz as f32;
        let n = c[NW] + (c[NE] - c[NW]) * fx;
        let s = c[SW] + (c[SE] - c[SW]) * fx;
        n + (s - n) * fz
    }

    pub fn ceil_at(&self, x: f32, z: f32) -> f32 {
        self.get(x.floor() as i32, z.floor() as i32).ceil_h()
    }

    /// Is a world point inside open space (ignoring heights)?
    pub fn point_open(&self, x: f32, z: f32) -> bool {
        let tx = x.floor() as i32;
        let tz = z.floor() as i32;
        let t = self.get(tx, tz);
        t.point_open(x - tx as f32, z - tz as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slope_interpolates() {
        let mut m = Map::new();
        let mut t = Tile::open(0, 16, Tex::Dirt, Tex::CeilingRock, Tex::RoughStone);
        t.shape = Shape::SlopeN;
        t.slope = 8;
        m.set(5, 5, t);
        assert!((m.floor_at(5.5, 5.0) - 1.0).abs() < 1e-5); // north edge high
        assert!((m.floor_at(5.5, 5.999) - 0.0).abs() < 1e-2);
        assert!((m.floor_at(5.5, 5.5) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn diagonal_open_halves() {
        let mut t = Tile::open(0, 8, Tex::Dirt, Tex::CeilingRock, Tex::RoughStone);
        t.shape = Shape::DiagSE;
        assert!(t.point_open(0.9, 0.9));
        assert!(!t.point_open(0.1, 0.1));
        assert!(t.edge_open(EDGE_E) && t.edge_open(EDGE_S));
        assert!(!t.edge_open(EDGE_N) && !t.edge_open(EDGE_W));
        t.shape = Shape::DiagNW;
        assert!(t.point_open(0.1, 0.1));
        assert!(!t.point_open(0.9, 0.9));
    }

    #[test]
    fn out_of_bounds_is_solid() {
        let m = Map::new();
        assert!(m.get(-1, 0).is_solid());
        assert!(m.get(64, 3).is_solid());
    }
}
