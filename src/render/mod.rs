//! The 3D world renderer: transforms the level mesh, dynamic polygons and
//! sprites into the 3D view window using the software rasteriser.

pub mod mesh;

use crate::art::Art;
use crate::gfx::palette::MAX_LIGHT;
use crate::gfx::raster::{self, SVert, Shading, View, LUT_RES};
use crate::gfx::Bitmap;
use crate::util::V3;
use crate::world::map::MAP_SIZE;
use mesh::{LevelMesh, Poly};

/// The visible 3D window inside the frame, as measured from the original.
pub const VIEW_W: i32 = 172;
pub const VIEW_H: i32 = 109;

pub const NEAR: f32 = 0.04;

/// Picking id categories (top 4 bits).
pub const ID_FLOOR: u32 = 1 << 28;
pub const ID_CEIL: u32 = 2 << 28;
pub const ID_WALL: u32 = 3 << 28;
pub const ID_ITEM: u32 = 4 << 28;
pub const ID_NPC: u32 = 5 << 28;
pub const ID_DOOR: u32 = 6 << 28;
pub const ID_MASK: u32 = 0x0FFF_FFFF;

pub fn surf_id(cat: u32, x: i32, z: i32, tex: crate::art::textures::Tex) -> u32 {
    cat | (((z * crate::world::map::MAP_SIZE + x) as u32 & 0xFFF) << 8) | tex as u32
}
pub const VIS_RADIUS: i32 = 18;

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    /// Eye position.
    pub pos: V3,
    /// Heading in radians: 0 = north (-z), positive turns clockwise (east).
    pub yaw: f32,
    /// Look up/down as a horizon shift in pixels (positive = look up).
    pub pitch: f32,
    /// Screen shake offset in pixels.
    pub shake: (f32, f32),
}

impl Camera {
    #[inline]
    pub fn to_cam(&self, p: V3) -> V3 {
        let dx = p.x - self.pos.x;
        let dy = p.y - self.pos.y;
        let dz = p.z - self.pos.z;
        let (s, c) = self.yaw.sin_cos();
        V3::new(dx * c + dz * s, dy, dx * s - dz * c)
    }
    pub fn forward(&self) -> V3 {
        V3::new(self.yaw.sin(), 0.0, -self.yaw.cos())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LightParams {
    /// Distance at which the carried light fades to darkness.
    pub radius: f32,
    /// Brightness 0..1 of the carried light.
    pub level: f32,
    /// Light everywhere, 0..31.
    pub ambient: f32,
}

pub struct SpriteInst<'a> {
    pub bmp: &'a Bitmap,
    pub id: u32,
    /// Bottom-centre in world space.
    pub pos: V3,
    pub w: f32,
    pub h: f32,
    pub flip: bool,
    pub fullbright: bool,
    pub stipple: bool,
    pub light: f32,
}

#[derive(Clone, Copy, Default)]
struct CVert {
    c: V3,
    u: f32,
    v: f32,
    l: f32,
}

pub struct Renderer {
    pub view: View,
    pub fx: f32,
    pub fy: f32,
    lut: Vec<f32>,
    clip_a: Vec<CVert>,
    clip_b: Vec<CVert>,
    sverts: Vec<SVert>,
}

impl Renderer {
    /// `fov_deg` is the horizontal field of view; `pixel_aspect` is the
    /// display height/width ratio of one pixel (1.2 for 320x200 on a 4:3 CRT).
    pub fn new(fov_deg: f32, pixel_aspect: f32) -> Renderer {
        let fx = (VIEW_W as f32 / 2.0) / (fov_deg.to_radians() / 2.0).tan();
        Renderer {
            view: View::new(VIEW_W, VIEW_H),
            fx,
            fy: fx / pixel_aspect,
            lut: vec![0.0; (64.0 * LUT_RES) as usize],
            clip_a: Vec::with_capacity(8),
            clip_b: Vec::with_capacity(8),
            sverts: Vec::with_capacity(8),
        }
    }

    pub fn set_pixel_aspect(&mut self, pixel_aspect: f32) {
        self.fy = self.fx / pixel_aspect;
    }

    fn build_lut(&mut self, lp: &LightParams) {
        let full = 0.9f32;
        for (i, l) in self.lut.iter_mut().enumerate() {
            let z = i as f32 / LUT_RES;
            // Gradual torch-light falloff: full brightness close by, about half
            // at the light's nominal radius, darkness at 1.6x the radius.
            let span = (lp.radius * 1.6 - full).max(0.1);
            let f = if z <= full { 1.0 } else { (1.0 - (z - full) / span).clamp(0.0, 1.0) };
            let carried = MAX_LIGHT as f32 * lp.level * f.powf(0.8);
            *l = carried.max(lp.ambient).min(MAX_LIGHT as f32);
        }
    }

