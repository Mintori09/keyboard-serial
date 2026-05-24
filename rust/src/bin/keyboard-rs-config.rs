use gtk::prelude::*;
use gtk::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, Entry, Label, ListBox,
    ListBoxRow, Notebook, Orientation, ScrolledWindow,
};
use gtk4 as gtk;
use keyboard_rs::types::{
    FIXED_KEYS, MacroConfig, MacroEntry, ProfileConfig, config_file_path, load_config_from_disk,
    normalize_config, save_config_to_disk, set_active_profile, validate_config_basic,
};
use std::cell::RefCell;
use std::fs;
use std::process::Command;
use std::rc::Rc;

const APP_ID: &str = "com.mintori.keyboard_rs.config";
const PID_FILE: &str = "/tmp/keyboard-rs.pid";

#[derive(Clone)]
struct ProfileRowWidgets {
    name_entry: Entry,
    selected_label: Label,
    select_btn: Button,
}

#[derive(Clone)]
struct FixedMacroRowWidgets {
    key: String,
    command_entry: Entry,
}

fn fixed_macro_rows(title: &str) -> (GtkBox, Vec<FixedMacroRowWidgets>) {
    let root = GtkBox::new(Orientation::Vertical, 6);
    root.set_margin_top(8);
    root.set_margin_bottom(8);
    root.set_margin_start(8);
    root.set_margin_end(8);

    root.append(&Label::new(Some(title)));

    let list = ListBox::new();
    list.set_selection_mode(gtk::SelectionMode::None);

    let mut rows = Vec::new();
    for key in FIXED_KEYS {
        let row_box = GtkBox::new(Orientation::Horizontal, 8);
        row_box.set_margin_top(2);
        row_box.set_margin_bottom(2);
        row_box.set_margin_start(4);
        row_box.set_margin_end(4);

        let key_label = Label::new(Some(key));
        key_label.set_width_chars(3);
        key_label.set_halign(Align::Start);

        let command_entry = Entry::new();
        command_entry.set_hexpand(true);
        command_entry.set_placeholder_text(Some("Command"));

        row_box.append(&key_label);
        row_box.append(&command_entry);

        let row = ListBoxRow::new();
        row.set_child(Some(&row_box));
        list.append(&row);

        rows.push(FixedMacroRowWidgets {
            key: key.to_string(),
            command_entry,
        });
    }

    let scroll = ScrolledWindow::new();
    scroll.set_hexpand(true);
    scroll.set_vexpand(true);
    scroll.set_child(Some(&list));

    root.append(&scroll);
    (root, rows)
}

fn macros_to_map(entries: &[MacroEntry]) -> std::collections::HashMap<String, String> {
    entries
        .iter()
        .map(|e| (e.key.clone(), e.command.clone()))
        .collect()
}

fn load_profile_into_editor(
    config: &Rc<RefCell<MacroConfig>>,
    selected_idx: usize,
    press_rows: &[FixedMacroRowWidgets],
    hold_rows: &[FixedMacroRowWidgets],
) {
    let cfg = config.borrow();
    let Some(profile) = cfg.profiles.get(selected_idx) else {
        return;
    };

    let press_map = macros_to_map(&profile.press_macros);
    for row in press_rows {
        row.command_entry.set_text(
            press_map
                .get(&row.key)
                .cloned()
                .unwrap_or_default()
                .as_str(),
        );
    }

    let hold_map = macros_to_map(&profile.hold_macros);
    for row in hold_rows {
        row.command_entry
            .set_text(hold_map.get(&row.key).cloned().unwrap_or_default().as_str());
    }
}

fn set_active_profile_in_memory(config: &Rc<RefCell<MacroConfig>>, index: usize) {
    let mut cfg = config.borrow_mut();
    let max_index = cfg.profiles.len().saturating_sub(1);
    cfg.active_profile = index.min(max_index);
}

fn persist_active_profile_and_signal(
    config: &Rc<RefCell<MacroConfig>>,
    index: usize,
) -> Result<(), String> {
    set_active_profile_in_memory(config, index);
    set_active_profile(index).map_err(|e| format!("Persist active profile failed: {}", e))?;
    signal_reload()
}

