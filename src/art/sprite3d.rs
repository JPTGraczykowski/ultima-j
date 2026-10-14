//! A tiny ray-casting renderer for "clay" models made of ellipsoids, round
//! cones and boxes. Used to pre-render creature sprites from several view
//! angles, the paperdoll and conversation portraits, then quantised to the
//! palette with an outline - giving sprites in the spirit of the original.

use crate::gfx::palette::{c, Ramp, BLACK};
use crate::gfx::Bitmap;
use crate::util::{bayer4, fbm3, hashf, vnoise3, M3, V3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Pattern {
    Plain,
    Noise(f32),
    Chain,
    Scales,
    Fur,
    Cloth,
    /// Coarse woven cloth: a fine checker over soft vertical folds.
    Weave,
    /// Bark: deep grooves running along the trunk.
    Bark,
    Wood,
    Bone,
    Metal,
}

#[derive(Clone, Copy, Debug)]
pub struct CMat {
    pub ramp: Ramp,
    pub lo: i32,
    pub hi: i32,
    pub pattern: Pattern,
    /// Emissive materials ignore lighting (eyes, flames).
    pub glow: bool,
}

impl CMat {
    pub const fn new(ramp: Ramp, lo: i32, hi: i32, pattern: Pattern) -> CMat {
        CMat { ramp, lo, hi, pattern, glow: false }
    }
    pub const fn glow(mut self) -> CMat {
        self.glow = true;
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Prim {
    Ellipsoid { c: V3, r: V3, rot: M3 },
    Cone { a: V3, b: V3, ra: f32, rb: f32 },
    Box { c: V3, h: V3, rot: M3 },
    /// A flat, two-sided triangle (wing membranes).
    Tri { a: V3, b: V3, c: V3 },
}

#[derive(Clone, Copy, Debug)]
pub struct Part {
    pub prim: Prim,
    pub mat: CMat,
    /// Part of a face: drawn facial features may be painted over it.
    pub face: bool,
}

impl Part {
    pub fn face(mut self) -> Part {
        self.face = true;
        self
    }
}

pub fn ell(c: V3, r: V3, mat: CMat) -> Part {
    Part { prim: Prim::Ellipsoid { c, r, rot: M3::IDENTITY }, mat, face: false }
}
pub fn ell_rot(c: V3, r: V3, rot: M3, mat: CMat) -> Part {
    Part { prim: Prim::Ellipsoid { c, r, rot }, mat, face: false }
}
pub fn cone(a: V3, b: V3, ra: f32, rb: f32, mat: CMat) -> Part {
    Part { prim: Prim::Cone { a, b, ra, rb }, mat, face: false }
}
pub fn bx(c: V3, h: V3, rot: M3, mat: CMat) -> Part {
    Part { prim: Prim::Box { c, h, rot }, mat, face: false }
}
pub fn tri(a: V3, b: V3, c: V3, mat: CMat) -> Part {
    Part { prim: Prim::Tri { a, b, c }, mat, face: false }
}

struct Hit {
    t: f32,
    n: V3,
    p: V3,
}

fn hit_ellipsoid(o: V3, d: V3, c0: V3, r: V3, rot: &M3) -> Option<Hit> {
    let rt = rot.transpose();
    let lo = rt.mul_v(o - c0);
    let ld = rt.mul_v(d);
    let so = V3::new(lo.x / r.x, lo.y / r.y, lo.z / r.z);
    let sd = V3::new(ld.x / r.x, ld.y / r.y, ld.z / r.z);
    let a = sd.dot(sd);
    let b = so.dot(sd);
    let cc = so.dot(so) - 1.0;
    let disc = b * b - a * cc;
    if disc < 0.0 {
        return None;
    }
    let t = (-b - disc.sqrt()) / a;
    if t < 0.0 {
        return None;
    }
    let u = so + sd * t;
    let nl = V3::new(u.x / r.x, u.y / r.y, u.z / r.z);
    Some(Hit { t, n: rot.mul_v(nl).norm(), p: V3::new(u.x * r.x, u.y * r.y, u.z * r.z) })
}

/// Round cone intersection (after Inigo Quilez). `d` must be normalised.
fn hit_cone(o: V3, d: V3, pa: V3, pb: V3, ra: f32, rb: f32) -> Option<Hit> {
    let ba = pb - pa;
    let oa = o - pa;
    let ob = o - pb;
    let rr = ra - rb;
    let m0 = ba.dot(ba);
    let m1 = ba.dot(oa);
    let m2 = ba.dot(d);
    let m3 = d.dot(oa);
    let m5 = oa.dot(oa);
    let m6 = ob.dot(d);
    let m7 = ob.dot(ob);
    let d2 = m0 - rr * rr;
    let k2 = d2 - m2 * m2;
    let k1 = d2 * m3 - m1 * m2 + m2 * rr * ra;
    let k0 = d2 * m5 - m1 * m1 + m1 * rr * ra * 2.0 - m0 * ra * ra;
    let h = k1 * k1 - k0 * k2;
    if h < 0.0 {
        return None;
    }
    if k2.abs() > 1e-9 {
        let t = (-h.sqrt() - k1) / k2;
        let y = m1 - ra * rr + t * m2;
        if y > 0.0 && y < d2 && t > 0.0 {
            let n = ((oa + d * t) * d2 - ba * y).norm();
            let p = o + d * t;
            return Some(Hit { t, n, p: p - pa });
        }
    }
    let h1 = m3 * m3 - m5 + ra * ra;
    let h2 = m6 * m6 - m7 + rb * rb;
    if h1.max(h2) < 0.0 {
        return None;
    }
    let mut best: Option<Hit> = None;
    if h1 > 0.0 {
        let t = -m3 - h1.sqrt();
        if t > 0.0 {
            best = Some(Hit { t, n: (oa + d * t) * (1.0 / ra), p: o + d * t - pa });
        }
    }
    if h2 > 0.0 {
        let t = -m6 - h2.sqrt();
        if t > 0.0 && best.as_ref().map(|b| t < b.t).unwrap_or(true) {
            best = Some(Hit { t, n: (ob + d * t) * (1.0 / rb), p: o + d * t - pa });
        }
    }
    best.map(|mut b| {
        b.n = b.n.norm();
        b
    })
}

fn hit_box(o: V3, d: V3, c0: V3, h: V3, rot: &M3) -> Option<Hit> {
    let rt = rot.transpose();
    let lo = rt.mul_v(o - c0);
    let ld = rt.mul_v(d);
    let mut tmin = f32::MIN;
    let mut tmax = f32::MAX;
    let mut axis = 0;
    let mut sign = 1.0;
    let lov = [lo.x, lo.y, lo.z];
    let ldv = [ld.x, ld.y, ld.z];
    let hv = [h.x, h.y, h.z];
    for i in 0..3 {
        if ldv[i].abs() < 1e-9 {
            if lov[i].abs() > hv[i] {
                return None;
            }
            continue;
        }
        let t1 = (-hv[i] - lov[i]) / ldv[i];
        let t2 = (hv[i] - lov[i]) / ldv[i];
        let (tn, tf, s) = if t1 < t2 { (t1, t2, -1.0) } else { (t2, t1, 1.0) };
        if tn > tmin {
            tmin = tn;
            axis = i;
            sign = s;
        }
        tmax = tmax.min(tf);
    }
    if tmin > tmax || tmax < 0.0 || tmin < 0.0 {
        return None;
    }
    let mut nl = [0.0f32; 3];
    nl[axis] = sign;
    let p = lo + ld * tmin;
    Some(Hit { t: tmin, n: rot.mul_v(V3::new(nl[0], nl[1], nl[2])), p })
}

/// Ray-triangle intersection (Moller-Trumbore); the normal faces the ray.
fn hit_tri(o: V3, d: V3, a: V3, b: V3, c: V3) -> Option<Hit> {
    let e1 = b - a;
    let e2 = c - a;
    let p = d.cross(e2);
    let det = e1.dot(p);
    if det.abs() < 1e-9 {
        return None;
    }
    let inv = 1.0 / det;
    let s = o - a;
    let u = s.dot(p) * inv;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = s.cross(e1);
    let v = d.dot(q) * inv;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = e2.dot(q) * inv;
    if t < 0.0 {
        return None;
    }
    let n = e1.cross(e2).norm();
    let n = if n.dot(d) > 0.0 { n * -1.0 } else { n };
    Some(Hit { t, n, p: s + d * t })
}

fn bounds(p: &Prim) -> (V3, f32) {
    match *p {
        Prim::Ellipsoid { c, r, .. } => (c, r.x.max(r.y).max(r.z)),
        Prim::Cone { a, b, ra, rb } => ((a + b) * 0.5, (b - a).len() * 0.5 + ra.max(rb)),
        Prim::Box { c, h, .. } => (c, h.len()),
        Prim::Tri { a, b, c } => {
            let m = (a + b + c) * (1.0 / 3.0);
            (m, (a - m).len().max((b - m).len()).max((c - m).len()))
        }
    }
}

pub struct View3 {
    /// Rotation of the model around the vertical axis (0 = model faces the viewer).
    pub yaw: f32,
    /// Camera elevation in radians (positive looks down on the model).
    pub pitch: f32,
    /// Pixels per model unit.
    pub scale: f32,
    /// Image size.
    pub w: i32,
    pub h: i32,
    /// Model-space point that maps to the image's bottom centre (x, z) and baseline y.
    pub center: V3,
    /// Pixel row of `center` (from the top).
    pub center_row: f32,
}

/// Render model parts into a palette bitmap with an outline.
pub fn render(parts: &[Part], v: &View3, outline: bool) -> Bitmap {
    render_lined(parts, v, outline, false)
}

/// Like `render`; `mat_lines` also draws a dark line where two different
/// materials meet (arm against tunic, tunic against belt), as a pixel
/// artist would.
pub fn render_lined(parts: &[Part], v: &View3, outline: bool, mat_lines: bool) -> Bitmap {
    render_ids(parts, v, outline, mat_lines).0
}

/// Render, also returning which part each pixel shows (`u16::MAX` = none).
pub fn render_ids(parts: &[Part], v: &View3, outline: bool, mat_lines: bool) -> (Bitmap, Vec<u16>) {
    let w = v.w;
    let h = v.h;
    let mut img = Bitmap::new(w, h);
    let mut depth = vec![f32::MAX; (w * h) as usize];
    let mut pid = vec![u16::MAX; (w * h) as usize];
    // Camera basis: rotate the model by yaw, then tilt the camera by pitch.
    let model_rot = M3::rot_y(v.yaw);
    let tilt = M3::rot_x(v.pitch);
    let cam = tilt.mul(&model_rot); // model -> camera
    let inv = cam.transpose(); // camera -> model
    let dir_cam = V3::new(0.0, 0.0, -1.0); // looking into the screen
    let d = inv.mul_v(dir_cam).norm();
    let light_cam = V3::new(-0.5, 0.65, 0.6).norm();
    let light = inv.mul_v(light_cam);
    // Per-part screen bounds for culling.
    let cc = cam.mul_v(v.center);
    let cull: Vec<(i32, i32, i32, i32)> = parts
        .iter()
        .map(|p| {
            let (bc, br) = bounds(&p.prim);
            let s = cam.mul_v(bc) - cc;
            let sx = w as f32 / 2.0 + s.x * v.scale;
            let sy = v.center_row - s.y * v.scale;
            let r = br * v.scale + 2.0;
            ((sx - r) as i32, (sy - r) as i32, (sx + r) as i32 + 1, (sy + r) as i32 + 1)
        })
        .collect();
    let mut normals = vec![V3::ZERO; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let px = (x as f32 + 0.5 - w as f32 / 2.0) / v.scale;
            let py = (v.center_row - (y as f32 + 0.5)) / v.scale;
            let o_cam = V3::new(px, py, 50.0) + cc;
            let o = inv.mul_v(o_cam);
            let mut best: Option<(Hit, usize)> = None;
            for (i, part) in parts.iter().enumerate() {
                let (x0, y0, x1, y1) = cull[i];
                if x < x0 || x > x1 || y < y0 || y > y1 {
                    continue;
                }
                let hit = match part.prim {
                    Prim::Ellipsoid { c, r, rot } => hit_ellipsoid(o, d, c, r, &rot),
                    Prim::Cone { a, b, ra, rb } => hit_cone(o, d, a, b, ra, rb),
                    Prim::Box { c, h, rot } => hit_box(o, d, c, h, &rot),
                    Prim::Tri { a, b, c } => hit_tri(o, d, a, b, c),
                };
                if let Some(hh) = hit {
                    if best.as_ref().map(|b| hh.t < b.0.t).unwrap_or(true) {
                        best = Some((hh, i));
                    }
                }
            }
            if let Some((hit, i)) = best {
                let idx = (y * w + x) as usize;
                depth[idx] = hit.t;
                pid[idx] = i as u16;
                normals[idx] = hit.n;
                let m = parts[i].mat;
                let col = shade(&m, hit.n, light, inv.mul_v(V3::new(0.0, 0.0, 1.0)), hit.p, x, y, i);
                img.set(x, y, col);
            }
        }
    }
    // Internal edges: darken pixels that sit behind a nearer neighbouring part.
    let src = img.clone();
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            if pid[i] == u16::MAX {
                continue;
            }
            // Leave the silhouette to the outline pass: a line here would
            // stick out of it as a notch.
            let empty = |x: i32, y: i32| x < 0 || y < 0 || x >= w || y >= h || pid[(y * w + x) as usize] == u16::MAX;
            if empty(x - 1, y) || empty(x + 1, y) || empty(x, y - 1) || empty(x, y + 1) {
                continue;
            }
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let nx = x + dx;
                let ny = y + dy;
                if nx < 0 || ny < 0 || nx >= w || ny >= h {
                    continue;
                }
                let j = (ny * w + nx) as usize;
                let other_mat = |a: &CMat, b: &CMat| a.ramp != b.ramp || a.pattern != b.pattern || a.lo != b.lo;
                let seam = mat_lines && pid[j] != u16::MAX && depth[i] > depth[j] && other_mat(&parts[pid[i] as usize].mat, &parts[pid[j] as usize].mat);
                if pid[j] != u16::MAX && pid[j] != pid[i] && (depth[i] - depth[j] > 0.06 || seam) {
                    let m = parts[pid[i] as usize].mat;
                    if !m.glow {
                        img.set(x, y, c(m.ramp, m.lo.max(1) - 1).min(src.get(x, y)).max(BLACK));
                        img.set(x, y, BLACK);
                    }
                }
            }
        }
    }
    (if outline { img.outlined(BLACK, false) } else { img }, pid)
}

