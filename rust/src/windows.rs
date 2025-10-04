use crate::{run_app, run_command};

pub fn press(key: &str) {
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
pub fn hold(key: &str) {
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
