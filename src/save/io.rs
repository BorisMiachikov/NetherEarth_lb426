use std::path::{Path, PathBuf};

use super::types::SaveData;

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
        .map_err(|e| format!("Ошибка сериализации: {e}"))?;
    std::fs::write(path, content).map_err(|e| format!("Ошибка записи файла: {e}"))
}

pub fn read_save(path: &Path) -> Result<SaveData, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Не удалось прочитать файл сохранения: {e}"))?;
    ron::from_str(&content).map_err(|e| format!("Ошибка парсинга сохранения: {e}"))
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
            .expect("сериализация");
        let loaded: SaveData = ron::from_str(&ron_str).expect("десериализация");
        assert_eq!(loaded.game_day, 5);
        assert_eq!(loaded.resources.general, 50);
        assert!((loaded.day_elapsed - 10.0).abs() < 1e-4);
    }
}
