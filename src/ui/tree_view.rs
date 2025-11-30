use egui::{Ui, ScrollArea, CollapsingHeader};
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use crate::app::AppState;
use tokio::runtime::Runtime;

pub struct TreeView {
    expanded: HashSet<PathBuf>,
}

impl TreeView {
    pub fn new() -> Self {
        Self {
            expanded: HashSet::new(),
        }
    }

    pub fn render(&mut self, ui: &mut Ui, app: &mut AppState, rt: &Runtime) {
        ScrollArea::vertical()
            .show(ui, |ui| {
                let drives = get_windows_drives();
                for drive in drives {
                    self.render_tree_node(ui, app, &drive, 0, rt);
                }
            });
    }

    fn render_tree_node(&mut self, ui: &mut Ui, app: &mut AppState, path: &Path, depth: usize, rt: &Runtime) {
        if depth > 10 {
            return;
        }

        let path_buf = path.to_path_buf();
        let mut is_expanded = self.expanded.contains(&path_buf);
        let is_current = app.navigation.current_path.starts_with(&path_buf) && app.navigation.current_path != path_buf;
        
        if is_current && !is_expanded {
            is_expanded = true;
            self.expanded.insert(path_buf.clone());
        }

        let label = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let header_response = CollapsingHeader::new(format!("📁 {}", label))
            .open(Some(is_expanded))
            .show(ui, |ui| {
                if let Ok(entries) = std::fs::read_dir(path) {
                    for entry in entries.flatten() {
                        let entry_path = entry.path();
                        if entry_path.is_dir() {
                            self.render_tree_node(ui, app, &entry_path, depth + 1, rt);
                        }
                    }
                }
            });

        // Handle expand/collapse state
        let is_now_open = header_response.body_response.is_some();
        if is_now_open {
            self.expanded.insert(path_buf.clone());
        } else {
            self.expanded.remove(&path_buf);
        }

        // Navigate on double-click, not single click
        if header_response.header_response.double_clicked() {
            rt.block_on(app.navigate_to(path_buf.clone()));
        }
    }
}

fn get_windows_drives() -> Vec<PathBuf> {
    #[cfg(windows)]
    {
        let mut drives = Vec::new();
        for letter in b'A'..=b'Z' {
            let drive = format!("{}:\\", letter as char);
            let path = PathBuf::from(&drive);
            if path.exists() {
                drives.push(path);
            }
        }
        drives
    }
    #[cfg(not(windows))]
    {
        vec![PathBuf::from("/")]
    }
}

