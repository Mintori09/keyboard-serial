use serde::Deserialize;
use std::{
    env::home_dir,
    fs,
    path::PathBuf,
    sync::{OnceLock, RwLock, RwLockReadGuard},
};

use crate::run_command;

#[derive(Debug, Deserialize, Clone)]
pub struct MacroEntry {
    pub key: String,
    pub command: String,
}

#[derive(Debug, Deserialize)]
pub struct MacroConfig {
    #[serde(default)]
    pub press_macros: Vec<MacroEntry>,
    #[serde(default)]
    pub anki_macros: Vec<MacroEntry>,
    #[serde(default)]
    pub hold_macros: Vec<MacroEntry>,
}

#[derive(Debug, Clone, Copy)]
pub enum MacroType {
    Press,
    Anki,
    Hold,
}

pub static MACROS: OnceLock<RwLock<MacroConfig>> = OnceLock::new();

static MACROS_FILE_PATH: OnceLock<PathBuf> = OnceLock::new();

fn get_macros_file_path() -> &'static PathBuf {
    MACROS_FILE_PATH.get_or_init(|| {
        home_dir()
            .expect("Cannot find home directory")
            .join(".config")
            .join("keyboard-rs")
            .join("config.json")
    })
}

pub fn init_macros() -> &'static RwLock<MacroConfig> {
    MACROS.get_or_init(|| {
        println!("Initializing macros from disk...");

        let path = get_macros_file_path();

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "Error reading config file {}: {}. Using empty configuration.",
                    path.display(),
                    e
                );
                r#"{}"#.to_string()
            }
        };

        let config: MacroConfig = serde_json::from_str(&content).unwrap_or_else(|e| {
            eprintln!(
                "Error parsing JSON config: {}. Using empty configuration.",
                e
            );
            MacroConfig {
                press_macros: Vec::new(),
                anki_macros: Vec::new(),
                hold_macros: Vec::new(),
            }
        });

        RwLock::new(config)
    })
}

fn get_macro_vector<'a>(
    macros: &'a RwLockReadGuard<MacroConfig>,
    macro_type: MacroType,
) -> &'a Vec<MacroEntry> {
    match macro_type {
        MacroType::Press => &macros.press_macros,
        MacroType::Anki => &macros.anki_macros,
        MacroType::Hold => &macros.hold_macros,
    }
}

pub fn run_macro(macro_type: MacroType, key: &str) {
    let macros_lock = init_macros();
    let macros = match macros_lock.read() {
        Ok(guard) => guard,
        Err(e) => {
            eprintln!("Error acquiring read lock on macros: {}", e);
            return;
        }
    };

    let macro_vector = get_macro_vector(&macros, macro_type);

    if let Some(entry) = macro_vector.iter().find(|entry| entry.key == key) {
        run_command(&entry.command);
    } else {
        println!("[WARN] Unmapped key '{:?}' for key: {}", macro_type, key);
    }
}
