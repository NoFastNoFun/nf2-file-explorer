use std::path::PathBuf;
use std::collections::VecDeque;
use crate::fs::directory::{DirectoryCache, FileEntry, list_directory};
use crate::fs::search::SearchOptions;
use crate::fs::watcher::FileSystemWatcher;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq)]
pub enum ClipboardOperation {
    Copy,
    Cut,
}

pub struct AppState {
    pub current_path: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected_indices: Vec<usize>,
    pub history: VecDeque<PathBuf>,
    pub nav_history_index: usize,
    pub search_options: SearchOptions,
    pub search_results: Vec<FileEntry>,
    pub is_searching: bool,
    pub show_properties: bool,
    pub properties_path: Option<PathBuf>,
    pub show_preview: bool,
    pub preview_path: Option<PathBuf>,
    pub cache: DirectoryCache,
    pub loading: bool,
    pub error_message: Option<String>,
    pub path_input: String,
    pub clipboard_files: Vec<PathBuf>,
    pub clipboard_operation: Option<ClipboardOperation>,
    pub rename_request: Option<usize>,
    pub delete_dialog: crate::ui::dialogs::DeleteDialog,
    pub overwrite_dialog: crate::ui::dialogs::OverwriteDialog,
    pub pending_delete: bool,
    pub pending_paste: bool,
    pub file_watcher: FileSystemWatcher,
    pub search_debounce_timer: Option<std::time::Instant>,
    pub settings: crate::ui::settings::Settings,
    pub show_settings: bool,
    pub show_home: bool,
    pub operation_history: VecDeque<OperationRecord>,
    pub undo_history_index: isize,
}

#[derive(Clone)]
pub enum OperationType {
    Copy { src: PathBuf, dst: PathBuf },
    Move { src: PathBuf, dst: PathBuf },
    Delete { path: PathBuf, was_dir: bool },
    Rename { old_path: PathBuf, new_path: PathBuf },
}

#[derive(Clone)]
pub struct OperationRecord {
    pub op_type: OperationType,
    pub timestamp: std::time::SystemTime,
}

impl AppState {
    pub fn new() -> Self {
        let current_path = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("C:\\"));

