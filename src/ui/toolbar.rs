use egui::{Ui, Button, TextEdit};
use std::path::PathBuf;
use crate::app::AppState;
use tokio::runtime::Runtime;
use crate::ui::breadcrumb;

pub fn render_toolbar(ui: &mut Ui, app: &mut AppState, rt: &Runtime) {
        ui.horizontal(|ui| {
        if ui.button("🏠").clicked() {
            app.ui.show_home = true;
        }

        if ui.add_enabled(app.can_go_back(), Button::new("◀")).clicked() {
            rt.block_on(app.go_back());
        }

        if ui.add_enabled(app.can_go_forward(), Button::new("▶")).clicked() {
            rt.block_on(app.go_forward());
        }

        if ui.button("↻").clicked() {
            rt.block_on(app.refresh());
        }

        ui.separator();

        let has_selection = !app.selected_indices.is_empty();
        if ui.add_enabled(has_selection, Button::new("Copy")).clicked() {
            app.copy_selected();
        }
        if ui.add_enabled(has_selection, Button::new("Cut")).clicked() {
            app.cut_selected();
        }
        if ui.add_enabled(app.has_clipboard_content(), Button::new("Paste")).clicked() {
            app.request_paste();
            match rt.block_on(app.paste_clipboard()) {
                Ok(_) => {}
                Err(e) => {
                    if !matches!(e, crate::fs::error::FileOperationError::Cancelled) {
                        app.ui.set_error(e.to_string());
                    }
                }
            }
        }
        if ui.add_enabled(has_selection, Button::new("Delete")).clicked() {
            app.request_delete();
        }

        ui.separator();

        if ui.button("Dual Pane").clicked() {
            app.use_multi_pane = !app.use_multi_pane;
            if app.use_multi_pane && app.multi_pane.is_none() {
                app.multi_pane = Some(crate::ui::multi_pane::MultiPaneView::new());
            }
        }

        ui.separator();

        if let Some(ref repo_info) = app.git_repo_info {
            ui.label(format!("Branch: {}", repo_info.branch));
            ui.separator();
        }

        breadcrumb::render_breadcrumb(ui, app, rt);

        ui.separator();

        let response = ui.add(
            TextEdit::singleline(&mut app.navigation.path_input)
                .desired_width(ui.available_width() * 0.4)
                .hint_text("Path")
        );

        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            let new_path = PathBuf::from(app.navigation.path_input.trim());
            rt.block_on(app.navigate_to(new_path));
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Settings").clicked() {
                app.ui.show_settings = true;
            }

            ui.menu_button("Recent", |ui| {
                if app.settings.recent_folders.is_empty() {
                    ui.label("No recent folders");
                } else {
                    let folders = app.settings.recent_folders.clone();
                    for folder in &folders {
                        if ui.button(folder).clicked() {
                            let path = PathBuf::from(folder);
                            rt.block_on(app.navigate_to(path));
                        }
                    }
                }
            });
            
            if ui.button("New Folder").clicked() {
                let mut folder_name = "New Folder".to_string();
                let mut counter = 1;
                let base_path = app.navigation.current_path.clone();
                
                loop {
                    let test_path = base_path.join(&folder_name);
                    if !test_path.exists() {
                        break;
                    }
                    folder_name = format!("New Folder ({})", counter);
                    counter += 1;
                }
                
                let new_folder_path = base_path.join(&folder_name);
                if let Err(e) = rt.block_on(tokio::fs::create_dir(&new_folder_path)) {
                    app.ui.set_error(format!("Failed to create folder: {}", e));
                } else {
                    rt.block_on(app.refresh());
                }
            }
        });
    });
}

