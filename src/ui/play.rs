//! The main game screen: input handling and drawing.

use super::panel;
use super::{Ctx, Overlay};
use crate::art::ui::Cursor;
use crate::audio::Sfx;
use crate::game::actions::Target;
use crate::game::dialogue::{self, ConvState};
use crate::game::items::{Class, Item, ItemKind};
use crate::game::magic::{Spell, Targeting};
use crate::game::state::{Event, MsgKind};
use crate::game::{ai, Game, MoveInput};
use crate::gfx::palette::{self, c, Ramp, BLACK, INK, MSG_GREEN};
use crate::gfx::Bitmap;
use crate::layout::*;
use crate::platform::{Input, Key};
use crate::render::mesh::{LevelMesh, Poly, WVert};
use crate::render::{self, Camera, LightParams, SpriteInst};
use crate::util::V3;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    None,
    Talk,
    Get,
    Look,
    Fight,
    Use,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Panel {
    Inventory,
    Stats,
    Runes,
}

pub struct Ui {
    pub mode: Mode,
    pub panel: Panel,
    pub held: Option<Item>,
    /// Path to an opened container: pack slot, then indices inside.
    pub open: Vec<usize>,
    pub charge: f32,
    pub charging: bool,
    pub attack_kind: usize,
    pub swing: f32,
    pub flash: Option<([f32; 3], f32)>,
    pub shake: f32,
    pub pitch: f32,
    pub right_down: Option<(f32, (i32, i32))>,
    pub left_in_view: bool,
    pub targeting: Option<Spell>,
    pub hover: (Target, f32),
    pub stats_scroll: i32,
    pub msg_back: usize,
    pub easy_move: Option<usize>,
    pub cursor_dir: Cursor,
    pub pack_page: usize,
    /// A text to show once the current overlay closes (dreams after sleep).
    pub pending_text: Option<String>,
    /// Map note being typed: (level, x, z, text).
    pub note: Option<(usize, i32, i32, String)>,
}

impl Ui {
    pub fn new() -> Ui {
        Ui {
            mode: Mode::None,
            panel: Panel::Inventory,
            held: None,
            open: Vec::new(),
            charge: 0.0,
            charging: false,
            attack_kind: 1,
            swing: 0.0,
            flash: None,
            shake: 0.0,
            pitch: 0.0,
            right_down: None,
            left_in_view: false,
            targeting: None,
            hover: (Target::None, 0.0),
            stats_scroll: 0,
            msg_back: 0,
            easy_move: None,
            cursor_dir: Cursor::Cross,
            pack_page: 0,
            pending_text: None,
            note: None,
        }
    }
}

pub struct PlayState {
    pub game: Game,
    pub mesh: LevelMesh,
    pub mesh_level: usize,
    pub ui: Ui,
    pub overlay: Overlay,
}

impl PlayState {
    pub fn new(game: Game) -> PlayState {
        let mesh = LevelMesh::build(&game.level().map);
        let lvl = game.cur;
        PlayState { game, mesh, mesh_level: lvl, ui: Ui::new(), overlay: Overlay::None }
    }

    fn view_mouse(input: &Input) -> Option<(f32, f32)> {
        if VIEW.contains(input.mouse) {
            Some(((input.mouse.0 - VIEW.x) as f32 + 0.5, (input.mouse.1 - VIEW.y) as f32 + 0.5))
        } else {
            None
        }
    }

    /// Which movement cursor applies at a view position.
    fn move_cursor(vx: f32, vy: f32) -> Cursor {
        let col = ((vx / VIEW.w as f32) * 3.0) as i32;
        let row = ((vy / VIEW.h as f32) * 3.0) as i32;
        match (row.clamp(0, 2), col.clamp(0, 2)) {
            (0, 0) => Cursor::FwdLeft,
            (0, 1) => Cursor::Forward,
            (0, 2) => Cursor::FwdRight,
            (1, 0) => Cursor::TurnLeft,
            (1, 1) => Cursor::Cross,
            (1, 2) => Cursor::TurnRight,
            (2, 0) => Cursor::SlideLeft,
            (2, 1) => Cursor::Back,
            _ => Cursor::SlideRight,
        }
    }

    fn pick(&self, ctx: &Ctx, vx: f32, vy: f32) -> (Target, f32) {
        let v = &ctx.renderer.view;
        let x = vx as i32;
        let y = vy as i32;
        if x < 0 || y < 0 || x >= v.w || y >= v.h {
            return (Target::None, 0.0);
        }
        let i = (y * v.w + x) as usize;
        let id = v.ids[i];
        let dist = if v.depth[i] > 0.0 { 1.0 / v.depth[i] } else { 99.0 };
        let cat = id & !render::ID_MASK;
        let low = id & render::ID_MASK;
        let tex = |l: u32| crate::art::textures::ALL_TEX[(l & 0xFF) as usize % crate::art::textures::TEX_COUNT];
        let tile = |l: u32| (((l >> 8) & 0xFFF) as i32 % 64, ((l >> 8) & 0xFFF) as i32 / 64);
        let t = match cat {
            render::ID_FLOOR => Target::Floor { tx: tile(low).0, tz: tile(low).1, tex: tex(low) },
            render::ID_CEIL => Target::Ceiling { tx: tile(low).0, tz: tile(low).1, tex: tex(low) },
            render::ID_WALL => Target::Wall { tx: tile(low).0, tz: tile(low).1, tex: tex(low) },
            render::ID_ITEM => Target::Item(low),
            render::ID_NPC => Target::Npc(low),
            render::ID_DOOR => Target::Door(low),
            _ => Target::None,
        };
        (t, dist)
    }