        Self {
            current_path: current_path.clone(),
            entries: Vec::new(),
            selected_indices: Vec::new(),
            history: {
                let mut h = VecDeque::new();
                h.push_back(current_path.clone());
                h
            },
            nav_history_index: 0,
            search_options: SearchOptions::default(),
            search_results: Vec::new(),
            is_searching: false,
            show_properties: false,
            properties_path: None,
            show_preview: false,
            preview_path: None,
            cache: DirectoryCache::new(Duration::from_secs(30)),
            loading: false,
            error_message: None,
            path_input: String::new(),
            clipboard_files: Vec::new(),
            clipboard_operation: None,
            rename_request: None,
            delete_dialog: crate::ui::dialogs::DeleteDialog::new(),
            overwrite_dialog: crate::ui::dialogs::OverwriteDialog::new(),
            pending_delete: false,
            pending_paste: false,
            file_watcher: FileSystemWatcher::new(),
            search_debounce_timer: None,
            settings: crate::ui::settings::Settings::load(),
            show_settings: false,
            show_home: true,
            operation_history: VecDeque::new(),
            undo_history_index: -1,
        }
    }

    pub async fn navigate_to(&mut self, path: PathBuf) {
        self.show_home = false;
        if !path.exists() || !path.is_dir() {
            self.error_message = Some(format!("Path does not exist or is not a directory: {}", path.display()));
            return;
        }

        self.current_path = path.clone();
        self.path_input = self.current_path.to_string_lossy().to_string();
        self.selected_indices.clear();
        self.loading = true;
        self.error_message = None;

        // Update file watcher
        self.file_watcher.unwatch();
        if let Err(e) = self.file_watcher.watch(self.current_path.clone()) {
            self.error_message = Some(format!("Failed to watch directory: {}", e));
        }

        if let Some(cached) = self.cache.get(&path).await {
            self.entries = cached;
            self.loading = false;
        } else {
            match list_directory(&path).await {
                Ok(entries) => {
                    self.entries = entries.clone();
                    self.cache.set(path, entries).await;
                    self.loading = false;
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to list directory: {}", e));
                    self.entries.clear();
                    self.loading = false;
                }
            }
        }

        if let Some(last) = self.history.back() {
            if last != &self.current_path {
                self.history.push_back(self.current_path.clone());
                self.nav_history_index = self.history.len() - 1;
            }
        }

        // Add to recent folders
        let path_str = self.current_path.to_string_lossy().to_string();
        if !self.settings.recent_folders.contains(&path_str) {
            self.settings.recent_folders.insert(0, path_str);
            if self.settings.recent_folders.len() > 20 {
                self.settings.recent_folders.truncate(20);
            }
            let _ = self.settings.save();
        }
    }

    pub fn add_to_recent_files(&mut self, path: PathBuf) {
        let path_str = path.to_string_lossy().to_string();
        if !self.settings.recent_files.contains(&path_str) {
            self.settings.recent_files.insert(0, path_str);
            if self.settings.recent_files.len() > 20 {
                self.settings.recent_files.truncate(20);
            }
            let _ = self.settings.save();
        }
    }

    pub fn toggle_favorite(&mut self, path: PathBuf) {
        let path_str = path.to_string_lossy().to_string();
        if let Some(pos) = self.settings.favorites.iter().position(|p| p == &path_str) {
            self.settings.favorites.remove(pos);
        } else {
            self.settings.favorites.push(path_str);
        }
        let _ = self.settings.save();
    }

    pub fn is_favorite(&self, path: &PathBuf) -> bool {
        let path_str = path.to_string_lossy().to_string();
        self.settings.favorites.contains(&path_str)
    }

    pub fn check_file_system_events(&mut self) -> bool {
        if let Some(_changed_path) = self.file_watcher.check_events() {
            // Return true to indicate refresh is needed
            true
        } else {
            false
        }
    }

    pub fn can_go_back(&self) -> bool {
        self.nav_history_index > 0
    }

    pub fn can_go_forward(&self) -> bool {
        self.nav_history_index < self.history.len() - 1
    }

    pub async fn go_back(&mut self) {
        if self.can_go_back() {
            self.nav_history_index -= 1;
            if let Some(path) = self.history.get(self.nav_history_index) {
                self.navigate_to(path.clone()).await;
            }
        }
    }

    pub async fn go_forward(&mut self) {
        if self.can_go_forward() {
            self.nav_history_index += 1;
            if let Some(path) = self.history.get(self.nav_history_index) {
                self.navigate_to(path.clone()).await;
            }
        }
    }

    pub async fn refresh(&mut self) {
        self.cache.invalidate(&self.current_path).await;
        let path = self.current_path.clone();
        self.navigate_to(path).await;
    }

    pub async fn perform_search(&mut self) {
        if self.search_options.query.is_empty() {
            self.is_searching = false;
            self.search_results.clear();
            return;
        }

        self.is_searching = true;
        self.loading = true;

        let search_path = if self.search_options.global {
            PathBuf::from("C:\\")
        } else {
            self.current_path.clone()
        };

        match crate::fs::search::search_directory(&search_path, &self.search_options).await {
            Ok(results) => {
                self.search_results = results;
                self.loading = false;
            }
            Err(e) => {
                self.error_message = Some(format!("Search failed: {}", e));
                self.search_results.clear();
                self.loading = false;
            }
        }
    }

    pub fn get_selected_entries(&self) -> Vec<&FileEntry> {
        self.selected_indices
            .iter()
            .filter_map(|&idx| self.entries.get(idx))
            .collect()
    }

    pub fn clear_selection(&mut self) {
        self.selected_indices.clear();
    }

    pub fn select_all(&mut self) {
        self.selected_indices = (0..self.entries.len()).collect();
    }

    pub fn copy_selected(&mut self) {
        self.clipboard_files = self.get_selected_entries()
            .iter()
            .map(|e| e.path.clone())
            .collect();
        self.clipboard_operation = Some(ClipboardOperation::Copy);
        
        // Copy to system clipboard
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let paths: Vec<String> = self.clipboard_files.iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            let _ = clipboard.set_text(paths.join("\n"));
        }
    }

    pub fn cut_selected(&mut self) {
        self.clipboard_files = self.get_selected_entries()
            .iter()
            .map(|e| e.path.clone())
            .collect();
        self.clipboard_operation = Some(ClipboardOperation::Cut);
        
        // Copy to system clipboard
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let paths: Vec<String> = self.clipboard_files.iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            let _ = clipboard.set_text(paths.join("\n"));
        }
    }

    pub fn has_clipboard_content(&self) -> bool {
        !self.clipboard_files.is_empty() && self.clipboard_operation.is_some()
    }

    pub fn request_delete(&mut self) {
        let selected = self.get_selected_entries();
        if selected.is_empty() {
            return;
        }

        let file_names: Vec<String> = selected.iter()
            .map(|e| e.name.clone())
            .collect();
        self.delete_dialog.show(selected.len(), file_names);
        self.pending_delete = true;
    }

    pub async fn execute_delete(&mut self) -> Result<(), String> {
        let selected = self.get_selected_entries();
        if selected.is_empty() {
            return Err("No files selected".to_string());
        }

        let mut records = Vec::new();
        for entry in &selected {
            if entry.is_dir {
                crate::fs::operations::delete_directory(&entry.path).await
                    .map_err(|e| format!("Failed to delete directory: {}", e))?;
            } else {
                crate::fs::operations::delete_file(&entry.path).await
                    .map_err(|e| format!("Failed to delete file: {}", e))?;
            }
            records.push(OperationRecord {
                op_type: OperationType::Delete {
                    path: entry.path.clone(),
                    was_dir: entry.is_dir,
                },
                timestamp: std::time::SystemTime::now(),
            });
        }

        // Add to history
        for record in records {
            self.add_to_history(record);
        }

        self.selected_indices.clear();
        self.refresh().await;
        Ok(())
    }

    fn add_to_history(&mut self, record: OperationRecord) {
        // Remove any operations after current index (when undoing and then doing new operation)
        while self.undo_history_index >= 0 && (self.undo_history_index as usize) < self.operation_history.len() {
            self.operation_history.pop_back();
        }
        self.operation_history.push_back(record);
        self.undo_history_index = self.operation_history.len() as isize - 1;
        
        // Limit history size
        if self.operation_history.len() > 50 {
            self.operation_history.pop_front();
            self.undo_history_index -= 1;
        }
    }

    pub async fn undo(&mut self) -> Result<(), String> {
        if self.undo_history_index < 0 {
            return Err("Nothing to undo".to_string());
        }

        let idx = self.undo_history_index as usize;
        if idx >= self.operation_history.len() {
            return Err("Invalid history index".to_string());
        }

        let record = self.operation_history[idx].clone();
        
        match &record.op_type {
            OperationType::Delete { path: _, was_dir: _ } => {
                // Undo delete by restoring (not implemented - would need to restore from trash)
                return Err("Undo delete not yet implemented".to_string());
            }
            OperationType::Move { src, dst } => {
                // Undo move by moving back
                if dst.is_dir() {
                    crate::fs::operations::move_directory(dst, src, None).await
                        .map_err(|e| format!("Failed to undo move: {}", e))?;
                } else {
                    crate::fs::operations::move_file(dst, src, None).await
                        .map_err(|e| format!("Failed to undo move: {}", e))?;
                }
            }
            OperationType::Copy { src: _, dst } => {
                // Undo copy by deleting destination
                if dst.is_dir() {
                    crate::fs::operations::delete_directory(dst).await
                        .map_err(|e| format!("Failed to undo copy: {}", e))?;
                } else {
                    crate::fs::operations::delete_file(dst).await
                        .map_err(|e| format!("Failed to undo copy: {}", e))?;
                }
            }
            OperationType::Rename { old_path, new_path } => {
                // Undo rename by renaming back
                tokio::fs::rename(new_path, old_path).await
                    .map_err(|e| format!("Failed to undo rename: {}", e))?;
            }
        }

        self.undo_history_index -= 1;
        self.refresh().await;
        Ok(())
    }

    pub fn can_undo(&self) -> bool {
        self.undo_history_index >= 0 && (self.undo_history_index as usize) < self.operation_history.len()
    }

    pub fn can_redo(&self) -> bool {
        (self.undo_history_index + 1) < self.operation_history.len() as isize
    }

    pub fn request_paste(&mut self) {
        if !self.has_clipboard_content() {
            return;
        }
        self.pending_paste = true;
    }

    pub async fn paste_clipboard(&mut self) -> Result<(), String> {
        if !self.has_clipboard_content() {
            return Err("No files in clipboard".to_string());
        }

        let operation = self.clipboard_operation.unwrap();
        let files_to_paste = self.clipboard_files.clone();
        let dest_dir = self.current_path.clone();

        for src_path in files_to_paste {
            if !src_path.exists() {
                continue;
            }

            let file_name = src_path.file_name()
                .ok_or_else(|| "Invalid file name".to_string())?;
            let dest_path = dest_dir.join(file_name);

            if dest_path.exists() {
                let file_name_str = file_name.to_string_lossy().to_string();
                self.overwrite_dialog.show(
                    file_name_str,
                    dest_path.clone(),
                    src_path.clone(),
                );
                return Err("File exists - showing dialog".to_string());
            }

            if src_path.is_dir() {
                match operation {
                    ClipboardOperation::Copy => {
                        crate::fs::operations::copy_directory(&src_path, &dest_path, None).await
                            .map_err(|e| format!("Failed to copy directory: {}", e))?;
                    }
                    ClipboardOperation::Cut => {
                        crate::fs::operations::move_directory(&src_path, &dest_path, None).await
                            .map_err(|e| format!("Failed to move directory: {}", e))?;
                    }
                }
            } else {
                match operation {
                    ClipboardOperation::Copy => {
                        crate::fs::operations::copy_file(&src_path, &dest_path, None).await
                            .map_err(|e| format!("Failed to copy file: {}", e))?;
                    }
                    ClipboardOperation::Cut => {
                        crate::fs::operations::move_file(&src_path, &dest_path, None).await
                            .map_err(|e| format!("Failed to move file: {}", e))?;
                    }
                }
            }
        }

        if operation == ClipboardOperation::Cut {
            self.clipboard_files.clear();
            self.clipboard_operation = None;
        }

        self.refresh().await;
        Ok(())
    }

    pub async fn execute_paste_with_overwrite(&mut self, overwrite: bool) -> Result<(), String> {
        if !overwrite {
            return Ok(());
        }

        if !self.has_clipboard_content() {
            return Err("No files in clipboard".to_string());
        }

        let operation = self.clipboard_operation.unwrap();
        let files_to_paste = self.clipboard_files.clone();
        let dest_dir = self.current_path.clone();

        for src_path in files_to_paste {
            if !src_path.exists() {
                continue;
            }

            let file_name = src_path.file_name()
                .ok_or_else(|| "Invalid file name".to_string())?;
            let dest_path = dest_dir.join(file_name);

            if dest_path.exists() && overwrite {
                if dest_path.is_dir() {
                    let _ = tokio::fs::remove_dir_all(&dest_path).await;
                } else {
                    let _ = tokio::fs::remove_file(&dest_path).await;
                }
            }

            if src_path.is_dir() {
                match operation {
                    ClipboardOperation::Copy => {
                        crate::fs::operations::copy_directory(&src_path, &dest_path, None).await
                            .map_err(|e| format!("Failed to copy directory: {}", e))?;
                    }
                    ClipboardOperation::Cut => {
                        crate::fs::operations::move_directory(&src_path, &dest_path, None).await
                            .map_err(|e| format!("Failed to move directory: {}", e))?;
                    }
                }
            } else {
                match operation {
                    ClipboardOperation::Copy => {
                        crate::fs::operations::copy_file(&src_path, &dest_path, None).await
                            .map_err(|e| format!("Failed to copy file: {}", e))?;
                    }
                    ClipboardOperation::Cut => {
                        crate::fs::operations::move_file(&src_path, &dest_path, None).await
                            .map_err(|e| format!("Failed to move file: {}", e))?;
                    }
                }
            }
        }

        if operation == ClipboardOperation::Cut {
            self.clipboard_files.clear();
            self.clipboard_operation = None;
        }

        self.refresh().await;
        Ok(())
    }
}

