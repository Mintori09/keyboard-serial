use keyboard_rs::types::{
    FIXED_KEYS, MacroConfig, MacroEntry, ProfileConfig, load_config_from_path, normalize_config,
    save_config_to_path, validate_config_basic,
};
use std::path::PathBuf;

fn fixed(commands_prefix: &str) -> Vec<MacroEntry> {
    FIXED_KEYS
        .iter()
        .map(|k| MacroEntry {
            key: (*k).to_string(),
            command: format!("{}-{}", commands_prefix, k),
        })
        .collect()
}

fn sample_config() -> MacroConfig {
    MacroConfig {
        active_profile: 0,
        profiles: vec![
            ProfileConfig {
                name: "Normal".to_string(),
                press_macros: fixed("p1"),
                hold_macros: fixed("h1"),
            },
            ProfileConfig {
                name: "Anki".to_string(),
                press_macros: fixed("p2"),
                hold_macros: fixed("h2"),
            },
        ],
    }
}

fn temp_path(name: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("keyboard-rs-test-{}-{}", std::process::id(), name));
    p
}

#[test]
fn config_roundtrip_preserves_profiles() {
    let path = temp_path("roundtrip.json");
    let cfg = sample_config();

    save_config_to_path(&path, &cfg).expect("save should succeed");
    let loaded = load_config_from_path(&path).expect("load should succeed");

    assert_eq!(loaded.profiles.len(), 2);
    assert_eq!(loaded.profiles[0].press_macros.len(), FIXED_KEYS.len());
    assert_eq!(loaded.profiles[0].hold_macros.len(), FIXED_KEYS.len());

    let _ = std::fs::remove_file(path);
}

#[test]
fn validate_rejects_empty_profile_name() {
    let mut cfg = sample_config();
    cfg.profiles[0].name = "".to_string();
    assert!(validate_config_basic(&cfg).is_err());
}

#[test]
fn normalize_fills_missing_fixed_keys() {
    let cfg = MacroConfig {
        active_profile: 0,
        profiles: vec![ProfileConfig {
            name: "Only".to_string(),
            press_macros: vec![MacroEntry {
                key: "1".to_string(),
                command: "x".to_string(),
            }],
            hold_macros: vec![],
        }],
    };

    let normalized = normalize_config(&cfg);
    assert_eq!(normalized.profiles[0].press_macros.len(), FIXED_KEYS.len());
    assert_eq!(normalized.profiles[0].hold_macros.len(), FIXED_KEYS.len());
}