    /// World point under a view pixel (on the depth buffer).
    fn world_point(&self, ctx: &Ctx, vx: f32, vy: f32) -> Option<V3> {
        let (_, dist) = self.pick(ctx, vx, vy);
        if dist > 50.0 {
            return None;
        }
        let cam = self.camera();
        let ray = ctx.renderer.pixel_ray(&cam, vx, vy);
        let fwd = cam.forward();
        let cosang = ray.dot(fwd).max(0.1);
        Some(cam.pos + ray * (dist / cosang))
    }

    pub fn camera(&self) -> Camera {
        let s = self.ui.shake;
        let shake = if s > 0.0 { (((self.game.time * 53.0).sin() as f32) * s * 8.0, ((self.game.time * 37.0).cos() as f32) * s * 6.0) } else { (0.0, 0.0) };
        Camera { pos: self.game.eye(), yaw: self.game.player.yaw, pitch: self.ui.pitch, shake }
    }

    fn aim_dir(&self, ctx: &Ctx, input: &Input) -> V3 {
        let cam = self.camera();
        match Self::view_mouse(input) {
            Some((vx, vy)) => ctx.renderer.pixel_ray(&cam, vx, vy),
            None => ctx.renderer.pixel_ray(&cam, VIEW.w as f32 / 2.0, VIEW.h as f32 / 2.0),
        }
    }

