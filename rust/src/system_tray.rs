use crate::types::{get_profiles, set_active_profile};
use ksni::menu::*;
use std::process::Command;
use std::sync::atomic::Ordering;
use std::sync::{Arc, atomic::AtomicUsize};

#[derive(Debug)]
pub struct MyTray {
    pub selected_option: Arc<AtomicUsize>,
}

impl ksni::Tray for MyTray {
    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
    }

    fn icon_name(&self) -> String {
        "help-about".into()
    }

    fn title(&self) -> String {
        let labels = get_profiles();
        let index = self.selected_option.load(Ordering::Relaxed);
        labels
            .get(index)
            .or_else(|| labels.first())
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "UNKNOWN".to_string())
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        let labels = get_profiles();
        let options: Vec<RadioItem> = labels
            .iter()
            .map(|profile| RadioItem {
                label: profile.name.clone(),
                ..Default::default()
            })
            .collect();

        vec![
            RadioGroup {
                selected: self.selected_option.load(Ordering::Relaxed),
                select: Box::new(|this: &mut MyTray, current| {
                    this.selected_option.store(current, Ordering::Relaxed);
                    if let Err(e) = set_active_profile(current) {
                        eprintln!("[WARN] Failed to persist active profile: {}", e);
                    }
                    println!("[TRAY] {}", current);
                }),
                options,
            }
            .into(),
            StandardItem {
                label: "Configure...".into(),
                activate: Box::new(|_| {
                    if let Err(e) = Command::new("keyboard-rs-config").spawn() {
                        eprintln!("[WARN] Failed to open configurator: {}", e);
                    }
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Exit".into(),
                activate: Box::new(|_| std::process::exit(0)),
                ..Default::default()
            }
            .into(),
        ]
    }
}
