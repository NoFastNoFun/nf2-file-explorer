mod app;
mod fs;
mod ui;
mod utils;

use eframe::egui;
use app::AppState;
use ui::{
    toolbar, status_bar, search_bar, properties, preview, settings, home,
    tree_view::TreeView, file_list::FileListView,
};

struct FileExplorerApp {
    app_state: AppState,
    tree_view: TreeView,
    file_list_view: FileListView,
    home_page: home::HomePage,
    rt: tokio::runtime::Runtime,
}

impl FileExplorerApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let rt = tokio::runtime::Runtime::new()
            .expect("Failed to create tokio runtime");
        let mut app_state = AppState::new();
        
        let current_path = app_state.navigation.current_path.clone();
        app_state.navigation.path_input = current_path.to_string_lossy().to_string();
        rt.block_on(app_state.navigate_to(current_path));

        Self {
            app_state,
            tree_view: TreeView::new(),
            file_list_view: FileListView::new(),
            home_page: home::HomePage::new(),
            rt,
        }
    }

}

impl eframe::App for FileExplorerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle keyboard shortcuts
        // Check for file system changes
        if self.app_state.check_file_system_events() {
            // Refresh if files changed
            self.rt.block_on(self.app_state.refresh());
        }

        ctx.input(|i| {
            let modifiers = i.modifiers;
            
            // Ctrl+C - Copy
            if modifiers.ctrl && i.key_pressed(egui::Key::C) {
                if !self.app_state.selected_indices.is_empty() {
                    self.app_state.copy_selected();
                }
            }
            
            // Ctrl+X - Cut
            if modifiers.ctrl && i.key_pressed(egui::Key::X) {
                if !self.app_state.selected_indices.is_empty() {
                    self.app_state.cut_selected();
                }
            }
            
            // Ctrl+V - Paste
            if modifiers.ctrl && i.key_pressed(egui::Key::V) {
                if self.app_state.has_clipboard_content() {
                    match self.rt.block_on(self.app_state.paste_clipboard()) {
                        Ok(_) => {}
                        Err(e) => {
                            if !matches!(e, crate::fs::error::FileOperationError::Cancelled) {
                                self.app_state.ui.set_error(e.to_string());
                            }
                        }
                    }
                }
            }
            
            // Delete - Delete selected
            if i.key_pressed(egui::Key::Delete) {
                if !self.app_state.selected_indices.is_empty() {
                    self.app_state.request_delete();
                }
            }
            
            // F2 - Rename
            if i.key_pressed(egui::Key::F2) {
                if let Some(&first_selected) = self.app_state.selected_indices.first() {
                    self.app_state.ui.rename_request = Some(first_selected);
                }
            }
            
            // F5 - Refresh
            if i.key_pressed(egui::Key::F5) {
                self.rt.block_on(self.app_state.refresh());
            }
            
            // Ctrl+A - Select all
            if modifiers.ctrl && i.key_pressed(egui::Key::A) {
                self.app_state.select_all();
            }
            
            // Escape - Clear selection and filter
            if i.key_pressed(egui::Key::Escape) {
                self.app_state.clear_selection();
                self.app_state.ui.filter_text.clear();
            }
            
            // Type-to-filter: capture typed characters when not in text input
            if !i.modifiers.ctrl && !i.modifiers.alt {
                for event in &i.events {
                    if let egui::Event::Text(text) = event {
                        if !text.is_empty() && text.chars().all(|c| c.is_alphanumeric() || c.is_whitespace()) {
                            self.app_state.ui.filter_text.push_str(text);
                        }
                    }
                }
            }
            
            // Ctrl+Z - Undo
            if modifiers.ctrl && i.key_pressed(egui::Key::Z) && !modifiers.shift {
                if self.app_state.can_undo() {
                    match self.rt.block_on(self.app_state.undo()) {
                        Ok(_) => {}
                        Err(e) => self.app_state.ui.set_error(e.to_string()),
                    }
                }
            }
            
            // Ctrl+Y or Ctrl+Shift+Z - Redo
            if (modifiers.ctrl && i.key_pressed(egui::Key::Y)) ||
               (modifiers.ctrl && modifiers.shift && i.key_pressed(egui::Key::Z)) {
                if self.app_state.can_redo() {
                    match self.rt.block_on(self.app_state.redo()) {
                        Ok(_) => {}
                        Err(e) => self.app_state.ui.set_error(e.to_string()),
                    }
                }
            }
            
            // Spacebar - Quick Look / Preview
            if i.key_pressed(egui::Key::Space) {
                if let Some(&first_selected) = self.app_state.selected_indices.first() {
                    if let Some(entry) = self.app_state.entries.get(first_selected) {
                        self.app_state.ui.preview_path = Some(entry.path.clone());
                        self.app_state.ui.show_preview = true;
                    }
                }
            }
        });

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            toolbar::render_toolbar(ui, &mut self.app_state, &self.rt);
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            status_bar::render_status_bar(ui, &self.app_state);
        });

        egui::SidePanel::left("tree_panel")
            .resizable(true)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Folders");
                self.tree_view.render(ui, &mut self.app_state, &self.rt);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.app_state.ui.show_home {
                self.home_page.set_show_home(true);
                self.home_page.render(ui, &mut self.app_state, &self.rt, ctx);
            } else if self.app_state.use_multi_pane {
                let multi_pane_opt = self.app_state.multi_pane.take();
                if let Some(mut multi_pane) = multi_pane_opt {
                    multi_pane.render(ui, ctx, &mut self.app_state, &self.rt);
                    self.app_state.multi_pane = Some(multi_pane);
                }
            } else {
                self.home_page.set_show_home(false);
                ui.vertical(|ui| {
                    search_bar::render_search_bar(ui, &mut self.app_state, &self.rt);

                    ui.separator();

                    if self.app_state.loading {
                        ui.centered_and_justified(|ui| {
                            ui.spinner();
                            ui.label("Loading...");
                        });
                    } else if let Some(error) = self.app_state.ui.error_message.clone() {
                        ui.horizontal(|ui| {
                            ui.colored_label(egui::Color32::RED, &error);
                            if ui.button("✕").clicked() {
                                self.app_state.ui.clear_error();
                            }
                        });
                        if self.app_state.ui.should_auto_dismiss_error() {
                            self.app_state.ui.clear_error();
                        }
                    } else {
                        self.file_list_view.render(ui, &mut self.app_state, &self.rt, ctx);
                    }
                });
            }
        });

        properties::render_properties(ctx, &mut self.app_state);
        preview::render_preview(ctx, &mut self.app_state, &self.rt);
        crate::ui::hash_dialog::render_hash_dialog(ctx, &mut self.app_state, &self.rt);
        crate::ui::progress_dialog::render_progress_dialog(ctx, &mut self.app_state.progress);
        
        if self.app_state.ui.show_create_link_dialog {
            let mut show = self.app_state.ui.show_create_link_dialog;
            let source_path = self.app_state.ui.link_source_path.clone();
            let link_is_symlink = self.app_state.ui.link_is_symlink;
            let current_path = self.app_state.navigation.current_path.clone();
            let mut link_target_name = self.app_state.ui.link_target_name.clone();
            egui::Window::new("Create Link")
                .collapsible(false)
                .resizable(false)
                .open(&mut show)
                .show(ctx, |ui| {
                    if let Some(ref source) = source_path {
                        ui.label(format!("Source: {}", source.display()));
                        ui.label("Link name:");
                        ui.text_edit_singleline(&mut link_target_name);
                        ui.horizontal(|ui| {
                            if ui.button("Create").clicked() {
                                let target = current_path.join(&link_target_name);
                                if link_is_symlink {
                                    if let Err(e) = std::os::windows::fs::symlink_file(source, &target) {
                                        self.app_state.ui.set_error(format!("Failed to create symlink: {}", e));
                                    } else {
                                        self.rt.block_on(self.app_state.refresh());
                                    }
                                } else {
                                    if let Err(e) = std::fs::hard_link(source, &target) {
                                        self.app_state.ui.set_error(format!("Failed to create hard link: {}", e));
                                    } else {
                                        self.rt.block_on(self.app_state.refresh());
                                    }
                                }
                                self.app_state.ui.show_create_link_dialog = false;
                                self.app_state.ui.link_target_name.clear();
                            }
                            if ui.button("Cancel").clicked() {
                                self.app_state.ui.show_create_link_dialog = false;
                                self.app_state.ui.link_target_name.clear();
                            }
                        });
                    }
                });
            self.app_state.ui.show_create_link_dialog = show;
            self.app_state.ui.link_target_name = link_target_name;
        }
        
        if self.app_state.ui.show_settings {
            let old_show_git_status = self.app_state.settings.show_git_status;
            let old_show_ignored_files = self.app_state.settings.show_ignored_files;
            let old_cache_ttl = self.app_state.settings.cache_ttl_seconds;
            
            settings::render_settings(ctx, &mut self.app_state.settings, &mut self.app_state.ui.show_settings);
            
            let settings_changed = old_show_git_status != self.app_state.settings.show_git_status
                || old_show_ignored_files != self.app_state.settings.show_ignored_files;
            
            if old_show_ignored_files != self.app_state.settings.show_ignored_files {
                self.app_state.show_ignored_files = self.app_state.settings.show_ignored_files;
            }
            
            if old_cache_ttl != self.app_state.settings.cache_ttl_seconds {
                self.app_state.cache = crate::fs::directory::DirectoryCache::new(
                    std::time::Duration::from_secs(self.app_state.settings.cache_ttl_seconds)
                );
            }
            
            if settings_changed {
                let _ = self.app_state.settings.save();
                self.rt.block_on(self.app_state.refresh());
            }
        }

        if self.app_state.ui.show_command_dialog {
            let mut show = true;
            egui::Window::new("Run Command")
                .collapsible(false)
                .resizable(false)
                .open(&mut show)
                .show(ctx, |ui| {
                    ui.label("Enter command to run:");
                    ui.text_edit_singleline(&mut self.app_state.ui.command_input);
                    ui.horizontal(|ui| {
                        if ui.button("Run").clicked() {
                            if let Some(ref path) = self.app_state.ui.command_dialog_path {
                                let target_path = if path.is_dir() {
                                    path.clone()
                                } else {
                                    path.parent().unwrap_or(path).to_path_buf()
                                };
                                if let Err(e) = crate::fs::terminal::run_command_in_terminal(&target_path, &self.app_state.ui.command_input) {
                                    self.app_state.ui.set_error(e);
                                }
                                self.app_state.ui.show_command_dialog = false;
                                self.app_state.ui.command_input.clear();
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            self.app_state.ui.show_command_dialog = false;
                            self.app_state.ui.command_input.clear();
                        }
                    });
                });
            if !show {
                self.app_state.ui.show_command_dialog = false;
            }
        }

        // Handle dialogs
        match self.app_state.delete_dialog.render(ctx) {
            crate::ui::dialogs::DialogAction::Confirm => {
                if self.app_state.pending_delete {
                    match self.rt.block_on(self.app_state.execute_delete()) {
                        Ok(_) => {}
                        Err(e) => self.app_state.ui.set_error(e.to_string()),
                    }
                    self.app_state.pending_delete = false;
                }
            }
            crate::ui::dialogs::DialogAction::Cancel => {
                self.app_state.pending_delete = false;
            }
            _ => {}
        }

        match self.app_state.overwrite_dialog.render(ctx) {
            crate::ui::dialogs::DialogAction::Confirm => {
                if self.app_state.pending_paste {
                    match self.rt.block_on(self.app_state.execute_paste_with_overwrite(true)) {
                        Ok(_) => {}
                        Err(e) => {
                            if !matches!(e, crate::fs::error::FileOperationError::Cancelled) {
                                self.app_state.ui.set_error(e.to_string());
                            }
                        }
                    }
                    self.app_state.pending_paste = false;
                }
            }
            crate::ui::dialogs::DialogAction::Cancel => {
                self.app_state.pending_paste = false;
            }
            _ => {}
        }

        if self.app_state.search.is_searching && !self.app_state.search.options.query.is_empty() {
            let search_results = self.app_state.search.results.clone();
            egui::Window::new("Search Results")
                .collapsible(true)
                .resizable(true)
                .default_size([400.0, 300.0])
                .show(ctx, |ui| {
                    ui.label(format!("Found {} results", search_results.len()));
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for result in &search_results {
                            let icon = if result.is_dir { "📁" } else { "📄" };
                            if ui.selectable_label(false, format!("{} {}", icon, result.name)).clicked() {
                                if result.is_dir {
                                    self.rt.block_on(self.app_state.navigate_to(result.path.clone()));
                                } else {
                                    self.app_state.ui.properties_path = Some(result.path.clone());
                                    self.app_state.ui.show_properties = true;
                                    self.app_state.add_to_recent_files(result.path.clone());
                                }
                            }
                        }
                    });
                });
        }

        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("NF2's File Explorer")
            .with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "NF2's File Explorer",
        options,
        Box::new(|cc| Box::new(FileExplorerApp::new(cc))),
    )
}

