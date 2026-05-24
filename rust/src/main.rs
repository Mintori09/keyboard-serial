use keyboard_rs::helper::*;
use keyboard_rs::system_tray::MyTray;
use keyboard_rs::types::{get_active_profile_index, reload_macros_from_disk};
use ksni::blocking::{Handle as TrayHandle, TrayMethods};
use signal_hook::consts::signal::SIGHUP;
use signal_hook::iterator::Signals;
use std::io::BufReader;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

const PID_FILE: &str = "/tmp/keyboard-rs.pid";

fn write_pid_file() {
    if let Err(e) = std::fs::write(PID_FILE, format!("{}\n", std::process::id())) {
        eprintln!("[WARN] Cannot write PID file {}: {}", PID_FILE, e);
    }
}

fn start_reload_signal_handler_with_option(
    option: Arc<AtomicUsize>,
    tray_handle: TrayHandle<MyTray>,
) {
    std::thread::spawn(move || {
        let mut signals = match Signals::new([SIGHUP]) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[WARN] Failed to initialize SIGHUP handler: {}", e);
                return;
            }
        };

        for _ in signals.forever() {
            match reload_macros_from_disk() {
                Ok(_) => {
                    let active = get_active_profile_index();
                    option.store(active, Ordering::Relaxed);
                    let _ = tray_handle.update(|tray| {
                        tray.selected_option.store(active, Ordering::Relaxed);
                    });
                    println!("[INFO] Reloaded config after SIGHUP");
                }
                Err(e) => eprintln!("[ERROR] Failed to reload config after SIGHUP: {}", e),
            }
        }
    });
}

fn main() {
    use rdev::listen;
    use std::thread;

    write_pid_file();

    let pressed_keys = Arc::new(Mutex::new(Vec::new()));

    let option = Arc::new(AtomicUsize::new(get_active_profile_index()));

    // --- Thread 1: Serial listener ---
    let serial_option = option.clone();
    let serial_thread = thread::spawn(move || {
        let serial_port = detect_or_exit();
        let port = open_serial_port(&serial_port);

        let mut reader = BufReader::new(port);
        read_serial_loop(&mut reader, serial_option);
    });

    // --- Thread 2: Keyboard listener ---
    let pressed_clone = pressed_keys.clone();
    let keyboard_thread = thread::spawn(move || {
        listen(move |event| {
            event_listener(event, &pressed_clone);
        })
        .unwrap();
    });

    // --- System tray ---
    let tray = MyTray {
        selected_option: option.clone(),
    };
    let tray_handle = tray.spawn().unwrap();

    start_reload_signal_handler_with_option(option.clone(), tray_handle);

    let _ = serial_thread.join();
    let _ = keyboard_thread.join();
}