    pub fn update(&mut self, ctx: &mut Ctx, input: &Input, dt: f32) -> Option<super::ScreenCmd> {
        // Decay presentation effects.
        if let Some((col, a)) = self.ui.flash {
            let na = a - dt * 1.2;
            self.ui.flash = if na > 0.0 { Some((col, na)) } else { None };
        }
        self.ui.shake = (self.ui.shake - dt).max(0.0);
        if self.ui.swing > 0.0 {
            self.ui.swing = (self.ui.swing - dt * 3.0).max(0.0);
        }

        // Overlays capture input.
        match &mut self.overlay {
            Overlay::None => {}
            _ => return super::overlay::update(self, ctx, input, dt),
        }

        let mut mv = MoveInput::default();
        let shift = input.shift;
        // Keyboard movement (original layout plus arrow keys).
        if input.key(Key::W) {
            mv.forward = 2.0;
        } else if input.key(Key::S) || input.key(Key::Up) {
            mv.forward = if shift || input.key(Key::Up) && shift { 2.0 } else { 1.0 };
        }
        if input.key(Key::Up) {
            mv.forward = if shift { 1.0 } else { 2.0 };
        }
        if input.key(Key::X) || input.key(Key::Down) {
            mv.forward = -1.0;
        }
        if input.key(Key::A) || input.key(Key::Left) {
            mv.turn = -1.0;
        }
        if input.key(Key::D) || input.key(Key::Right) {
            mv.turn = 1.0;
        }
        if input.key(Key::Z) {
            mv.strafe = -1.0;
        }
        if input.key(Key::C) {
            mv.strafe = 1.0;
        }
        if input.hit(Key::J) || input.hit(Key::Space) {
            mv.jump = true;
        }
        mv.fly_up = input.key(Key::E);
        mv.fly_down = input.key(Key::Q);
        // Look up/down.
        if input.key(Key::Num1) || input.key(Key::PageDown) {
            self.ui.pitch = (self.ui.pitch - dt * 90.0).max(-48.0);
        }
        if input.key(Key::Num3) || input.key(Key::PageUp) {
            self.ui.pitch = (self.ui.pitch + dt * 90.0).min(48.0);
        }
        if input.hit(Key::Num2) || input.hit(Key::Home) {
            self.ui.pitch = 0.0;
        }
        // Function keys.
        if input.hit(Key::F1) || input.hit(Key::Escape) {
            self.overlay = Overlay::Options(0);
            ctx.sounds.push(Sfx::Click);
        }
        let mode_keys = [(Key::F2, Mode::Talk), (Key::F3, Mode::Get), (Key::F4, Mode::Look), (Key::F5, Mode::Fight), (Key::F6, Mode::Use)];
        for (k, m) in mode_keys {
            if input.hit(k) {
                self.ui.mode = if self.ui.mode == m { Mode::None } else { m };
            }
        }
        if input.hit(Key::F7) {
            self.ui.panel = if self.ui.panel == Panel::Inventory { Panel::Stats } else { Panel::Inventory };
        }
        if input.hit(Key::F8) {
            self.cast(ctx, input);
        }
        if input.hit(Key::F9) {
            self.game.track();
        }
        if input.hit(Key::F10) {
            self.game.sleep();
        }
        if input.hit(Key::H) {
            self.overlay = Overlay::Text(super::overlay::HELP_TEXT.to_string(), 0);
        }
        if input.hit(Key::M) || input.hit(Key::Tab) {
            self.overlay = Overlay::Map(self.game.cur);
        }
        if input.ctrl && input.hit(Key::S) {
            super::save::quick_save(self, ctx);
        }
        if input.ctrl && input.hit(Key::R) {
            return Some(super::ScreenCmd::LoadSlot(0));
        }
        if input.ctrl && input.hit(Key::Q) {
            return Some(super::ScreenCmd::Quit);
        }
        // Keyboard attacks.
        for (k, kind) in [(Key::P, 0usize), (Key::Semicolon, 1), (Key::Period, 2)] {
            if input.hit(k) && self.ui.swing <= 0.0 {
                self.ui.mode = Mode::Fight;
                let aim = self.camera().forward();
                ai::player_attack(&mut self.game, aim, kind, 0.6);
                self.ui.swing = 1.0;
                self.ui.attack_kind = kind;
            }
        }

        // Mouse.
        let m = input.mouse;
        let in_view = Self::view_mouse(input);
        if let Some((vx, vy)) = in_view {
            self.ui.hover = self.pick(ctx, vx, vy);
            self.ui.cursor_dir = Self::move_cursor(vx, vy);
        }
        if input.pressed[0] {
            if let Some((vx, vy)) = in_view {
                if let Some(spell) = self.ui.targeting.take() {
                    let aim = self.aim_dir(ctx, input);
                    let _ = spell;
                    self.game.cast_shelf(aim);
                } else if let Some(it) = self.ui.held.take() {
                    let at = self.world_point(ctx, vx, vy);
                    self.throw_or_drop(it, at);
                } else {
                    self.ui.left_in_view = true;
                }
            } else {
                self.click_ui(ctx, input, m);
            }
        }
        if !input.down[0] {
            self.ui.left_in_view = false;
            self.ui.easy_move = None;
        }
        if self.ui.left_in_view {
            if let Some((vx, vy)) = in_view {
                let dx = (vx - VIEW.w as f32 / 2.0) / (VIEW.w as f32 / 2.0);
                let dy = (vy - VIEW.h as f32 / 2.0) / (VIEW.h as f32 / 2.0);
                let speed = (dx.abs().max(dy.abs()) * 1.4 + 0.4).min(2.0);
                match Self::move_cursor(vx, vy) {
                    Cursor::Forward => mv.forward = speed,
                    Cursor::Back => mv.forward = -speed.min(1.0),
                    Cursor::FwdLeft => {
                        mv.forward = speed * 0.8;
                        mv.turn = -0.7;
                    }
                    Cursor::FwdRight => {
                        mv.forward = speed * 0.8;
                        mv.turn = 0.7;
                    }
                    Cursor::TurnLeft => mv.turn = -(dx.abs() * 1.2).max(0.5),
                    Cursor::TurnRight => mv.turn = (dx.abs() * 1.2).max(0.5),
                    Cursor::SlideLeft => mv.strafe = -1.0,
                    Cursor::SlideRight => mv.strafe = 1.0,
                    _ => {}
                }
                if input.pressed[1] {
                    mv.jump = true;
                }
            }
        }
        if let Some(b) = self.ui.easy_move {
            match b {
                0 => mv.turn = -0.8,
                1 => mv.forward = 1.0,
                _ => mv.turn = 0.8,
            }
        }
        // Right button: actions.
        if input.pressed[1] && !self.ui.left_in_view {
            self.ui.right_down = Some((self.game.time as f32, m));
            if in_view.is_some() && self.ui.mode == Mode::Fight && self.ui.swing <= 0.0 {
                self.ui.charging = true;
                self.ui.charge = 0.0;
                let vy = in_view.unwrap().1;
                self.ui.attack_kind = if vy < VIEW.h as f32 * 0.33 { 0 } else if vy < VIEW.h as f32 * 0.66 { 1 } else { 2 };
            }
        }
        if self.ui.charging {
            let speed = 1.0 / (0.9 * self.weapon_delay());
            self.ui.charge = (self.ui.charge + dt * speed).min(1.0);
        }
        if input.released[1] {
            if self.ui.charging {
                self.ui.charging = false;
                if self.ui.charge >= 0.2 {
                    let aim = self.aim_dir(ctx, input);
                    let p = self.ui.charge;
                    ai::player_attack(&mut self.game, aim, self.ui.attack_kind, p);
                    self.ui.swing = 1.0;
                }
                self.ui.charge = 0.0;
            } else if let Some((t0, p0)) = self.ui.right_down.take() {
                let held_long = self.game.time as f32 - t0 > 0.3 || (p0.0 - m.0).abs() + (p0.1 - m.1).abs() > 4;
                if let Some((vx, vy)) = Self::view_mouse(&Input { mouse: p0, ..Default::default() }) {
                    if let Some(spell) = self.ui.targeting.take() {
                        let _ = spell;
                        let aim = self.aim_dir(ctx, input);
                        self.game.cast_shelf(aim);
                    } else {
                        let (t, dist) = self.pick(ctx, vx, vy);
                        self.view_action(t, dist, held_long);
                    }
                } else {
                    self.right_click_ui(ctx, input, p0);
                }
            }
        }
        if !input.down[1] {
            self.ui.right_down = if input.released[1] { None } else { self.ui.right_down };
        }

        self.game.update(&mv, dt);
        self.handle_events(ctx);
        if self.game.cur != self.mesh_level {
            self.mesh = LevelMesh::build(&self.game.level().map);
            self.mesh_level = self.game.cur;
        }
        None
    }

