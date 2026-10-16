//! Procedural creature models, posed per animation frame and rendered from
//! five view angles with the clay renderer.

use super::sprite3d::{bx, cone, ell, ell_rot, render, CMat, Part, Pattern, Prim, View3};
use crate::game::creatures::Race;
use crate::gfx::palette::Ramp;
use crate::gfx::Bitmap;
use crate::util::{M3, V3};
use std::f32::consts::PI;

/// Frames: 0 idle, 1-4 walk, 5-7 attack, 8 hit, 9-11 dying (11 = corpse).
pub const FRAMES: usize = 12;
pub const VIEWS: usize = 5;
/// Sprite pixels per world unit (gives the original's chunky look up close).
pub const SCALE: f32 = 70.0;
/// World units per model unit: models are built at a smaller scale and
/// enlarged so creatures are as big relative to the halls as in the original.
pub const WORLD_SCALE: f32 = 1.25;

pub const F_IDLE: usize = 0;
pub const F_WALK: usize = 1;
pub const F_ATTACK: usize = 5;
pub const F_HIT: usize = 8;
pub const F_DIE: usize = 9;
pub const F_CORPSE: usize = 11;

#[derive(Clone, Copy, Debug, Default)]
pub struct Pose {
    pub walk: f32,
    pub stride: f32,
    pub attack: f32,
    pub hit: f32,
    pub death: f32,
}

pub fn pose_for_frame(f: usize) -> Pose {
    match f {
        1..=4 => Pose { walk: (f - 1) as f32 * PI / 2.0, stride: 1.0, ..Default::default() },
        5..=7 => Pose { attack: (f - 4) as f32 / 3.0, ..Default::default() },
        8 => Pose { hit: 1.0, ..Default::default() },
        9..=11 => Pose { death: (f - 8) as f32 / 3.0, ..Default::default() },
        _ => Pose::default(),
    }
}

pub struct CreatureSprites {
    /// frames[frame][view]
    pub frames: Vec<Vec<Bitmap>>,
    /// World size of each bitmap.
    pub size: (f32, f32),
}

fn m(r: Ramp, lo: i32, hi: i32, p: Pattern) -> CMat {
    CMat::new(r, lo, hi, p)
}

#[derive(Clone, Copy, PartialEq)]
enum Weapon {
    None,
    Sword,
    Axe,
    Club,
    Staff,
    Claws,
}

#[derive(Clone)]
struct Humanoid {
    h: f32,
    girth: f32,
    head: f32,
    skin: CMat,
    torso: CMat,
    legs: CMat,
    feet: CMat,
    sleeves: Option<CMat>,
    hair: Option<CMat>,
    beard: Option<CMat>,
    helmet: Option<CMat>,
    crown: Option<CMat>,
    robe: Option<CMat>,
    weapon: Weapon,
    weapon_mat: CMat,
    shield: Option<CMat>,
    ears: f32,
    snout: f32,
    tail: Option<CMat>,
    horns: Option<CMat>,
    wings: Option<CMat>,
    headless: bool,
    bony: bool,
    hunch: f32,
    eyes: CMat,
    belt: Option<CMat>,
    nose: f32,
    arm_len: f32,
    muscular: f32,
    loincloth: Option<CMat>,
    fangs: bool,
    hump: bool,
}

impl Humanoid {
    fn base(h: f32, skin: CMat) -> Humanoid {
        Humanoid {
            h,
            girth: 1.0,
            head: 1.1,
            skin,
            torso: skin,
            legs: skin,
            feet: skin,
            sleeves: None,
            hair: None,
            beard: None,
            helmet: None,
            crown: None,
            robe: None,
            weapon: Weapon::None,
            weapon_mat: m(Ramp::Silver, 4, 14, Pattern::Metal),
            shield: None,
            ears: 0.0,
            snout: 0.0,
            tail: None,
            horns: None,
            wings: None,
            headless: false,
            bony: false,
            hunch: 0.0,
            eyes: CMat::new(Ramp::Grey, 0, 1, Pattern::Plain),
            belt: None,
            nose: 0.0,
            arm_len: 1.0,
            muscular: 0.0,
            loincloth: None,
            fangs: false,
            hump: false,
        }
    }
}

/// Direction for a limb pitched by `p` radians about the x axis (0 = down, PI/2 = forward).
fn limb_dir(p: f32, splay: f32) -> V3 {
    V3::new(splay, -p.cos(), p.sin()).norm()
}

fn lerp3(a: f32, b: f32, c: f32, t: f32) -> f32 {
    if t < 0.5 { a + (b - a) * (t * 2.0) } else { b + (c - b) * ((t - 0.5) * 2.0) }
}

fn transform(parts: &mut [Part], pivot: V3, rot: &M3, shift: V3) {
    for p in parts.iter_mut() {
        p.prim = match p.prim {
            Prim::Ellipsoid { c, r, rot: r0 } => Prim::Ellipsoid { c: rot.mul_v(c - pivot) + pivot + shift, r, rot: rot.mul(&r0) },
            Prim::Cone { a, b, ra, rb } => Prim::Cone { a: rot.mul_v(a - pivot) + pivot + shift, b: rot.mul_v(b - pivot) + pivot + shift, ra, rb },
            Prim::Box { c, h, rot: r0 } => Prim::Box { c: rot.mul_v(c - pivot) + pivot + shift, h, rot: rot.mul(&r0) },
        };
    }
}

