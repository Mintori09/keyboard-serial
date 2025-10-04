pub mod linux;
pub mod windows;
use serialport::{SerialPortType, available_ports};
use std::process::Command;

pub fn detect_serial_port() -> Option<String> {
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
pub fn run_command(cmd: &str) {
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
pub fn run_app(cmd: &str) {
    println!("[APP] {}", cmd);
    if cfg!(target_os = "windows") {
        Command::new("cmd").args(&["/C", cmd]).spawn().unwrap();
    } else if cfg!(target_os = "linux") {
        Command::new("sh").arg("-c").arg(cmd).spawn().unwrap();
    }
}