fn save_editor_to_profile(
    config: &Rc<RefCell<MacroConfig>>,
    selected_idx: usize,
    press_rows: &[FixedMacroRowWidgets],
    hold_rows: &[FixedMacroRowWidgets],
) {
    let mut cfg = config.borrow_mut();
    let Some(profile) = cfg.profiles.get_mut(selected_idx) else {
        return;
    };

    profile.press_macros = press_rows
        .iter()
        .map(|row| MacroEntry {
            key: row.key.clone(),
            command: row.command_entry.text().to_string(),
        })
        .collect();

    profile.hold_macros = hold_rows
        .iter()
        .map(|row| MacroEntry {
            key: row.key.clone(),
            command: row.command_entry.text().to_string(),
        })
        .collect();
}

fn sync_profile_names(
    rows: &Rc<RefCell<Vec<ProfileRowWidgets>>>,
    config: &Rc<RefCell<MacroConfig>>,
) {
    let names = rows
        .borrow()
        .iter()
        .map(|r| r.name_entry.text().to_string())
        .collect::<Vec<_>>();

    let mut cfg = config.borrow_mut();
    for (idx, name) in names.into_iter().enumerate() {
        if let Some(profile) = cfg.profiles.get_mut(idx) {
            profile.name = name;
        }
    }
}

fn refresh_selected_markers(rows: &Rc<RefCell<Vec<ProfileRowWidgets>>>, selected_index: usize) {
    for (idx, row) in rows.borrow().iter().enumerate() {
        let is_selected = idx == selected_index;
        row.selected_label.set_visible(is_selected);
        row.select_btn.set_sensitive(!is_selected);
    }
}

fn signal_reload() -> Result<(), String> {
    let pid_text = fs::read_to_string(PID_FILE)
        .map_err(|e| format!("Cannot read PID file {}: {}", PID_FILE, e))?;
    let pid = pid_text.trim();
    if pid.is_empty() {
        return Err("PID file is empty.".to_string());
    }

    let status = Command::new("kill")
        .arg("-HUP")
        .arg(pid)
        .status()
        .map_err(|e| format!("Failed to run kill -HUP: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("kill -HUP {} failed with status {}", pid, status))
    }
}

fn set_status(status: &Label, msg: &str, is_error: bool) {
    status.set_text(msg);
    if is_error {
        status.add_css_class("error");
    } else {
        status.remove_css_class("error");
    }
}

fn render_profiles_tab(
    list: &ListBox,
    rows: &Rc<RefCell<Vec<ProfileRowWidgets>>>,
    config: &Rc<RefCell<MacroConfig>>,
    selected_index: &Rc<RefCell<usize>>,
    status_label: &Label,
    press_rows: &[FixedMacroRowWidgets],
    hold_rows: &[FixedMacroRowWidgets],
) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
    rows.borrow_mut().clear();

    for (idx, profile) in config.borrow().profiles.iter().enumerate() {
        let row_box = GtkBox::new(Orientation::Horizontal, 8);
        row_box.set_margin_top(2);
        row_box.set_margin_bottom(2);
        row_box.set_margin_start(4);
        row_box.set_margin_end(4);

        let name_entry = Entry::new();
        name_entry.set_hexpand(true);
        name_entry.set_text(&profile.name);

        let selected_label = Label::new(Some("Selected"));
        selected_label.add_css_class("accent");
        selected_label.set_visible(idx == *selected_index.borrow());

        let select_btn = Button::with_label("Select");
        select_btn.set_sensitive(idx != *selected_index.borrow());
        let remove_btn = Button::with_label("Remove");

        row_box.append(&name_entry);
        row_box.append(&selected_label);
        row_box.append(&select_btn);
        row_box.append(&remove_btn);

        let row = ListBoxRow::new();
        row.set_child(Some(&row_box));
        list.append(&row);

        let selected_index_clone = selected_index.clone();
        let cfg_for_select = config.clone();
        let rows_for_select = rows.clone();
        let status_for_select = status_label.clone();
        let list_for_select = list.clone();
        let press_for_select = press_rows.to_vec();
        let hold_for_select = hold_rows.to_vec();
        select_btn.connect_clicked(move |_| {
            sync_profile_names(&rows_for_select, &cfg_for_select);
            let current_idx = *selected_index_clone.borrow();
            save_editor_to_profile(
                &cfg_for_select,
                current_idx,
                &press_for_select,
                &hold_for_select,
            );

            *selected_index_clone.borrow_mut() = idx;
            let sync_result = persist_active_profile_and_signal(&cfg_for_select, idx);
            load_profile_into_editor(&cfg_for_select, idx, &press_for_select, &hold_for_select);

            render_profiles_tab(
                &list_for_select,
                &rows_for_select,
                &cfg_for_select,
                &selected_index_clone,
                &status_for_select,
                &press_for_select,
                &hold_for_select,
            );
            refresh_selected_markers(&rows_for_select, idx);

            match sync_result {
                Ok(()) => set_status(
                    &status_for_select,
                    &format!("Selected profile #{} and synced runtime", idx + 1),
                    false,
                ),
                Err(e) => set_status(
                    &status_for_select,
                    &format!(
                        "Selected profile #{} but runtime sync failed: {}",
                        idx + 1,
                        e
                    ),
                    true,
                ),
            }
        });

        let cfg_for_remove = config.clone();
        let selected_for_remove = selected_index.clone();
        let status_for_remove = status_label.clone();
        let list_for_remove = list.clone();
        let rows_for_remove = rows.clone();
        let press_for_remove = press_rows.to_vec();
        let hold_for_remove = hold_rows.to_vec();
        remove_btn.connect_clicked(move |_| {
            sync_profile_names(&rows_for_remove, &cfg_for_remove);
            if cfg_for_remove.borrow().profiles.len() <= 1 {
                set_status(
                    &status_for_remove,
                    "At least one profile is required.",
                    true,
                );
                return;
            }

            if idx < cfg_for_remove.borrow().profiles.len() {
                cfg_for_remove.borrow_mut().profiles.remove(idx);
                let len = cfg_for_remove.borrow().profiles.len();
                let mut sel = selected_for_remove.borrow_mut();
                if *sel >= len {
                    *sel = len.saturating_sub(1);
                }
                let new_sel = *sel;
                drop(sel);
                set_active_profile_in_memory(&cfg_for_remove, new_sel);
                if let Err(e) = set_active_profile(new_sel) {
                    eprintln!(
                        "[WARN] Failed to persist active profile after remove: {}",
                        e
                    );
                }
            }

            render_profiles_tab(
                &list_for_remove,
                &rows_for_remove,
                &cfg_for_remove,
                &selected_for_remove,
                &status_for_remove,
                &press_for_remove,
                &hold_for_remove,
            );
            load_profile_into_editor(
                &cfg_for_remove,
                *selected_for_remove.borrow(),
                &press_for_remove,
                &hold_for_remove,
            );
            refresh_selected_markers(&rows_for_remove, *selected_for_remove.borrow());
        });

        rows.borrow_mut().push(ProfileRowWidgets {
            name_entry,
            selected_label,
            select_btn,
        });
    }
}