    fn weapon_delay(&self) -> f32 {
        self.game.player.equip.weapon.as_ref().and_then(|w| w.def().weapon).map(|w| w.delay).unwrap_or(0.8)
    }

    pub fn handle_events(&mut self, ctx: &mut Ctx) {
        let events: Vec<Event> = std::mem::take(&mut self.game.events);
        for e in events {
            match e {
                Event::Sound(s) => ctx.sounds.push(s),
                Event::Flash(c, a) => self.ui.flash = Some((c, a)),
                Event::Shake(s) => self.ui.shake = self.ui.shake.max(s),
                Event::LevelChanged => {
                    self.mesh = LevelMesh::build(&self.game.level().map);
                    self.mesh_level = self.game.cur;
                }
                Event::StartConversation(id) => {
                    if let Some(n) = self.game.level().npc(id) {
                        if let Some(conv) = n.talk {
                            let mut st = ConvState { conv, npc: id, node: String::new(), text: String::new(), options: Vec::new(), ended: false };
                            dialogue::enter_node(&mut self.game, &ctx.db, &mut st, "start");
                            // NPC turns to face the player.
                            let eye = self.game.eye();
                            if let Some(n) = self.game.level_mut().npc_mut(id) {
                                n.yaw = (eye.x - n.pos.x).atan2(-(eye.z - n.pos.z));
                                n.ai = crate::game::creatures::AiState::Idle;
                            }
                            self.overlay = Overlay::Conversation(Box::new(st), 0);
                        }
                    }
                }
                Event::ReadText(t) => {
                    if t == "@sleep" {
                        self.overlay = Overlay::Sleep(0.0);
                    } else if matches!(self.overlay, Overlay::Sleep(_)) {
                        self.ui.pending_text = Some(t);
                    } else {
                        self.overlay = Overlay::Text(t, 0);
                    }
                }
                Event::PlayerDied => {
                    self.overlay = Overlay::Dead(0.0);
                }
                Event::Victory => {
                    self.overlay = Overlay::Victory(0.0);
                }
            }
        }
    }

    fn view_action(&mut self, t: Target, dist: f32, held_long: bool) {
        let mode = self.ui.mode;
        match mode {
            Mode::Look => self.game.look(t),
            Mode::Talk => self.game.talk(t, dist),
            Mode::Get => self.do_get(t, dist),
            Mode::Use => {
                let held = self.ui.held.clone();
                self.game.use_world(t, dist, held.as_ref());
            }
            Mode::Fight => {}
            Mode::None => {
                if !held_long {
                    self.game.look(t);
                } else {
                    match t {
                        Target::Item(id) => {
                            let fixture = self.game.level().items.iter().find(|i| i.id == id).map(|i| i.item.kind.is_fixture()).unwrap_or(true);
                            if fixture {
                                self.game.use_world(t, dist, None);
                            } else {
                                self.do_get(t, dist);
                            }
                        }
                        Target::Npc(_) => self.game.talk(t, dist),
                        Target::Door(_) | Target::Wall { .. } => self.game.use_world(t, dist, None),
                        _ => self.game.look(t),
                    }
                }
            }
        }
    }

    fn do_get(&mut self, t: Target, dist: f32) {
        if self.ui.held.is_some() {
            self.game.msg("You are already holding something.");
            return;
        }
        if let Some(it) = self.game.get(t, dist) {
            self.ui.held = Some(it);
        }
    }

    fn throw_or_drop(&mut self, it: Item, at: Option<V3>) {
        self.game.drop_item(it, at);
    }

    pub fn cast(&mut self, ctx: &Ctx, input: &Input) {
        let shelf = self.game.player.shelf.clone();
        if let Some(sp) = Spell::from_runes(&shelf) {
            if sp.def().target != Targeting::Caster && Self::view_mouse(input).is_none() {
                self.ui.targeting = Some(sp);
                self.game.msg("Select a target for the spell.");
                return;
            }
        }
        let aim = self.aim_dir(ctx, input);
        self.game.cast_shelf(aim);
    }