fn humanoid(s: &Humanoid, pose: &Pose) -> Vec<Part> {
    let h = s.h;
    let g = s.girth;
    let mut parts: Vec<Part> = Vec::new();
    let stride = pose.stride;
    let bob = (pose.walk * 2.0).sin().abs() * 0.015 * h * stride;
    let hip_y = 0.47 * h + bob;
    let thigh = 0.25 * h;
    let shin = 0.23 * h;
    let hip_dx = 0.07 * h * g;
    let swing = pose.walk.sin() * 0.5 * stride;
    let limb_r = if s.bony { 0.018 * h } else { 0.045 * h * g };

    // Legs (or robe).
    if let Some(robe) = s.robe {
        let sway = swing * 0.06 * h;
        parts.push(cone(V3::new(0.0, hip_y + 0.05 * h, 0.0), V3::new(sway, 0.05 * h, 0.0), 0.12 * h * g, 0.19 * h * g, robe));
        for side in [-1.0f32, 1.0] {
            let fwd = (swing * side).max(0.0) * 0.12 * h;
            parts.push(ell(V3::new(side * 0.06 * h, 0.03 * h, 0.08 * h + fwd), V3::new(0.04 * h, 0.03 * h, 0.07 * h), s.feet));
        }
    } else {
        for side in [-1.0f32, 1.0] {
            let a = swing * side;
            let bend = (0.65 * stride * (pose.walk * side + PI * 0.5).sin().max(0.0)).max(if s.hunch > 0.0 { 0.2 } else { 0.05 });
            let hip = V3::new(side * hip_dx, hip_y, 0.0);
            let knee = hip + limb_dir(a, 0.0) * thigh;
            let ankle = knee + limb_dir(a - bend, 0.0) * shin;
            let r0 = if s.bony { limb_r * 1.3 } else { 0.06 * h * g };
            parts.push(cone(hip, knee, r0, limb_r * 1.1, s.legs));
            parts.push(cone(knee, ankle, limb_r * 1.1, limb_r * 0.85, s.legs));
            if s.muscular > 0.0 {
                let calf = knee + (ankle - knee) * 0.35 + V3::new(0.0, 0.0, -0.012 * h);
                parts.push(ell(calf, V3::new(limb_r * 1.25, shin * 0.25, limb_r * 1.3), s.legs));
            }
            parts.push(ell(ankle + V3::new(0.0, -0.015 * h, 0.04 * h), V3::new(0.04 * h, 0.03 * h, 0.075 * h), s.feet));
        }
    }

    // Torso.
    let torso_c = V3::new(0.0, hip_y + 0.17 * h, 0.0);
    let mut upper: Vec<Part> = Vec::new();
    if s.bony {
        upper.push(cone(V3::new(0.0, hip_y, 0.0), V3::new(0.0, hip_y + 0.3 * h, 0.0), 0.02 * h, 0.02 * h, s.torso));
        for k in 0..4 {
            let y = hip_y + 0.14 * h + k as f32 * 0.045 * h;
            upper.push(ell(V3::new(0.0, y, 0.01 * h), V3::new(0.11 * h - k as f32 * 0.008 * h, 0.014 * h, 0.07 * h), s.torso));
        }
        upper.push(ell(V3::new(0.0, hip_y + 0.02 * h, 0.0), V3::new(0.09 * h, 0.04 * h, 0.05 * h), s.torso));
    } else {
        // A V-shaped torso: broad chest over a narrower waist.
        upper.push(ell(torso_c + V3::new(0.0, 0.06 * h, 0.0), V3::new(0.15 * h * g, 0.13 * h, 0.085 * h * g), s.torso));
        upper.push(ell(torso_c + V3::new(0.0, -0.07 * h, 0.0), V3::new(0.115 * h * g, 0.11 * h, 0.075 * h * g), s.torso));
        upper.push(ell(V3::new(0.0, hip_y + 0.02 * h, 0.0), V3::new(0.12 * h * g, 0.07 * h, 0.08 * h * g), if s.robe.is_some() { s.torso } else { s.legs }));
        if let Some(b) = s.belt {
            upper.push(ell(V3::new(0.0, hip_y + 0.07 * h, 0.0), V3::new(0.135 * h * g, 0.02 * h, 0.095 * h * g), b));
        }
        if s.muscular > 0.0 {
            let mu = s.muscular;
            for side in [-1.0f32, 1.0] {
                // Pectorals.
                upper.push(ell(torso_c + V3::new(side * 0.068 * h * g, 0.075 * h, 0.06 * h * g), V3::new(0.075 * h * g, 0.055 * h, 0.035 * h * mu.max(0.5)), s.torso));
            }
            // Abdomen.
            upper.push(ell(torso_c + V3::new(0.0, -0.06 * h, 0.06 * h * g), V3::new(0.065 * h * g, 0.085 * h, 0.03 * h * mu.max(0.5)), s.torso));
        }
        if s.hump {
            upper.push(ell(V3::new(0.0, hip_y + 0.3 * h, -0.07 * h), V3::new(0.13 * h * g, 0.1 * h, 0.08 * h), s.torso));
        }
        if let Some(lc) = s.loincloth {
            upper.push(ell(V3::new(0.0, hip_y - 0.05 * h, 0.085 * h * g), V3::new(0.07 * h, 0.1 * h, 0.012 * h), lc));
            upper.push(ell(V3::new(0.0, hip_y - 0.04 * h, -0.085 * h * g), V3::new(0.08 * h, 0.09 * h, 0.012 * h), lc));
        }
    }
    // Head.
    let neck_y = hip_y + 0.36 * h;
    let hr = 0.075 * h * s.head;
    let head_c = V3::new(0.0, neck_y + hr * 0.95, 0.01 * h);
    if !s.headless {
        upper.push(cone(V3::new(0.0, neck_y - 0.03 * h, 0.0), head_c, 0.035 * h, 0.03 * h, s.skin));
        upper.push(ell(head_c, V3::new(hr * 0.9, hr, hr * 0.95), s.skin));
        if s.snout > 0.0 {
            upper.push(cone(head_c + V3::new(0.0, -0.01 * h, 0.03 * h), head_c + V3::new(0.0, -0.025 * h, hr + s.snout * h), hr * 0.6, hr * 0.35, s.skin));
        }
        if s.nose > 0.0 {
            upper.push(ell(head_c + V3::new(0.0, -0.01 * h, hr * 0.92), V3::new(0.02 * h * s.nose, 0.025 * h * s.nose, 0.025 * h * s.nose), s.skin));
        }
        if s.ears > 0.0 {
            for side in [-1.0f32, 1.0] {
                upper.push(ell_rot(
                    head_c + V3::new(side * hr * 1.1, 0.01 * h, -0.01 * h),
                    V3::new(0.05 * h * s.ears, 0.018 * h, 0.03 * h),
                    M3::rot_z(side * 0.35),
                    s.skin,
                ));
            }
        }
        if let Some(hair) = s.hair {
            upper.push(ell(head_c + V3::new(0.0, hr * 0.25, -hr * 0.2), V3::new(hr * 0.98, hr * 0.85, hr * 0.92), hair));
        }
        if let Some(beard) = s.beard {
            upper.push(ell(head_c + V3::new(0.0, -hr * 0.7, hr * 0.45), V3::new(hr * 0.75, hr * 0.85, hr * 0.5), beard));
        }
        if let Some(helm) = s.helmet {
            upper.push(ell(head_c + V3::new(0.0, hr * 0.3, -hr * 0.05), V3::new(hr * 1.05, hr * 0.8, hr * 1.02), helm));
        }
        if let Some(crown) = s.crown {
            // A band with a ring of spikes, tallest at the front.
            let top = head_c + V3::new(0.0, hr * 0.72, -hr * 0.05);
            upper.push(ell(top, V3::new(hr * 0.92, hr * 0.16, hr * 0.92), crown));
            for k in 0..8 {
                let a = k as f32 * PI / 4.0;
                let (sx, cz) = (a.sin(), a.cos());
                let tall = 0.35 + 0.25 * cz.max(0.0);
                let base = top + V3::new(sx * hr * 0.85, hr * 0.08, cz * hr * 0.85);
                upper.push(cone(base, base + V3::new(sx * hr * 0.08, hr * tall, cz * hr * 0.08), 0.014 * h, 0.003 * h, crown));
            }
        }
        if let Some(horn) = s.horns {
            for side in [-1.0f32, 1.0] {
                upper.push(cone(
                    head_c + V3::new(side * hr * 0.6, hr * 0.6, 0.0),
                    head_c + V3::new(side * hr * 1.5, hr * 1.8, -hr * 0.4),
                    0.025 * h,
                    0.006 * h,
                    horn,
                ));
            }
        }
        // Face: brow ridge, jaw, mouth and eyes set under the brow.
        if s.helmet.is_none() || s.snout == 0.0 {
            upper.push(ell(head_c + V3::new(0.0, hr * 0.36, hr * 0.62), V3::new(hr * 0.78, hr * 0.2, hr * 0.3), s.skin));
        }
        if s.snout == 0.0 {
            upper.push(ell(head_c + V3::new(0.0, -hr * 0.52, hr * 0.3), V3::new(hr * 0.66, hr * 0.42, hr * 0.6), s.skin));
            let mouth = CMat::new(Ramp::Red, 0, 3, Pattern::Plain);
            let mw = if s.fangs { 0.48 } else { 0.32 };
            upper.push(ell(head_c + V3::new(0.0, -hr * 0.5, hr * 0.84), V3::new(hr * mw, hr * if s.fangs { 0.12 } else { 0.06 }, hr * 0.1), mouth));
            if s.fangs {
                let tooth = CMat::new(Ramp::Parch, 9, 15, Pattern::Plain);
                for side in [-1.0f32, 1.0] {
                    upper.push(cone(head_c + V3::new(side * hr * 0.26, -hr * 0.42, hr * 0.9), head_c + V3::new(side * hr * 0.24, -hr * 0.62, hr * 0.92), hr * 0.07, hr * 0.02, tooth));
                }
            }
        }
        for side in [-1.0f32, 1.0] {
            let e = head_c + V3::new(side * hr * 0.36, hr * 0.13, hr * 0.84);
            upper.push(ell(e, V3::new(hr * 0.15, hr * 0.1, hr * 0.12), s.eyes));
        }
    } else {
        // Headless: hunched shoulder mound.
        upper.push(ell(V3::new(0.0, neck_y, 0.0), V3::new(0.13 * h * g, 0.06 * h, 0.08 * h), s.skin));
    }
    if let Some(w) = s.wings {
        for side in [-1.0f32, 1.0] {
            let flap = (pose.walk * 2.0).sin() * 0.3;
            upper.push(ell_rot(
                V3::new(side * 0.2 * h, hip_y + 0.3 * h, -0.08 * h),
                V3::new(0.2 * h, 0.13 * h, 0.012 * h),
                M3::rot_z(side * (0.5 + flap)).mul(&M3::rot_y(side * 0.4)),
                w,
            ));
        }
    }

    // Arms.
    let sh_y = hip_y + 0.33 * h;
    let sh_dx = 0.16 * h * g;
    let upper_len = 0.17 * h * s.arm_len;
    let fore_len = 0.16 * h * s.arm_len;
    let arm_mat = s.sleeves.unwrap_or(s.skin);
    let mut right_wrist = V3::ZERO;
    let mut right_fore_dir = V3::new(0.0, -1.0, 0.0);
    for side in [-1.0f32, 1.0] {
        // side -1 = the model's right arm (weapon arm).
        let base_swing = -swing * side * 0.8;
        let (pu, pf) = if side < 0.0 && pose.attack > 0.0 {
            let t = pose.attack;
            (lerp3(2.7, 1.5, 0.6, t), lerp3(3.4, 1.4, 0.4, t))
        } else if pose.hit > 0.0 {
            (0.6, 1.2)
        } else {
            let idle = if s.weapon != Weapon::None && side < 0.0 { 0.35 } else { 0.1 };
            (base_swing + idle + s.hunch * 0.6, base_swing + idle + 0.25 + s.hunch * 0.8)
        };
        let splay = side * 0.15;
        let sh = V3::new(side * sh_dx, sh_y, 0.0);
        let elbow = sh + limb_dir(pu, splay) * upper_len;
        let fdir = limb_dir(pf, splay * 0.5);
        let wrist = elbow + fdir * fore_len;
        let r0 = if s.bony { limb_r * 1.2 } else { 0.042 * h * g };
        upper.push(ell(sh, V3::new(0.046 * h * g, 0.044 * h, 0.046 * h * g), arm_mat));
        upper.push(cone(sh, elbow, r0, r0 * 0.85, arm_mat));
        if s.muscular > 0.0 {
            let mid = sh + (elbow - sh) * 0.45;
            upper.push(ell(mid + V3::new(0.0, 0.0, 0.012 * h), V3::new(r0 * 1.15, upper_len * 0.32, r0 * 1.2), arm_mat));
        }
        upper.push(cone(elbow, wrist, r0 * 0.85, r0 * 0.7, if s.bony { arm_mat } else { s.skin }));
        let hand_r = if s.weapon == Weapon::Claws { 0.045 * h } else { 0.032 * h };
        upper.push(ell(wrist + fdir * (0.02 * h), V3::new(hand_r, hand_r * 1.1, hand_r), s.skin));
        if side < 0.0 {
            right_wrist = wrist + fdir * (0.02 * h);
            right_fore_dir = fdir;
        } else if let Some(sm) = s.shield {
            let c = wrist + V3::new(0.03 * h, 0.02 * h, 0.04 * h);
            upper.push(ell_rot(c, V3::new(0.12 * h, 0.14 * h, 0.02 * h), M3::rot_y(0.25), sm));
        }
    }
    // Weapon in the right hand.
    if s.weapon != Weapon::None && s.weapon != Weapon::Claws {
        // Blade points roughly along the forearm rotated forward/up.
        let fwd = V3::new(0.0, 0.0, 1.0);
        let up = V3::new(0.0, 1.0, 0.0);
        let dir = (right_fore_dir * 0.45 + fwd * 0.55 + up * 0.55).norm();
        let w = s.weapon_mat;
        match s.weapon {
            Weapon::Sword => {
                let tip = right_wrist + dir * (0.4 * h);
                upper.push(cone(right_wrist + dir * (0.04 * h), tip, 0.022 * h, 0.008 * h, w));
                upper.push(cone(right_wrist + dir * 0.035 * h + V3::new(-0.06 * h, 0.0, 0.0), right_wrist + dir * 0.035 * h + V3::new(0.06 * h, 0.0, 0.0), 0.012 * h, 0.012 * h, m(Ramp::Gold, 3, 11, Pattern::Metal)));
            }
            Weapon::Axe => {
                let top = right_wrist + dir * (0.32 * h);
                upper.push(cone(right_wrist - dir * 0.03 * h, top, 0.014 * h, 0.012 * h, m(Ramp::Earth, 2, 8, Pattern::Wood)));
                let side = V3::new(-1.0, 0.0, 0.0);
                upper.push(ell_rot(top - dir * 0.04 * h + side * 0.06 * h, V3::new(0.07 * h, 0.06 * h, 0.012 * h), M3::rot_y(PI / 2.0), w));
            }
            Weapon::Club => {
                let top = right_wrist + dir * (0.3 * h);
                upper.push(cone(right_wrist - dir * 0.02 * h, top, 0.02 * h, 0.04 * h, m(Ramp::Earth, 2, 8, Pattern::Wood)));
            }
            Weapon::Staff => {
                upper.push(cone(right_wrist - dir * 0.35 * h, right_wrist + dir * 0.45 * h, 0.016 * h, 0.016 * h, m(Ramp::Earth, 2, 9, Pattern::Wood)));
                upper.push(ell(right_wrist + dir * 0.47 * h, V3::new(0.03 * h, 0.03 * h, 0.03 * h), m(Ramp::Blue, 8, 14, Pattern::Plain).glow()));
            }
            _ => {}
        }
    }
    // Tail.
    if let Some(t) = s.tail {
        let wag = (pose.walk).sin() * 0.06 * h;
        parts.push(cone(V3::new(0.0, hip_y, -0.06 * h), V3::new(wag, hip_y * 0.35, -0.3 * h), 0.05 * h, 0.03 * h, t));
        parts.push(cone(V3::new(wag, hip_y * 0.35, -0.3 * h), V3::new(-wag, 0.03 * h, -0.5 * h), 0.03 * h, 0.01 * h, t));
    }

    // Upper body leans: hunch forward, recoil when hit.
    let lean = s.hunch * 0.35 - pose.hit * 0.3 + if pose.attack > 0.0 { (pose.attack * PI).sin() * 0.12 } else { 0.0 };
    let pivot = V3::new(0.0, hip_y, 0.0);
    transform(&mut upper, pivot, &M3::rot_x(lean), V3::ZERO);
    parts.extend(upper);

    // Death: topple sideways onto the ground.
    if pose.death > 0.0 {
        let t = pose.death.min(1.0);
        let ang = t * t * PI * 0.5;
        transform(&mut parts, V3::new(0.0, 0.0, 0.0), &M3::rot_z(ang), V3::new(0.0, -t * 0.03 * h, 0.0));
        // Make sure nothing sinks far below the floor.
        let lift = lowest_point(&parts).min(0.0);
        if lift < 0.0 {
            transform(&mut parts, V3::ZERO, &M3::IDENTITY, V3::new(0.0, -lift * 0.9, 0.0));
        }
    }
    parts
}

