use egui::{Context, Window};
use crate::app::AppState;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

pub fn render_hash_dialog(ctx: &Context, app: &mut AppState, rt: &Runtime) {
    if let Some(ref mut rx) = app.ui.hash_receiver {
        if let Ok(result) = rx.try_recv() {
            match result {
                Ok(hash) => {
                    app.ui.hash_result = Some(hash);
                    app.ui.computing_hash = false;
                }
                Err(e) => {
                    app.ui.set_error(e);
                    app.ui.computing_hash = false;
                }
            }
            app.ui.hash_receiver = None;
        }
    }

    if !app.ui.show_hash_dialog {
        return;
    }

    let mut show = true;
    Window::new("File Hash")
        .collapsible(false)
        .resizable(false)
        .open(&mut show)
        .show(ctx, |ui| {
            if let Some(ref path) = app.ui.hash_dialog_path {
                ui.label(format!("File: {}", path.display()));
                ui.separator();
                
                ui.horizontal(|ui| {
                    if ui.button("MD5").clicked() {
                        app.ui.computing_hash = true;
                        app.ui.hash_result = None;
                        let path = path.clone();
                        let rt_handle = rt.handle().clone();
                        let ctx_clone = ctx.clone();
                        let (tx, rx) = mpsc::unbounded_channel();
                        app.ui.hash_receiver = Some(rx);
                        rt_handle.spawn(async move {
                            let result = crate::fs::hash::compute_hash(&path, crate::fs::hash::HashType::MD5)
                                .await
                                .map_err(|e| e.to_string());
                            let _ = tx.send(result);
                            ctx_clone.request_repaint();
                        });
                    }
                    if ui.button("SHA-1").clicked() {
                        app.ui.computing_hash = true;
                        app.ui.hash_result = None;
                        let path = path.clone();
                        let rt_handle = rt.handle().clone();
                        let ctx_clone = ctx.clone();
                        let (tx, rx) = mpsc::unbounded_channel();
                        app.ui.hash_receiver = Some(rx);
                        rt_handle.spawn(async move {
                            let result = crate::fs::hash::compute_hash(&path, crate::fs::hash::HashType::SHA1)
                                .await
                                .map_err(|e| e.to_string());
                            let _ = tx.send(result);
                            ctx_clone.request_repaint();
                        });
                    }
                    if ui.button("SHA-256").clicked() {
                        app.ui.computing_hash = true;
                        app.ui.hash_result = None;
                        let path = path.clone();
                        let rt_handle = rt.handle().clone();
                        let ctx_clone = ctx.clone();
                        let (tx, rx) = mpsc::unbounded_channel();
                        app.ui.hash_receiver = Some(rx);
                        rt_handle.spawn(async move {
                            let result = crate::fs::hash::compute_hash(&path, crate::fs::hash::HashType::SHA256)
                                .await
                                .map_err(|e| e.to_string());
                            let _ = tx.send(result);
                            ctx_clone.request_repaint();
                        });
                    }
                });
                
                if app.ui.computing_hash {
                    ui.spinner();
                    ui.label("Computing hash...");
                }
                
                if let Some(ref hash) = app.ui.hash_result {
                    ui.separator();
                    ui.label(egui::RichText::new(hash.clone()).monospace());
                    if ui.button("Copy").clicked() {
                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                            let _ = clipboard.set_text(hash.clone());
                        }
                    }
                }
            }
            
            ui.separator();
            if ui.button("Close").clicked() {
                app.ui.show_hash_dialog = false;
            }
        });
    
    app.ui.show_hash_dialog = show;
}

