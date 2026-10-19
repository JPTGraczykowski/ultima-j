//! Hand-drawn facial features for the modelled creatures. The clay renderer
//! gives a head its volume and shading; eyes, brows, mouths and teeth are then
//! painted on as pixel art, the way the original's artists drew them.

use super::sprite3d::{project, to_camera, CMat, Part, Prim, View3};
use crate::gfx::palette::{c, ramp_of, Ramp, BLACK};
use crate::gfx::Bitmap;
use crate::util::V3;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Face {
    Human,
    Goblin,
    Troll,
    Skull,
    Ghoul,
    Lizard,
    Ghost,
    Imp,
    Tree,
    Spider,
    Lurker,
}

/// A face drawn for a head of a given size. Keys: `K` black, `e` eye colour,
/// `w` teeth, `m` mouth, `d`/`l` darken/lighten the rendered skin underneath,
/// `.` leave it.
struct Design {
    rows: &'static [&'static str],
    /// Texel over the front of the head, level with its centre.
    anchor: (i32, i32),
    /// Head radius in pixels the design is drawn for.
    radius: f32,
}

fn design(f: Face) -> Design {
    match f {
        Face::Human => Design {
            rows: &[
                ".ddd.ddd.",
                "..e...e..",
                "....l....",
                "...ddd...",
                ".........",
                "...dmd...",
            ],
            anchor: (4, 2),
            radius: 5.9,
        },
        Face::Goblin => Design {
            rows: &[
                "KK.....KK",
                ".Kee.eeK.",
                "....l....",
                "...KdK...",
                ".KwKwKwK.",
                "..KKKKK..",
            ],
            anchor: (4, 2),
            radius: 5.4,
        },
        Face::Troll => Design {
            rows: &[
                ".dKKKd.dKKKd.",
                "..dKedddeKd..",
                "....d...d....",
                ".....lll.....",
                "....dKdKd....",
                ".w.........w.",
                ".wKKmmmmmKKw.",
                "..KwKmmmKwK..",
                "...KKKKKKK...",
            ],
            anchor: (6, 2),
            radius: 7.9,
        },
        Face::Skull => Design {
            rows: &[
                ".KKK.KKK.",
                ".KeK.KeK.",
                "....K....",
                "...KKK...",
                ".........",
                ".KlKlKlK.",
            ],
            anchor: (4, 2),
            radius: 5.9,
        },
        Face::Ghoul => Design {
            rows: &[
                ".ddd.ddd.",
                ".KeK.KeK.",
                "....d....",
                "...KdK...",
                "..KwmwK..",
                "..KmmmK..",
                "...KKK...",
            ],
            anchor: (4, 2),
            radius: 5.8,
        },
        Face::Lizard => Design {
            rows: &[
                ".........",
                "eK.....Ke",
                ".........",
                "...K.K...",
            ],
            anchor: (4, 2),
            radius: 5.8,
        },
        Face::Ghost => Design {
            rows: &[
                ".KKK.KKK.",
                ".KeK.KeK.",
                ".........",
                "...KKK...",
                "...KKK...",
            ],
            anchor: (4, 2),
            radius: 6.1,
        },
        Face::Tree => Design {
            rows: &[
                "KK.........KK",
                ".KKK.....KKK.",
                "..KeeK.KeeK..",
                "...KK...KK...",
                ".............",
                "......d......",
                ".....ddd.....",
                ".............",
                "...KKKKKKK...",
                "..KmmmmmmmK..",
                "..KmKmmmKmK..",
                "...K.KKK.K...",
            ],
            anchor: (6, 5),
            radius: 6.0,
        },
        Face::Spider => Design {
            rows: &[
                ".e.....e.",
                "..eK.Ke..",
                ".........",
                "..KwKwK..",
                "...KmK...",
            ],
            anchor: (4, 2),
            radius: 4.4,
        },
        Face::Lurker => Design {
            rows: &[
                "...KKK...KKK...",
                "..KeeeK.KeeeK..",
                "...KKK...KKK...",
                "...............",
                ".KKKKKKKKKKKKK.",
                "KKwKwKwKwKwKwKK",
                "KmmmmmmmmmmmmmK",
                "KmmmmmmmmmmmmmK",
                ".KwKwKwKwKwKwK.",
                "..KKKKKKKKKKK..",
            ],
            anchor: (7, 3),
            radius: 8.75,
        },
        Face::Imp => Design {
            rows: &[
                "K.....K",
                ".eK.Ke.",
                "...l...",
                ".KwKwK.",
            ],
            anchor: (3, 2),
            radius: 3.8,
        },
    }
}

/// Paint `face` over the head in a rendered image; `pid` holds the part shown
/// by each pixel, so features only land on visible parts of the face.
pub fn paint(b: &mut Bitmap, pid: &[u16], parts: &[Part], v: &View3, face: Face, eyes: CMat) {
    // The head is the first face part, an ellipsoid.
    let Some(head) = parts.iter().find(|p| p.face && matches!(p.prim, Prim::Ellipsoid { .. })) else {
        return;
    };
    let Prim::Ellipsoid { c: centre, r, rot } = head.prim else {
        return;
    };
    let fwd = rot.mul_v(V3::new(0.0, 0.0, 1.0));
    let f = to_camera(v, fwd);
    let up = to_camera(v, rot.mul_v(V3::new(0.0, 1.0, 0.0)));
    // Facing away, or lying down.
    if f.z < 0.35 || up.y < 0.75 {
        return;
    }
    let (ax, ay, _) = project(v, centre + fwd * r.z);
    let d = design(face);
    let k = (r.y * v.scale / d.radius).round().max(1.0);
    // Seen from three quarters the far side of the face is foreshortened.
    let turn = if f.z < 0.85 { f.x.signum() } else { 0.0 };
    let eye = if eyes.glow { c(eyes.ramp, eyes.lo + 1) } else { c(Ramp::Grey, 1) };
    for (gy, row) in d.rows.iter().enumerate() {
        for (gx, ch) in row.chars().enumerate() {
            if ch == '.' {
                continue;
            }
            let dx = (gx as i32 - d.anchor.0) as f32;
            let dy = (gy as i32 - d.anchor.1) as f32;
            let dx = if turn == 0.0 { dx } else { dx * if dx * turn > 0.0 { 0.45 } else { 0.8 } };
            let x0 = (ax + (dx - 0.5) * k).round() as i32;
            let y0 = (ay + (dy - 0.5) * k).round() as i32;
            for y in y0..y0 + k as i32 {
                for x in x0..x0 + k as i32 {
                    if x < 0 || y < 0 || x >= b.w || y >= b.h {
                        continue;
                    }
                    let p = pid[(y * b.w + x) as usize];
                    if p == u16::MAX || !parts[p as usize].face {
                        continue;
                    }
                    let under = b.get(x, y);
                    let col = match ch {
                        'K' => BLACK,
                        'e' => eye,
                        'w' => c(Ramp::Parch, 13),
                        'm' => c(Ramp::Red, 2),
                        'd' => ramp_of(under).map(|(r, s)| c(r, s - 3)).unwrap_or(BLACK),
                        'l' => ramp_of(under).map(|(r, s)| c(r, s + 2)).unwrap_or(under),
                        _ => continue,
                    };
                    b.set(x, y, col);
                }
            }
        }
    }
}