fn lowest_point(parts: &[Part]) -> f32 {
    parts
        .iter()
        .map(|p| match p.prim {
            Prim::Ellipsoid { c, r, .. } => c.y - r.x.min(r.y).min(r.z),
            Prim::Cone { a, b, ra, rb } => (a.y - ra).min(b.y - rb),
            Prim::Box { c, h, .. } => c.y - h.y,
        })
        .fold(f32::MAX, f32::min)
}

fn rat(pose: &Pose) -> Vec<Part> {
    let fur = m(Ramp::Earth, 1, 8, Pattern::Fur);
    let pink = m(Ramp::Flesh, 4, 11, Pattern::Plain);
    let eye = m(Ramp::Red, 10, 13, Pattern::Plain).glow();
    let mut p = Vec::new();
    let bob = (pose.walk * 2.0).sin().abs() * 0.01;
    let y = 0.13 + bob;
    let lunge = if pose.attack > 0.0 { (pose.attack * PI).sin() * 0.08 } else { 0.0 };
    p.push(ell(V3::new(0.0, y, -0.04 + lunge * 0.5), V3::new(0.1, 0.09, 0.17), fur));
    let head = V3::new(0.0, y + 0.02, 0.15 + lunge);
    p.push(ell(head, V3::new(0.065, 0.06, 0.08), fur));
    p.push(cone(head, head + V3::new(0.0, -0.02, 0.1), 0.05, 0.018, fur));
    p.push(ell(head + V3::new(0.0, -0.02, 0.11), V3::new(0.014, 0.014, 0.014), pink));
    for s in [-1.0f32, 1.0] {
        p.push(ell(head + V3::new(s * 0.045, 0.05, -0.02), V3::new(0.025, 0.03, 0.01), pink));
        p.push(ell(head + V3::new(s * 0.03, 0.02, 0.05), V3::new(0.012, 0.012, 0.012), eye));
    }
    for (i, (lx, lz)) in [(-1.0f32, 0.08f32), (1.0, 0.08), (-1.0, -0.13), (1.0, -0.13)].iter().enumerate() {
        let ph = pose.walk + if i % 3 == 0 { 0.0 } else { PI };
        let sw = ph.sin() * 0.05 * pose.stride;
        let top = V3::new(lx * 0.07, y - 0.03, *lz);
        p.push(cone(top, V3::new(lx * 0.08, 0.01, lz + sw), 0.025, 0.012, fur));
    }
    let wag = pose.walk.sin() * 0.05;
    p.push(cone(V3::new(0.0, y - 0.02, -0.2), V3::new(wag, 0.03, -0.42), 0.018, 0.005, pink));
    finish_death(p, pose, 0.0)
}