    fn click_ui(&mut self, ctx: &mut Ctx, input: &Input, m: (i32, i32)) {
        // Command icons.
        for (i, &(cx, cy)) in ICON_CENTERS.iter().enumerate() {
            if (m.0 - cx).abs() <= ICON_HIT.0 && (m.1 - cy).abs() <= ICON_HIT.1 {
                ctx.sounds.push(Sfx::Click);
                if i == 0 {
                    self.overlay = Overlay::Options(0);
                    return;
                }
                let modes = [Mode::None, Mode::Talk, Mode::Get, Mode::Look, Mode::Fight, Mode::Use];
                self.ui.mode = if self.ui.mode == modes[i] { Mode::None } else { modes[i] };
                return;
            }
        }
        if CHAIN.contains(m) {
            ctx.sounds.push(Sfx::Click);
            self.ui.panel = match self.ui.panel {
                Panel::Inventory => Panel::Stats,
                _ => Panel::Inventory,
            };
            return;
        }
        if PANEL.contains(m) {
            panel::click(self, ctx, m, false);
            return;
        }
        let (ccx, ccy) = COMPASS_C;
        if ((m.0 - ccx) as f32 / 30.0).powi(2) + ((m.1 - ccy) as f32 / 11.0).powi(2) < 1.0 {
            self.game.status_report();
            return;
        }
        for (i, r) in MOVE_BUTTONS.iter().enumerate() {
            if r.contains(m) {
                self.ui.easy_move = Some(i);
                return;
            }
        }
        if RUNE_SHELF.contains(m) {
            self.cast(ctx, input);
            return;
        }
        let vit = Rect::new(FLASK_VIT.0 - 12, FLASK_VIT.1 - 30, 24, 32);
        let mana = Rect::new(FLASK_MANA.0 - 12, FLASK_MANA.1 - 30, 24, 32);
        if vit.contains(m) {
            let p = &self.game.player;
            let s = format!("Your current vitality is {} out of {}.", p.vit, p.max_vit);
            self.game.msg(s);
            return;
        }
        if mana.contains(m) {
            let p = &self.game.player;
            let s = format!("Your current mana points are {} out of {}.", p.mana, p.max_mana);
            self.game.msg(s);
            return;
        }
        if Rect::new(0, 166, 320, 34).contains(m) {
            // Scroll back through messages.
            let total = self.game.messages.len();
            self.ui.msg_back = if m.1 < 183 { (self.ui.msg_back + 1).min(total.saturating_sub(SCROLL_LINES as usize)) } else { self.ui.msg_back.saturating_sub(1) };
        }
    }

    fn right_click_ui(&mut self, ctx: &mut Ctx, _input: &Input, m: (i32, i32)) {
        if PANEL.contains(m) {
            panel::click(self, ctx, m, true);
        } else if RUNE_SHELF.contains(m) {
            self.game.player.shelf.clear();
            ctx.sounds.push(Sfx::Click);
        }
    }

    // ---------------------------------------------------------------- drawing

    pub fn draw(&mut self, ctx: &mut Ctx, fb: &mut Bitmap, input_mouse: (i32, i32)) {
        if let Overlay::Map(lvl) = self.overlay {
            super::automap::draw(ctx, fb, &self.game, lvl, self.ui.note.clone());
            return;
        }
        fb.blit_opaque(&ctx.art.ui.frame, 0, 0);
        let in_convo = matches!(self.overlay, Overlay::Conversation(..));
        if !in_convo {
            self.render_view(ctx);
            let mut vb = ctx.renderer.view.to_bitmap();
            if let Overlay::Sleep(t) = self.overlay {
                let k = ((1.0 - (t * 2.0 - 1.0).abs()) * 1.2).min(1.0);
                let level = ((1.0 - k) * palette::MAX_LIGHT as f32) as usize;
                let tab = ctx.art.pal.shade[level];
                vb.remap(&tab);
            }
            self.draw_weapon(ctx, &mut vb);
            fb.blit_opaque(&vb, VIEW.x, VIEW.y);
        }
        self.draw_icons(ctx, fb);
        panel::draw(self, ctx, fb);
        self.draw_compass(ctx, fb);
        self.draw_gem(ctx, fb);
        self.draw_gargoyle(ctx, fb);
        self.draw_flasks(ctx, fb);
        self.draw_shelf(ctx, fb);
        self.draw_spells(ctx, fb);
        if in_convo {
            super::convo::draw(self, ctx, fb);
        } else {
            self.draw_messages(ctx, fb);
        }
        super::overlay::draw(self, ctx, fb);
        self.draw_cursor(ctx, fb, input_mouse);
    }

