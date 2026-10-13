//! A 2.5D "height canvas" for drawing small pixel-art objects procedurally.
//! Shapes are rasterised as height fields; the result is lit from the upper
//! left, quantised into palette ramps and outlined in black, giving the
//! hand-shaded look of the original's icons and object sprites.

use crate::gfx::palette::{c, Ramp, BLACK};
use crate::gfx::Bitmap;
use crate::util::{bayer4, hashf, V3};

#[derive(Clone, Copy, Debug)]
pub struct Mat {
    pub ramp: Ramp,
    pub lo: i32,
    pub hi: i32,
    /// Strength of per-pixel noise (0 = clean metal, 0.3 = rough).
    pub noise: f32,
    /// Specular highlight strength.
    pub spec: f32,
    /// Overall brightness bias.
    pub bias: f32,
}

impl Mat {
    pub const fn new(ramp: Ramp, lo: i32, hi: i32) -> Mat {
        Mat { ramp, lo, hi, noise: 0.0, spec: 0.0, bias: 0.0 }
    }
    pub const fn noise(mut self, n: f32) -> Mat {
        self.noise = n;
        self
    }
    pub const fn spec(mut self, s: f32) -> Mat {
        self.spec = s;
        self
    }
}

pub struct HCanvas {
    pub w: i32,
    pub h: i32,
    z: Vec<f32>,
    mat: Vec<u8>,
    part: Vec<u16>,
    next_part: u16,
    pub mats: Vec<Mat>,
    /// Pixels forced to a specific palette colour after shading (details).
    overlay: Vec<u8>,
}

const EMPTY: f32 = f32::NEG_INFINITY;

impl HCanvas {
    pub fn new(w: i32, h: i32) -> HCanvas {
        let n = (w * h) as usize;
        HCanvas { w, h, z: vec![EMPTY; n], mat: vec![0; n], part: vec![0; n], next_part: 1, mats: Vec::new(), overlay: vec![0; n] }
    }

    pub fn add_mat(&mut self, m: Mat) -> u8 {
        self.mats.push(m);
        (self.mats.len() - 1) as u8
    }

    fn new_part(&mut self) -> u16 {
        let p = self.next_part;
        self.next_part = self.next_part.wrapping_add(1).max(1);
        p
    }

