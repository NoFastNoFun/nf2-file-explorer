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
    
    if response.secondary_clicked() {
        ctx.memory_mut(|mem| {
            mem.open_popup(popup_id);
        });
    }

    if ctx.memory(|mem| mem.is_popup_open(popup_id)) {
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
                        Err(e) => {
                            if !matches!(e, crate::fs::error::FileOperationError::Cancelled) {
                                app.ui.set_error(e.to_string());
                            }
                        }
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
                    app.ui.rename_request = Some(entry_index);
                    ctx.memory_mut(|mem| {
                        mem.close_popup();
                    });
                }

        if ui.button("Properties").clicked() {
            app.ui.properties_path = Some(entry_path.clone());
            app.ui.show_properties = true;
            ctx.memory_mut(|mem| {
                mem.close_popup();
            });
        }

        ui.separator();

        if let Some(_repo_info) = &app.git_repo_info {
            if let Some(git_status) = crate::fs::git::get_git_status(&entry_path) {
                ui.separator();
                ui.label("Git:");
                
                if git_status == crate::fs::git::GitFileStatus::Modified || 
                   git_status == crate::fs::git::GitFileStatus::Untracked {
                    if ui.button("Stage File").clicked() {
                        if let Some(repo_path) = crate::fs::git::find_git_repo(&entry_path) {
                            if let Ok(repo) = git2::Repository::open(&repo_path) {
                                if let Ok(mut index) = repo.index() {
                                    if let Ok(relative_path) = entry_path.strip_prefix(&repo_path) {
                                        let _ = index.add_path(relative_path);
                                        let _ = index.write();
                                    }
                                }
                            }
                        }
                        ctx.memory_mut(|mem| {
                            mem.close_popup();
                        });
                    }
                }
                
                if git_status == crate::fs::git::GitFileStatus::Staged {
                    if ui.button("Unstage File").clicked() {
                        if let Some(repo_path) = crate::fs::git::find_git_repo(&entry_path) {
                            if let Ok(repo) = git2::Repository::open(&repo_path) {
                                if let Ok(mut index) = repo.index() {
                                    if let Ok(relative_path) = entry_path.strip_prefix(&repo_path) {
                                        let _ = index.remove_path(relative_path);
                                        let _ = index.write();
                                    }
                                }
                            }
                        }
                        ctx.memory_mut(|mem| {
                            mem.close_popup();
                        });
                    }
                }
                
                if !entry_path.is_dir() {
                    if ui.button("Git Diff").clicked() {
                        app.ui.preview_path = Some(entry_path.clone());
                        app.ui.show_preview = true;
                        ctx.memory_mut(|mem| {
                            mem.close_popup();
                        });
                    }
                    
                    if ui.button("Git Blame").clicked() {
                        app.ui.preview_path = Some(entry_path.clone());
                        app.ui.show_preview = true;
                        ctx.memory_mut(|mem| {
                            mem.close_popup();
                        });
                    }
                }
            }
        }

        ui.separator();

        if ui.button("Open Terminal Here").clicked() {
            let target_path = if entry_path.is_dir() {
                entry_path.clone()
            } else {
                entry_path.parent().unwrap_or(&entry_path).to_path_buf()
            };
            if let Err(e) = crate::fs::terminal::open_terminal(&target_path) {
                app.ui.set_error(e);
            }
            ctx.memory_mut(|mem| {
                mem.close_popup();
            });
        }

        if ui.button("Run in Terminal...").clicked() {
            app.ui.show_command_dialog = true;
            app.ui.command_dialog_path = Some(entry_path.clone());
            ctx.memory_mut(|mem| {
                mem.close_popup();
            });
        }

        if !entry_path.is_dir() {
            if ui.button("Calculate Hash").clicked() {
                app.ui.show_hash_dialog = true;
                app.ui.hash_dialog_path = Some(entry_path.clone());
                ctx.memory_mut(|mem| {
                    mem.close_popup();
                });
            }
        }

        ui.separator();

        if ui.button("Create Symbolic Link...").clicked() {
            app.ui.show_create_link_dialog = true;
            app.ui.link_source_path = Some(entry_path.clone());
            app.ui.link_is_symlink = true;
            ctx.memory_mut(|mem| {
                mem.close_popup();
            });
        }

        if !entry_path.is_dir() {
            if ui.button("Create Hard Link...").clicked() {
                app.ui.show_create_link_dialog = true;
                app.ui.link_source_path = Some(entry_path.clone());
                app.ui.link_is_symlink = false;
                ctx.memory_mut(|mem| {
                    mem.close_popup();
                });
            }
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