    pub fn render_view(&mut self, ctx: &mut Ctx) {
        let cam = self.camera();
        let g = &self.game;
        let lv = g.level();
        let (radius, level) = g.player.light();
        let lp = LightParams { radius, level, ambient: lv.ambient };
        // Ensure sprites exist.
        for wi in &lv.items {
            ctx.cache.ensure_item(&wi.item);
        }
        for n in &lv.npcs {
            ctx.cache.ensure_creature(n.race, n.look);
        }
        let viewer = cam.pos;
        let mut sprites: Vec<SpriteInst> = Vec::new();
        let tile_light = |x: f32, z: f32| lv.map.get(x.floor() as i32, z.floor() as i32).light as f32;
        for wi in &lv.items {
            if let Some((bmp, size)) = ctx.cache.world_sprite(&wi.item) {
                let glow = matches!(wi.item.kind, ItemKind::Campfire | ItemKind::Brazier) || (wi.item.lit && wi.item.def().class == Class::Light);
                sprites.push(SpriteInst { bmp, id: render::ID_ITEM | wi.id, pos: wi.pos, w: size.0, h: size.1, flip: false, fullbright: glow, stipple: false, light: tile_light(wi.pos.x, wi.pos.z) });
            }
        }
        for n in &lv.npcs {
            if let Some(cs) = ctx.cache.creature(n.race, n.look) {
                let (f, v, flip) = super::sprites::creature_frame(n, viewer);
                let bmp = &cs.frames[f][v];
                let ghost = n.race == crate::game::creatures::Race::Ghost;
                sprites.push(SpriteInst { bmp, id: render::ID_NPC | n.id, pos: n.pos, w: cs.size.0, h: cs.size.1, flip, fullbright: false, stipple: ghost, light: tile_light(n.pos.x, n.pos.z) });
            }
        }
        let frame = (g.time * 6.0) as usize;
        for p in &g.projectiles {
            let k = match p.kind {
                crate::game::ProjKind::MagicArrow => 0,
                crate::game::ProjKind::Lightning => 1,
                crate::game::ProjKind::Fireball => 2,
                crate::game::ProjKind::Poison => 3,
                _ => 4,
            };
            let bmp = &ctx.cache.effects[k][frame % 4];
            let pos = V3::new(p.pos.x, p.pos.y - 0.12, p.pos.z);
            sprites.push(SpriteInst { bmp, id: 0, pos, w: 0.25, h: 0.25, flip: false, fullbright: true, stipple: false, light: 31.0 });
        }
        for e in &g.effects {
            let k = match e.kind {
                crate::game::EffectKind::Blood => 5,
                crate::game::EffectKind::Sparks => 6,
                crate::game::EffectKind::Explosion => 7,
                crate::game::EffectKind::Splash => 8,
                crate::game::EffectKind::Smoke => 9,
            };
            let fr = ((e.t / 0.6) * 4.0) as usize;
            let bmp = &ctx.cache.effects[k][fr.min(3)];
            let pos = V3::new(e.pos.x, e.pos.y - 0.2, e.pos.z);
            sprites.push(SpriteInst { bmp, id: 0, pos, w: 0.4, h: 0.4, flip: false, fullbright: k != 5, stipple: false, light: 31.0 });
        }
        let doors = door_polys(lv);
        let tframe = (g.time * 4.0) as usize;
        ctx.renderer.render(&ctx.art, &self.mesh, &cam, &doors, &sprites, &lp, tframe);
    }

    fn draw_weapon(&self, ctx: &mut Ctx, vb: &mut Bitmap) {
        if self.ui.mode != Mode::Fight {
            return;
        }
        let kind = self.game.player.equip.weapon.as_ref().map(|w| w.kind);
        let phase = if self.ui.swing > 0.0 {
            if self.ui.swing > 0.6 { 2 } else { 3 }
        } else if self.ui.charging {
            1
        } else {
            0
        };
        let bmp = ctx.cache.hand_weapon(kind, phase);
        let x = VIEW.w / 2 - bmp.w / 2 + match phase {
            1 => 22,
            2 => -10,
            3 => -34,
            _ => 18,
        };
        let y = VIEW.h - bmp.h + 6 + if phase == 1 { -6 } else { 0 };
        vb.blit(bmp, x, y);
    }

    fn draw_icons(&self, ctx: &Ctx, fb: &mut Bitmap) {
        let sel = match self.ui.mode {
            Mode::None => None,
            Mode::Talk => Some(1),
            Mode::Get => Some(2),
            Mode::Look => Some(3),
            Mode::Fight => Some(4),
            Mode::Use => Some(5),
        };
        for i in 0..ICON_CENTERS.len() {
            let (cx, cy) = ICON_CENTERS[i];
            let ic = if Some(i) == sel {
                crate::art::ui::oval_rim(fb, crate::art::ui::ICON_OVALS[i], true);
                &ctx.art.ui.icons_lit[i]
            } else {
                &ctx.art.ui.icons[i]
            };
            fb.blit(ic, cx - ic.w / 2, cy - ic.h / 2);
        }
    }

    fn draw_compass(&self, ctx: &Ctx, fb: &mut Bitmap) {
        let yaw = self.game.player.yaw;
        let dir = ((yaw / std::f32::consts::TAU * 16.0).round() as i32).rem_euclid(16) as usize;
        let b = &ctx.art.ui.compass[dir];
        fb.blit(b, COMPASS_C.0 - b.w / 2, COMPASS_C.1 - b.h / 2);
    }

    fn draw_gem(&self, ctx: &Ctx, fb: &mut Bitmap) {
        let state = if !self.ui.charging {
            0
        } else if self.ui.charge < 0.35 {
            1
        } else if self.ui.charge < 0.7 {
            2
        } else if self.ui.charge < 0.98 {
            3
        } else {
            4
        };
        let b = &ctx.art.ui.gem[state.min(4)];
        let b = if state == 4 { &ctx.art.ui.gem[4] } else { b };
        fb.blit(b, GEM_C.0 - b.w / 2, GEM_C.1 - 6);
    }

    fn draw_gargoyle(&self, ctx: &Ctx, fb: &mut Bitmap) {
        let state = match self.game.foe.and_then(|id| self.game.level().npc(id)) {
            Some(n) if n.alive() => {
                let f = n.hp as f32 / n.max_hp.max(1) as f32;
                if f > 0.66 { 1 } else if f > 0.33 { 2 } else { 3 }
            }
            _ => 0,
        };
        if state == 0 {
            return;
        }
        let col = ctx.art.ui.gargoyle_eye[state];
        let (gx, _) = GARGOYLE_C;
        let ox = gx - 13;
        for &(x, y) in &[(8, 10), (9, 10), (16, 10), (17, 10)] {
            fb.set(ox + x, y, col);
        }
    }