    #[inline]
    fn plot(&mut self, x: i32, y: i32, z: f32, mat: u8, part: u16) {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            return;
        }
        let i = (y * self.w + x) as usize;
        if z > self.z[i] {
            self.z[i] = z;
            self.mat[i] = mat;
            self.part[i] = part;
        }
    }

    /// A tapered capsule (round cone) from a to b with radii ra, rb.
    /// `z0` is the base height; the profile is a half-cylinder of height r*zs.
    #[allow(clippy::too_many_arguments)]
    pub fn capsule(&mut self, a: (f32, f32), b: (f32, f32), ra: f32, rb: f32, z0: f32, zs: f32, mat: u8) {
        let part = self.new_part();
        let rmax = ra.max(rb);
        let x0 = (a.0.min(b.0) - rmax - 1.0).floor() as i32;
        let x1 = (a.0.max(b.0) + rmax + 1.0).ceil() as i32;
        let y0 = (a.1.min(b.1) - rmax - 1.0).floor() as i32;
        let y1 = (a.1.max(b.1) + rmax + 1.0).ceil() as i32;
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let l2 = (dx * dx + dy * dy).max(1e-6);
        for y in y0..=y1 {
            for x in x0..=x1 {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;
                let t = (((px - a.0) * dx + (py - a.1) * dy) / l2).clamp(0.0, 1.0);
                let cx = a.0 + dx * t;
                let cy = a.1 + dy * t;
                let r = ra + (rb - ra) * t;
                let d2 = (px - cx).powi(2) + (py - cy).powi(2);
                if d2 <= r * r {
                    let h = (r * r - d2).sqrt();
                    self.plot(x, y, z0 + h * zs, mat, part);
                }
            }
        }
    }

    /// A domed ellipse.
    #[allow(clippy::too_many_arguments)]
    pub fn ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, z0: f32, zs: f32, mat: u8) {
        let part = self.new_part();
        let x0 = (cx - rx - 1.0).floor() as i32;
        let x1 = (cx + rx + 1.0).ceil() as i32;
        let y0 = (cy - ry - 1.0).floor() as i32;
        let y1 = (cy + ry + 1.0).ceil() as i32;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let nx = (x as f32 + 0.5 - cx) / rx;
                let ny = (y as f32 + 0.5 - cy) / ry;
                let d2 = nx * nx + ny * ny;
                if d2 <= 1.0 {
                    let h = (1.0 - d2).sqrt() * rx.min(ry);
                    self.plot(x, y, z0 + h * zs, mat, part);
                }
            }
        }
    }

    /// A flat-topped polygon with bevelled edges of width `bevel`.
    pub fn poly(&mut self, pts: &[(f32, f32)], z0: f32, bevel: f32, mat: u8) {
        let part = self.new_part();
        let minx = pts.iter().map(|p| p.0).fold(f32::MAX, f32::min).floor() as i32 - 1;
        let maxx = pts.iter().map(|p| p.0).fold(f32::MIN, f32::max).ceil() as i32 + 1;
        let miny = pts.iter().map(|p| p.1).fold(f32::MAX, f32::min).floor() as i32 - 1;
        let maxy = pts.iter().map(|p| p.1).fold(f32::MIN, f32::max).ceil() as i32 + 1;
        for y in miny..=maxy {
            for x in minx..=maxx {
                let p = (x as f32 + 0.5, y as f32 + 0.5);
                if !point_in_poly(p, pts) {
                    continue;
                }
                let d = dist_to_edges(p, pts);
                let h = if bevel > 0.0 { (d / bevel).min(1.0) * bevel } else { 0.0 };
                self.plot(x, y, z0 + h, mat, part);
            }
        }
    }

    /// A ring (torus seen from above) - for rings, the options medallion etc.
    #[allow(clippy::too_many_arguments)]
    pub fn ring(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, thick: f32, z0: f32, mat: u8) {
        let part = self.new_part();
        let x0 = (cx - rx - thick - 1.0).floor() as i32;
        let x1 = (cx + rx + thick + 1.0).ceil() as i32;
        let y0 = (cy - ry - thick - 1.0).floor() as i32;
        let y1 = (cy + ry + thick + 1.0).ceil() as i32;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let px = x as f32 + 0.5 - cx;
                let py = y as f32 + 0.5 - cy;
                // Distance to the ellipse centre line, approximated in normalised space.
                let ang = py.atan2(px * ry / rx);
                let ex = rx * ang.cos();
                let ey = ry * ang.sin();
                let d = ((px - ex).powi(2) + (py - ey).powi(2)).sqrt();
                if d <= thick {
                    let h = (thick * thick - d * d).sqrt();
                    self.plot(x, y, z0 + h, mat, part);
                }
            }
        }
    }

    /// Render to an indexed bitmap. `light` points towards the light.
    pub fn render(&self, outline: bool, inner_lines: bool) -> Bitmap {
        let mut out = Bitmap::new(self.w, self.h);
        let light = V3::new(-0.55, -0.65, 0.55).norm();
        let zat = |x: i32, y: i32, fallback: f32| -> f32 {
            if x < 0 || y < 0 || x >= self.w || y >= self.h {
                return fallback;
            }
            let z = self.z[(y * self.w + x) as usize];
            if z == EMPTY { fallback } else { z }
        };
        for y in 0..self.h {
            for x in 0..self.w {
                let i = (y * self.w + x) as usize;
                let z = self.z[i];
                if z == EMPTY {
                    continue;
                }
                let m = self.mats[self.mat[i] as usize];
                // Normal from the height field (fall back to own height at edges,
                // with a drop so silhouettes curve away).
                let edge_drop = z - 1.5;
                let zl = zat(x - 1, y, edge_drop);
                let zr = zat(x + 1, y, edge_drop);
                let zu = zat(x, y - 1, edge_drop);
                let zd = zat(x, y + 1, edge_drop);
                let n = V3::new((zl - zr) * 0.5, (zu - zd) * 0.5, 1.0).norm();
                let diff = n.dot(light).max(0.0);
                let refl = (n * (2.0 * n.dot(light)) - light).z.max(0.0);
                let spec = refl.powf(12.0) * m.spec;
                let noise = (hashf(x, y, self.mat[i] as i32, 77) - 0.5) * m.noise;
                let v = 0.18 + diff * 0.8 + spec + noise + m.bias;
                let s = m.lo as f32 + v.clamp(0.0, 1.0) * (m.hi - m.lo) as f32 + (bayer4(x, y) - 0.5) * 0.6;
                out.set(x, y, c(m.ramp, (s.round() as i32).clamp(m.lo, m.hi)));
            }
        }
        if inner_lines {
            // Dark line where a part overlaps a lower part.
            let src = out.clone();
            for y in 0..self.h {
                for x in 0..self.w {
                    let i = (y * self.w + x) as usize;
                    if self.z[i] == EMPTY {
                        continue;
                    }
                    for (dx, dy) in [(1, 0), (0, 1), (-1, 0), (0, -1)] {
                        let (nx, ny) = (x + dx, y + dy);
                        if nx < 0 || ny < 0 || nx >= self.w || ny >= self.h {
                            continue;
                        }
                        let j = (ny * self.w + nx) as usize;
                        if self.z[j] != EMPTY && self.part[j] != self.part[i] && self.z[j] > self.z[i] + 1.2 {
                            out.set(x, y, BLACK);
                        }
                    }
                }
            }
            let _ = src;
        }
        for (i, &o) in self.overlay.iter().enumerate() {
            if o != 0 {
                out.px[i] = o;
            }
        }
        if outline { out.outlined(BLACK, false) } else { out }
    }
}

pub fn point_in_poly(p: (f32, f32), pts: &[(f32, f32)]) -> bool {
    let mut inside = false;
    let n = pts.len();
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = pts[i];
        let (xj, yj) = pts[j];
        if ((yi > p.1) != (yj > p.1)) && (p.0 < (xj - xi) * (p.1 - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn dist_to_edges(p: (f32, f32), pts: &[(f32, f32)]) -> f32 {
    let mut best = f32::MAX;
    let n = pts.len();
    for i in 0..n {
        let a = pts[i];
        let b = pts[(i + 1) % n];
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let l2 = (dx * dx + dy * dy).max(1e-6);
        let t = (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / l2).clamp(0.0, 1.0);
        let d = ((p.0 - a.0 - dx * t).powi(2) + (p.1 - a.1 - dy * t).powi(2)).sqrt();
        best = best.min(d);
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capsule_renders_with_outline() {
        let mut cv = HCanvas::new(16, 16);
        let m = cv.add_mat(Mat::new(Ramp::Silver, 3, 14));
        cv.capsule((4.0, 8.0), (12.0, 8.0), 2.5, 2.5, 0.0, 1.0, m);
        let b = cv.render(true, true);
        assert_eq!(b.get(8, 8) != 0, true);
        assert_eq!(b.get(8, 4), BLACK); // outline above the capsule
        assert_eq!(b.get(0, 0), 0);
    }
}