fn bat(pose: &Pose) -> Vec<Part> {
    let fur = m(Ramp::Earth, 1, 6, Pattern::Fur);
    let membrane = m(Ramp::Red, 1, 4, Pattern::Noise(0.25));
    let bone = m(Ramp::Earth, 2, 7, Pattern::Plain);
    let eye = m(Ramp::Red, 10, 13, Pattern::Plain).glow();
    let mut p = Vec::new();
    let y = 0.16;
    p.push(ell(V3::new(0.0, y, 0.0), V3::new(0.045, 0.065, 0.04), fur));
    let head = V3::new(0.0, y + 0.065, 0.012);
    p.push(ell(head, V3::new(0.034, 0.03, 0.032), fur));
    p.push(ell(head + V3::new(0.0, -0.012, 0.028), V3::new(0.014, 0.01, 0.012), bone));
    for s in [-1.0f32, 1.0] {
        // Big pointed ears and glowing eyes.
        p.push(cone(head + V3::new(s * 0.018, 0.018, 0.0), head + V3::new(s * 0.04, 0.072, -0.005), 0.016, 0.003, fur));
        p.push(ell(head + V3::new(s * 0.015, 0.006, 0.029), V3::new(0.008, 0.007, 0.007), eye));
        // Wing: arm bone, three fingers and the membrane between them.
        let flap = (pose.walk * 2.0 + if pose.attack > 0.0 { pose.attack * 6.0 } else { 0.0 }).sin() * 0.65;
        let sh = V3::new(s * 0.035, y + 0.03, 0.0);
        let wrist = sh + V3::new(s * 0.1, 0.05, -0.01);
        let tips = [V3::new(s * 0.31, y + 0.1, -0.02), V3::new(s * 0.3, y - 0.01, -0.02), V3::new(s * 0.2, y - 0.08, -0.015)];
        let mut w = vec![cone(sh, wrist, 0.011, 0.008, bone)];
        for t in tips {
            w.push(cone(wrist, t, 0.006, 0.003, bone));
        }
        w.push(ell(sh + V3::new(s * 0.12, -0.01, -0.012), V3::new(0.12, 0.06, 0.005), membrane));
        w.push(ell(V3::new(s * 0.25, y + 0.035, -0.018), V3::new(0.07, 0.055, 0.005), membrane));
        w.push(ell(V3::new(s * 0.2, y - 0.035, -0.016), V3::new(0.07, 0.04, 0.005), membrane));
        transform(&mut w, sh, &M3::rot_z(s * flap), V3::ZERO);
        p.extend(w);
    }
    finish_death(p, pose, 0.0)
}

