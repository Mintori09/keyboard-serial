use crate::{run_app, run_command};
pub fn press(key: &str) {
    match key {
        "1" => run_command("xdg-open 'https://chatgpt.com/?temporary-chat=true'"),
        "2" => run_command("xdg-open \"https://www.google.com/search?q=$(wl-paste)\""),
        "3" => run_command("qdbus org.kde.KWin /KWin org.kde.KWin.nextDesktop"),
        "4" => run_app("kitty"),
        "5" => run_app(
            "sh -c 'tmp=$(mktemp /tmp/clipXXXX.md); wl-paste > $tmp; kitty -e nvim -c \"autocmd VimLeave * call delete(\\\"$tmp\\\")\" $tmp'",
        ),
        "6" => run_command("qdbus org.kde.KWin /KWin org.kde.KWin.previousDesktop"),
        "7" => run_command("xdg-open \"https://www.youtube.com/\""),
        "8" => run_command("xdg-open \"https://onedrive.live.com/?view=0\""),
        "9" => run_app("mpv \"$(wl-paste)\""),
        "*" => run_app("kitty -e nvim ~/Documents/Obsidian/"),
        "0" => run_app("/home/mintori/.config/rofi/launchers/rofi-power-menu.sh"),
        "#" => run_command("qdbus org.freedesktop.ScreenSaver /ScreenSaver Lock"),
        _ => println!("[WARN] Unmapped key: {}", key),
    }
}

pub fn hold(key: &str) {
    match key {
        "1" => run_command("ydotool type \"git add . && git commit -m \'Update\' && git push\""),
        "2" => run_command("xdg-open \"https://shopee.vn/search?keyword=$(wl-paste)\""),
        "4" => run_app(
            "kitty -e zsh -c '$(wl-paste); echo; read -n 1 -s -r -p \"Press any key to exit...\"'",
        ),
        "5" => run_app("kitty -e bash -c 'cd ~/Documents/Obsidian && nvim'"),
        "7" => run_app("xdg-open \"https://www.youtube.com/results?search_query=$( wl-paste )\""),
        "0" => run_command(
            r#"
                . ~/.config/shell/apikey.sh &&
                curl -s -H "Authorization: Bearer $HASS_TOKEN" \
                    -H "Content-Type: application/json" \
                    -d '{"entity_id": "input_boolean.cong_tac_dieu_hoa_phong_tri"}' \
                    http://192.168.10.80:8123/api/services/input_boolean/toggle
                "#,
        ),
        "#" => run_command("poweroff"),
        _ => println!("[WARN] Unmapped hold key: {}", key),
    }
}
