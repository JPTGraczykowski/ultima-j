//! Software rasteriser for the 3D view: perspective-correct texture-mapped
//! convex polygons with a depth buffer, palette shade tables for lighting,
//! and depth-tested scaled sprites (billboards).

use super::bitmap::Bitmap;
use super::palette::MAX_LIGHT;
use crate::util::bayer4;

/// Colour + depth target for the 3D view.
pub struct View {
    pub w: i32,
    pub h: i32,
    pub color: Vec<u8>,
    /// 1/z per pixel; 0 = infinitely far.
    pub depth: Vec<f32>,
    /// Object id per pixel (for mouse picking); 0 = nothing.
    pub ids: Vec<u32>,
}

impl View {
    pub fn new(w: i32, h: i32) -> View {
        View { w, h, color: vec![0; (w * h) as usize], depth: vec![0.0; (w * h) as usize], ids: vec![0; (w * h) as usize] }
    }
    pub fn clear(&mut self, c: u8) {
        self.color.iter_mut().for_each(|p| *p = c);
        self.depth.iter_mut().for_each(|d| *d = 0.0);
        self.ids.iter_mut().for_each(|d| *d = 0);
    }
    pub fn to_bitmap(&self) -> Bitmap {
        Bitmap { w: self.w, h: self.h, px: self.color.clone() }
    }
}

/// Power-of-two indexed texture.
#[derive(Clone)]
pub struct Texture {
    pub w: i32,
    pub h: i32,
    pub px: Vec<u8>,
    wmask: i32,
    hmask: i32,
    pub has_holes: bool,
}

impl Texture {
    pub fn from_bitmap(b: &Bitmap) -> Texture {
        assert!(b.w.count_ones() == 1 && b.h.count_ones() == 1, "texture must be power of two");
        let has_holes = b.px.contains(&0);
        Texture { w: b.w, h: b.h, px: b.px.clone(), wmask: b.w - 1, hmask: b.h - 1, has_holes }
    }
    #[inline]
    pub fn sample(&self, u: f32, v: f32) -> u8 {
        // u, v in texture repeats; offset keeps truncation equal to floor.
        let x = ((u * self.w as f32 + 65536.0) as i32) & self.wmask;
        let y = ((v * self.h as f32 + 65536.0) as i32) & self.hmask;
        self.px[(y * self.w + x) as usize]
    }
}

/// A projected vertex. `iz` = 1/z, `uz` = u/z, `vz` = v/z, `l` = vertex light (0..MAX_LIGHT).
#[derive(Clone, Copy, Debug, Default)]
pub struct SVert {
    pub x: f32,
    pub y: f32,
    pub iz: f32,
    pub uz: f32,
    pub vz: f32,
    pub l: f32,
}

impl SVert {
    #[inline]
    fn lerp(&self, o: &SVert, t: f32) -> SVert {
        SVert {
            x: self.x + (o.x - self.x) * t,
            y: self.y + (o.y - self.y) * t,
            iz: self.iz + (o.iz - self.iz) * t,
            uz: self.uz + (o.uz - self.uz) * t,
            vz: self.vz + (o.vz - self.vz) * t,
            l: self.l + (o.l - self.l) * t,
        }
    }
}

/// Lighting context for a draw call.
pub struct Shading<'a> {
    pub tables: &'a [[u8; 256]],
    /// Light from the player's light source as a function of depth:
    /// lut[(z * LUT_RES) as usize], values 0..MAX_LIGHT.
    pub lut: &'a [f32],
    pub fullbright: bool,
    pub dither: bool,
}

pub const LUT_RES: f32 = 16.0;

impl Shading<'_> {
    #[inline]
    fn light_at(&self, z: f32, vertex_light: f32) -> f32 {
        let i = ((z * LUT_RES) as usize).min(self.lut.len() - 1);
        self.lut[i].max(vertex_light)
    }
    #[inline]
    fn level(&self, light: f32, x: i32, y: i32) -> usize {
        let l = if self.dither { light + bayer4(x, y) - 0.5 } else { light };
        (l.round().max(0.0) as usize).min(MAX_LIGHT)
    }
}

