use std::path::{Path, PathBuf};

use super::types::{SaveData, SAVE_VERSION};

pub const SAVES_DIR: &str = "saves";
pub const AUTOSAVE_FILE: &str = "saves/autosave.ron";

pub fn slot_path(slot: usize) -> PathBuf {
    PathBuf::from(format!("{SAVES_DIR}/slot_{slot}.ron"))
}

fn ensure_saves_dir() {
    let _ = std::fs::create_dir_all(SAVES_DIR);
}

pub fn write_save(path: &Path, data: &SaveData) -> Result<(), String> {
    ensure_saves_dir();
    let content = ron::ser::to_string_pretty(data, ron::ser::PrettyConfig::default())
        .map_err(|e| format!("Serialization error: {e}"))?;
    std::fs::write(path, content).map_err(|e| format!("Write error: {e}"))
}

pub fn read_save(path: &Path) -> Result<SaveData, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read save file: {e}"))?;
    let data: SaveData = ron::from_str(&content)
        .map_err(|e| format!("Failed to parse save: {e}"))?;
    migrate_save(data)
}

/// Версионирование: проверяет совместимость и применяет миграции.
fn migrate_save(data: SaveData) -> Result<SaveData, String> {
    if data.version > SAVE_VERSION {
        return Err(format!(
            "Save version {} is newer than supported {}",
            data.version, SAVE_VERSION
        ));
    }
    // v1 — текущая версия, миграция не нужна.
    // При добавлении v2: match data.version { 1 => migrate_v1_to_v2(data), _ => Ok(data) }
    Ok(data)
}

pub fn slot_exists(slot: usize) -> bool {
    slot_path(slot).exists()
}

pub fn autosave_exists() -> bool {
    Path::new(AUTOSAVE_FILE).exists()
}

/// Возвращает (game_day, unix_timestamp_сек) для слота, если файл существует.
pub fn slot_info(slot: usize) -> Option<(u32, u64)> {
    let path = slot_path(slot);
    let data = read_save(&path).ok()?;
    let ts = path
        .metadata()
        .ok()?
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    Some((data.game_day, ts))
}

pub fn autosave_info() -> Option<u32> {
    let data = read_save(Path::new(AUTOSAVE_FILE)).ok()?;
    Some(data.game_day)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::save::types::*;

    fn dummy_save() -> SaveData {
        SaveData {
            version: SAVE_VERSION,
            game_day: 5,
            day_elapsed: 10.0,
            seconds_per_day: 30.0,
            resources: SavedResources {
                general: 50,
                chassis: 20,
                cannon: 15,
                missile: 10,
                phasers: 10,
                electronics: 10,
                nuclear: 5,
            },
            scout_position: [32.0, 3.0, 32.0],
            robots: vec![],
            factories: vec![],
            warbases: vec![],
            ai: SavedAI::default(),
        }
    }

    #[test]
    fn serialize_deserialize_roundtrip() {
        let data = dummy_save();
        let ron_str = ron::ser::to_string_pretty(&data, ron::ser::PrettyConfig::default())
            .expect("serialization");
        let loaded: SaveData = ron::from_str(&ron_str).expect("deserialization");
        assert_eq!(loaded.game_day, 5);
        assert_eq!(loaded.resources.general, 50);
        assert!((loaded.day_elapsed - 10.0).abs() < 1e-4);
    }

    #[test]
    fn future_version_rejected() {
        let mut data = dummy_save();
        data.version = SAVE_VERSION + 1;
        let ron_str = ron::ser::to_string_pretty(&data, ron::ser::PrettyConfig::default())
            .expect("serialization");
        let path = std::env::temp_dir().join("ne_test_future_ver.ron");
        std::fs::write(&path, ron_str).expect("write");
        let result = read_save(&path);
        let _ = std::fs::remove_file(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("newer than supported"));
    }

    #[test]
    fn corrupt_ron_rejected() {
        let path = std::env::temp_dir().join("ne_test_corrupt.ron");
        std::fs::write(&path, "this is not valid RON {{{").expect("write");
        let result = read_save(&path);
        let _ = std::fs::remove_file(&path);
        assert!(result.is_err());
    }

    #[test]
    fn file_roundtrip() {
        let data = dummy_save();
        let path = std::env::temp_dir().join("ne_test_roundtrip.ron");
        write_save(&path, &data).expect("write");
        let loaded = read_save(&path).expect("read");
        let _ = std::fs::remove_file(&path);
        assert_eq!(loaded.game_day, data.game_day);
        assert_eq!(loaded.resources.chassis, data.resources.chassis);
        assert_eq!(loaded.version, SAVE_VERSION);
    }
}