    fn draw_flasks(&self, ctx: &Ctx, fb: &mut Bitmap) {
        let p = &self.game.player;
        let vit_ramp = if p.poison > 0.0 { Ramp::Green } else { Ramp::Red };
        for (k, (fx, fy)) in [FLASK_VIT, FLASK_MANA].iter().enumerate() {
            let glass = &ctx.art.ui.flask_glass;
            let ins = &ctx.art.ui.flask_inside;
            let ox = fx - glass.w / 2;
            let oy = fy - 31;
            let (frac, ramp) = if k == 0 { (p.vit as f32 / p.max_vit.max(1) as f32, vit_ramp) } else { (p.mana as f32 / p.max_mana.max(1) as f32, Ramp::Blue) };
            let top = oy as f32 + 34.0 - frac.clamp(0.0, 1.0) * 29.0;
            let wobble = (self.game.time as f32 * 3.0).sin() * 0.6;
            for y in 0..ins.h {
                for x in 0..ins.w {
                    if ins.get(x, y) != 0 {
                        let surface = top + if x % 4 < 2 { wobble } else { -wobble } * 0.5;
                        let filled = (oy + y) as f32 >= surface;
                        let col = if filled {
                            let edge = x < 9 || ins.get(x - 2, y) == 0;
                            c(ramp, if edge { 11 } else if (oy + y) as f32 <= surface + 1.0 { 12 } else { 8 })
                        } else {
                            c(Ramp::Slate, 1 + ((x + y) % 3 == 0) as i32)
                        };
                        fb.set(ox + x, oy + y, col);
                    }
                }
            }
            fb.blit(glass, ox, oy);
        }
        // pull chain
        fb.blit(&ctx.art.ui.chain_bmp, CHAIN.x, CHAIN.y);
    }

    fn draw_shelf(&self, ctx: &Ctx, fb: &mut Bitmap) {
        let shelf = &self.game.player.shelf;
        for (i, r) in shelf.iter().enumerate() {
            let x = RUNE_SHELF.x + i as i32 * RUNE_SHELF.w / 3 + 2;
            let y = RUNE_SHELF.y + 2;
            draw_rune_tile(fb, ctx, *r, x, y, 11, 11);
        }
        if self.ui.targeting.is_some() {
            fb.rect(RUNE_SHELF.x, RUNE_SHELF.y, RUNE_SHELF.w, RUNE_SHELF.h, c(Ramp::Blue, 12));
        }
    }

    fn draw_spells(&self, ctx: &Ctx, fb: &mut Bitmap) {
        for (i, a) in self.game.player.spells.iter().take(3).enumerate() {
            let x = SPELLS.x + 2 + i as i32 * 15;
            let y = SPELLS.y + 2;
            let r = a.spell.def().runes[0];
            // A glowing sun-like disc with the spell's first rune.
            fb.fill_circle_f(x as f32 + 5.5, y as f32 + 5.5, 5.5, c(Ramp::Gold, 6));
            fb.fill_circle_f(x as f32 + 5.5, y as f32 + 5.5, 4.0, c(Ramp::Gold, 11));
            let g = crate::art::items::rune_glyph(r as usize);
            fb.blit_mapped(&g, x + 3, y + 2, &solid_map(c(Ramp::Copper, 3)));
            let _ = ctx;
        }
    }

    fn draw_messages(&self, ctx: &Ctx, fb: &mut Bitmap) {
        let msgs = &self.game.messages;
        // Wrap all recent messages into lines.
        let mut lines: Vec<(String, MsgKind)> = Vec::new();
        for m in msgs.iter().rev().take(30).collect::<Vec<_>>().into_iter().rev() {
            for l in ctx.art.font.wrap(&m.text, SCROLL.w - 6) {
                lines.push((l, m.kind));
            }
        }
        let n = SCROLL_LINES as usize;
        let end = lines.len().saturating_sub(self.ui.msg_back);
        let start = end.saturating_sub(n);
        for (i, (l, kind)) in lines[start..end].iter().enumerate() {
            let col = match kind {
                MsgKind::System => MSG_GREEN,
                MsgKind::Speech => palette::INK_BLUE,
                MsgKind::Normal => INK,
            };
            ctx.art.font.draw(fb, SCROLL_TEXT_X, SCROLL_TEXT_Y + i as i32 * 6, l, col);
        }
    }

    fn draw_cursor(&self, ctx: &mut Ctx, fb: &mut Bitmap, m: (i32, i32)) {
        if let Some(it) = &self.ui.held {
            let b = ctx.cache.icon_mut(it).clone();
            fb.blit(&b, m.0 - b.w / 2, m.1 - b.h / 2);
            return;
        }
        let cur = if self.ui.targeting.is_some() {
            if self.ui.targeting.map(|s| s.def().target == Targeting::Combat).unwrap_or(false) { Cursor::Target } else { Cursor::Spell }
        } else if VIEW.contains(m) && !matches!(self.overlay, Overlay::Conversation(..) | Overlay::Text(..)) {
            self.ui.cursor_dir
        } else {
            Cursor::Cross
        };
        let (b, hs) = &ctx.art.ui.cursors[cur as usize];
        fb.blit(b, m.0 - hs.0, m.1 - hs.1);
    }
}

