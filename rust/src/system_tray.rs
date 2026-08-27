use crate::types::{get_profiles, set_active_profile};
use image::GenericImageView;
use ksni::menu::*;
use std::process::Command;
use std::sync::atomic::Ordering;
use std::sync::{Arc, atomic::AtomicUsize, LazyLock};

static ICON: LazyLock<ksni::Icon> = LazyLock::new(|| {
    let img = image::load_from_memory_with_format(
        include_bytes!(concat!(env!("OUT_DIR"), "/icon.png")),
        image::ImageFormat::Png,
    )
    .expect("valid tray icon");
    let (width, height) = img.dimensions();
    let mut data = img.into_rgba8().into_vec();
    assert_eq!(data.len() % 4, 0);
    for pixel in data.chunks_exact_mut(4) {
        pixel.rotate_right(1);
    }
    ksni::Icon {
        width: width as i32,
        height: height as i32,
        data,
    }
});

#[derive(Debug)]
pub struct MyTray {
    pub selected_option: Arc<AtomicUsize>,
}

impl ksni::Tray for MyTray {
    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
    }

    fn icon_name(&self) -> String {
        String::new()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        vec![ICON.clone()]
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
