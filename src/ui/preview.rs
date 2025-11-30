use egui::{Context, Window, ScrollArea, Image, Vec2, Color32};
use crate::app::AppState;
use tokio::runtime::Runtime;
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;
use crate::ui::json_viewer;

#[derive(Clone, Copy, PartialEq)]
pub enum PreviewMode {
    Raw,
    Structured,
}

pub fn render_preview(ctx: &Context, app: &mut AppState, _rt: &Runtime) {
    if !app.ui.show_preview {
        return;
    }

    let preview_path = if let Some(path) = &app.ui.preview_path {
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
                    "json" => {
                        if let Ok(content) = std::fs::read_to_string(&preview_path) {
                            ui.horizontal(|ui| {
                                ui.radio_value(&mut app.ui.preview_mode, PreviewMode::Structured, "Structured");
                                ui.radio_value(&mut app.ui.preview_mode, PreviewMode::Raw, "Raw");
                            });
                            ui.separator();
                            
                            match app.ui.preview_mode {
                                PreviewMode::Structured => {
                                    json_viewer::render_json_viewer(ui, &content);
                                }
                                PreviewMode::Raw => {
                                    let syntax_set = SyntaxSet::load_defaults_newlines();
                                    let theme_set = ThemeSet::load_defaults();
                                    let theme = &theme_set.themes["base16-ocean.dark"];
                                    let syntax = syntax_set.find_syntax_by_extension("json")
                                        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());
                                    let mut highlighter = HighlightLines::new(syntax, theme);
                                    let lines: Vec<&str> = content.lines().take(500).collect();
                                    for (line_num, line) in lines.iter().enumerate() {
                                        let regions = highlighter.highlight_line(line, &syntax_set).unwrap_or_default();
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new(format!("{:4} ", line_num + 1)).monospace().color(Color32::GRAY));
                                            for (style, text) in regions {
                                                let color = Color32::from_rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                                                ui.colored_label(color, egui::RichText::new(text).monospace());
                                            }
                                        });
                                    }
                                }
                            }
                        } else {
                            ui.label("Failed to read file");
                        }
                    }
                    "yaml" | "yml" => {
                        if let Ok(content) = std::fs::read_to_string(&preview_path) {
                            ui.horizontal(|ui| {
                                ui.radio_value(&mut app.ui.preview_mode, PreviewMode::Structured, "Structured");
                                ui.radio_value(&mut app.ui.preview_mode, PreviewMode::Raw, "Raw");
                            });
                            ui.separator();
                            
                            match app.ui.preview_mode {
                                PreviewMode::Structured => {
                                    json_viewer::render_yaml_viewer(ui, &content);
                                }
                                PreviewMode::Raw => {
                                    let syntax_set = SyntaxSet::load_defaults_newlines();
                                    let theme_set = ThemeSet::load_defaults();
                                    let theme = &theme_set.themes["base16-ocean.dark"];
                                    let syntax = syntax_set.find_syntax_by_extension("yaml")
                                        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());
                                    let mut highlighter = HighlightLines::new(syntax, theme);
                                    let lines: Vec<&str> = content.lines().take(500).collect();
                                    for (line_num, line) in lines.iter().enumerate() {
                                        let regions = highlighter.highlight_line(line, &syntax_set).unwrap_or_default();
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new(format!("{:4} ", line_num + 1)).monospace().color(Color32::GRAY));
                                            for (style, text) in regions {
                                                let color = Color32::from_rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                                                ui.colored_label(color, egui::RichText::new(text).monospace());
                                            }
                                        });
                                    }
                                }
                            }
                        } else {
                            ui.label("Failed to read file");
                        }
                    }
                    "txt" | "md" | "log" | "rs" | "py" | "js" | "ts" | "xml" | "html" | "css" => {
                        // Show syntax-highlighted preview
                        if let Ok(content) = std::fs::read_to_string(&preview_path) {
                            let syntax_set = SyntaxSet::load_defaults_newlines();
                            let theme_set = ThemeSet::load_defaults();
                            let theme = &theme_set.themes["base16-ocean.dark"];
                            
                            let syntax = syntax_set.find_syntax_by_extension(&ext)
                                .or_else(|| syntax_set.find_syntax_by_name(&ext))
                                .unwrap_or_else(|| syntax_set.find_syntax_plain_text());
                            
                            let mut highlighter = HighlightLines::new(syntax, theme);
                            
                            let lines: Vec<&str> = content.lines().take(500).collect();
                            
                            for (line_num, line) in lines.iter().enumerate() {
                                let regions = highlighter.highlight_line(line, &syntax_set).unwrap_or_default();
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(format!("{:4} ", line_num + 1)).monospace().color(Color32::GRAY));
                                    for (style, text) in regions {
                                        let color = Color32::from_rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                                        ui.colored_label(color, egui::RichText::new(text).monospace());
                                    }
                                });
                            }
                            
                            if content.lines().count() > 500 {
                                ui.label(format!("... ({} more lines)", content.lines().count() - 500));
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

