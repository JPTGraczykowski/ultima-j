//! Headless driver: runs the game without a window from a tiny script, for
//! automated tests and screenshots.
//!
//! Script commands (separated by `;`):
//! `title` | `chargen` | `play` - choose the starting screen
//! `wait N` - run N frames (1/30 s each)
//! `key K N` - hold key K for N frames (K: W, A, F5, Up, ...)
//! `tap K` - press key K for one frame
//! `mouse X Y` - move the mouse (320x200 coordinates)
//! `click X Y` / `rclick X Y` - left/right click
//! `rhold X Y N` - hold the right button for N frames
//! `type TEXT` - type characters
//! `goto X Z [YAW]` - teleport the player
//! `level N X Z` - change level
//! `give Kind` - put an item in the pack
//! `equip Kind Slot` - equip an item
//! `set NAME VALUE` - player attribute (vit, mana, level, exp) or skill (traps, lore...)
//! `runes all` - learn all runes
//! `shelf A B C` - put runes on the shelf
//! `shot PATH` - save a screenshot (3x, 4:3 aspect)
//! `shot1 PATH` - save a 1x screenshot

use crate::gfx::{Bitmap, SCREEN_H, SCREEN_W};
use crate::platform::{App, Input, Key};
use crate::ui::{GameApp, Screen};

fn parse_key(s: &str) -> Option<Key> {
    use Key::*;
    let k = match s.to_ascii_uppercase().as_str() {
        "A" => A, "B" => B, "C" => C, "D" => D, "E" => E, "F" => F, "G" => G, "H" => H, "I" => I, "J" => J,
        "K" => K, "L" => L, "M" => M, "N" => N, "O" => O, "P" => P, "Q" => Q, "R" => R, "S" => S, "T" => T,
        "U" => U, "V" => V, "W" => W, "X" => X, "Y" => Y, "Z" => Z,
        "1" => Num1, "2" => Num2, "3" => Num3, "4" => Num4, "5" => Num5, "6" => Num6, "7" => Num7, "8" => Num8, "9" => Num9, "0" => Num0,
        "F1" => F1, "F2" => F2, "F3" => F3, "F4" => F4, "F5" => F5, "F6" => F6, "F7" => F7, "F8" => F8, "F9" => F9, "F10" => F10,
        "UP" => Up, "DOWN" => Down, "LEFT" => Left, "RIGHT" => Right, "ENTER" => Enter, "ESC" => Escape, "SPACE" => Space,
        "TAB" => Tab, "PERIOD" => Period, "SEMICOLON" => Semicolon, "BACKSPACE" => Backspace,
        _ => return None,
    };
    Some(k)
}

pub fn save_screenshot(app: &mut GameApp, path: &str, scale: i32) -> Result<(), String> {
    let mut fb = Bitmap::new(SCREEN_W, SCREEN_H);
    let tint = app.draw(&mut fb);
    let pal = app.palette();
    let mut rgba = vec![0u8; (SCREEN_W * SCREEN_H * 4) as usize];
    pal.to_rgba(&fb.px, &mut rgba, tint);
    // Scale up with 4:3 aspect correction (each row is 1.2x as tall).
    let w = SCREEN_W * scale;
    let h = (SCREEN_H as f32 * scale as f32 * if scale > 1 { 1.2 } else { 1.0 }) as i32;
    let mut out = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        let sy = ((y as f32 / h as f32) * SCREEN_H as f32) as i32;
        for x in 0..w {
            let sx = x / scale;
            let si = ((sy * SCREEN_W + sx) * 4) as usize;
            let di = ((y * w + x) * 4) as usize;
            out[di..di + 4].copy_from_slice(&rgba[si..si + 4]);
        }
    }
    crate::gfx::save_png_rgba(path, w as u32, h as u32, &out)
}