/// Draw a convex polygon (already near-clipped and projected).
/// Front faces are clockwise on screen (y down); back faces are skipped when `cull`.
pub fn draw_poly(view: &mut View, verts: &[SVert], tex: &Texture, sh: &Shading, cull: bool, id: u32) {
    let n = verts.len();
    if n < 3 {
        return;
    }
    if cull {
        let mut area = 0.0;
        for i in 0..n {
            let a = &verts[i];
            let b = &verts[(i + 1) % n];
            area += a.x * b.y - b.x * a.y;
        }
        if area <= 0.0 {
            return;
        }
    }
    let miny = verts.iter().map(|v| v.y).fold(f32::MAX, f32::min);
    let maxy = verts.iter().map(|v| v.y).fold(f32::MIN, f32::max);
    let y0 = ((miny - 0.5).ceil() as i32).max(0);
    let y1 = ((maxy - 0.5).ceil() as i32).min(view.h);
    if y0 >= y1 {
        return;
    }
    for y in y0..y1 {
        let yc = y as f32 + 0.5;
        let mut left: Option<SVert> = None;
        let mut right: Option<SVert> = None;
        for i in 0..n {
            let a = &verts[i];
            let b = &verts[(i + 1) % n];
            let (top, bot) = if a.y < b.y { (a, b) } else { (b, a) };
            if yc >= top.y && yc < bot.y {
                let t = (yc - top.y) / (bot.y - top.y);
                let p = top.lerp(bot, t);
                match left {
                    None => {
                        left = Some(p);
                        right = Some(p);
                    }
                    Some(l) => {
                        if p.x < l.x {
                            left = Some(p);
                        }
                        if p.x > right.unwrap().x {
                            right = Some(p);
                        }
                    }
                }
            }
        }
        let (Some(l), Some(r)) = (left, right) else { continue };
        let xs = ((l.x - 0.5).ceil() as i32).max(0);
        let xe = ((r.x - 0.5).ceil() as i32).min(view.w);
        if xs >= xe {
            continue;
        }
        let span = r.x - l.x;
        let inv = if span > 1e-6 { 1.0 / span } else { 0.0 };
        let diz = (r.iz - l.iz) * inv;
        let duz = (r.uz - l.uz) * inv;
        let dvz = (r.vz - l.vz) * inv;
        let dl = (r.l - l.l) * inv;
        let off = xs as f32 + 0.5 - l.x;
        let mut iz = l.iz + diz * off;
        let mut uz = l.uz + duz * off;
        let mut vz = l.vz + dvz * off;
        let mut lv = l.l + dl * off;
        let row = (y * view.w) as usize;
        for x in xs..xe {
            let idx = row + x as usize;
            if iz > view.depth[idx] && iz > 0.0 {
                let z = 1.0 / iz;
                let texel = tex.sample(uz * z, vz * z);
                if texel != 0 || !tex.has_holes {
                    let c = if sh.fullbright {
                        texel
                    } else {
                        let light = sh.light_at(z, lv);
                        sh.tables[sh.level(light, x, y)][texel as usize]
                    };
                    view.color[idx] = c;
                    view.depth[idx] = iz;
                    view.ids[idx] = id;
                }
            }
            iz += diz;
            uz += duz;
            vz += dvz;
            lv += dl;
        }
    }
}

