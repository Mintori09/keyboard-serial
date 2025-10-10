use crate::{run_app, run_command};

pub fn press(key: &str) {
    match key {
        "1" => run_command("start https://chat.openai.com"),
        "2" => run_command(
            "powershell -Command \"Start-Process 'https://www.google.com/search?q=' + (Get-Clipboard)\"",
        ),
        "3" => println!("Virtual desktop navigation not implemented on Windows."), // Requires external tool like PowerShell script or AutoHotkey
        "4" => run_app("wt.exe"), // Windows Terminal
        "5" => run_command(
            "powershell -Command \"$tmp = New-TemporaryFile; Get-Clipboard | Out-File -FilePath $tmp; wt.exe new-tab nvim $tmp\"",
        ),
        "6" => println!("Previous desktop navigation not implemented on Windows."),
        "7" => run_command("start https://www.youtube.com/"),
        "8" => run_command("start https://onedrive.live.com/?view=0"),
        "9" => run_command("powershell -Command \"Start-Process 'mpv' (Get-Clipboard)\""),
        "*" => run_app("wt.exe -d %USERPROFILE%\\Documents\\Obsidian nvim ."),
        "0" => run_app(
            "powershell -Command \"Start-Process 'C:\\Users\\mintori\\.config\\rofi\\launchers\\rofi-power-menu.bat'\"",
        ),
        "#" => run_command("rundll32.exe user32.dll,LockWorkStation"),
        _ => println!("[WARN] Unmapped key: {}", key),
    }
}

pub fn hold(key: &str) {
    match key {
        "1" => run_command("powershell -Command \"git add .; git commit -m 'Update'; git push\""),
        "2" => run_command(
            "powershell -Command \"Start-Process 'https://shopee.vn/search?keyword=' + (Get-Clipboard)\"",
        ),
        "4" => run_app(
            "wt.exe new-tab powershell -NoExit -Command \"(Get-Clipboard); Read-Host 'Press Enter to exit...'\"",
        ),
        "5" => run_app("wt.exe -d %USERPROFILE%\\Documents\\Obsidian nvim ."),
        "7" => run_command(
            "powershell -Command \"Start-Process 'https://www.youtube.com/results?search_query=' + (Get-Clipboard)\"",
        ),
        "8" => run_command(
            "powershell -Command \"Start-Process 'C:\\Users\\mintori\\.config\\rofi\\launchers\\install-yay.ps1'\"",
        ),
        "#" => run_command("shutdown /s /t 0"),
        _ => println!("[WARN] Unmapped hold key: {}", key),
    }
}