fn build_ui(app: &Application) {
    let cfg = load_config_from_disk().unwrap_or_else(|_| MacroConfig {
        profiles: Vec::new(),
        active_profile: 0,
    });
    let config = Rc::new(RefCell::new(normalize_config(&cfg)));

    let selected_index = Rc::new(RefCell::new(config.borrow().active_profile));

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Keyboard-rs Configurator")
        .default_width(980)
        .default_height(680)
        .build();

    let root = GtkBox::new(Orientation::Vertical, 8);
    root.set_margin_top(8);
    root.set_margin_bottom(8);
    root.set_margin_start(8);
    root.set_margin_end(8);

    let notebook = Notebook::new();
    notebook.set_hexpand(true);
    notebook.set_vexpand(true);

    let status_label = Label::new(None);
    status_label.set_halign(Align::Start);
    status_label.set_text(&format!("Editing {}", config_file_path().display()));

    let profiles_tab = GtkBox::new(Orientation::Vertical, 8);
    profiles_tab.set_margin_top(8);
    profiles_tab.set_margin_bottom(8);
    profiles_tab.set_margin_start(8);
    profiles_tab.set_margin_end(8);

    let add_profile_btn = Button::with_label("Add Profile");
    add_profile_btn.set_halign(Align::Start);

    let profile_list = ListBox::new();
    profile_list.set_selection_mode(gtk::SelectionMode::None);

    profiles_tab.append(&add_profile_btn);
    profiles_tab.append(&profile_list);

    let macros_tab = GtkBox::new(Orientation::Vertical, 8);
    let (press_panel, press_rows) = fixed_macro_rows("Press Commands");
    let (hold_panel, hold_rows) = fixed_macro_rows("Hold Commands");
    macros_tab.append(&press_panel);
    macros_tab.append(&hold_panel);

    notebook.append_page(&profiles_tab, Some(&Label::new(Some("Profiles"))));
    notebook.append_page(&macros_tab, Some(&Label::new(Some("Macros"))));

    let action_bar = GtkBox::new(Orientation::Horizontal, 8);
    action_bar.set_halign(Align::End);
    let reload_btn = Button::with_label("Reload from Disk");
    let apply_btn = Button::with_label("Apply");
    action_bar.append(&reload_btn);
    action_bar.append(&apply_btn);

    root.append(&notebook);
    root.append(&action_bar);
    root.append(&status_label);
    window.set_child(Some(&root));

    let profile_rows = Rc::new(RefCell::new(Vec::<ProfileRowWidgets>::new()));

    let press_rows_shared = press_rows.clone();
    let hold_rows_shared = hold_rows.clone();
    render_profiles_tab(
        &profile_list,
        &profile_rows,
        &config,
        &selected_index,
        &status_label,
        &press_rows_shared,
        &hold_rows_shared,
    );
    refresh_selected_markers(&profile_rows, *selected_index.borrow());
    load_profile_into_editor(&config, *selected_index.borrow(), &press_rows, &hold_rows);

    let config_for_add = config.clone();
    let selected_for_add = selected_index.clone();
    let status_for_add = status_label.clone();
    let list_for_add = profile_list.clone();
    let rows_for_add = profile_rows.clone();
    let press_for_add = press_rows.clone();
    let hold_for_add = hold_rows.clone();
    add_profile_btn.connect_clicked(move |_| {
        sync_profile_names(&rows_for_add, &config_for_add);
        save_editor_to_profile(
            &config_for_add,
            *selected_for_add.borrow(),
            &press_for_add,
            &hold_for_add,
        );

        config_for_add.borrow_mut().profiles.push(ProfileConfig {
            name: format!("Profile {}", config_for_add.borrow().profiles.len() + 1),
            press_macros: FIXED_KEYS
                .iter()
                .map(|k| MacroEntry {
                    key: (*k).to_string(),
                    command: String::new(),
                })
                .collect(),
            hold_macros: FIXED_KEYS
                .iter()
                .map(|k| MacroEntry {
                    key: (*k).to_string(),
                    command: String::new(),
                })
                .collect(),
        });

        *selected_for_add.borrow_mut() = config_for_add.borrow().profiles.len().saturating_sub(1);
        set_active_profile_in_memory(&config_for_add, *selected_for_add.borrow());

        render_profiles_tab(
            &list_for_add,
            &rows_for_add,
            &config_for_add,
            &selected_for_add,
            &status_for_add,
            &press_for_add,
            &hold_for_add,
        );
        refresh_selected_markers(&rows_for_add, *selected_for_add.borrow());
        load_profile_into_editor(
            &config_for_add,
            *selected_for_add.borrow(),
            &press_for_add,
            &hold_for_add,
        );
    });

    let status_for_reload = status_label.clone();
    let win_clone = window.clone();
    reload_btn.connect_clicked(move |_| {
        if let Ok(exe) = std::env::current_exe() {
            if let Err(e) = Command::new(exe).spawn() {
                set_status(
                    &status_for_reload,
                    &format!("Cannot relaunch configurator: {}", e),
                    true,
                );
                return;
            }
            win_clone.close();
        } else {
            set_status(
                &status_for_reload,
                "Cannot resolve current executable path.",
                true,
            );
        }
    });

    let config_for_apply = config.clone();
    let selected_for_apply = selected_index.clone();
    let profile_rows_for_apply = profile_rows.clone();
    let status_for_apply = status_label.clone();
    let press_for_apply = press_rows.clone();
    let hold_for_apply = hold_rows.clone();
    apply_btn.connect_clicked(move |_| {
        sync_profile_names(&profile_rows_for_apply, &config_for_apply);
        set_active_profile_in_memory(&config_for_apply, *selected_for_apply.borrow());
        save_editor_to_profile(
            &config_for_apply,
            *selected_for_apply.borrow(),
            &press_for_apply,
            &hold_for_apply,
        );

        let normalized = normalize_config(&config_for_apply.borrow());
        if let Err(e) = validate_config_basic(&normalized) {
            set_status(&status_for_apply, &format!("Validation error: {}", e), true);
            return;
        }

        if let Err(e) = save_config_to_disk(&normalized) {
            set_status(&status_for_apply, &format!("Save failed: {}", e), true);
            return;
        }

        *config_for_apply.borrow_mut() = normalized;

        match signal_reload() {
            Ok(()) => set_status(&status_for_apply, "Saved and applied via SIGHUP.", false),
            Err(e) => set_status(
                &status_for_apply,
                &format!("Saved, but reload signal failed: {}", e),
                true,
            ),
        }
    });

    window.present();
}

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
}