    fn tile_visible(&self, cam: &Camera, tx: i32, tz: i32) -> bool {
        let t = (VIEW_W as f32 / 2.0 + 4.0) / self.fx;
        let mut out_left = true;
        let mut out_right = true;
        let mut behind = true;
        for (ox, oz) in [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)] {
            let c = cam.to_cam(V3::new(tx as f32 + ox, cam.pos.y, tz as f32 + oz));
            if c.z > NEAR {
                behind = false;
            }
            if c.x >= -c.z * t - 0.05 {
                out_left = false;
            }
            if c.x <= c.z * t + 0.05 {
                out_right = false;
            }
        }
        !(behind || out_left || out_right)
    }

    fn shading<'a>(art: &'a Art, lut: &'a [f32], fullbright: bool) -> Shading<'a> {
        Shading { tables: &art.pal.shade, lut, fullbright, dither: true }
    }

    pub fn draw_poly(&mut self, art: &Art, cam: &Camera, poly: &Poly, frame: usize) {
        // Transform.
        self.clip_a.clear();
        for v in poly.verts() {
            self.clip_a.push(CVert { c: cam.to_cam(v.p), u: v.u, v: v.v, l: v.l });
        }
        // Near-plane clip (Sutherland-Hodgman).
        self.clip_b.clear();
        let n = self.clip_a.len();
        for i in 0..n {
            let a = self.clip_a[i];
            let b = self.clip_a[(i + 1) % n];
            let ain = a.c.z >= NEAR;
            let bin = b.c.z >= NEAR;
            if ain {
                self.clip_b.push(a);
            }
            if ain != bin {
                let t = (NEAR - a.c.z) / (b.c.z - a.c.z);
                self.clip_b.push(CVert {
                    c: a.c.lerp(b.c, t),
                    u: a.u + (b.u - a.u) * t,
                    v: a.v + (b.v - a.v) * t,
                    l: a.l + (b.l - a.l) * t,
                });
            }
        }
        if self.clip_b.len() < 3 {
            return;
        }
        let hw = VIEW_W as f32 / 2.0 + cam.shake.0;
        let hh = VIEW_H as f32 / 2.0 + cam.pitch + cam.shake.1;
        self.sverts.clear();
        for cv in &self.clip_b {
            let iz = 1.0 / cv.c.z;
            self.sverts.push(SVert {
                x: hw + cv.c.x * iz * self.fx,
                y: hh - cv.c.y * iz * self.fy,
                iz,
                uz: cv.u * iz,
                vz: cv.v * iz,
                l: cv.l,
            });
        }
        let tex = art.tex(poly.tex, frame);
        let sh = Self::shading(art, &self.lut, poly.fullbright);
        raster::draw_poly(&mut self.view, &self.sverts, tex, &sh, !poly.two_sided, poly.id);
    }

    pub fn draw_sprite(&mut self, art: &Art, cam: &Camera, s: &SpriteInst) {
        let c = cam.to_cam(s.pos);
        if c.z < NEAR * 2.0 {
            return;
        }
        let hw = VIEW_W as f32 / 2.0 + cam.shake.0;
        let hh = VIEW_H as f32 / 2.0 + cam.pitch + cam.shake.1;
        let iz = 1.0 / c.z;
        let sx = hw + c.x * iz * self.fx;
        let sy_bottom = hh - c.y * iz * self.fy;
        let w = s.w * iz * self.fx;
        let h = s.h * iz * self.fy;
        let sh = Self::shading(art, &self.lut, s.fullbright);
        raster::draw_sprite(&mut self.view, s.bmp, sx - w / 2.0, sy_bottom - h, w, h, c.z, &sh, s.light, s.flip, s.stipple, s.id);
    }

    /// Direction of the ray through a view pixel, in world space.
    pub fn pixel_ray(&self, cam: &Camera, px: f32, py: f32) -> V3 {
        let cx = (px - VIEW_W as f32 / 2.0) / self.fx;
        let cy = -(py - VIEW_H as f32 / 2.0 - cam.pitch) / self.fy;
        let (s, c) = cam.yaw.sin_cos();
        // inverse of to_cam rotation: world = right*cx + up*cy + forward*1
        V3::new(c * cx + s, cy, s * cx - c).norm()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        art: &Art,
        mesh: &LevelMesh,
        cam: &Camera,
        extra: &[Poly],
        sprites: &[SpriteInst],
        lp: &LightParams,
        frame: usize,
    ) {
        self.view.clear(0);
        self.build_lut(lp);
        let cx = cam.pos.x.floor() as i32;
        let cz = cam.pos.z.floor() as i32;
        let r = VIS_RADIUS;
        for tz in (cz - r).max(0)..=(cz + r).min(MAP_SIZE - 1) {
            for tx in (cx - r).max(0)..=(cx + r).min(MAP_SIZE - 1) {
                let polys = mesh.tile_polys(tx, tz);
                if polys.is_empty() || !self.tile_visible(cam, tx, tz) {
                    continue;
                }
                for p in polys {
                    self.draw_poly(art, cam, p, frame);
                }
            }
        }
        for p in extra {
            self.draw_poly(art, cam, p, frame);
        }
        for s in sprites {
            self.draw_sprite(art, cam, s);
        }
    }
}
