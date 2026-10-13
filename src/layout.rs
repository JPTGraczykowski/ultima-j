//! Screen layout of the main game screen (320x200), measured from the
//! original game's interface.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect { x, y, w, h }
    }
    pub fn contains(&self, p: (i32, i32)) -> bool {
        p.0 >= self.x && p.1 >= self.y && p.0 < self.x + self.w && p.1 < self.y + self.h
    }
}

/// The 3D view window.
pub const VIEW: Rect = Rect::new(52, 19, 172, 109);
/// Left column of command icons.
pub const ICON_PANEL: Rect = Rect::new(1, 4, 38, 117);
/// Command icon centres: options, talk, get, look, fight, use.
pub const ICON_CENTERS: [(i32, i32); 6] = [(20, 16), (20, 35), (19, 53), (19, 72), (21, 89), (21, 108)];
pub const ICON_W: i32 = 30;
pub const ICON_H: i32 = 19;
/// Character panel (inventory / stats / runes).
pub const PANEL: Rect = Rect::new(236, 4, 83, 116);
/// Message scroll text area.
pub const SCROLL: Rect = Rect::new(14, 168, 292, 31);
pub const SCROLL_LINES: i32 = 5;
pub const SCROLL_TEXT_X: i32 = 16;
pub const SCROLL_TEXT_Y: i32 = 169;
/// Compass rose centre and radius.
pub const COMPASS_C: (i32, i32) = (137, 143);
pub const COMPASS_R: f32 = 29.0;
/// Easy-move arrow buttons on the compass base: turn left, forward, turn right.
pub const MOVE_BUTTONS: [Rect; 3] = [Rect::new(111, 154, 12, 11), Rect::new(132, 155, 11, 11), Rect::new(152, 154, 12, 11)];
/// Rune shelf (3 slots) and current-spell display.
pub const RUNE_SHELF: Rect = Rect::new(174, 137, 45, 15);
pub const SPELLS: Rect = Rect::new(57, 137, 45, 15);
/// Power gem centre.
pub const GEM_C: (i32, i32) = (20, 142);
/// Flasks: bottom-centre of each flask body.
pub const FLASK_VIT: (i32, i32) = (259, 154);
pub const FLASK_MANA: (i32, i32) = (295, 154);
pub const FLASK_R: i32 = 11;
/// Pull chain (switches the character panel).
pub const CHAIN: Rect = Rect::new(271, 117, 11, 27);
/// Gargoyle face (centre).
pub const GARGOYLE_C: (i32, i32) = (136, 9);
/// Inventory slot centres inside the panel: 8 backpack slots (2 rows of 4).
pub const PACK_SLOTS: [(i32, i32); 8] =
    [(251, 90), (270, 90), (289, 90), (308, 90), (251, 108), (270, 108), (289, 108), (308, 108)];
/// Equipment slot centres around the paperdoll.
pub const EQ_SHOULDER_L: (i32, i32) = (251, 18);
pub const EQ_SHOULDER_R: (i32, i32) = (307, 18);
pub const EQ_HAND_L: (i32, i32) = (251, 44);
pub const EQ_HAND_R: (i32, i32) = (307, 44);
pub const EQ_RING_L: (i32, i32) = (262, 60);
pub const EQ_RING_R: (i32, i32) = (296, 60);
pub const PAPERDOLL_C: (i32, i32) = (279, 44);
