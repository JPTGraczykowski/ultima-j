//! Platform layer: input snapshot types and the window/presentation loop.
//! The game itself only sees `Input` and draws into an indexed framebuffer,
//! so it can also run headless (for automated screenshots and tests).

use crate::audio::{Sfx, SoundBank};
use crate::gfx::{Bitmap, Palette, SCREEN_H, SCREEN_W};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Key {
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    Up, Down, Left, Right,
    Enter, Escape, Space, Tab, Backspace, PageUp, PageDown, Home, End,
    Period, Semicolon, Comma, Minus, Equal, Slash,
}

#[derive(Clone, Default, Debug)]
pub struct Input {
    /// Mouse position in 320x200 screen coordinates.
    pub mouse: (i32, i32),
    pub down: [bool; 3],
    pub pressed: [bool; 3],
    pub released: [bool; 3],
    pub keys_down: Vec<Key>,
    pub keys_pressed: Vec<Key>,
    pub chars: Vec<char>,
    pub shift: bool,
    pub ctrl: bool,
}

impl Input {
    pub fn key(&self, k: Key) -> bool {
        self.keys_down.contains(&k)
    }
    pub fn hit(&self, k: Key) -> bool {
        self.keys_pressed.contains(&k)
    }
    pub fn any_key(&self) -> bool {
        !self.keys_pressed.is_empty()
    }
    pub fn any_click(&self) -> bool {
        self.pressed[0] || self.pressed[1]
    }
}

/// What the app asks the platform to do after a frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AppCmd {
    None,
    Quit,
    ToggleFullscreen,
    ToggleAspect,
}

pub trait App {
    fn update(&mut self, input: &Input, dt: f32) -> AppCmd;
    /// Draw into the 320x200 framebuffer. Returns an optional full-screen
    /// colour tint (r, g, b, amount) for damage/drowning flashes.
    fn draw(&mut self, fb: &mut Bitmap) -> Option<([f32; 3], f32)>;
    fn palette(&self) -> &Palette;
    fn take_sounds(&mut self) -> Vec<Sfx>;
    /// Called when the display aspect correction changes.
    fn set_aspect(&mut self, _corrected: bool) {}
    /// Which music track should be playing (None = silence).
    fn music(&self) -> Option<u8> {
        None
    }
}

/// Run the app in a window until it quits.
pub fn run_window(mut app: Box<dyn App>) {
    use macroquad::prelude as mq;
    let conf = mq::Conf {
        window_title: "Ultima-J: The Abyss Below".to_owned(),
        window_width: 960,
        window_height: 720,
        high_dpi: false,
        window_resizable: true,
        ..Default::default()
    };
    macroquad::Window::from_config(conf, async move {
        mq::show_mouse(false);
        let mut bank = SoundBank::load().await;
        let mut fb = Bitmap::new(SCREEN_W, SCREEN_H);
        let mut image = mq::Image::gen_image_color(SCREEN_W as u16, SCREEN_H as u16, mq::BLACK);
        let texture = mq::Texture2D::from_image(&image);
        texture.set_filter(mq::FilterMode::Nearest);
        let mut aspect_correct = true;
        let mut fullscreen = false;
        let mut prev_down = [false; 3];
        // Self-test: save the presented frame after a few frames and quit.
        let selftest = std::env::var("ULTIMA_J_SELFTEST").ok();
        let mut frames = 0u32;
        loop {
            frames += 1;
            let sw = mq::screen_width();
            let sh = mq::screen_height();
            let target_aspect = if aspect_correct { 4.0 / 3.0 } else { SCREEN_W as f32 / SCREEN_H as f32 };
            let (dw, dh) = if sw / sh > target_aspect { (sh * target_aspect, sh) } else { (sw, sw / target_aspect) };
            let dx = (sw - dw) / 2.0;
            let dy = (sh - dh) / 2.0;

            // Gather input.
            let (mx, my) = mq::mouse_position();
            let fx = ((mx - dx) / dw * SCREEN_W as f32).floor() as i32;
            let fy = ((my - dy) / dh * SCREEN_H as f32).floor() as i32;
            let buttons = [mq::MouseButton::Left, mq::MouseButton::Right, mq::MouseButton::Middle];
            let mut input = Input { mouse: (fx.clamp(0, SCREEN_W - 1), fy.clamp(0, SCREEN_H - 1)), ..Default::default() };
            for (i, b) in buttons.iter().enumerate() {
                let d = mq::is_mouse_button_down(*b);
                input.down[i] = d;
                input.pressed[i] = mq::is_mouse_button_pressed(*b) || (d && !prev_down[i]);
                input.released[i] = mq::is_mouse_button_released(*b) || (!d && prev_down[i]);
                prev_down[i] = d;
            }
            for (mk, k) in KEYMAP {
                if mq::is_key_down(*mk) {
                    input.keys_down.push(*k);
                }
                if mq::is_key_pressed(*mk) {
                    input.keys_pressed.push(*k);
                }
            }
            input.shift = mq::is_key_down(mq::KeyCode::LeftShift) || mq::is_key_down(mq::KeyCode::RightShift);
            input.ctrl = mq::is_key_down(mq::KeyCode::LeftControl)
                || mq::is_key_down(mq::KeyCode::RightControl)
                || mq::is_key_down(mq::KeyCode::LeftSuper)
                || mq::is_key_down(mq::KeyCode::RightSuper);
            while let Some(ch) = mq::get_char_pressed() {
                if !ch.is_control() {
                    input.chars.push(ch);
                }
            }

            let dt = mq::get_frame_time().min(0.1);
            match app.update(&input, dt) {
                AppCmd::Quit => break,
                AppCmd::ToggleFullscreen => {
                    fullscreen = !fullscreen;
                    mq::set_fullscreen(fullscreen);
                }
                AppCmd::ToggleAspect => {
                    aspect_correct = !aspect_correct;
                    app.set_aspect(aspect_correct);
                }
                AppCmd::None => {}
            }
            for s in app.take_sounds() {
                bank.play(s);
            }
            bank.set_music(app.music());

            let tint = app.draw(&mut fb);
            app.palette().to_rgba(&fb.px, &mut image.bytes, tint);
            texture.update(&image);
            mq::clear_background(mq::BLACK);
            mq::draw_texture_ex(
                &texture,
                dx,
                dy,
                mq::WHITE,
                mq::DrawTextureParams { dest_size: Some(mq::vec2(dw, dh)), ..Default::default() },
            );
            if let Some(path) = &selftest {
                if frames == 90 {
                    let img = mq::get_screen_data();
                    let (w, h) = (img.width as u32, img.height as u32);
                    // Screen data is bottom-up; flip it.
                    let mut flipped = vec![0u8; img.bytes.len()];
                    let row = (w * 4) as usize;
                    for y in 0..h as usize {
                        flipped[y * row..(y + 1) * row].copy_from_slice(&img.bytes[(h as usize - 1 - y) * row..(h as usize - y) * row]);
                    }
                    let _ = crate::gfx::save_png_rgba(path, w, h, &flipped);
                    break;
                }
            }
            mq::next_frame().await;
        }
    });
}

