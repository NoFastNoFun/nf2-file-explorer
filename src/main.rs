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
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut app_state = AppState::new();
        
        let current_path = app_state.current_path.clone();
        app_state.path_input = current_path.to_string_lossy().to_string();
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
                        Err(e) => self.app_state.error_message = Some(e),
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
                    self.app_state.rename_request = Some(first_selected);
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
            
            // Escape - Clear selection
            if i.key_pressed(egui::Key::Escape) {
                self.app_state.clear_selection();
            }
            
            // Ctrl+Z - Undo
            if modifiers.ctrl && i.key_pressed(egui::Key::Z) && !modifiers.shift {
                if self.app_state.can_undo() {
                    match self.rt.block_on(self.app_state.undo()) {
                        Ok(_) => {}
                        Err(e) => self.app_state.error_message = Some(e),
                    }
                }
            }
            
            // Ctrl+Y or Ctrl+Shift+Z - Redo
            if (modifiers.ctrl && i.key_pressed(egui::Key::Y)) ||
               (modifiers.ctrl && modifiers.shift && i.key_pressed(egui::Key::Z)) {
                // Redo not yet implemented
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
            if self.app_state.show_home {
                self.home_page.set_show_home(true);
                self.home_page.render(ui, &mut self.app_state, &self.rt, ctx);
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
                    } else if let Some(ref error) = self.app_state.error_message {
                        ui.colored_label(egui::Color32::RED, error);
                    } else {
                        self.file_list_view.render(ui, &mut self.app_state, &self.rt, ctx);
                    }
                });
            }
        });

        properties::render_properties(ctx, &mut self.app_state);
        preview::render_preview(ctx, &mut self.app_state, &self.rt);
        
        if self.app_state.show_settings {
            settings::render_settings(ctx, &mut self.app_state.settings);
        }

        // Handle dialogs
        match self.app_state.delete_dialog.render(ctx) {
            crate::ui::dialogs::DialogAction::Confirm => {
                if self.app_state.pending_delete {
                    match self.rt.block_on(self.app_state.execute_delete()) {
                        Ok(_) => {}
                        Err(e) => self.app_state.error_message = Some(e),
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
                        Err(e) => self.app_state.error_message = Some(e),
                    }
                    self.app_state.pending_paste = false;
                }
            }
            crate::ui::dialogs::DialogAction::Cancel => {
                self.app_state.pending_paste = false;
            }
            _ => {}
        }

        if self.app_state.is_searching && !self.app_state.search_options.query.is_empty() {
            let search_results = self.app_state.search_results.clone();
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
                                    self.app_state.properties_path = Some(result.path.clone());
                                    self.app_state.show_properties = true;
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

