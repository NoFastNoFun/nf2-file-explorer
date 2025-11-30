use egui::{Ui, Context, ScrollArea, Grid};
use std::path::PathBuf;
use crate::app::AppState;
use crate::ui::icons;
use tokio::runtime::Runtime;

pub struct HomePage {
    show_home: bool,
}

impl HomePage {
    pub fn new() -> Self {
        Self {
            show_home: true,
        }
    }

    pub fn should_show_home(&self) -> bool {
        self.show_home
    }

    pub fn set_show_home(&mut self, show: bool) {
        self.show_home = show;
    }

    pub fn render(&mut self, ui: &mut Ui, app: &mut AppState, rt: &Runtime, _ctx: &Context) {
        ScrollArea::vertical().show(ui, |ui| {
            ui.vertical(|ui| {
                ui.heading("Home");

                // Special Folders Section
                ui.separator();
                ui.label(egui::RichText::new("Special Folders").heading());
                
                let special_folders = get_special_folders();
                Grid::new("special_folders_grid")
                    .num_columns(4)
                    .spacing([16.0, 8.0])
                    .show(ui, |ui| {
                        for folder in &special_folders {
                            if let Some(path) = &folder.path {
                                let (icon, color) = if folder.is_dir {
                                    ("📁", egui::Color32::from_rgb(100, 150, 255))
                                } else {
                                    icons::get_file_icon(&crate::fs::directory::FileEntry {
                                        name: folder.name.clone(),
                                        path: path.clone(),
                                        is_dir: false,
                                        size: 0,
                                        modified: std::time::SystemTime::now(),
                                        extension: None,
                                        git_status: None,
                                        is_symlink: false,
                                        is_hardlink: false,
                                        link_target: None,
                                    })
                                };
                                
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.colored_label(color, icon);
                                        if ui.button(&folder.name).clicked() {
                                            rt.block_on(app.navigate_to(path.clone()));
                                            self.show_home = false;
                                        }
                                    });
                                });
                            }
                            ui.end_row();
                        }
                    });

                // Favorites Section
                ui.separator();
                ui.label(egui::RichText::new("Favorites").heading());
                
                let favorites = app.settings.favorites.clone();
                if favorites.is_empty() {
                    ui.label("No favorites yet. Right-click a folder to add it to favorites.");
                } else {
                    Grid::new("favorites_grid")
                        .num_columns(4)
                        .spacing([16.0, 8.0])
                        .show(ui, |ui| {
                            for favorite in &favorites {
                                let path = PathBuf::from(favorite);
                                if path.exists() {
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.colored_label(egui::Color32::from_rgb(100, 150, 255), "📁");
                                            let display_name = path.file_name()
                                                .and_then(|n| n.to_str())
                                                .map(|s| s.to_string())
                                                .unwrap_or_else(|| favorite.clone());
                                            if ui.button(&display_name).clicked() {
                                                rt.block_on(app.navigate_to(path.clone()));
                                                self.show_home = false;
                                            }
                                        });
                                    });
                                }
                                ui.end_row();
                            }
                        });
                }

                // Recent Files and Folders Section
                ui.separator();
                ui.label(egui::RichText::new("Recent").heading());
                
                let recent_items = get_recent_items(app);
                if recent_items.is_empty() {
                    ui.label("No recent items.");
                } else {
                    Grid::new("recent_grid")
                        .num_columns(4)
                        .spacing([16.0, 8.0])
                        .show(ui, |ui| {
                            for item in &recent_items {
                                let (icon, color) = icons::get_file_icon(item);
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.colored_label(color, icon);
                                        let label = if item.is_dir {
                                            format!("{} (folder)", item.name)
                                        } else {
                                            item.name.clone()
                                        };
                                        if ui.button(&label).clicked() {
                                            if item.is_dir {
                                                rt.block_on(app.navigate_to(item.path.clone()));
                                                self.show_home = false;
                                            } else {
                                                app.ui.properties_path = Some(item.path.clone());
                                                app.ui.show_properties = true;
                                            }
                                        }
                                    });
                                    ui.label(item.format_modified());
                                });
                                ui.end_row();
                            }
                        });
                }
            });
        });
    }
}

struct SpecialFolder {
    name: String,
    path: Option<PathBuf>,
    is_dir: bool,
}

fn get_special_folders() -> Vec<SpecialFolder> {
    let mut folders = Vec::new();

    // Get user home directory
    if let Some(home) = dirs::home_dir() {
        // Desktop
        let desktop = home.join("Desktop");
        if desktop.exists() {
            folders.push(SpecialFolder {
                name: "Desktop".to_string(),
                path: Some(desktop),
                is_dir: true,
            });
        }

        // Downloads
        let downloads = home.join("Downloads");
        if downloads.exists() {
            folders.push(SpecialFolder {
                name: "Downloads".to_string(),
                path: Some(downloads),
                is_dir: true,
            });
        }

        // Documents
        let documents = home.join("Documents");
        if documents.exists() {
            folders.push(SpecialFolder {
                name: "Documents".to_string(),
                path: Some(documents),
                is_dir: true,
            });
        }

        // Pictures
        let pictures = home.join("Pictures");
        if pictures.exists() {
            folders.push(SpecialFolder {
                name: "Pictures".to_string(),
                path: Some(pictures),
                is_dir: true,
            });
        }

        // Videos
        let videos = home.join("Videos");
        if videos.exists() {
            folders.push(SpecialFolder {
                name: "Videos".to_string(),
                path: Some(videos),
                is_dir: true,
            });
        }

        // Music
        let music = home.join("Music");
        if music.exists() {
            folders.push(SpecialFolder {
                name: "Music".to_string(),
                path: Some(music),
                is_dir: true,
            });
        }
    }

    // Windows-specific: Add user directory
    #[cfg(windows)]
    {
        if let Ok(user_profile) = std::env::var("USERPROFILE") {
            let user_path = PathBuf::from(&user_profile);
            if user_path.exists() {
                folders.push(SpecialFolder {
                    name: "User Directory".to_string(),
                    path: Some(user_path),
                    is_dir: true,
                });
            }
        }
    }

    folders
}

fn get_recent_items(app: &AppState) -> Vec<crate::fs::directory::FileEntry> {
    let mut items = Vec::new();
    
    // Get recent folders from settings
    for folder_path in &app.settings.recent_folders {
        let path = PathBuf::from(folder_path);
        if let Ok(metadata) = std::fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                items.push(crate::fs::directory::FileEntry {
                    name: path.file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| folder_path.clone()),
                    path: path.clone(),
                    is_dir: metadata.is_dir(),
                    size: metadata.len(),
                    modified,
                    extension: path.extension()
                        .and_then(|e| e.to_str())
                        .map(|s| s.to_string()),
                    git_status: None,
                    is_symlink: false,
                    is_hardlink: false,
                    link_target: None,
                });
            }
        }
    }

    // Add recent files from recent_files in settings
    for file_path in &app.settings.recent_files {
        let path = PathBuf::from(file_path);
        if let Ok(metadata) = std::fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                items.push(crate::fs::directory::FileEntry {
                    name: path.file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| file_path.clone()),
                    path: path.clone(),
                    is_dir: false,
                    size: metadata.len(),
                    modified,
                    extension: path.extension()
                        .and_then(|e| e.to_str())
                        .map(|s| s.to_string()),
                    git_status: None,
                    is_symlink: false,
                    is_hardlink: false,
                    link_target: None,
                });
            }
        }
    }

    // Sort by modified time, most recent first
    items.sort_by(|a, b| b.modified.cmp(&a.modified));
    items.truncate(20); // Limit to 20 most recent
    items
}