/// Where a model-space point lands in the image (fractional pixels).
pub fn project(v: &View3, p: V3) -> (f32, f32, V3) {
    let cam = M3::rot_x(v.pitch).mul(&M3::rot_y(v.yaw));
    let s = cam.mul_v(p) - cam.mul_v(v.center);
    (v.w as f32 / 2.0 + s.x * v.scale, v.center_row - s.y * v.scale, s)
}

/// A model-space direction in camera space (+z points at the viewer).
pub fn to_camera(v: &View3, d: V3) -> V3 {
    M3::rot_x(v.pitch).mul(&M3::rot_y(v.yaw)).mul_v(d)
}

#[allow(clippy::too_many_arguments)]
fn shade(m: &CMat, n: V3, light: V3, to_eye: V3, p: V3, x: i32, y: i32, part: usize) -> u8 {
    if m.glow {
        let facing = n.dot(to_eye).max(0.0).sqrt();
        let s = m.lo as f32 + facing * (m.hi - m.lo) as f32 + (bayer4(x, y) - 0.5) * 0.8;
        return c(m.ramp, (s.round() as i32).clamp(m.lo, m.hi));
    }
    let diff = n.dot(light).max(0.0);
    let facing = n.dot(to_eye).max(0.0);
    let mut v = 0.06 + diff * 0.86 + facing * 0.16;
    let seed = part as u32 * 31 + 7;
    match m.pattern {
        Pattern::Plain => {}
        Pattern::Noise(a) => v += (fbm3(p * 18.0, 2, seed) - 0.5) * a,
        Pattern::Chain => {
            let q = p * 55.0;
            let ring = ((q.x.floor() as i32 + q.y.floor() as i32) & 1) == 0;
            v += if ring { 0.12 } else { -0.18 };
        }
        Pattern::Scales => {
            let q = p * 40.0;
            let row = q.y.floor();
            let off = if (row as i32) & 1 == 0 { 0.0 } else { 0.5 };
            let fx = (q.x + off).fract();
            let fy = q.y.fract();
            let edge = (fx - 0.5).abs() * 2.0 + fy;
            v += if edge > 1.2 { -0.25 } else { (0.5 - fy) * 0.2 };
        }
        Pattern::Fur => {
            v += (vnoise3(p.x * 60.0, p.y * 14.0, p.z * 60.0, seed) - 0.5) * 0.45;
        }
        Pattern::Cloth => {
            v += ((p.x * 30.0 + vnoise3(p.x * 8.0, p.y * 8.0, p.z * 8.0, seed) * 3.0).sin()) * 0.08;
            v += (hashf(x, y, part as i32, 3) - 0.5) * 0.1;
        }
        Pattern::Weave => {
            v += if (x + y) & 1 == 0 { 0.07 } else { -0.07 };
            v += (vnoise3(p.x * 30.0, p.y * 5.0, p.z * 30.0, seed) - 0.5) * 0.3;
        }
        Pattern::Bark => {
            let a = p.x.atan2(p.z) * 9.0 + vnoise3(p.x * 9.0, p.y * 3.0, p.z * 9.0, seed) * 4.0;
            v += 0.1 - (1.0 - a.sin().abs()).powi(3) * 0.7;
        }
        Pattern::Wood => {
            v += ((p.y * 50.0 + vnoise3(p.x * 6.0, p.y * 6.0, p.z * 6.0, seed) * 6.0).sin()) * 0.12;
        }
        Pattern::Bone => {
            v += (vnoise3(p.x * 25.0, p.y * 25.0, p.z * 25.0, seed) - 0.5) * 0.2;
        }
        Pattern::Metal => {
            let refl = (n * (2.0 * n.dot(light)) - light).dot(to_eye).max(0.0);
            v += refl.powf(8.0) * 0.6 - 0.05;
        }
    }
    let s = m.lo as f32 + v.clamp(0.0, 1.0) * (m.hi - m.lo) as f32 + (bayer4(x, y) - 0.5) * 0.7;
    c(m.ramp, (s.round() as i32).clamp(m.lo, m.hi))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sphere_renders_round_and_outlined() {
        let m = CMat::new(Ramp::Grey, 2, 12, Pattern::Plain);
        let parts = vec![ell(V3::new(0.0, 0.5, 0.0), V3::new(0.3, 0.3, 0.3), m)];
        let v = View3 { yaw: 0.0, pitch: 0.0, scale: 40.0, w: 40, h: 40, center: V3::ZERO, center_row: 39.0 };
        let b = render(&parts, &v, true);
        // centre of the sphere is at y = 0.5 -> row 39 - 20 = 19
        assert_ne!(b.get(20, 19), 0);
        assert_eq!(b.get(2, 2), 0);
        let filled = b.px.iter().filter(|&&p| p != 0).count();
        // roughly pi * 12^2 + outline
        assert!(filled > 400 && filled < 600, "filled {}", filled);
    }

    #[test]
    fn cone_and_box_hit() {
        let m = CMat::new(Ramp::Grey, 2, 12, Pattern::Plain);
        let parts = vec![
            cone(V3::new(-0.3, 0.3, 0.0), V3::new(0.3, 0.3, 0.0), 0.1, 0.05, m),
            bx(V3::new(0.0, 0.7, 0.0), V3::new(0.1, 0.1, 0.1), M3::rot_y(0.5), m),
        ];
        let v = View3 { yaw: 0.3, pitch: 0.1, scale: 40.0, w: 48, h: 48, center: V3::ZERO, center_row: 46.0 };
        let b = render(&parts, &v, false);
        assert!(b.px.iter().filter(|&&p| p != 0).count() > 100);
    }
}
