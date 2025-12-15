use keyboard_rs::helper::*;
use keyboard_rs::system_tray::MyTray;
use ksni::blocking::TrayMethods;
use std::io::BufReader;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

const NORMAL: usize = 0;

fn main() {
    use rdev::listen;
    use std::thread;

    let pressed_keys = Arc::new(Mutex::new(Vec::new()));

    let option = Arc::new(AtomicUsize::new(NORMAL));

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
        checked: false,
    };
    tray.spawn().unwrap();

    let _ = serial_thread.join();
    let _ = keyboard_thread.join();
}