/// A box (slab) running from a to b, for blocky creatures like golems.
fn slab(a: V3, b: V3, hw: f32, hd: f32, mat: CMat) -> Part {
    let y = (b - a).norm();
    let up = if y.y.abs() > 0.9 { V3::new(1.0, 0.0, 0.0) } else { V3::new(0.0, 1.0, 0.0) };
    let x = V3::new(up.y * y.z - up.z * y.y, up.z * y.x - up.x * y.z, up.x * y.y - up.y * y.x).norm();
    let z = V3::new(x.y * y.z - x.z * y.y, x.z * y.x - x.x * y.z, x.x * y.y - x.y * y.x).norm();
    let rot = M3([[x.x, y.x, z.x], [x.y, y.y, z.y], [x.z, y.z, z.z]]);
    bx((a + b) * 0.5, V3::new(hw, (b - a).len() * 0.5, hd), rot, mat)
}

/// The earth golem: a hulking figure of rough stone slabs.
fn golem(pose: &Pose) -> Vec<Part> {
    let rock = m(Ramp::Stone, 2, 9, Pattern::Noise(0.5));
    let dark = m(Ramp::Stone, 1, 6, Pattern::Noise(0.45));
    let eye = m(Ramp::Fire, 9, 13, Pattern::Plain).glow();
    let h = 1.1;
    let mut p = Vec::new();
    let swing = pose.walk.sin() * 0.35 * pose.stride;
    let hip_y = 0.42 * h + (pose.walk * 2.0).sin().abs() * 0.01 * pose.stride;
    for side in [-1.0f32, 1.0] {
        let a = swing * side;
        let hip = V3::new(side * 0.1, hip_y, 0.0);
        let knee = hip + limb_dir(a, 0.0) * 0.22;
        let ankle = knee + limb_dir(a - 0.1, 0.0) * 0.2;
        p.push(slab(hip, knee, 0.075, 0.08, rock));
        p.push(slab(knee, ankle, 0.065, 0.07, dark));
        p.push(bx(ankle + V3::new(0.0, -0.01, 0.03), V3::new(0.08, 0.035, 0.1), M3::rot_y(side * 0.1), rock));
    }
    let mut upper = Vec::new();
    upper.push(bx(V3::new(0.0, hip_y + 0.05, 0.0), V3::new(0.17, 0.07, 0.1), M3::IDENTITY, dark));
    upper.push(bx(V3::new(0.0, hip_y + 0.22, 0.0), V3::new(0.22, 0.13, 0.13), M3::rot_z(0.04), rock));
    upper.push(bx(V3::new(0.0, hip_y + 0.36, 0.01), V3::new(0.26, 0.07, 0.14), M3::rot_z(-0.05), rock));
    upper.push(bx(V3::new(0.0, hip_y + 0.48, 0.05), V3::new(0.075, 0.07, 0.07), M3::rot_y(0.15), dark));
    for side in [-1.0f32, 1.0] {
        upper.push(ell(V3::new(side * 0.03, hip_y + 0.49, 0.12), V3::new(0.018, 0.012, 0.01), eye));
        let sh = V3::new(side * 0.3, hip_y + 0.38, 0.0);
        let (pu, pf) = if pose.attack > 0.0 {
            let t = pose.attack;
            (lerp3(2.6, 1.4, 0.5, t), lerp3(2.9, 1.2, 0.3, t))
        } else if pose.hit > 0.0 {
            (0.5, 0.9)
        } else {
            (-swing * side * 0.6 + 0.1, -swing * side * 0.6 + 0.35)
        };
        let elbow = sh + limb_dir(pu, side * 0.2) * 0.23;
        let wrist = elbow + limb_dir(pf, side * 0.1) * 0.22;
        upper.push(slab(sh, elbow, 0.07, 0.075, rock));
        upper.push(slab(elbow, wrist, 0.075, 0.08, dark));
        upper.push(bx(wrist + limb_dir(pf, 0.0) * 0.05, V3::new(0.08, 0.07, 0.08), M3::rot_y(0.3), rock));
    }
    let lean = -pose.hit * 0.25 + if pose.attack > 0.0 { (pose.attack * PI).sin() * 0.15 } else { 0.05 };
    transform(&mut upper, V3::new(0.0, hip_y, 0.0), &M3::rot_x(lean), V3::ZERO);
    p.extend(upper);
    finish_death(p, pose, h)
}

fn spider(pose: &Pose) -> Vec<Part> {
    let body = m(Ramp::Grey, 3, 12, Pattern::Fur);
    let leg = m(Ramp::Grey, 2, 10, Pattern::Plain);
    let eye = m(Ramp::Red, 10, 14, Pattern::Plain).glow();
    let mut p = Vec::new();
    let y = 0.17 + (pose.walk * 2.0).sin().abs() * 0.01;
    let lunge = if pose.attack > 0.0 { (pose.attack * PI).sin() * 0.06 } else { 0.0 };
    p.push(ell(V3::new(0.0, y + 0.04, -0.16), V3::new(0.15, 0.12, 0.17), body));
    p.push(ell(V3::new(0.0, y + 0.01, 0.06 + lunge), V3::new(0.1, 0.08, 0.1), body));
    let head = V3::new(0.0, y + 0.02, 0.15 + lunge);
    p.push(ell(head, V3::new(0.06, 0.05, 0.05), body));
    for s in [-1.0f32, 1.0] {
        p.push(ell(head + V3::new(s * 0.022, 0.025, 0.035), V3::new(0.014, 0.014, 0.012), eye));
        p.push(cone(head + V3::new(s * 0.02, -0.03, 0.03), head + V3::new(s * 0.012, -0.07, 0.06), 0.012, 0.005, leg));
        for k in 0..4 {
            let ang = (k as f32 - 1.5) * 0.45;
            let ph = pose.walk + k as f32 * PI / 2.0 + if s > 0.0 { PI } else { 0.0 };
            let sw = ph.sin() * 0.1 * pose.stride;
            let root = V3::new(s * 0.07, y + 0.01, 0.06 + (1.5 - k as f32) * 0.035);
            let knee = root + V3::new(s * 0.2 * ang.cos(), 0.13, 0.2 * ang.sin() + sw);
            let foot = knee + V3::new(s * 0.13 * ang.cos(), -0.3, 0.12 * ang.sin() + sw * 0.5);
            p.push(cone(root, knee, 0.022, 0.016, leg));
            p.push(cone(knee, foot, 0.016, 0.006, leg));
        }
    }
    finish_death(p, pose, 0.0)
}

