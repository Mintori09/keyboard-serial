use ksni::menu::*;
use std::sync::atomic::Ordering;
use std::sync::{Arc, atomic::AtomicUsize};

const NORMAL: usize = 0;
const ANKI: usize = 1;

#[derive(Debug)]
pub struct MyTray {
    pub selected_option: Arc<AtomicUsize>,
    pub checked: bool,
}

impl ksni::Tray for MyTray {
    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
    }

    fn icon_name(&self) -> String {
        "help-about".into()
    }

    fn title(&self) -> String {
        match self.selected_option.load(Ordering::Relaxed) {
            NORMAL => "NORMAL",
            ANKI => "ANKI",
            _ => "UNKNOWN",
        }
        .into()
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        vec![
            RadioGroup {
                selected: self.selected_option.load(Ordering::Relaxed),
                select: Box::new(|this: &mut MyTray, current| {
                    this.selected_option.store(current, Ordering::Relaxed);
                    println!("[TRAY] {}", current);
                }),
                options: vec![
                    RadioItem {
                        label: "Normal".into(),
                        ..Default::default()
                    },
                    RadioItem {
                        label: "Anki".into(),
                        ..Default::default()
                    },
                ],
                // ..Default::default()
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
