//! Saving and restoring games (gzipped JSON files in ~/.ultima-j/), and the
//! settings kept between launches (settings.json there).

use super::play::PlayState;
use super::Ctx;
use crate::game::Game;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::{Read, Write};
use std::path::PathBuf;

fn dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".ultima-j")
}

fn path(slot: usize) -> PathBuf {
    dir().join(format!("save{}.json.gz", slot + 1))
}

/// Saves from before compression; still loaded, and removed on the next save to the slot.
fn legacy_path(slot: usize) -> PathBuf {
    dir().join(format!("save{}.json", slot + 1))
}

fn existing_path(slot: usize) -> Option<PathBuf> {
    [path(slot), legacy_path(slot)].into_iter().find(|p| p.exists())
}

fn to_bytes(g: &Game) -> Result<Vec<u8>, String> {
    let mut enc = GzEncoder::new(Vec::new(), Compression::default());
    serde_json::to_writer(&mut enc, g).map_err(|e| e.to_string())?;
    enc.finish().map_err(|e| e.to_string())
}

/// Reads gzipped JSON, or plain JSON from an old save.
fn from_bytes(bytes: &[u8]) -> Result<Game, String> {
    if bytes.starts_with(&[0x1f, 0x8b]) {
        let mut json = Vec::new();
        GzDecoder::new(bytes).read_to_end(&mut json).map_err(|e| e.to_string())?;
        serde_json::from_slice(&json).map_err(|e| e.to_string())
    } else {
        serde_json::from_slice(bytes).map_err(|e| e.to_string())
    }
}

pub fn save(g: &Game, slot: usize) -> Result<(), String> {
    std::fs::create_dir_all(dir()).map_err(|e| e.to_string())?;
    let bytes = to_bytes(g)?;
    std::fs::File::create(path(slot)).and_then(|mut f| f.write_all(&bytes)).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(legacy_path(slot));
    Ok(())
}

pub fn load(slot: usize) -> Result<Game, String> {
    let p = existing_path(slot).ok_or_else(|| "no saved game in that slot".to_string())?;
    let bytes = std::fs::read(p).map_err(|e| e.to_string())?;
    from_bytes(&bytes)
}

pub fn slot_exists(slot: usize) -> bool {
    existing_path(slot).is_some()
}

/// The most recently written slot (0 if none).
pub fn latest_slot() -> usize {
    (0..4)
        .filter_map(|i| existing_path(i).and_then(|p| std::fs::metadata(p).ok()).and_then(|m| m.modified().ok()).map(|t| (i, t)))
        .max_by_key(|(_, t)| *t)
        .map(|(i, _)| i)
        .unwrap_or(0)
}

pub fn describe(slot: usize) -> Option<String> {
    let g = load(slot).ok()?;
    Some(format!("{}, level {} {}, day {}", g.player.name, g.player.level, g.player.class.name(), g.day()))
}

/// Options remembered between launches.
#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct Settings {
    #[serde(default)]
    pub modern_controls: bool,
}

fn settings_path() -> PathBuf {
    dir().join("settings.json")
}

/// The saved settings, or the defaults if there are none (or they can't be read).
pub fn load_settings() -> Settings {
    std::fs::read(settings_path()).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
}

pub fn save_settings(s: &Settings) -> Result<(), String> {
    std::fs::create_dir_all(dir()).map_err(|e| e.to_string())?;
    let json = serde_json::to_vec_pretty(s).map_err(|e| e.to_string())?;
    std::fs::write(settings_path(), json).map_err(|e| e.to_string())
}

pub fn quick_save(st: &mut PlayState, _ctx: &mut Ctx) {
    match save(&st.game, 0) {
        Ok(()) => st.game.sys("Save game succeeded."),
        Err(e) => st.game.sys(format!("Save failed: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::{from_bytes, to_bytes};
    use crate::game::{Game, PClass, Player};

    #[test]
    fn game_state_round_trips_through_a_save() {
        let mut p = Player::new("Saver", PClass::Mage, true);
        crate::ui::menus::starting_kit(&mut p);
        let mut g = Game::new(p, 3);
        g.set_flag("rhugg_open", 1);
        g.player.runes[5] = true;
        g.levels[0].notes.push((3, 4, "hello".into()));
        let bytes = to_bytes(&g).expect("serialize");
        let back = from_bytes(&bytes).expect("deserialize");
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

    #[test]
    fn settings_round_trip_and_default_when_missing() {
        let s = super::Settings { modern_controls: true };
        let back: super::Settings = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
        assert!(back.modern_controls);
        let empty: super::Settings = serde_json::from_str("{}").unwrap();
        assert!(!empty.modern_controls);
    }

    #[test]
    fn plain_json_saves_still_load() {
        let g = Game::new(Player::new("Old", PClass::Fighter, false), 3);
        let json = serde_json::to_vec(&g).expect("serialize");
        assert_eq!(from_bytes(&json).expect("deserialize").player.name, "Old");
    }
}
