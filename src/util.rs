//! Small math, random and noise helpers shared by the art generator and game.

use std::ops::{Add, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct V3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl V3 {
    pub const ZERO: V3 = V3 { x: 0.0, y: 0.0, z: 0.0 };
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> V3 {
        V3 { x, y, z }
    }
    #[inline]
    pub fn cross(self, o: V3) -> V3 {
        V3::new(self.y * o.z - self.z * o.y, self.z * o.x - self.x * o.z, self.x * o.y - self.y * o.x)
    }
    #[inline]
    pub fn dot(self, o: V3) -> f32 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }
    #[inline]
    pub fn len(self) -> f32 {
        self.dot(self).sqrt()
    }
    #[inline]
    pub fn norm(self) -> V3 {
        let l = self.len();
        if l > 1e-9 { self * (1.0 / l) } else { self }
    }
    #[inline]
    pub fn lerp(self, o: V3, t: f32) -> V3 {
        self + (o - self) * t
    }
}

impl Add for V3 {
    type Output = V3;
    #[inline]
    fn add(self, o: V3) -> V3 {
        V3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
}
impl Sub for V3 {
    type Output = V3;
    #[inline]
    fn sub(self, o: V3) -> V3 {
        V3::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }
}
impl Mul<f32> for V3 {
    type Output = V3;
    #[inline]
    fn mul(self, s: f32) -> V3 {
        V3::new(self.x * s, self.y * s, self.z * s)
    }
}
impl Neg for V3 {
    type Output = V3;
    #[inline]
    fn neg(self) -> V3 {
        V3::new(-self.x, -self.y, -self.z)
    }
}

/// 3x3 rotation matrix (row-major).
#[derive(Clone, Copy, Debug)]
pub struct M3(pub [[f32; 3]; 3]);

impl M3 {
    pub const IDENTITY: M3 = M3([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);
    pub fn rot_x(a: f32) -> M3 {
        let (s, c) = a.sin_cos();
        M3([[1.0, 0.0, 0.0], [0.0, c, -s], [0.0, s, c]])
    }
    pub fn rot_y(a: f32) -> M3 {
        let (s, c) = a.sin_cos();
        M3([[c, 0.0, s], [0.0, 1.0, 0.0], [-s, 0.0, c]])
    }
    pub fn rot_z(a: f32) -> M3 {
        let (s, c) = a.sin_cos();
        M3([[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]])
    }
    #[inline]
    pub fn mul_v(&self, v: V3) -> V3 {
        let m = &self.0;
        V3::new(
            m[0][0] * v.x + m[0][1] * v.y + m[0][2] * v.z,
            m[1][0] * v.x + m[1][1] * v.y + m[1][2] * v.z,
            m[2][0] * v.x + m[2][1] * v.y + m[2][2] * v.z,
        )
    }
    pub fn mul(&self, o: &M3) -> M3 {
        let mut r = [[0.0; 3]; 3];
        for (i, row) in r.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                *cell = (0..3).map(|k| self.0[i][k] * o.0[k][j]).sum();
            }
        }
        M3(r)
    }
    pub fn transpose(&self) -> M3 {
        let m = &self.0;
        M3([[m[0][0], m[1][0], m[2][0]], [m[0][1], m[1][1], m[2][1]], [m[0][2], m[1][2], m[2][2]]])
    }
}

/// xorshift64* pseudo random generator; deterministic across platforms.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        let mut s = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(0x6A09_E667_F3BC_C909);
        if s == 0 {
            s = 0x1234_5678;
        }
        let mut r = Rng(s);
        r.next_u32();
        r
    }
    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        (x.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 32) as u32
    }
    /// Uniform in [0, 1).
    #[inline]
    pub fn f32(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 / (1u32 << 24) as f32
    }
    /// Uniform integer in [lo, hi). Returns lo if the range is empty.
    pub fn range(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        lo + (self.next_u32() % (hi - lo) as u32) as i32
    }
    pub fn rangef(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.f32()
    }
    pub fn chance(&mut self, p: f32) -> bool {
        self.f32() < p
    }
    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.range(0, items.len() as i32) as usize]
    }
}

#[inline]
pub fn hash3(x: i32, y: i32, z: i32, seed: u32) -> u32 {
    let mut h = seed.wrapping_mul(0x27d4_eb2d);
    h ^= (x as u32).wrapping_mul(0x8da6_b343);
    h = h.rotate_left(13);
    h ^= (y as u32).wrapping_mul(0xd816_3841);
    h = h.rotate_left(11);
    h ^= (z as u32).wrapping_mul(0xcb1a_b31f);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297a_2d39);
    h ^= h >> 15;
    h
}

#[inline]
pub fn hashf(x: i32, y: i32, z: i32, seed: u32) -> f32 {
    (hash3(x, y, z, seed) >> 8) as f32 / (1u32 << 24) as f32
}

#[inline]
fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Periodic 2D value noise in [0,1]; `period` is in lattice cells.
pub fn vnoise2(x: f32, y: f32, period: i32, seed: u32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let fx = smooth(x - xi as f32);
    let fy = smooth(y - yi as f32);
    let p = period.max(1);
    let h = |a: i32, b: i32| hashf(a.rem_euclid(p), b.rem_euclid(p), 0, seed);
    let v00 = h(xi, yi);
    let v10 = h(xi + 1, yi);
    let v01 = h(xi, yi + 1);
    let v11 = h(xi + 1, yi + 1);
    let a = v00 + (v10 - v00) * fx;
    let b = v01 + (v11 - v01) * fx;
    a + (b - a) * fy
}

