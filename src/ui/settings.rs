use egui::{Context, Window};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub show_hidden_files: bool,
    pub default_view_mode: String,
    pub theme: String,
    pub cache_ttl_seconds: u64,
    pub recent_folders: Vec<String>,
    pub recent_files: Vec<String>,
    pub favorites: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_hidden_files: false,
            default_view_mode: "list".to_string(),
            theme: "light".to_string(),
            cache_ttl_seconds: 30,
            recent_folders: Vec::new(),
            recent_files: Vec::new(),
            favorites: Vec::new(),
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let settings_path = Self::settings_path();
        if let Ok(content) = fs::read_to_string(&settings_path) {
            if let Ok(settings) = toml::from_str(&content) {
                return settings;
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = Self::settings_path();
        if let Some(parent) = settings_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string(self)?;
        fs::write(&settings_path, content)?;
        Ok(())
    }

    fn settings_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("file-explorer");
        path.push("settings.toml");
        path
    }
}

pub fn render_settings(ctx: &Context, settings: &mut Settings) {
    let mut show = true;
    Window::new("Settings")
        .collapsible(true)
        .resizable(true)
        .open(&mut show)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("Preferences");

                ui.separator();

                ui.checkbox(&mut settings.show_hidden_files, "Show hidden files");

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Default view mode:");
                    ui.radio_value(&mut settings.default_view_mode, "list".to_string(), "List");
                    ui.radio_value(&mut settings.default_view_mode, "grid".to_string(), "Grid");
                    ui.radio_value(&mut settings.default_view_mode, "details".to_string(), "Details");
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Theme:");
                    ui.radio_value(&mut settings.theme, "light".to_string(), "Light");
                    ui.radio_value(&mut settings.theme, "dark".to_string(), "Dark");
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Cache TTL (seconds):");
                    ui.add(egui::Slider::new(&mut settings.cache_ttl_seconds, 1..=300));
                });

                ui.separator();

                if ui.button("Save").clicked() {
                    if let Err(e) = settings.save() {
                        eprintln!("Failed to save settings: {}", e);
                    }
                }
            });
        });
}