use macroquad::prelude::KeyCode as MK;
const KEYMAP: &[(MK, Key)] = &[
    (MK::A, Key::A), (MK::B, Key::B), (MK::C, Key::C), (MK::D, Key::D), (MK::E, Key::E),
    (MK::F, Key::F), (MK::G, Key::G), (MK::H, Key::H), (MK::I, Key::I), (MK::J, Key::J),
    (MK::K, Key::K), (MK::L, Key::L), (MK::M, Key::M), (MK::N, Key::N), (MK::O, Key::O),
    (MK::P, Key::P), (MK::Q, Key::Q), (MK::R, Key::R), (MK::S, Key::S), (MK::T, Key::T),
    (MK::U, Key::U), (MK::V, Key::V), (MK::W, Key::W), (MK::X, Key::X), (MK::Y, Key::Y),
    (MK::Z, Key::Z),
    (MK::Key0, Key::Num0), (MK::Key1, Key::Num1), (MK::Key2, Key::Num2), (MK::Key3, Key::Num3),
    (MK::Key4, Key::Num4), (MK::Key5, Key::Num5), (MK::Key6, Key::Num6), (MK::Key7, Key::Num7),
    (MK::Key8, Key::Num8), (MK::Key9, Key::Num9),
    (MK::F1, Key::F1), (MK::F2, Key::F2), (MK::F3, Key::F3), (MK::F4, Key::F4), (MK::F5, Key::F5),
    (MK::F6, Key::F6), (MK::F7, Key::F7), (MK::F8, Key::F8), (MK::F9, Key::F9), (MK::F10, Key::F10),
    (MK::F11, Key::F11), (MK::F12, Key::F12),
    (MK::Up, Key::Up), (MK::Down, Key::Down), (MK::Left, Key::Left), (MK::Right, Key::Right),
    (MK::Enter, Key::Enter), (MK::KpEnter, Key::Enter), (MK::Escape, Key::Escape), (MK::Space, Key::Space),
    (MK::Tab, Key::Tab), (MK::Backspace, Key::Backspace), (MK::PageUp, Key::PageUp),
    (MK::PageDown, Key::PageDown), (MK::Home, Key::Home), (MK::End, Key::End),
    (MK::Period, Key::Period), (MK::Semicolon, Key::Semicolon), (MK::Comma, Key::Comma),
    (MK::Minus, Key::Minus), (MK::Equal, Key::Equal), (MK::Slash, Key::Slash),
];
