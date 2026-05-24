use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env, fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
    sync::{OnceLock, RwLock},
};

use crate::run_command;

pub const FIXED_KEYS: [&str; 12] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "*", "#"];

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MacroEntry {
    pub key: String,
    pub command: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProfileConfig {
    pub name: String,
    #[serde(default)]
    pub press_macros: Vec<MacroEntry>,
    #[serde(default)]
    pub hold_macros: Vec<MacroEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MacroConfig {
    #[serde(default)]
    pub profiles: Vec<ProfileConfig>,
    #[serde(default)]
    pub active_profile: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum MacroType {
    Press,
    Hold,
}

#[derive(Debug, Deserialize, Clone)]
struct LegacyProfile {
    pub name: String,
    #[serde(default)]
    pub press_set: String,
    #[serde(default)]
    pub press_macros: Vec<MacroEntry>,
    #[serde(default)]
    pub hold_macros: Vec<MacroEntry>,
}

#[derive(Debug, Deserialize, Clone)]
struct LegacyConfig {
    #[serde(default)]
    pub press_macros: Vec<MacroEntry>,
    #[serde(default)]
    pub anki_macros: Vec<MacroEntry>,
    #[serde(default)]
    pub hold_macros: Vec<MacroEntry>,
    #[serde(default)]
    pub profiles: Vec<LegacyProfile>,
}

pub static MACROS: OnceLock<RwLock<MacroConfig>> = OnceLock::new();

pub fn config_file_path() -> PathBuf {
    if let Ok(path) = env::var("KEYBOARD_RS_CONFIG_PATH") {
        return PathBuf::from(path);
    }

    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".config")
        .join("keyboard-rs")
        .join("config.json")
}

fn normalize_macro_entries(entries: &[MacroEntry]) -> Vec<MacroEntry> {
    let mut map: HashMap<String, String> = HashMap::new();
    for entry in entries {
        if entry.key != "_default" {
            map.insert(entry.key.clone(), entry.command.clone());
        }
    }

    FIXED_KEYS
        .iter()
        .map(|key| MacroEntry {
            key: (*key).to_string(),
            command: map.get(*key).cloned().unwrap_or_default(),
        })
        .collect()
}

fn normalize_profile(profile: &ProfileConfig) -> ProfileConfig {
    ProfileConfig {
        name: profile.name.clone(),
        press_macros: normalize_macro_entries(&profile.press_macros),
        hold_macros: normalize_macro_entries(&profile.hold_macros),
    }
}

fn normalize_config_inner(config: &MacroConfig) -> MacroConfig {
    let profiles = config
        .profiles
        .iter()
        .filter(|p| !p.name.trim().is_empty())
        .map(normalize_profile)
        .collect::<Vec<_>>();

    let active_profile = if profiles.is_empty() {
        0
    } else {
        config.active_profile.min(profiles.len().saturating_sub(1))
    };

    MacroConfig {
        profiles,
        active_profile,
    }
}

fn default_profiles() -> Vec<ProfileConfig> {
    vec![
        ProfileConfig {
            name: "Normal".to_string(),
            press_macros: normalize_macro_entries(&[]),
            hold_macros: normalize_macro_entries(&[]),
        },
        ProfileConfig {
            name: "Anki".to_string(),
            press_macros: normalize_macro_entries(&[]),
            hold_macros: normalize_macro_entries(&[]),
        },
    ]
}

fn default_config() -> MacroConfig {
    MacroConfig {
        profiles: default_profiles(),
        active_profile: 0,
    }
}

fn migrate_legacy_config(legacy: LegacyConfig) -> MacroConfig {
    let base_profiles = if legacy.profiles.is_empty() {
        vec![
            LegacyProfile {
                name: "Normal".to_string(),
                press_set: "normal".to_string(),
                press_macros: Vec::new(),
                hold_macros: Vec::new(),
            },
            LegacyProfile {
                name: "Anki".to_string(),
                press_set: "anki".to_string(),
                press_macros: Vec::new(),
                hold_macros: Vec::new(),
            },
        ]
    } else {
        legacy.profiles
    };

    let profiles = base_profiles
        .into_iter()
        .filter(|p| !p.name.trim().is_empty())
        .map(|p| {
            let press_source = if !p.press_macros.is_empty() {
                p.press_macros
            } else if p.press_set.trim().eq_ignore_ascii_case("anki") {
                legacy.anki_macros.clone()
            } else {
                legacy.press_macros.clone()
            };

            let hold_source = if !p.hold_macros.is_empty() {
                p.hold_macros
            } else {
                legacy.hold_macros.clone()
            };

            ProfileConfig {
                name: p.name,
                press_macros: normalize_macro_entries(&press_source),
                hold_macros: normalize_macro_entries(&hold_source),
            }
        })
        .collect::<Vec<_>>();

    if profiles.is_empty() {
        default_config()
    } else {
        MacroConfig {
            profiles,
            active_profile: 0,
        }
    }
}

pub fn normalize_config(config: &MacroConfig) -> MacroConfig {
    let mut normalized = normalize_config_inner(config);
    if normalized.profiles.is_empty() {
        normalized = default_config();
    }
    normalized
}

pub fn load_config_from_path(path: &Path) -> io::Result<MacroConfig> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(default_config()),
        Err(e) => return Err(e),
    };

    let value: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("Error parsing JSON config {}: {}", path.display(), e),
        )
    })?;

    if let Ok(new_cfg) = serde_json::from_value::<MacroConfig>(value.clone()) {
        return Ok(normalize_config(&new_cfg));
    }

    let legacy_cfg: LegacyConfig = serde_json::from_value(value).map_err(|e| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("Unsupported config format {}: {}", path.display(), e),
        )
    })?;

    Ok(migrate_legacy_config(legacy_cfg))
}

