//! Saving and restoring games (JSON files in ~/.ultima-j/).

use super::play::PlayState;
use super::Ctx;
use crate::game::Game;
use std::path::PathBuf;

fn dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".ultima-j")
}

fn path(slot: usize) -> PathBuf {
    dir().join(format!("save{}.json", slot + 1))
}

pub fn save(g: &Game, slot: usize) -> Result<(), String> {
    std::fs::create_dir_all(dir()).map_err(|e| e.to_string())?;
    let json = serde_json::to_string(g).map_err(|e| e.to_string())?;
    std::fs::write(path(slot), json).map_err(|e| e.to_string())
}

pub fn load(slot: usize) -> Result<Game, String> {
    let s = std::fs::read_to_string(path(slot)).map_err(|_| "no saved game in that slot".to_string())?;
    serde_json::from_str(&s).map_err(|e| e.to_string())
}

pub fn slot_exists(slot: usize) -> bool {
    path(slot).exists()
}

/// The most recently written slot (0 if none).
pub fn latest_slot() -> usize {
    (0..4)
        .filter_map(|i| std::fs::metadata(path(i)).ok().and_then(|m| m.modified().ok()).map(|t| (i, t)))
        .max_by_key(|(_, t)| *t)
        .map(|(i, _)| i)
        .unwrap_or(0)
}

pub fn describe(slot: usize) -> Option<String> {
    let g = load(slot).ok()?;
    Some(format!("{}, level {} {}, day {}", g.player.name, g.player.level, g.player.class.name(), g.day()))
}

pub fn quick_save(st: &mut PlayState, _ctx: &mut Ctx) {
    match save(&st.game, 0) {
        Ok(()) => st.game.sys("Save game succeeded."),
        Err(e) => st.game.sys(format!("Save failed: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use crate::game::{Game, PClass, Player};

    #[test]
    fn game_state_round_trips_through_json() {
        let mut p = Player::new("Saver", PClass::Mage, true);
        crate::ui::menus::starting_kit(&mut p);
        let mut g = Game::new(p, 3);
        g.set_flag("rhugg_open", 1);
        g.player.runes[5] = true;
        g.levels[0].notes.push((3, 4, "hello".into()));
        let json = serde_json::to_string(&g).expect("serialize");
        let back: Game = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.levels.len(), g.levels.len());
        assert_eq!(back.player.name, "Saver");
        assert!(back.player.female);
        assert_eq!(back.flag("rhugg_open"), 1);
        assert!(back.player.runes[5]);
        assert_eq!(back.levels[0].notes.len(), 1);
        for (a, b) in g.levels.iter().zip(back.levels.iter()) {
            assert_eq!(a.items.len(), b.items.len());
            assert_eq!(a.npcs.len(), b.npcs.len());
            assert_eq!(a.doors.len(), b.doors.len());
            assert_eq!(a.triggers.len(), b.triggers.len());
        }
        // A restored game keeps running.
        let mut back = back;
        back.update(&crate::game::MoveInput { forward: 1.0, ..Default::default() }, 0.1);
    }
}
