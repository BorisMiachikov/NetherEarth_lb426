use nether_earth::save::{
    io::{read_save, write_save},
    types::{SaveData, SavedAI, SavedResources, SAVE_VERSION},
};

fn dummy_save() -> SaveData {
    SaveData {
        version: SAVE_VERSION,
        game_day: 7,
        day_elapsed: 15.5,
        seconds_per_day: 30.0,
        resources: SavedResources {
            general: 100,
            chassis: 40,
            cannon: 30,
            missile: 20,
            phasers: 20,
            electronics: 15,
            nuclear: 8,
        },
        scout_position: [16.0, 3.0, 16.0],
        robots: vec![],
        factories: vec![],
        warbases: vec![],
        ai: SavedAI::default(),
    }
}

#[test]
fn file_roundtrip_preserves_all_fields() {
    let original = dummy_save();
    let path = std::env::temp_dir().join("ne_integration_roundtrip.ron");
    write_save(&path, &original).expect("write failed");
    let loaded = read_save(&path).expect("read failed");
    let _ = std::fs::remove_file(&path);

    assert_eq!(loaded.version, original.version);
    assert_eq!(loaded.game_day, original.game_day);
    assert!((loaded.day_elapsed - original.day_elapsed).abs() < 1e-4);
    assert_eq!(loaded.resources.general, original.resources.general);
    assert_eq!(loaded.resources.nuclear, original.resources.nuclear);
    assert_eq!(loaded.scout_position, original.scout_position);
}

#[test]
fn corrupt_file_returns_error() {
    let path = std::env::temp_dir().join("ne_integration_corrupt.ron");
    std::fs::write(&path, b"not valid ron at all {{").expect("write");
    let result = read_save(&path);
    let _ = std::fs::remove_file(&path);
    assert!(result.is_err(), "Expected Err for corrupt file");
}

#[test]
fn missing_file_returns_error() {
    let path = std::env::temp_dir().join("ne_integration_nonexistent_zzz.ron");
    let _ = std::fs::remove_file(&path);
    let result = read_save(&path);
    assert!(result.is_err(), "Expected Err for missing file");
}

#[test]
fn future_version_rejected() {
    let mut data = dummy_save();
    data.version = SAVE_VERSION + 1;
    let path = std::env::temp_dir().join("ne_integration_future_ver.ron");
    write_save(&path, &data).expect("write");
    let result = read_save(&path);
    let _ = std::fs::remove_file(&path);
    assert!(result.is_err(), "Expected Err for future version");
    let msg = result.unwrap_err();
    assert!(msg.contains("newer than supported"), "Error message: {msg}");
}

#[test]
fn current_version_accepted() {
    let data = dummy_save();
    assert_eq!(data.version, SAVE_VERSION);
    let path = std::env::temp_dir().join("ne_integration_current_ver.ron");
    write_save(&path, &data).expect("write");
    let result = read_save(&path);
    let _ = std::fs::remove_file(&path);
    assert!(result.is_ok(), "Expected Ok for current version: {:?}", result);
}