pub fn run(script: &str) -> Result<(), String> {
    let cmds: Vec<Vec<String>> = script.split(';').map(|c| c.split_whitespace().map(|s| s.to_string()).collect()).filter(|c: &Vec<String>| !c.is_empty()).collect();
    let mut app = GameApp::new_playing(7);
    let mut mouse = (138, 60);
    let dt = 1.0 / 30.0;
    let step = |app: &mut GameApp, input: &Input| {
        app.update(input, dt);
        let mut fb = Bitmap::new(SCREEN_W, SCREEN_H);
        app.draw(&mut fb);
    };
    for c in cmds {
        let arg = |i: usize| c.get(i).map(|s| s.as_str()).unwrap_or("");
        let num = |i: usize| c.get(i).and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
        let base = Input { mouse, ..Default::default() };
        match arg(0) {
            "title" => app.screen = Screen::Title(crate::ui::menus::Title::new()),
            "chargen" => app.screen = Screen::CharGen(crate::ui::menus::CharGen::new()),
            "intro" => app.screen = Screen::Intro(crate::ui::menus::Intro::new()),
            "play" => app = GameApp::new_playing(7),
            "wait" => {
                for _ in 0..num(1) as i32 {
                    step(&mut app, &base);
                }
            }
            "key" => {
                let k = parse_key(arg(1)).ok_or(format!("bad key {}", arg(1)))?;
                let n = num(2) as i32;
                for i in 0..n {
                    let mut inp = base.clone();
                    inp.keys_down.push(k);
                    if i == 0 {
                        inp.keys_pressed.push(k);
                    }
                    step(&mut app, &inp);
                }
            }
            "tap" => {
                let k = parse_key(arg(1)).ok_or(format!("bad key {}", arg(1)))?;
                let mut inp = base.clone();
                inp.keys_down.push(k);
                inp.keys_pressed.push(k);
                step(&mut app, &inp);
                step(&mut app, &base);
            }
            "type" => {
                let text = c[1..].join(" ");
                let mut inp = base.clone();
                inp.chars = text.chars().collect();
                step(&mut app, &inp);
            }
            "mouse" => {
                mouse = (num(1) as i32, num(2) as i32);
                step(&mut app, &Input { mouse, ..Default::default() });
            }
            "click" | "rclick" => {
                mouse = (num(1) as i32, num(2) as i32);
                let b = if arg(0) == "click" { 0 } else { 1 };
                let mut inp = Input { mouse, ..Default::default() };
                inp.down[b] = true;
                inp.pressed[b] = true;
                step(&mut app, &inp);
                let mut up = Input { mouse, ..Default::default() };
                up.released[b] = true;
                step(&mut app, &up);
            }
            "rhold" => {
                mouse = (num(1) as i32, num(2) as i32);
                let n = num(3) as i32;
                for i in 0..n {
                    let mut inp = Input { mouse, ..Default::default() };
                    inp.down[1] = true;
                    inp.pressed[1] = i == 0;
                    step(&mut app, &inp);
                }
                let mut up = Input { mouse, ..Default::default() };
                up.released[1] = true;
                step(&mut app, &up);
            }
            "skill" => {
                if let Screen::Play(st) = &mut app.screen {
                    if let Some(s) = crate::game::items::ALL_SKILLS.iter().find(|s| s.name().eq_ignore_ascii_case(arg(1))) {
                        st.game.player.skills[*s as usize] = num(2) as i32;
                    }
                }
            }
            "goto" | "level" | "give" | "equip" | "set" | "runes" | "shelf" => {
                if let Screen::Play(st) = &mut app.screen {
                    let g = &mut st.game;
                    match arg(0) {
                        "goto" => {
                            let (x, z) = (num(1), num(2));
                            g.player.pos = crate::util::V3::new(x, g.level().map.floor_at(x, z), z);
                            if c.len() > 3 {
                                g.player.yaw = num(3).to_radians();
                            }
                            g.explore();
                        }
                        "level" => g.change_level(num(1) as usize, num(2), num(3), 0.0),
                        "give" => {
                            let k = crate::game::dialogue::item_kind_by_name(arg(1)).ok_or("bad item")?;
                            let mut it = crate::game::items::Item::new(k);
                            if c.len() > 2 {
                                it.data = num(2) as u16;
                            }
                            g.player.give(it);
                        }
                        "equip" => {
                            let k = crate::game::dialogue::item_kind_by_name(arg(1)).ok_or("bad item")?;
                            let slot = match arg(2) {
                                "weapon" => crate::game::EqSlot::Weapon,
                                "shield" => crate::game::EqSlot::Shield,
                                "head" => crate::game::EqSlot::Head,
                                "chest" => crate::game::EqSlot::Chest,
                                "legs" => crate::game::EqSlot::Legs,
                                "feet" => crate::game::EqSlot::Feet,
                                "hands" => crate::game::EqSlot::Hands,
                                "shoulder" => crate::game::EqSlot::ShoulderL,
                                _ => crate::game::EqSlot::ShoulderR,
                            };
                            let mut it = crate::game::items::Item::new(k);
                            if it.def().light.is_some() {
                                it.lit = true;
                            }
                            *g.player.equip.get_mut(slot) = Some(it);
                        }
                        "set" => {
                            let v = num(2) as i32;
                            match arg(1) {
                                "vit" => g.player.vit = v,
                                "mana" => {
                                    g.player.max_mana = g.player.max_mana.max(v);
                                    g.player.mana = v
                                }
                                "level" => g.player.level = v,
                                "exp" => g.gain_exp(v),
                                name => {
                                    if let Some(sk) = crate::game::items::ALL_SKILLS.iter().find(|k| k.name().eq_ignore_ascii_case(name)) {
                                        g.player.skills[*sk as usize] = v;
                                    }
                                }
                            }
                        }
                        "runes" => g.player.runes = [true; 24],
                        "shelf" => {
                            g.player.shelf = c[1..]
                                .iter()
                                .filter_map(|n| crate::game::magic::RUNE_NAMES.iter().position(|r| r.eq_ignore_ascii_case(n)))
                                .map(|i| i as u8)
                                .collect();
                        }
                        _ => {}
                    }
                    st.handle_events(&mut app.ctx);
                    st.mesh = crate::render::mesh::LevelMesh::build(&st.game.level().map);
                    st.mesh_level = st.game.cur;
                }
            }
            "shot" | "shot1" => {
                let scale = if arg(0) == "shot" { 3 } else { 1 };
                save_screenshot(&mut app, arg(1), scale)?;
                eprintln!("saved {}", arg(1));
            }
            other => return Err(format!("unknown command '{}'", other)),
        }
    }
    if let Screen::Play(st) = &app.screen {
        for m in st.game.recent_messages(24) {
            eprintln!("msg: {}", m.text);
        }
        for n in &st.game.level().npcs {
            let d = (n.pos - st.game.player.pos).len();
            if d < 12.0 {
                eprintln!("npc {:?} {:?} hp {}/{} {:?} {:?} dist {:.1}", n.race, n.name, n.hp, n.max_hp, n.anim, n.ai, d);
            }
        }
        if !matches!(st.overlay, crate::ui::Overlay::None) {
            eprintln!("overlay: {}", match st.overlay { crate::ui::Overlay::Victory(_) => "victory", crate::ui::Overlay::Dead(_) => "dead", crate::ui::Overlay::Conversation(..) => "conversation", _ => "other" });
        }
        let p = &st.game.player;
        eprintln!("player at ({:.2}, {:.2}, {:.2}) yaw {:.0} vit {}/{} level {} exp {}", p.pos.x, p.pos.y, p.pos.z, p.yaw.to_degrees(), p.vit, p.max_vit, p.level, p.exp);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::Rng;

    /// Random clicks and key presses on all screens must never panic.
    #[test]
    fn ui_fuzz() {
        let mut rng = Rng::new(3);
        let keys = [
            Key::W, Key::A, Key::S, Key::D, Key::X, Key::Z, Key::C, Key::J, Key::F1, Key::F2, Key::F3, Key::F4, Key::F5, Key::F6,
            Key::F7, Key::F8, Key::F9, Key::Escape, Key::Enter, Key::Num1, Key::Num2, Key::Num3, Key::M, Key::Up, Key::Down,
            Key::Q, Key::E, Key::R, Key::T, Key::V, Key::B, Key::N, Key::Num4, Key::Num5, Key::Num6, Key::Num7, Key::Num8,
        ];
        for start in 0..4 {
            let mut app = GameApp::new_playing(11);
            match start {
                1 => app.screen = Screen::Title(crate::ui::menus::Title::new()),
                2 => app.screen = Screen::CharGen(crate::ui::menus::CharGen::new()),
                3 => app.ctx.modern_controls = true,
                _ => {}
            }
            let mut fb = Bitmap::new(SCREEN_W, SCREEN_H);
            for _ in 0..1500 {
                let mut inp = Input { mouse: (rng.range(0, 320), rng.range(0, 200)), ..Default::default() };
                if rng.chance(0.15) {
                    let b = rng.range(0, 2) as usize;
                    inp.pressed[b] = true;
                    inp.down[b] = true;
                }
                if rng.chance(0.1) {
                    inp.released[rng.range(0, 2) as usize] = true;
                }
                if rng.chance(0.08) {
                    let k = *rng.pick(&keys);
                    // Never quit or save to disk during the fuzz.
                    inp.keys_pressed.push(k);
                    inp.keys_down.push(k);
                }
                if let Screen::Play(st) = &mut app.screen {
                    st.game.player.vit = st.game.player.max_vit;
                    // Avoid the options menu actions that touch the filesystem.
                    if matches!(st.overlay, crate::ui::Overlay::Options(_) | crate::ui::Overlay::Slots(..)) {
                        st.overlay = crate::ui::Overlay::None;
                    }
                }
                let cmd = app.update(&inp, 1.0 / 30.0);
                let _ = cmd;
                app.draw(&mut fb);
                if !matches!(app.screen, Screen::Play(_) | Screen::Title(_) | Screen::CharGen(_) | Screen::Intro(_) | Screen::Credits(_)) {
                    break;
                }
            }
        }
    }

    /// Modern controls: the number row picks commands and Q/E sidestep instead of flying.
    #[test]
    fn modern_controls() {
        use crate::ui::play::Mode;
        use crate::util::V3;
        let mut app = GameApp::new_playing(11);
        app.ctx.modern_controls = true;
        let key = |k: Key| Input { keys_pressed: vec![k], keys_down: vec![k], ..Default::default() };
        let play = |app: &GameApp| match &app.screen {
            Screen::Play(st) => (st.ui.mode, st.game.player.pos, st.game.player.yaw),
            _ => panic!("left the game"),
        };
        app.update(&key(Key::Num4), 1.0 / 30.0);
        assert_eq!(play(&app).0, Mode::Look);
        app.update(&key(Key::Num4), 1.0 / 30.0);
        assert_eq!(play(&app).0, Mode::None);

        let (_, p0, yaw0) = play(&app);
        for _ in 0..15 {
            app.update(&Input { keys_down: vec![Key::E], ..Default::default() }, 1.0 / 30.0);
        }
        let (_, p1, yaw1) = play(&app);
        assert_eq!(yaw0, yaw1);
        let side = V3::new(yaw0.cos(), 0.0, yaw0.sin());
        assert!(((p1 - p0).dot(side)).abs() > 0.1, "E should sidestep: moved {:?}", p1 - p0);
    }
}