/// Depth-tested, shaded, scaled sprite. (`sx`, `sy`) is the top-left on screen,
/// (`sw`, `sh_px`) the on-screen size, `z` the sprite's depth.
#[allow(clippy::too_many_arguments)]
pub fn draw_sprite(
    view: &mut View,
    bmp: &Bitmap,
    sx: f32,
    sy: f32,
    sw: f32,
    sh_px: f32,
    z: f32,
    sh: &Shading,
    vertex_light: f32,
    flip: bool,
    stipple: bool,
    id: u32,
) {
    if sw < 0.5 || sh_px < 0.5 || z <= 0.0 {
        return;
    }
    let iz = 1.0 / z;
    let x0 = ((sx - 0.5).ceil() as i32).max(0);
    let x1 = ((sx + sw - 0.5).ceil() as i32).min(view.w);
    let y0 = ((sy - 0.5).ceil() as i32).max(0);
    let y1 = ((sy + sh_px - 0.5).ceil() as i32).min(view.h);
    if x0 >= x1 || y0 >= y1 {
        return;
    }
    let light = if sh.fullbright { MAX_LIGHT as f32 } else { sh.light_at(z, vertex_light) };
    let su = bmp.w as f32 / sw;
    let sv = bmp.h as f32 / sh_px;
    for y in y0..y1 {
        let ty = (((y as f32 + 0.5 - sy) * sv) as i32).clamp(0, bmp.h - 1);
        let row = (y * view.w) as usize;
        for x in x0..x1 {
            if stipple && ((x + y) & 1) == 1 {
                continue;
            }
            let idx = row + x as usize;
            if iz <= view.depth[idx] {
                continue;
            }
            let mut tx = (((x as f32 + 0.5 - sx) * su) as i32).clamp(0, bmp.w - 1);
            if flip {
                tx = bmp.w - 1 - tx;
            }
            let texel = bmp.px[(ty * bmp.w + tx) as usize];
            if texel == 0 {
                continue;
            }
            let c = if sh.fullbright { texel } else { sh.tables[sh.level(light, x, y)][texel as usize] };
            view.color[idx] = c;
            view.depth[idx] = iz;
            view.ids[idx] = id;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_tex() -> Texture {
        Texture::from_bitmap(&Bitmap::filled(4, 4, 7))
    }

    fn tables() -> Vec<[u8; 256]> {
        let mut t = vec![[0u8; 256]; MAX_LIGHT + 1];
        for (l, tab) in t.iter_mut().enumerate() {
            for (i, v) in tab.iter_mut().enumerate() {
                *v = if l == MAX_LIGHT { i as u8 } else { 1 };
            }
        }
        t
    }

    fn v(x: f32, y: f32) -> SVert {
        SVert { x, y, iz: 1.0, uz: 0.0, vz: 0.0, l: 0.0 }
    }

    #[test]
    fn adjacent_quads_cover_without_gaps_or_overlap() {
        let mut view = View::new(32, 32);
        let tab = tables();
        let lut = vec![MAX_LIGHT as f32; 64];
        let sh = Shading { tables: &tab, lut: &lut, fullbright: false, dither: false };
        let tex = flat_tex();
        // Two triangles sharing a slanted edge, clockwise on screen.
        let a = [v(1.3, 1.1), v(30.2, 2.7), v(5.5, 29.9)];
        let b = [v(30.2, 2.7), v(29.1, 30.6), v(5.5, 29.9)];
        draw_poly(&mut view, &a, &tex, &sh, true, 1);
        let covered_a: Vec<usize> = (0..view.color.len()).filter(|&i| view.color[i] == 7).collect();
        let mut view2 = View::new(32, 32);
        draw_poly(&mut view2, &b, &tex, &sh, true, 1);
        for &i in &covered_a {
            assert_ne!(view2.color[i], 7, "pixel {} covered twice", i);
        }
        // The union must have no holes along the shared edge interior.
        let mut union = view.color.clone();
        for (i, c) in view2.color.iter().enumerate() {
            if *c == 7 {
                union[i] = 7;
            }
        }
        for y in 4..28 {
            let row: Vec<u8> = (0..32).map(|x| union[y * 32 + x]).collect();
            let first = row.iter().position(|&c| c == 7);
            let last = row.iter().rposition(|&c| c == 7);
            if let (Some(f), Some(l)) = (first, last) {
                assert!(row[f..=l].iter().all(|&c| c == 7), "gap in row {}", y);
            }
        }
    }

    #[test]
    fn backfaces_are_culled() {
        let mut view = View::new(16, 16);
        let tab = tables();
        let lut = vec![MAX_LIGHT as f32; 64];
        let sh = Shading { tables: &tab, lut: &lut, fullbright: false, dither: false };
        let ccw = [v(1.0, 1.0), v(1.0, 14.0), v(14.0, 1.0)];
        draw_poly(&mut view, &ccw, &flat_tex(), &sh, true, 1);
        assert!(view.color.iter().all(|&c| c == 0));
    }

    #[test]
    fn depth_test_keeps_nearest() {
        let mut view = View::new(8, 8);
        let tab = tables();
        let lut = vec![MAX_LIGHT as f32; 64];
        let sh = Shading { tables: &tab, lut: &lut, fullbright: false, dither: false };
        let near: Vec<SVert> = [v(0.0, 0.0), v(8.0, 0.0), v(8.0, 8.0), v(0.0, 8.0)]
            .iter()
            .map(|p| SVert { iz: 2.0, ..*p })
            .collect();
        let far: Vec<SVert> = near.iter().map(|p| SVert { iz: 0.5, ..*p }).collect();
        let t1 = Texture::from_bitmap(&Bitmap::filled(2, 2, 3));
        let t2 = Texture::from_bitmap(&Bitmap::filled(2, 2, 4));
        draw_poly(&mut view, &near, &t1, &sh, true, 1);
        draw_poly(&mut view, &far, &t2, &sh, true, 2);
        assert!(view.color.iter().all(|&c| c == 3));
    }
}
