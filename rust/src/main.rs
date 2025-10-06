use rdev::{Event, EventType, Key, listen};
use std::env::temp_dir;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use keyboard_rs::{detect_serial_port, linux, windows};

const BAUD_RATE: u32 = 9600;

fn handle_event(event: &str, key: &str) {
    match event {
        "PRESS" => {
            println!("[EVENT] Key {} pressed", key);
            handle_key(key);
        }
        "HOLD" => {
            println!("[EVENT] Key {} held", key);
            handle_key_hold(key);
        }
        _ => println!("[WARN] Unknown event: {}", event),
    }
}

fn handle_key_hold(key: &str) {
    if cfg!(target_os = "linux") {
        linux::hold(key);
    } else if cfg!(target_os = "windows") {
        windows::hold(key);
    }
}

fn handle_key(key: &str) {
    if cfg!(target_os = "linux") {
        linux::press(key);
    } else if cfg!(target_os = "windows") {
        windows::press(key);
    }
}

fn detect_or_exit() -> String {
    detect_serial_port().unwrap_or_else(|| {
        eprintln!("[FATAL] Could not auto-detect serial port.");
        std::process::exit(1);
    })
}

fn open_serial_port(port_name: &str) -> Box<dyn serialport::SerialPort> {
    serialport::new(port_name, BAUD_RATE)
        .timeout(Duration::from_millis(200))
        .open()
        .unwrap_or_else(|e| {
            eprintln!("Failed to open {}: {}", port_name, e);
            std::process::exit(1);
        })
}

fn read_serial_loop(reader: &mut BufReader<Box<dyn serialport::SerialPort>>) {
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(n) if n > 0 => handle_serial_line(line.trim()),
            Ok(_) => {} // Empty line or nothing read
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(e) => {
                eprintln!("[ERROR] Serial read failed: {}", e);
                break;
            }
        }
    }
}

fn handle_serial_line(line: &str) {
    if let Some((event, key)) = line.split_once(':') {
        println!("[DEBUG] {}:{}", event, key);
        handle_event(event, key);
    }
}

fn event_listener(event: Event, pressed_keys: &Arc<Mutex<Vec<Key>>>) {
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

fn run_vim_anywhere() {
    // file tạm
    let mut tmpfile: PathBuf = temp_dir();
    tmpfile.push(format!("vim-anywhere-{}.txt", std::process::id()));

    // Mở kitty + nvim
    let mut child = Command::new("kitty")
        .arg("-e")
        .arg("nvim")
        .arg(&tmpfile)
        .spawn()
        .expect("Không mở được kitty/nvim");

    // Chờ cho đến khi nvim thoát (cửa sổ kitty cũng sẽ đóng nếu cấu hình mặc định)
    let _ = child.wait();

    // Sau đó đọc nội dung và paste lại
    if let Ok(mut content) = fs::read_to_string(&tmpfile) {
        // Xóa các newline ở cuối (cả \n và \r\n)
        while content.ends_with('\n') || content.ends_with('\r') {
            content.pop();
        }

        if !content.is_empty() {
            // Copy vào clipboard
            let mut wl = Command::new("wl-copy")
                .stdin(std::process::Stdio::piped())
                .spawn()
                .expect("Không chạy được wl-copy");

            use std::io::Write;
            if let Some(stdin) = wl.stdin.as_mut() {
                let _ = stdin.write_all(content.as_bytes());
            }
            let _ = wl.wait();

            // Giả lập Ctrl+V để paste nhanh
            let _ = Command::new("ydotool")
                .args(&["key", "29:1", "47:1", "47:0", "29:0"]) // Ctrl down, V down, V up, Ctrl up
                .status();
        }
    }

    let _ = fs::remove_file(&tmpfile);
}

fn main() {
    use rdev::listen;
    use std::thread;

    let pressed_keys = Arc::new(Mutex::new(Vec::new()));
    let pressed_clone = pressed_keys.clone();

    // --- Thread 1: Serial listener ---
    let serial_thread = thread::spawn(|| {
        let serial_port = detect_or_exit();
        let port = open_serial_port(&serial_port);
        println!(
            "[INFO] Listening on {} at {} baud...",
            serial_port, BAUD_RATE
        );
        let mut reader = BufReader::new(port);
        read_serial_loop(&mut reader);
    });

    // --- Thread 2: Keyboard listener ---
    let keyboard_thread = thread::spawn(move || {
        println!("[INFO] Listening for Ctrl+Alt+I hotkey...");
        if let Err(error) = listen(move |event| {
            event_listener(event, &pressed_clone);
        }) {
            eprintln!("[ERROR] Keyboard listener failed: {:?}", error);
        }
    });

    // Wait for both threads to finish
    let _ = serial_thread.join();
    let _ = keyboard_thread.join();
}
