use device_query::{DeviceQuery, DeviceState, Keycode};
use serialport::{SerialPortType, available_ports};
use std::io::{BufRead, BufReader};
use std::process::Command;
use std::time::Duration;

const BAUD_RATE: u32 = 9600;

/// Tìm cổng serial (ưu tiên USB)
fn detect_serial_port() -> Option<String> {
    match available_ports() {
        Ok(ports) => {
            if ports.is_empty() {
                eprintln!("[ERROR] No serial ports found");
                return None;
            }
            println!("[INFO] Available ports:");
            for p in &ports {
                println!(" - {}", p.port_name);
                if let SerialPortType::UsbPort(info) = &p.port_type {
                    println!(
                        "   [USB] VID: {:?}, PID: {:?}, Serial: {:?}, Manufacturer: {:?}, Product: {:?}",
                        info.vid, info.pid, info.serial_number, info.manufacturer, info.product
                    );
                }
            }
            // Ưu tiên USB
            if let Some(usb) = ports
                .iter()
                .find(|p| matches!(p.port_type, SerialPortType::UsbPort(_)))
            {
                Some(usb.port_name.clone())
            } else {
                Some(ports[0].port_name.clone())
            }
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to list serial ports: {}", e);
            None
        }
    }
}

/// Chạy lệnh shell
fn run_command(cmd: &str) {
    println!("[CMD] {}", cmd);
    let status = if cfg!(target_os = "windows") {
        Command::new("cmd").args(&["/C", cmd]).status().unwrap()
    } else {
        Command::new("sh").arg("-c").arg(cmd).status().unwrap()
    };
    if !status.success() {
        eprintln!("[ERROR] command failed");
    }
}

/// Chạy ứng dụng
fn run_app(cmd: &str) {
    println!("[APP] {}", cmd);
    if cfg!(target_os = "windows") {
        Command::new("cmd").args(&["/C", cmd]).spawn().unwrap();
    } else {
        Command::new("sh").arg("-c").arg(cmd).spawn().unwrap();
    }
}

/// Gọi wtype để gõ chuỗi git command
fn wtype_git_command(message: &str) {
    let git_cmd = format!("git add . && git commit -m \"{}\" && git push", message);

    let _ = Command::new("wtype")
        .args(&["-s", "20", &git_cmd]) // -s 20 = delay 20ms mỗi ký tự
        .status();
}

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
        match key {
            "#" => run_command("poweroff"),
            "2" => run_command("xdg-open \"https://shopee.vn/search?keyword=$(wl-paste)\""),
            "4" => run_command("git add . && git commit -m 'auto' && git push"),
            "5" => run_app("alacritty -e nvim ~/Documents/Obsidian/"),
            "0" => run_command(
                r#"
                . ~/.config/shell/apikey.sh &&
                curl -s -H "Authorization: Bearer $HASS_TOKEN" \
                    -H "Content-Type: application/json" \
                    -d '{"entity_id": "input_boolean.cong_tac_dieu_hoa_phong_tri"}' \
                    http://192.168.10.80:8123/api/services/input_boolean/toggle
                "#,
            ),
            _ => println!("[WARN] Unmapped hold key: {}", key),
        }
    } else if cfg!(target_os = "windows") {
        match key {
            "1" => run_command("start https://chatgpt.com"),
            "2" => run_command(
                "powershell -command \"Start-Process 'https://www.google.com/search?q=' + (Get-Clipboard)\"",
            ),
            "4" => run_app("wt.exe"),
            "5" => run_app("notepad.exe"),
            "7" => run_command("start https://www.youtube.com/"),
            "8" => run_command("start https://onedrive.live.com/?view=0"),
            "9" => run_command(
                "powershell -command \"$wshell = New-Object -ComObject wscript.shell; $wshell.SendKeys('^+i')\"",
            ),
            "#" => run_command("rundll32.exe user32.dll,LockWorkStation"),
            _ => println!("[WARN] Unmapped hold key: {}", key),
        }
    }
}

fn handle_key(key: &str) {
    if cfg!(target_os = "linux") {
        match key {
            "1" => run_command("xdg-open 'https://chatgpt.com/?temporary-chat=true'"),
            "2" => run_command("xdg-open \"https://www.google.com/search?q=$(wl-paste)\""),
            "3" => run_command("qdbus org.kde.KWin /KWin org.kde.KWin.nextDesktop"),
            "4" => run_app("alacritty"),
            "5" => run_app(
                "sh -c 'tmp=$(mktemp /tmp/clipXXXX.md); wl-paste > $tmp; alacritty -e nvim -c \"autocmd VimLeave * call delete(\\\"$tmp\\\")\" $tmp'",
            ),
            "6" => run_command("qdbus org.kde.KWin /KWin org.kde.KWin.previousDesktop"),
            "7" => run_command("xdg-open \"https://www.youtube.com/\""),
            "8" => run_command("xdg-open \"https://onedrive.live.com/?view=0\""),
            "9" => run_command("mpv \"$(wl-paste)\""),
            "*" => run_app("alacritty -e nvim ~/Documents/Obsidian/"),
            "0" => run_app("/home/mintori/.config/rofi/launchers/rofi-power-menu.sh"),
            "#" => run_command("qdbus org.freedesktop.ScreenSaver /ScreenSaver Lock"),
            _ => println!("[WARN] Unmapped key: {}", key),
        }
    } else if cfg!(target_os = "windows") {
        match key {
            "1" => run_command("start https://chatgpt.com"),
            "2" => run_command(
                "powershell -command \"Start-Process 'https://www.google.com/search?q=' + (Get-Clipboard)\"",
            ),
            "4" => run_app("wt.exe"),
            "5" => run_app("notepad.exe"),
            "7" => run_command("start https://www.youtube.com/"),
            "8" => run_command("start https://onedrive.live.com/?view=0"),
            "9" => run_command(
                "powershell -command \"$wshell = New-Object -ComObject wscript.shell; $wshell.SendKeys('^+i')\"",
            ),
            "#" => run_command("rundll32.exe user32.dll,LockWorkStation"),
            _ => println!("[WARN] Unmapped key: {}", key),
        }
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