pub fn load_config_from_disk() -> io::Result<MacroConfig> {
    load_config_from_path(&config_file_path())
}

pub fn save_config_to_path(path: &Path, config: &MacroConfig) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let normalized = normalize_config(config);
    let content = serde_json::to_string_pretty(&normalized).map_err(|e| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("Error serializing config to JSON: {}", e),
        )
    })?;

    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, format!("{}\n", content))?;
    fs::rename(&tmp_path, path)?;
    Ok(())
}

pub fn save_config_to_disk(config: &MacroConfig) -> io::Result<()> {
    save_config_to_path(&config_file_path(), config)
}

pub fn validate_config_basic(config: &MacroConfig) -> Result<(), String> {
    if config.profiles.is_empty() {
        return Err("At least one profile is required.".to_string());
    }

    for profile in &config.profiles {
        if profile.name.trim().is_empty() {
            return Err("Profile name cannot be empty.".to_string());
        }
    }

    Ok(())
}

pub fn init_macros() -> &'static RwLock<MacroConfig> {
    MACROS.get_or_init(|| {
        println!("Initializing macros from disk...");
        let config = load_config_from_disk().unwrap_or_else(|e| {
            eprintln!("Error loading config: {}. Using default configuration.", e);
            default_config()
        });
        RwLock::new(config)
    })
}

pub fn reload_macros_from_disk() -> io::Result<()> {
    let config = load_config_from_disk()?;
    let macros_lock = init_macros();

    match macros_lock.write() {
        Ok(mut guard) => {
            *guard = config;
            Ok(())
        }
        Err(e) => Err(io::Error::other(format!(
            "Error acquiring write lock on macros: {}",
            e
        ))),
    }
}

pub fn get_profiles() -> Vec<ProfileConfig> {
    let macros_lock = init_macros();
    match macros_lock.read() {
        Ok(guard) => {
            if guard.profiles.is_empty() {
                default_profiles()
            } else {
                guard.profiles.clone()
            }
        }
        Err(e) => {
            eprintln!("Error acquiring read lock on macros: {}", e);
            default_profiles()
        }
    }
}

pub fn get_active_profile_index() -> usize {
    let macros_lock = init_macros();
    match macros_lock.read() {
        Ok(guard) => {
            if guard.profiles.is_empty() {
                0
            } else {
                guard
                    .active_profile
                    .min(guard.profiles.len().saturating_sub(1))
            }
        }
        Err(e) => {
            eprintln!("Error acquiring read lock on macros: {}", e);
            0
        }
    }
}

pub fn set_active_profile(index: usize) -> io::Result<()> {
    let macros_lock = init_macros();
    let snapshot = {
        let mut guard = macros_lock
            .write()
            .map_err(|e| io::Error::other(format!("Error acquiring write lock: {}", e)))?;

        let max_index = guard.profiles.len().saturating_sub(1);
        guard.active_profile = index.min(max_index);
        guard.clone()
    };

    save_config_to_disk(&snapshot)
}

fn macro_vec_for_type(profile: &ProfileConfig, macro_type: MacroType) -> &Vec<MacroEntry> {
    match macro_type {
        MacroType::Press => &profile.press_macros,
        MacroType::Hold => &profile.hold_macros,
    }
}

pub fn run_profile_macro(profile_index: usize, macro_type: MacroType, key: &str) {
    let macros_lock = init_macros();
    let macros = match macros_lock.read() {
        Ok(guard) => guard,
        Err(e) => {
            eprintln!("Error acquiring read lock on macros: {}", e);
            return;
        }
    };

    let profile = macros
        .profiles
        .get(profile_index)
        .or_else(|| macros.profiles.first());

    let Some(profile) = profile else {
        eprintln!("[WARN] No profile available");
        return;
    };

    let macro_vector = macro_vec_for_type(profile, macro_type);
    let map: HashMap<&str, &MacroEntry> = macro_vector
        .iter()
        .map(|entry| (entry.key.as_str(), entry))
        .collect();

    if let Some(entry) = map.get(key) {
        run_command(&entry.command);
    } else {
        println!(
            "[WARN] Unmapped key '{:?}' for key '{}' in profile '{}'",
            macro_type, key, profile.name
        );
    }
}
