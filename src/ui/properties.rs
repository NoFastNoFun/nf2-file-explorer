use egui::{Context, Window};
use crate::app::AppState;

pub fn render_properties(ctx: &Context, app: &mut AppState) {
    if !app.ui.show_properties {
        return;
    }

    if let Some(ref path) = app.ui.properties_path {
        Window::new("Properties")
            .collapsible(false)
            .resizable(true)
            .show(ctx, |ui| {
                if let Ok(metadata) = std::fs::metadata(path) {
                    ui.vertical(|ui| {
                        ui.heading("File Properties");

                        ui.separator();

                        ui.label(format!("Name: {}", path.file_name().unwrap_or_default().to_string_lossy()));
                        ui.label(format!("Path: {}", path.to_string_lossy()));

                        ui.separator();

                        if metadata.is_dir() {
                            ui.label("Type: Folder");
                        } else {
                            ui.label(format!("Type: File"));
                            if let Some(ext) = path.extension() {
                                ui.label(format!("Extension: {}", ext.to_string_lossy()));
                            }
                        }

                        ui.separator();

                        ui.label(format!("Size: {}", format_bytes(metadata.len())));

                        if let Ok(modified) = metadata.modified() {
                            let dt: chrono::DateTime<chrono::Local> = modified.into();
                            ui.label(format!("Modified: {}", dt.format("%Y-%m-%d %H:%M:%S")));
                        }

                        if let Ok(created) = metadata.created() {
                            let dt: chrono::DateTime<chrono::Local> = created.into();
                            ui.label(format!("Created: {}", dt.format("%Y-%m-%d %H:%M:%S")));
                        }

                        ui.separator();

                        #[cfg(windows)]
                        {
                            use winapi::um::winnt::{FILE_ATTRIBUTE_READONLY, FILE_ATTRIBUTE_HIDDEN, FILE_ATTRIBUTE_SYSTEM, FILE_ATTRIBUTE_ARCHIVE};
                            use winapi::um::fileapi::GetFileAttributesW;
                            use std::os::windows::ffi::OsStrExt;
                            
                            let mut attr_list = Vec::new();
                            
                            let wide_path: Vec<u16> = path.as_os_str()
                                .encode_wide()
                                .chain(std::iter::once(0))
                                .collect();
                            
                            unsafe {
                                let attrs = GetFileAttributesW(wide_path.as_ptr());
                                if attrs != 0xFFFFFFFF {
                                    if attrs & FILE_ATTRIBUTE_READONLY != 0 {
                                        attr_list.push("Read-only");
                                    }
                                    if attrs & FILE_ATTRIBUTE_HIDDEN != 0 {
                                        attr_list.push("Hidden");
                                    }
                                    if attrs & FILE_ATTRIBUTE_SYSTEM != 0 {
                                        attr_list.push("System");
                                    }
                                    if attrs & FILE_ATTRIBUTE_ARCHIVE != 0 {
                                        attr_list.push("Archive");
                                    }
                                }
                            }

                            if !attr_list.is_empty() {
                                ui.label(format!("Attributes: {}", attr_list.join(", ")));
                            }
                        }

                        ui.separator();

                        if ui.button("Close").clicked() {
                            app.ui.show_properties = false;
                        }
                    });
                } else {
                    ui.label("Failed to read file properties");
                    if ui.button("Close").clicked() {
                        app.ui.show_properties = false;
                    }
                }
            });
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