fn slug(pose: &Pose) -> Vec<Part> {
    let skin = m(Ramp::Moss, 3, 12, Pattern::Noise(0.3));
    let belly = m(Ramp::Green, 4, 11, Pattern::Plain);
    let eye = m(Ramp::Gold, 11, 14, Pattern::Plain).glow();
    let mut p = Vec::new();
    for k in 0..6 {
        let t = k as f32 / 5.0;
        let z = 0.2 - t * 0.5;
        let wave = (pose.walk + t * 4.0).sin() * 0.03 * (1.0 + pose.stride);
        let r = 0.11 * (1.0 - t * 0.6);
        p.push(ell(V3::new(wave, r * 0.85, z), V3::new(r, r * 0.85, r * 1.3), if k % 2 == 0 { skin } else { belly }));
    }
    let head = V3::new(0.0, 0.1, 0.25 + if pose.attack > 0.0 { (pose.attack * PI).sin() * 0.06 } else { 0.0 });
    p.push(ell(head, V3::new(0.09, 0.08, 0.08), skin));
    for s in [-1.0f32, 1.0] {
        let top = head + V3::new(s * 0.04, 0.14, 0.03);
        p.push(cone(head + V3::new(s * 0.03, 0.05, 0.02), top, 0.012, 0.008, skin));
        p.push(ell(top, V3::new(0.018, 0.018, 0.018), eye));
    }
    finish_death(p, pose, 0.0)
}

fn lurker(pose: &Pose) -> Vec<Part> {
    let skin = m(Ramp::Slate, 2, 10, Pattern::Scales);
    let maw = m(Ramp::Red, 2, 8, Pattern::Plain);
    let eye = m(Ramp::Gold, 11, 14, Pattern::Plain).glow();
    let mut p = Vec::new();
    let rise = 0.05 + if pose.attack > 0.0 { (pose.attack * PI).sin() * 0.1 } else { (pose.walk).sin() * 0.02 };
    let head = V3::new(0.0, 0.18 + rise, 0.0);
    p.push(ell(head, V3::new(0.24, 0.2, 0.2), skin));
    p.push(ell(head + V3::new(0.0, -0.06, 0.16), V3::new(0.16, 0.06, 0.06), maw));
    for s in [-1.0f32, 1.0] {
        p.push(ell(head + V3::new(s * 0.1, 0.1, 0.15), V3::new(0.035, 0.035, 0.03), eye));
        for k in 0..2 {
            let a = pose.walk + k as f32;
            let base = head + V3::new(s * 0.2, -0.05, 0.05 - k as f32 * 0.1);
            p.push(cone(base, base + V3::new(s * (0.18 + a.sin() * 0.05), 0.15 + a.cos() * 0.05, 0.05), 0.04, 0.012, skin));
        }
    }
    finish_death(p, pose, 0.0)
}

fn reaper(pose: &Pose) -> Vec<Part> {
    let bark = m(Ramp::Earth, 1, 7, Pattern::Wood);
    let leaf = m(Ramp::Moss, 2, 8, Pattern::Noise(0.4));
    let eye = m(Ramp::Gold, 11, 15, Pattern::Plain).glow();
    let mut p = Vec::new();
    let sway = (pose.walk).sin() * 0.04;
    p.push(cone(V3::new(0.0, 0.0, 0.0), V3::new(sway, 0.85, 0.0), 0.16, 0.11, bark));
    for (k, s) in [-1.0f32, 1.0].iter().enumerate() {
        p.push(cone(V3::new(s * 0.1, 0.02, 0.0), V3::new(s * 0.25, 0.0, 0.08), 0.06, 0.02, bark));
        let raise = if pose.attack > 0.0 && k == 0 { lerp3(0.6, 0.0, -0.4, pose.attack) } else { 0.2 + (pose.walk + k as f32).sin() * 0.1 };
        let sh = V3::new(s * 0.1, 0.72, 0.0);
        let el = sh + V3::new(s * 0.22, 0.12 + raise * 0.2, 0.1);
        let tip = el + V3::new(s * 0.08, 0.22 * (1.0 - raise), 0.18 + raise * 0.2);
        p.push(cone(sh, el, 0.05, 0.035, bark));
        p.push(cone(el, tip, 0.035, 0.01, bark));
    }
    p.push(ell(V3::new(sway, 0.95, -0.02), V3::new(0.2, 0.14, 0.16), leaf));
    for s in [-1.0f32, 1.0] {
        p.push(ell(V3::new(sway + s * 0.05, 0.7, 0.11), V3::new(0.025, 0.02, 0.02), eye));
    }
    p.push(ell(V3::new(sway, 0.6, 0.12), V3::new(0.06, 0.02, 0.02), m(Ramp::Grey, 0, 1, Pattern::Plain)));
    finish_death(p, pose, 0.0)
}

fn finish_death(mut parts: Vec<Part>, pose: &Pose, _h: f32) -> Vec<Part> {
    if pose.hit > 0.0 {
        transform(&mut parts, V3::ZERO, &M3::rot_x(-0.15), V3::new(0.0, 0.0, -0.02));
    }
    if pose.death > 0.0 {
        let t = pose.death.min(1.0);
        transform(&mut parts, V3::ZERO, &M3::rot_z(t * PI * 0.95), V3::ZERO);
        let low = lowest_point(&parts);
        transform(&mut parts, V3::ZERO, &M3::IDENTITY, V3::new(0.0, -low, 0.0));
    }
    parts
}

