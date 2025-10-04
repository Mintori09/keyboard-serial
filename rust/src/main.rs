use std::io::{BufRead, BufReader};
use std::time::Duration;

use keyboard_rs::{detect_serial_port, linux, windows};

const BAUD_RATE: u32 = 9600;

/// Map sự kiện
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

fn main() {
    let serial_port = detect_serial_port().unwrap_or_else(|| {
        eprintln!("[FATAL] Could not auto-detect serial port.");
        std::process::exit(1);
    });

    let port = serialport::new(&serial_port, BAUD_RATE)
        .timeout(Duration::from_millis(200))
        .open()
        .unwrap_or_else(|e| {
            eprintln!("Failed to open {}: {}", serial_port, e);
            std::process::exit(1);
        });

    println!(
        "[INFO] Listening on {} at {} baud...",
        serial_port, BAUD_RATE
    );

    let mut reader = BufReader::new(port);

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(n) if n > 0 => {
                let line = line.trim();
                if let Some((event, key)) = line.split_once(':') {
                    println!("[DEBUG] {}:{}", event, key);

                    match event {
                        "PRESS" => {
                            handle_event(event, key);
                        }
                        "HOLD" => {
                            handle_event(event, key);
                        }
                        _ => handle_event(event, key),
                    }
                }
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
