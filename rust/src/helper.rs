use crate::detect_serial_port;
use crate::types::*;
use rdev::{Event, EventType, Key};
use std::env::temp_dir;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
pub const BAUD_RATE: u32 = 9600;

pub fn detect_or_exit() -> String {
    detect_serial_port().unwrap_or_else(|| {
        eprintln!("[FATAL] Could not auto-detect serial port.");
        std::process::exit(1);
    })
}

pub fn open_serial_port(port_name: &str) -> Box<dyn serialport::SerialPort> {
    serialport::new(port_name, BAUD_RATE)
        .timeout(Duration::from_millis(200))
        .open()
        .unwrap_or_else(|e| {
            eprintln!("Failed to open {}: {}", port_name, e);
            std::process::exit(1);
        })
}

pub fn read_serial_loop(
    reader: &mut BufReader<Box<dyn serialport::SerialPort>>,
    option: Arc<AtomicUsize>,
) {
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(n) if n > 0 => {
                let current = option.load(Ordering::Relaxed);
                handle_serial_line(line.trim(), current);
            }
            Ok(_) => {}
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(e) => {
                eprintln!("[ERROR] Serial read failed: {}", e);
                break;
            }
        }
    }
}

pub fn handle_serial_line(line: &str, selected_profile_index: usize) {
    init_macros();

    if let Some((event, key)) = line.split_once(':') {
        let normalized_event = event.to_uppercase();

        println!(
            "[DEBUG] {}:{}:profile-{}",
            normalized_event, key, selected_profile_index
        );

        if normalized_event == "HOLD" {
            run_profile_macro(selected_profile_index, MacroType::Hold, key);
        } else if normalized_event == "PRESS" {
            run_profile_macro(selected_profile_index, MacroType::Press, key);
        } else {
            println!("[WARN] Unhandled event type: {}", normalized_event);
        }
    } else {
        println!("[WARN] Invalid serial line format: {}", line);
    }
}

pub fn event_listener(event: Event, pressed_keys: &Arc<Mutex<Vec<Key>>>) {
    match event.event_type {
        EventType::KeyPress(key) => {
            let mut keys = pressed_keys.lock().unwrap();
            if !keys.contains(&key) {
                keys.push(key);
            }

            if (keys.contains(&Key::ControlLeft) || keys.contains(&Key::ControlRight))
                && keys.contains(&Key::Alt)
                && keys.contains(&Key::KeyI)
            {
                println!("Hotkey detected: Ctrl+Alt+V");
                run_vim_anywhere();
                keys.clear();
            }
        }
        EventType::KeyRelease(key) => {
            let mut keys = pressed_keys.lock().unwrap();
            keys.retain(|&k| k != key);
        }
        _ => {}
    }
}

pub fn run_vim_anywhere() {
    let mut tmpfile: PathBuf = temp_dir();
    tmpfile.push(format!("vim-anywhere-{}.txt", std::process::id()));

    let mut child = Command::new("kitty")
        .arg("-e")
        .arg("nvim")
        .arg(&tmpfile)
        .spawn()
        .expect("Cannot open kitty/nvim");
    let _ = child.wait();

    if let Ok(mut content) = fs::read_to_string(&tmpfile) {
        while content.ends_with('\n') || content.ends_with('\r') {
            content.pop();
        }

        if !content.is_empty() {
            let mut wl = Command::new("wl-copy")
                .stdin(std::process::Stdio::piped())
                .spawn()
                .expect("Cannot run wl-copy");

            use std::io::Write;
            if let Some(stdin) = wl.stdin.as_mut() {
                let _ = stdin.write_all(content.as_bytes());
            }
            let _ = wl.wait();

            let _ = Command::new("ydotool")
                .args(["key", "29:1", "47:1", "47:0", "29:0"])
                .status();
        }
    }

    let _ = fs::remove_file(&tmpfile);
}
