use std::fs;

use keyboard_rs::types::load_config_from_disk;

#[test]
fn integration_test_read_live_config() {
    let content = load_config_from_disk().expect("Không thể đọc config từ disk");

    let total = content
        .profiles
        .iter()
        .map(|p| p.press_macros.len() + p.hold_macros.len())
        .sum::<usize>();

    assert!(
        !content.profiles.is_empty(),
        "Config phải có ít nhất 1 profile"
    );
    assert!(total > 0, "Config không chứa macro press/hold nào");

    println!(
        "Đọc config thành công: {} profiles, {} macro entries",
        content.profiles.len(),
        total
    );

    let raw = fs::read_to_string(keyboard_rs::types::config_file_path())
        .expect("Không thể đọc raw config file");
    let _: serde_json::Value = serde_json::from_str(&raw).expect("Raw JSON không hợp lệ");
}
