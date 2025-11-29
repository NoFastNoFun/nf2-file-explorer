use egui::{Ui, ScrollArea, TextEdit, Grid, Context};
use crate::app::AppState;
use tokio::runtime::Runtime;
use crate::ui::context_menu;
use crate::ui::icons;

#[derive(PartialEq, Clone, Copy)]
pub enum SortColumn {
    Name,
    Size,
    Type,
    Modified,
}

pub struct FileListView {
    sort_column: SortColumn,
    sort_ascending: bool,
    rename_index: Option<usize>,
    rename_text: String,
}

impl FileListView {
    pub fn new() -> Self {
        Self {
            sort_column: SortColumn::Name,
            sort_ascending: true,
            rename_index: None,
            rename_text: String::new(),
        }
    }

    pub fn render(&mut self, ui: &mut Ui, app: &mut AppState, rt: &Runtime, ctx: &Context) {
        let entries: Vec<_> = if app.is_searching {
            app.search_results.clone()
        } else {
            app.entries.clone()
        };

        let mut sorted_entries = entries.clone();
        sorted_entries.sort_by(|a, b| {
            let ordering = match self.sort_column {
                SortColumn::Name => a.name.cmp(&b.name),
                SortColumn::Size => a.size.cmp(&b.size),
                SortColumn::Type => a.file_type().cmp(&b.file_type()),
                SortColumn::Modified => a.modified.cmp(&b.modified),
            };

            if self.sort_ascending {
                ordering
            } else {
                ordering.reverse()
            }
        });

        ScrollArea::vertical()
            .show(ui, |ui| {
                Grid::new("file_list_grid")
                    .num_columns(4)
                    .spacing([8.0, 4.0])
                    .show(ui, |ui| {
                        // Header row
                        let name_header = if self.sort_column == SortColumn::Name {
                            if self.sort_ascending { "Name ▲" } else { "Name ▼" }
                        } else {
                            "Name"
                        };
                        if ui.selectable_label(false, name_header).clicked() {
                            if self.sort_column == SortColumn::Name {
                                self.sort_ascending = !self.sort_ascending;
                            } else {
                                self.sort_column = SortColumn::Name;
                                self.sort_ascending = true;
                            }
                        }

                        let size_header = if self.sort_column == SortColumn::Size {
                            if self.sort_ascending { "Size ▲" } else { "Size ▼" }
                        } else {
                            "Size"
                        };
                        if ui.selectable_label(false, size_header).clicked() {
                            if self.sort_column == SortColumn::Size {
                                self.sort_ascending = !self.sort_ascending;
                            } else {
                                self.sort_column = SortColumn::Size;
                                self.sort_ascending = true;
                            }
                        }

                        let type_header = if self.sort_column == SortColumn::Type {
                            if self.sort_ascending { "Type ▲" } else { "Type ▼" }
                        } else {
                            "Type"
                        };
                        if ui.selectable_label(false, type_header).clicked() {
                            if self.sort_column == SortColumn::Type {
                                self.sort_ascending = !self.sort_ascending;
                            } else {
                                self.sort_column = SortColumn::Type;
                                self.sort_ascending = true;
                            }
                        }

                        let modified_header = if self.sort_column == SortColumn::Modified {
                            if self.sort_ascending { "Modified ▲" } else { "Modified ▼" }
                        } else {
                            "Modified"
                        };
                        if ui.selectable_label(false, modified_header).clicked() {
                            if self.sort_column == SortColumn::Modified {
                                self.sort_ascending = !self.sort_ascending;
                            } else {
                                self.sort_column = SortColumn::Modified;
                                self.sort_ascending = true;
                            }
                        }
                        ui.end_row();

                        // Data rows
                        for (idx, entry) in sorted_entries.iter().enumerate() {
                            let original_idx = entries.iter().position(|e| e.path == entry.path).unwrap_or(idx);
                            let is_selected = app.selected_indices.contains(&original_idx);

                            if let Some(requested_idx) = app.rename_request.take() {
                                if requested_idx == original_idx {
                                    self.rename_index = Some(original_idx);
                                    self.rename_text = entry.name.clone();
                                }
                            }

                            if self.rename_index == Some(original_idx) {
                                let _response = ui.add(
                                    TextEdit::singleline(&mut self.rename_text)
                                        .desired_width(ui.available_width())
                                );

                                if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                    let new_path = entry.path.parent().unwrap().join(&self.rename_text);
                                    if let Err(e) = rt.block_on(tokio::fs::rename(&entry.path, &new_path)) {
                                        app.error_message = Some(format!("Rename failed: {}", e));
                                    } else {
                                        rt.block_on(app.refresh());
                                    }
                                    self.rename_index = None;
                                }

                                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                                    self.rename_index = None;
                                }

                                ui.label("");
                                ui.label("");
                                ui.label("");
                            } else {
                                let (icon, color) = icons::get_file_icon(entry);
                                ui.horizontal(|ui| {
                                    ui.colored_label(color, icon);
                                    let response = ui.selectable_label(is_selected, &entry.name);

                                    if response.clicked() {
                                        if ui.input(|i| i.modifiers.ctrl) {
                                            if is_selected {
                                                app.selected_indices.retain(|&i| i != original_idx);
                                            } else {
                                                app.selected_indices.push(original_idx);
                                            }
                                        } else if ui.input(|i| i.modifiers.shift) {
                                            if let Some(&last_idx) = app.selected_indices.last() {
                                                let start = last_idx.min(original_idx);
                                                let end = last_idx.max(original_idx);
                                                app.selected_indices = (start..=end).collect();
                                            } else {
                                                app.selected_indices = vec![original_idx];
                                            }
                                        } else {
                                            app.selected_indices = vec![original_idx];
                                        }
                                    }

                                    if response.double_clicked() {
                                        if entry.is_dir {
                                            rt.block_on(app.navigate_to(entry.path.clone()));
                                        } else {
                                            app.properties_path = Some(entry.path.clone());
                                            app.show_properties = true;
                                            app.add_to_recent_files(entry.path.clone());
                                        }
                                    }

                                    if response.secondary_clicked() {
                                        if !app.selected_indices.contains(&original_idx) {
                                            app.selected_indices = vec![original_idx];
                                        }
                                        context_menu::show_file_context_menu(
                                            ctx,
                                            &response,
                                            app,
                                            rt,
                                            entry.path.clone(),
                                            original_idx,
                                        );
                                    }
                                });

                                ui.label(entry.format_size());
                                ui.label(entry.file_type());
                                ui.label(entry.format_modified());
                            }
                            ui.end_row();
                        }
                    });
            });
    }
}
