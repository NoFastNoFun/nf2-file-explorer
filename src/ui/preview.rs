use egui::{Context, Window, ScrollArea, Image, Vec2};
use crate::app::AppState;
use tokio::runtime::Runtime;

pub fn render_preview(ctx: &Context, app: &mut AppState, rt: &Runtime) {
    if !app.show_preview {
        return;
    }

    let preview_path = if let Some(path) = &app.preview_path {
        path.clone()
    } else if let Some(selected) = app.selected_indices.first() {
        if let Some(entry) = app.entries.get(*selected) {
            entry.path.clone()
        } else {
            return;
        }
    } else {
        return;
    };

    Window::new("Preview")
        .collapsible(true)
        .resizable(true)
        .default_size([300.0, 400.0])
        .show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                if preview_path.is_dir() {
                    ui.label("Preview not available for folders");
                    return;
                }

                let ext = preview_path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                match ext.as_str() {
                    "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" => {
                        // Try to load and display image
                        if let Ok(bytes) = std::fs::read(&preview_path) {
                            if let Ok(img) = image::load_from_memory(&bytes) {
                                let rgba = img.to_rgba8();
                                let size = [rgba.width() as usize, rgba.height() as usize];
                                let pixels = rgba.as_flat_samples();
                                
                                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                                let texture = ctx.load_texture(
                                    "preview_image",
                                    color_image,
                                    Default::default()
                                );
                                ui.add(Image::new(&texture).max_size(Vec2::new(280.0, 280.0)));
                            } else {
                                ui.label("Failed to decode image");
                            }
                        } else {
                            ui.label("Failed to read image file");
                        }
                    }
                    "txt" | "md" | "log" | "rs" | "py" | "js" | "ts" | "json" | "xml" | "html" | "css" => {
                        // Show text preview
                        if let Ok(content) = std::fs::read_to_string(&preview_path) {
                            let lines: Vec<&str> = content.lines().take(100).collect();
                            let preview_text = lines.join("\n");
                            ui.label(egui::RichText::new(preview_text).monospace());
                            if content.lines().count() > 100 {
                                ui.label(format!("... ({} more lines)", content.lines().count() - 100));
                            }
                        } else {
                            ui.label("Failed to read file");
                        }
                    }
                    _ => {
                        ui.label("Preview not available for this file type");
                    }
                }

                ui.separator();
                ui.label(format!("Path: {}", preview_path.display()));
            });
        });
}