pub fn solid_map(col: u8) -> [u8; 256] {
    let mut m = [col; 256];
    m[0] = 0;
    m
}

/// A rune on a dark stone tile (rune bag, shelf).
pub fn draw_rune_tile(fb: &mut Bitmap, _ctx: &Ctx, r: u8, x: i32, y: i32, w: i32, h: i32) {
    fb.fill_rect(x, y, w, h, c(Ramp::Slate, 4));
    fb.hline(x, x + w - 1, y, c(Ramp::Slate, 8));
    fb.vline(x, y, y + h - 1, c(Ramp::Slate, 8));
    fb.hline(x, x + w - 1, y + h - 1, c(Ramp::Slate, 1));
    fb.vline(x + w - 1, y, y + h - 1, c(Ramp::Slate, 1));
    let g = crate::art::items::rune_glyph(r as usize);
    fb.blit_mapped(&g, x + (w - 5) / 2, y + (h - 7) / 2 + 1, &solid_map(c(Ramp::Gold, 12)));
    let _ = BLACK;
}

/// Door panels as dynamic polygons.
pub fn door_polys(lv: &crate::game::Level) -> Vec<Poly> {
    use crate::art::textures::Tex;
    use crate::game::{DoorAxis, DoorKind};
    let mut out = Vec::new();
    for d in &lv.doors {
        let t = lv.map.get(d.tx, d.tz);
        let floor = lv.map.floor_at(d.tx as f32 + 0.5, d.tz as f32 + 0.5);
        let ceil = t.ceil_h();
        let height = (ceil - floor).min(1.15);
        let tex = d.tex.unwrap_or(match d.kind {
            DoorKind::Wood => Tex::DoorWood,
            DoorKind::Iron => Tex::DoorIron,
            DoorKind::Portcullis => Tex::DoorPortcullis,
            DoorKind::Secret => t.wall_tex,
        });
        let light = t.light as f32;
        let id = render::ID_DOOR | d.id;
        // Local frame: door spans u in [0,1] along `along`, thickness along `across`.
        let (origin, along, across) = match d.axis {
            DoorAxis::NS => ((d.tx as f32, d.tz as f32 + 0.5), (1.0, 0.0), (0.0, 1.0)),
            DoorAxis::EW => ((d.tx as f32 + 0.5, d.tz as f32), (0.0, 1.0), (1.0, 0.0)),
        };
        let thick = 0.06;
        let mut lift = 0.0;
        let mut ang = 0.0f32;
        if d.kind == DoorKind::Portcullis {
            lift = d.open * height * 0.95;
        } else {
            ang = d.open * std::f32::consts::FRAC_PI_2 * 0.97;
        }
        let (s, c0) = ang.sin_cos();
        // Rotate around the hinge at u = 0.
        let pt = |u: f32, w: f32| -> (f32, f32) {
            let lu = u * c0 - w * s;
            let lw = u * s + w * c0;
            (origin.0 + along.0 * lu + across.0 * lw, origin.1 + along.1 * lu + across.1 * lw)
        };
        let y0 = floor + lift;
        let y1 = floor + height + lift;
        let corners = [pt(0.02, -thick / 2.0), pt(0.98, -thick / 2.0), pt(0.98, thick / 2.0), pt(0.02, thick / 2.0)];
        let mk = |p: (f32, f32), y: f32, u: f32| WVert { p: V3::new(p.0, y, p.1), u, v: (floor + height - y + lift) / height.max(0.1), l: light };
        let mut quad = |a: (f32, f32), b: (f32, f32), ua: f32, ub: f32| {
            out.push(Poly { v: [mk(a, y1, ua), mk(b, y1, ub), mk(b, y0, ub), mk(a, y0, ua)], n: 4, tex, two_sided: false, fullbright: false, id });
        };
        // Faces (wound clockwise when seen from outside).
        quad(corners[0], corners[1], 0.0, 1.0);
        quad(corners[2], corners[3], 1.0, 0.0);
        quad(corners[1], corners[2], 0.0, 0.06);
        quad(corners[3], corners[0], 0.0, 0.06);
        // Lintel above the door to the ceiling.
        if ceil > floor + height + 0.01 && d.kind != DoorKind::Portcullis || ceil > floor + height + 0.01 {
            let a = (origin.0, origin.1);
            let b = (origin.0 + along.0, origin.1 + along.1);
            let wt = t.wall_tex;
            for (p, q) in [(a, b), (b, a)] {
                let mkw = |pp: (f32, f32), y: f32, u: f32| WVert { p: V3::new(pp.0, y, pp.1), u, v: -y, l: light };
                let (u0, u1) = (0.0, 1.0);
                out.push(Poly { v: [mkw(p, ceil, u0), mkw(q, ceil, u1), mkw(q, floor + height, u1), mkw(p, floor + height, u0)], n: 4, tex: wt, two_sided: false, fullbright: false, id: 0 });
            }
        }
    }
    // Fix winding: we can't easily know orientation for rotated boxes, so emit
    // every door face as two-sided.
    for p in out.iter_mut() {
        p.two_sided = true;
    }
    out
}