fn spec_for(race: Race, look: u8) -> Option<Humanoid> {
    use Pattern::*;
    use Ramp::*;
    let flesh = m(Flesh, 3, 12, Noise(0.1));
    let eyes_dark = m(Grey, 0, 2, Plain);
    let red_eyes = m(Red, 10, 14, Plain).glow();
    let yellow_eyes = m(Gold, 11, 15, Plain).glow();
    let outfit = |k: u8| -> CMat {
        match k % 4 {
            0 => m(Earth, 2, 8, Cloth),
            1 => m(Red, 2, 8, Cloth),
            2 => m(Blue, 2, 8, Cloth),
            _ => m(Moss, 2, 8, Cloth),
        }
    };
    let s = match race {
        Race::GreenGoblin | Race::GrayGoblin => {
            let skin = if race == Race::GreenGoblin { m(Green, 2, 10, Noise(0.2)) } else { m(Grey, 2, 9, Noise(0.2)) };
            let mut s = Humanoid::base(0.66, skin);
            s.girth = 1.05;
            s.head = 1.25;
            s.ears = 1.0;
            s.nose = 1.4;
            s.torso = skin;
            s.legs = skin;
            s.feet = skin;
            s.belt = Some(m(Earth, 1, 6, Cloth));
            s.eyes = yellow_eyes;
            s.weapon = if race == Race::GreenGoblin { Weapon::Club } else { Weapon::Axe };
            s.hunch = 0.3;
            s.muscular = 1.0;
            s.fangs = true;
            s.loincloth = Some(m(Earth, 1, 6, Cloth));
            s
        }
        Race::Skeleton => {
            let bone = m(Stone, 5, 14, Bone);
            let mut s = Humanoid::base(0.82, bone);
            s.bony = true;
            s.torso = bone;
            s.legs = bone;
            s.feet = bone;
            s.eyes = red_eyes;
            s.weapon = Weapon::Sword;
            s.weapon_mat = m(Grey, 3, 11, Metal);
            s
        }
        Race::Lizardman => {
            let skin = m(Green, 2, 9, Scales);
            let mut s = Humanoid::base(0.8, skin);
            s.snout = 0.09;
            s.tail = Some(skin);
            s.torso = if look % 2 == 0 { skin } else { m(Earth, 2, 7, Noise(0.2)) };
            s.legs = skin;
            s.feet = skin;
            s.eyes = yellow_eyes;
            s.weapon = Weapon::Sword;
            s.belt = Some(m(Copper, 2, 7, Plain));
            s.hunch = 0.15;
            s.muscular = 0.6;
            s
        }
        Race::Dwarf => {
            let mut s = Humanoid::base(0.66, flesh);
            s.girth = 1.35;
            s.head = 1.15;
            s.beard = Some(match look % 3 {
                0 => m(Earth, 1, 6, Fur),
                1 => m(Grey, 6, 13, Fur),
                _ => m(Copper, 2, 9, Fur),
            });
            s.helmet = if look % 2 == 0 { Some(m(Grey, 3, 12, Metal)) } else { None };
            s.hair = Some(m(Earth, 1, 6, Fur));
            s.torso = if look % 2 == 0 { m(Grey, 3, 10, Chain) } else { m(Earth, 2, 8, Cloth) };
            s.sleeves = Some(m(Earth, 2, 7, Cloth));
            s.legs = m(Earth, 1, 6, Cloth);
            s.feet = m(Earth, 1, 5, Plain);
            s.belt = Some(m(Gold, 3, 10, Metal));
            s.eyes = eyes_dark;
            s.weapon = Weapon::Axe;
            s.nose = 1.2;
            s.muscular = 0.4;
            s
        }
        Race::Human if look == 4 => {
            let mut s = Humanoid::base(0.78, flesh);
            let dress = m(Red, 2, 9, Cloth);
            s.girth = 0.85;
            s.robe = Some(dress);
            s.torso = dress;
            s.sleeves = Some(dress);
            s.hair = Some(m(Gold, 4, 12, Fur));
            s.beard = None;
            s.feet = m(Earth, 1, 5, Plain);
            s.eyes = eyes_dark;
            s.belt = Some(m(Gold, 4, 12, Metal));
            s
        }
        Race::Human => {
            let mut s = Humanoid::base(0.82, flesh);
            s.nose = 0.9;
            s.hair = Some(match look % 4 {
                0 => m(Earth, 1, 6, Fur),
                1 => m(Gold, 3, 10, Fur),
                2 => m(Grey, 1, 4, Fur),
                _ => m(Copper, 2, 8, Fur),
            });
            if look % 3 == 1 {
                s.beard = Some(m(Earth, 1, 6, Fur));
            }
            s.torso = if look == 3 { m(Grey, 3, 11, Chain) } else { outfit(look) };
            s.sleeves = Some(outfit(look + 1));
            s.legs = m(Earth, 1, 6, Cloth);
            s.feet = m(Earth, 1, 5, Plain);
            s.belt = Some(m(Earth, 1, 5, Plain));
            s.eyes = eyes_dark;
            s.weapon = if look == 3 { Weapon::Sword } else if look == 2 { Weapon::Club } else { Weapon::None };
            if look == 3 {
                s.shield = Some(m(Earth, 2, 8, Wood));
            }
            s
        }
        Race::Mage => {
            let mut s = Humanoid::base(0.82, flesh);
            let robe = match look % 4 {
                0 => m(Purple, 2, 9, Cloth),
                1 => m(Red, 2, 8, Cloth),
                2 => m(Blue, 2, 8, Cloth),
                _ => m(Grey, 2, 9, Cloth),
            };
            s.robe = Some(robe);
            s.torso = robe;
            s.sleeves = Some(robe);
            s.hair = Some(m(Grey, 6, 13, Fur));
            s.beard = if look % 2 == 0 { Some(m(Grey, 7, 14, Fur)) } else { None };
            s.helmet = Some(robe);
            s.feet = m(Earth, 1, 5, Plain);
            s.eyes = eyes_dark;
            s.weapon = Weapon::Staff;
            s
        }
        Race::Ghoul => {
            let skin = m(Moss, 2, 9, Noise(0.35));
            let mut s = Humanoid::base(0.8, skin);
            s.torso = m(Grey, 1, 5, Cloth);
            s.legs = m(Grey, 1, 5, Cloth);
            s.feet = skin;
            s.eyes = red_eyes;
            s.weapon = Weapon::Claws;
            s.hunch = 0.45;
            s.arm_len = 1.2;
            s.muscular = 0.5;
            s.fangs = true;
            s
        }
        Race::Troll => {
            let skin = m(Moss, 2, 10, Noise(0.3));
            let mut s = Humanoid::base(1.05, skin);
            s.girth = 1.22;
            s.head = 0.85;
            s.hump = true;
            s.nose = 1.6;
            s.ears = 0.6;
            s.torso = skin;
            s.legs = m(Earth, 1, 6, Fur);
            s.feet = skin;
            s.eyes = yellow_eyes;
            s.weapon = Weapon::Club;
            s.hunch = 0.5;
            s.arm_len = 1.25;
            s.muscular = 1.2;
            s.fangs = true;
            s.loincloth = Some(m(Earth, 1, 5, Fur));
            s
        }
        Race::Ghost => {
            let pale = m(Silver, 6, 15, Cloth);
            let mut s = Humanoid::base(0.85, pale);
            s.robe = Some(pale);
            s.torso = pale;
            s.feet = pale;
            s.hair = Some(m(Silver, 8, 15, Fur));
            s.eyes = m(Blue, 11, 15, Plain).glow();
            s.weapon = Weapon::Claws;
            s
        }
        Race::Golem => {
            let rock = m(Stone, 2, 9, Noise(0.45));
            let mut s = Humanoid::base(1.1, rock);
            s.girth = 1.6;
            s.head = 0.8;
            s.torso = rock;
            s.legs = rock;
            s.feet = rock;
            s.eyes = m(Fire, 9, 13, Plain).glow();
            s.weapon = Weapon::Claws;
            s.arm_len = 1.15;
            s
        }
        Race::Headless => {
            let skin = m(Flesh, 2, 9, Noise(0.15));
            let mut s = Humanoid::base(0.85, skin);
            s.headless = true;
            s.girth = 1.25;
            s.torso = skin;
            s.legs = m(Earth, 1, 6, Cloth);
            s.feet = skin;
            s.weapon = Weapon::Club;
            s.muscular = 1.1;
            s
        }
        Race::Imp => {
            let skin = m(Red, 2, 10, Noise(0.15));
            let mut s = Humanoid::base(0.45, skin);
            s.head = 1.3;
            s.ears = 0.8;
            s.horns = Some(m(Stone, 3, 11, Plain));
            s.wings = Some(m(Red, 1, 6, Noise(0.2)));
            s.tail = Some(skin);
            s.torso = skin;
            s.legs = skin;
            s.feet = skin;
            s.eyes = yellow_eyes;
            s.weapon = Weapon::Claws;
            s
        }
        Race::Demon => {
            // The Lord of Souls: a gaunt, ashen king in tattered robes.
            let skin = m(Grey, 3, 10, Noise(0.3));
            let robe = m(Purple, 1, 9, Cloth);
            let mut s = Humanoid::base(1.35, skin);
            s.girth = 0.92;
            s.head = 1.0;
            s.robe = Some(robe);
            s.sleeves = Some(robe);
            s.torso = m(Slate, 2, 9, Metal);
            s.belt = Some(m(Gold, 2, 8, Metal));
            s.feet = m(Grey, 0, 4, Plain);
            s.hair = Some(m(Silver, 7, 15, Fur));
            s.beard = Some(m(Silver, 7, 15, Fur));
            s.crown = Some(m(Gold, 5, 14, Metal));
            s.eyes = m(Purple, 11, 15, Plain).glow();
            s.nose = 1.1;
            s.weapon = Weapon::Sword;
            s.weapon_mat = m(Fire, 8, 15, Metal).glow();
            s.hunch = 0.15;
            s.arm_len = 1.05;
            s
        }
        _ => return None,
    };
    Some(s)
}

