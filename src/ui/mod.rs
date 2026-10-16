//! User interface: screens, the main game screen and overlays.

pub mod automap;
pub mod convo;
pub mod menus;
pub mod overlay;
pub mod panel;
pub mod play;
pub mod save;
pub mod sprites;

use crate::art::Art;
use crate::audio::Sfx;
use crate::game::dialogue::{ConvState, DialogueDb};
use crate::game::{Game, Player};
use crate::gfx::{Bitmap, Palette};
use crate::platform::{App, AppCmd, Input, Key};
use crate::render::Renderer;
use play::PlayState;
use sprites::SpriteCache;

pub struct Ctx {
    pub art: Art,
    pub cache: SpriteCache,
    pub renderer: Renderer,
    pub db: DialogueDb,
    pub sounds: Vec<Sfx>,
    pub time: f32,
    pub sound_on: bool,
    pub music_on: bool,
    /// Modern key layout (WASD, number row for commands) instead of the original's.
    pub modern_controls: bool,
}

pub enum Overlay {
    None,
    /// Options menu with the selected line.
    Options(usize),
    /// Save (true) or restore (false) slot picker.
    Slots(bool, usize),
    /// Automap of a level.
    Map(usize),
    /// Conversation state and the current text page.
    Conversation(Box<ConvState>, usize),
    /// A text to read and the current page.
    Text(String, usize),
    Sleep(f32),
    Dead(f32),
    Victory(f32),
}

pub enum ScreenCmd {
    Quit,
    Title,
    NewGame(Box<Player>),
    LoadSlot(usize),
    CharGen,
    Intro,
    Credits,
}

pub enum Screen {
    Title(menus::Title),
    Intro(menus::Intro),
    CharGen(menus::CharGen),
    Credits(f32),
    Play(Box<PlayState>),
}

pub struct GameApp {
    pub ctx: Ctx,
    pub screen: Screen,
    mouse: (i32, i32),
}

impl GameApp {
    pub fn new() -> GameApp {
        let art = Art::generate();
        let ctx = Ctx {
            art,
            cache: SpriteCache::new(),
            renderer: Renderer::new(72.0, 1.2),
            db: DialogueDb::load(),
            sounds: Vec::new(),
            time: 0.0,
            sound_on: true,
            music_on: true,
            modern_controls: false,
        };
        GameApp { ctx, screen: Screen::Title(menus::Title::new()), mouse: (160, 100) }
    }

    /// Start directly in the game with a default character (for testing/screenshots).
    pub fn new_playing(seed: u64) -> GameApp {
        let mut app = GameApp::new();
        let mut p = Player::new("Avatar", crate::game::PClass::Fighter, false);
        menus::starting_kit(&mut p);
        let game = Game::new(p, seed);
        app.screen = Screen::Play(Box::new(PlayState::new(game)));
        app.start_messages();
        app
    }

    fn start_messages(&mut self) {
        if let Screen::Play(st) = &mut self.screen {
            st.game.sys("Welcome to the Abyss.");
            st.game.msg("You claw your way out of a shallow grave. You are cold, and grey, and dead.");
            st.game.sys("(Press H for help on the controls.)");
            st.game.explore();
        }
    }

    fn apply(&mut self, cmd: ScreenCmd) -> AppCmd {
        match cmd {
            ScreenCmd::Quit => return AppCmd::Quit,
            ScreenCmd::Title => self.screen = Screen::Title(menus::Title::new()),
            ScreenCmd::CharGen => self.screen = Screen::CharGen(menus::CharGen::new()),
            ScreenCmd::Intro => self.screen = Screen::Intro(menus::Intro::new()),
            ScreenCmd::Credits => self.screen = Screen::Credits(0.0),
            ScreenCmd::NewGame(p) => {
                let seed = (self.ctx.time * 1000.0) as u64 ^ 0x5EED;
                let game = Game::new(*p, seed);
                self.screen = Screen::Play(Box::new(PlayState::new(game)));
                self.start_messages();
            }
            ScreenCmd::LoadSlot(slot) => match save::load(slot) {
                Ok(game) => {
                    let mut st = PlayState::new(game);
                    st.game.sys("Game restored.");
                    self.screen = Screen::Play(Box::new(st));
                }
                Err(e) => {
                    if let Screen::Play(st) = &mut self.screen {
                        st.game.sys(format!("Could not restore: {}", e));
                    } else if let Screen::Title(t) = &mut self.screen {
                        t.note = Some("No saved game found.".into());
                    }
                }
            },
        }
        AppCmd::None
    }
}

impl Default for GameApp {
    fn default() -> Self {
        Self::new()
    }
}

impl App for GameApp {
    fn update(&mut self, input: &Input, dt: f32) -> AppCmd {
        self.ctx.time += dt;
        self.mouse = input.mouse;
        if input.hit(Key::F11) || (input.ctrl && input.hit(Key::F)) {
            return AppCmd::ToggleFullscreen;
        }
        if input.hit(Key::F12) {
            return AppCmd::ToggleAspect;
        }
        let cmd = match &mut self.screen {
            Screen::Title(t) => t.update(&mut self.ctx, input, dt),
            Screen::Intro(i) => i.update(&mut self.ctx, input, dt),
            Screen::CharGen(c) => c.update(&mut self.ctx, input, dt),
            Screen::Credits(t) => {
                *t += dt;
                if (*t > 1.0 && (input.any_click() || input.any_key())) || *t > 60.0 {
                    Some(ScreenCmd::Title)
                } else {
                    None
                }
            }
            Screen::Play(st) => st.update(&mut self.ctx, input, dt),
        };
        match cmd {
            Some(c) => self.apply(c),
            None => AppCmd::None,
        }
    }

    fn draw(&mut self, fb: &mut Bitmap) -> Option<([f32; 3], f32)> {
        match &mut self.screen {
            Screen::Title(t) => t.draw(&mut self.ctx, fb, self.mouse),
            Screen::Intro(i) => i.draw(&mut self.ctx, fb),
            Screen::CharGen(c) => c.draw(&mut self.ctx, fb, self.mouse),
            Screen::Credits(t) => menus::draw_credits(&mut self.ctx, fb, *t),
            Screen::Play(st) => {
                st.draw(&mut self.ctx, fb, self.mouse);
                return st.ui.flash.map(|(c, a)| (c, a.min(0.6)));
            }
        }
        None
    }

    fn palette(&self) -> &Palette {
        &self.ctx.art.pal
    }

    fn set_aspect(&mut self, corrected: bool) {
        self.ctx.renderer.set_pixel_aspect(if corrected { 1.2 } else { 1.0 });
    }

    fn music(&self) -> Option<u8> {
        if !self.ctx.music_on {
            return None;
        }
        match &self.screen {
            Screen::Play(st) if matches!(st.overlay, Overlay::Dead(_)) => None,
            Screen::Play(_) => Some(1),
            _ => Some(0),
        }
    }

    fn take_sounds(&mut self) -> Vec<Sfx> {
        let s = std::mem::take(&mut self.ctx.sounds);
        if self.ctx.sound_on { s } else { Vec::new() }
    }
}
