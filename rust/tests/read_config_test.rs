use std::env::home_dir;
use std::fs;
use std::path::PathBuf;

use keyboard_rs::types::MacroConfig;

fn get_expected_config_path() -> PathBuf {
    home_dir()
        .expect("Không tìm thấy thư mục Home")
        .join(".config")
        .join("keyboard-rs")
        .join("config.json")
}

#[test]
fn integration_test_read_live_config() {
    let config_path = get_expected_config_path();

    if !config_path.exists() {
        panic!(
            "Kiểm thử thất bại: File cấu hình không tồn tại tại {}",
            config_path.display()
        );
    }

    let content = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(e) => panic!(
            "Kiểm thử thất bại: Không thể đọc file {}. Lỗi: {}",
            config_path.display(),
            e
        ),
    };

    let config: MacroConfig = match serde_json::from_str(&content) {
        Ok(cfg) => cfg,
        Err(e) => panic!(
            "Kiểm thử thất bại: File JSON bị lỗi định dạng tại {}. Lỗi: {}",
            config_path.display(),
            e
        ),
    };

    assert!(
        config.press_macros.len() > 0
            || config.anki_macros.len() > 0
            || config.hold_macros.len() > 0,
        "Kiểm thử thành công việc đọc file, nhưng file cấu hình không chứa macro nào."
    );

    println!(
        "Kiểm thử tích hợp thành công: Đọc {} macros từ {}\n",
        config.press_macros.len() + config.anki_macros.len() + config.hold_macros.len(),
        config_path.display()
    );

    // assert_eq!(config.press_macros.get("1"), Some(&"command_a".to_string()));
}