/// Build the posed model parts for a creature.
pub fn model(race: Race, look: u8, pose: &Pose) -> Vec<Part> {
    match race {
        Race::Rat => rat(pose),
        Race::Bat => bat(pose),
        Race::Spider => spider(pose),
        Race::Slug => slug(pose),
        Race::Lurker => lurker(pose),
        Race::Reaper => reaper(pose),
        Race::Golem => golem(pose),
        _ => humanoid(&spec_for(race, look).expect("humanoid spec"), pose),
    }
}

/// Render all frames and views for one creature.
pub fn render_creature(race: Race, look: u8) -> CreatureSprites {
    let h = race.def().height / WORLD_SCALE;
    let px = SCALE * WORLD_SCALE;
    // Canvas big enough for a corpse lying on its side.
    let cw = ((h.max(0.45) * 1.25 + 0.15) * px).ceil() as i32 + 2;
    let ch = ((h * 1.3 + 0.12) * px).ceil() as i32 + 2;
    let mut frames = Vec::with_capacity(FRAMES);
    for f in 0..FRAMES {
        let pose = pose_for_frame(f);
        let parts = model(race, look, &pose);
        let mut views = Vec::with_capacity(VIEWS);
        for vi in 0..VIEWS {
            let v = View3 {
                yaw: vi as f32 * PI / 4.0,
                pitch: 0.12,
                scale: px,
                w: cw,
                h: ch,
                center: V3::ZERO,
                center_row: ch as f32 - 2.0,
            };
            views.push(render(&parts, &v, true));
        }
        frames.push(views);
    }
    CreatureSprites { frames, size: (cw as f32 / SCALE, ch as f32 / SCALE) }
}

/// A head-and-shoulders portrait for conversations.
pub fn portrait(race: Race, look: u8, size: i32) -> Bitmap {
    let parts = model(race, look, &Pose::default());
    let h = race.def().height / WORLD_SCALE;
    // (centre height, visible height) in world units.
    let (center_y, span) = match race {
        Race::Rat => (0.13, 0.3),
        Race::Bat => (0.18, 0.3),
        Race::Spider => (0.2, 0.4),
        Race::Slug => (0.15, 0.35),
        Race::Lurker => (0.22, 0.5),
        Race::Reaper => (0.8, 0.5),
        Race::Headless => (h * 0.75, h * 0.45),
        Race::Golem | Race::Troll | Race::Demon => (h * 0.82, h * 0.36),
        _ => (h * 0.86, h * 0.3),
    };
    let scale = size as f32 / span;
    let v = View3 {
        yaw: 0.25,
        pitch: 0.05,
        scale,
        w: size,
        h: size,
        center: V3::new(0.0, center_y, 0.0),
        center_row: size as f32 * 0.5,
    };
    render(&parts, &v, true)
}

/// The paperdoll body for the inventory panel.
pub fn paperdoll(parts: &[Part], w: i32, h: i32, scale: f32) -> Bitmap {
    let v = View3 { yaw: 0.0, pitch: 0.0, scale, w, h, center: V3::ZERO, center_row: h as f32 - 2.0 };
    render(parts, &v, true)
}

/// Player model for the paperdoll, dressed according to equipment colours.
pub struct Outfit {
    pub female: bool,
    pub beard: bool,
    pub skin: u8,
    pub hair: u8,
    pub head: Option<CMat>,
    pub chest: Option<CMat>,
    pub legs: Option<CMat>,
    pub feet: Option<CMat>,
    pub hands: Option<CMat>,
}

pub fn player_parts(o: &Outfit) -> Vec<Part> {
    use Pattern::*;
    let skin = match o.skin % 3 {
        0 => m(Ramp::Flesh, 3, 12, Noise(0.08)),
        1 => m(Ramp::Flesh, 2, 9, Noise(0.08)),
        _ => m(Ramp::Copper, 2, 9, Noise(0.08)),
    };
    let mut s = Humanoid::base(0.82, skin);
    s.girth = if o.female { 0.88 } else { 1.0 };
    s.head = 1.2;
    s.nose = 0.9;
    let hair = match o.hair % 4 {
        0 => m(Ramp::Earth, 1, 6, Fur),
        1 => m(Ramp::Gold, 3, 10, Fur),
        2 => m(Ramp::Grey, 1, 4, Fur),
        _ => m(Ramp::Copper, 2, 8, Fur),
    };
    s.hair = Some(hair);
    if o.beard && !o.female {
        s.beard = Some(hair);
    }
    s.torso = o.chest.unwrap_or(m(Ramp::Earth, 2, 7, Cloth));
    s.sleeves = Some(o.chest.unwrap_or(m(Ramp::Earth, 2, 7, Cloth)));
    s.legs = o.legs.unwrap_or(m(Ramp::Earth, 1, 6, Cloth));
    s.feet = o.feet.unwrap_or(skin);
    s.helmet = o.head;
    s.eyes = m(Ramp::Grey, 0, 2, Plain);
    s.belt = Some(m(Ramp::Earth, 1, 5, Plain));
    let mut parts = humanoid(&s, &Pose::default());
    if let Some(hm) = o.hands {
        // Recolour hands: hands are the small ellipsoids at the wrists.
        for p in parts.iter_mut() {
            if let Prim::Ellipsoid { r, .. } = p.prim {
                if (r.x - 0.032 * 0.82).abs() < 1e-4 {
                    p.mat = hm;
                }
            }
        }
    }
    let _ = bx;
    parts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::creatures::ALL_RACES;

    #[test]
    fn every_race_renders_nonempty_frames() {
        for race in ALL_RACES {
            let s = render_creature(race, 0);
            assert_eq!(s.frames.len(), FRAMES);
            for (fi, f) in s.frames.iter().enumerate() {
                for (vi, b) in f.iter().enumerate() {
                    let n = b.px.iter().filter(|&&p| p != 0).count();
                    assert!(n > 30, "{:?} frame {} view {} is empty", race, fi, vi);
                }
            }
        }
    }
}
