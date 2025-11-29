use egui::{Context, Response, Window};
use std::path::PathBuf;
use crate::app::AppState;
use tokio::runtime::Runtime;

pub fn show_file_context_menu(
    ctx: &Context,
    response: &Response,
    app: &mut AppState,
    rt: &Runtime,
    entry_path: PathBuf,
    entry_index: usize,
) {
    let popup_id = response.id.with("context_menu");
    let mut show_popup = ctx.memory(|mem| mem.is_popup_open(popup_id));
    
    if response.secondary_clicked() {
        show_popup = true;
        ctx.memory_mut(|mem| {
            mem.open_popup(popup_id);
        });
    }

    if show_popup {
        let pos = response.rect.left_bottom();
        Window::new("")
            .id(popup_id)
            .collapsible(false)
            .resizable(false)
            .title_bar(false)
            .fixed_pos(pos)
            .show(ctx, |ui| {
                ui.set_min_width(150.0);

                if ui.button("Copy").clicked() {
                    app.selected_indices = vec![entry_index];
                    app.copy_selected();
                    ctx.memory_mut(|mem| {
                        mem.close_popup();
                    });
                }

                if ui.button("Cut").clicked() {
                    app.selected_indices = vec![entry_index];
                    app.cut_selected();
                    ctx.memory_mut(|mem| {
                        mem.close_popup();
                    });
                }

                if ui.button("Paste").clicked() {
                    match rt.block_on(app.paste_clipboard()) {
                        Ok(_) => {}
                        Err(e) => app.error_message = Some(e),
                    }
                    ctx.memory_mut(|mem| {
                        mem.close_popup();
                    });
                }

                ui.separator();

                if ui.button("Delete").clicked() {
                    app.selected_indices = vec![entry_index];
                    app.request_delete();
                    ctx.memory_mut(|mem| {
                        mem.close_popup();
                    });
                }

                if ui.button("Rename").clicked() {
                    app.rename_request = Some(entry_index);
                    ctx.memory_mut(|mem| {
                        mem.close_popup();
                    });
                }

        if ui.button("Properties").clicked() {
            app.properties_path = Some(entry_path.clone());
            app.show_properties = true;
            ctx.memory_mut(|mem| {
                mem.close_popup();
            });
        }

        ui.separator();

        if entry_path.is_dir() {
            let is_favorite = app.is_favorite(&entry_path);
            let favorite_text = if is_favorite { "Remove from Favorites" } else { "Add to Favorites" };
            if ui.button(favorite_text).clicked() {
                app.toggle_favorite(entry_path);
                ctx.memory_mut(|mem| {
                    mem.close_popup();
                });
            }
        }
            });
    }
}