/// Tileable fractal noise over a texture of `size` pixels.
/// `cells` is the lattice period of the first octave.
pub fn fbm2(px: f32, py: f32, size: f32, cells: i32, octaves: i32, seed: u32) -> f32 {
    let mut sum = 0.0;
    let mut amp = 1.0;
    let mut norm = 0.0;
    let mut c = cells;
    for o in 0..octaves {
        let s = c as f32 / size;
        sum += vnoise2(px * s, py * s, c, seed.wrapping_add(o as u32 * 101)) * amp;
        norm += amp;
        amp *= 0.5;
        c *= 2;
    }
    sum / norm
}

/// Non-periodic 3D value noise in [0,1].
pub fn vnoise3(x: f32, y: f32, z: f32, seed: u32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let zi = z.floor() as i32;
    let fx = smooth(x - xi as f32);
    let fy = smooth(y - yi as f32);
    let fz = smooth(z - zi as f32);
    let h = |a, b, c| hashf(a, b, c, seed);
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let x00 = lerp(h(xi, yi, zi), h(xi + 1, yi, zi), fx);
    let x10 = lerp(h(xi, yi + 1, zi), h(xi + 1, yi + 1, zi), fx);
    let x01 = lerp(h(xi, yi, zi + 1), h(xi + 1, yi, zi + 1), fx);
    let x11 = lerp(h(xi, yi + 1, zi + 1), h(xi + 1, yi + 1, zi + 1), fx);
    lerp(lerp(x00, x10, fy), lerp(x01, x11, fy), fz)
}

pub fn fbm3(p: V3, octaves: i32, seed: u32) -> f32 {
    let mut sum = 0.0;
    let mut amp = 1.0;
    let mut norm = 0.0;
    let mut f = 1.0;
    for o in 0..octaves {
        sum += vnoise3(p.x * f, p.y * f, p.z * f, seed.wrapping_add(o as u32 * 57)) * amp;
        norm += amp;
        amp *= 0.5;
        f *= 2.0;
    }
    sum / norm
}

/// Result of a tileable Worley (cellular) lookup.
#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub f1: f32,
    pub f2: f32,
    pub id: u32,
    /// Offset from the sample to the nearest feature point (in cell units).
    pub dx: f32,
    pub dy: f32,
}

/// Tileable Worley noise. `cells` cells across `size` pixels; `jitter` 0..1.
/// `aspect` stretches cells horizontally (>1 makes wide stones).
pub fn worley2(px: f32, py: f32, size: f32, cells: i32, jitter: f32, aspect: f32, seed: u32) -> Cell {
    let cx_count = ((cells as f32) / aspect).round().max(1.0) as i32;
    let cy_count = cells.max(1);
    let x = px / size * cx_count as f32;
    let y = py / size * cy_count as f32;
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let mut f1 = f32::MAX;
    let mut f2 = f32::MAX;
    let mut id = 0u32;
    let (mut bdx, mut bdy) = (0.0, 0.0);
    for oy in -1..=1 {
        for ox in -1..=1 {
            let gx = xi + ox;
            let gy = yi + oy;
            let wx = gx.rem_euclid(cx_count);
            let wy = gy.rem_euclid(cy_count);
            let fx = gx as f32 + 0.5 + (hashf(wx, wy, 1, seed) - 0.5) * jitter;
            let fy = gy as f32 + 0.5 + (hashf(wx, wy, 2, seed) - 0.5) * jitter;
            let dx = (fx - x) * aspect;
            let dy = fy - y;
            let d = (dx * dx + dy * dy).sqrt();
            if d < f1 {
                f2 = f1;
                f1 = d;
                id = hash3(wx, wy, 3, seed);
                bdx = dx;
                bdy = dy;
            } else if d < f2 {
                f2 = d;
            }
        }
    }
    Cell { f1, f2, id, dx: bdx, dy: bdy }
}

/// 4x4 ordered-dither threshold in [0,1).
#[inline]
pub fn bayer4(x: i32, y: i32) -> f32 {
    const M: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];
    (M[(y & 3) as usize][(x & 3) as usize] as f32 + 0.5) / 16.0
}

/// Normalise an angle to [-PI, PI).
pub fn wrap_angle(a: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    let mut a = (a + std::f32::consts::PI).rem_euclid(tau);
    if a < 0.0 {
        a += tau;
    }
    a - std::f32::consts::PI
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rng_is_deterministic_and_in_range() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            let x = a.range(3, 9);
            assert_eq!(x, b.range(3, 9));
            assert!((3..9).contains(&x));
            let f = a.f32();
            b.f32();
            assert!((0.0..1.0).contains(&f));
        }
    }

    #[test]
    fn noise_tiles() {
        let size = 64.0;
        for i in 0..16 {
            let y = i as f32 * 3.7;
            let a = fbm2(0.0, y, size, 4, 3, 7);
            let b = fbm2(size, y, size, 4, 3, 7);
            assert!((a - b).abs() < 1e-4);
        }
        let c1 = worley2(0.3, 10.0, size, 4, 0.8, 1.0, 9);
        let c2 = worley2(size + 0.3, 10.0, size, 4, 0.8, 1.0, 9);
        assert!((c1.f1 - c2.f1).abs() < 1e-4);
        assert_eq!(c1.id, c2.id);
    }

    #[test]
    fn wrap_angle_range() {
        for k in -20..20 {
            let a = wrap_angle(k as f32 * 1.3);
            assert!((-std::f32::consts::PI..std::f32::consts::PI).contains(&a));
        }
    }
}
