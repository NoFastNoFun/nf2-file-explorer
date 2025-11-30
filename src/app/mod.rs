pub mod navigation;
pub mod clipboard;
pub mod ui_state;
pub mod search_state;
pub mod operation_history;
pub mod background_tasks;

pub use background_tasks::PendingOperations;

pub use navigation::NavigationState;
pub use clipboard::{ClipboardState, ClipboardOperation};
pub use ui_state::UIState;
pub use search_state::SearchState;
pub use operation_history::{OperationHistory, OperationRecord, OperationType};

use std::path::PathBuf;
use std::sync::Arc;
use crate::fs::directory::{DirectoryCache, FileEntry, list_directory};
use crate::fs::watcher::FileSystemWatcher;
use std::time::Duration;
use crate::ui::multi_pane::MultiPaneView;
use crate::fs::gitignore::GitIgnoreFilter;
use crate::fs::git::GitStatusManager;
use crate::ui::dialogs::{DeleteDialog, OverwriteDialog};

pub struct AppState {
    pub navigation: NavigationState,
    pub entries: Vec<FileEntry>,
    pub selected_indices: Vec<usize>,
    pub clipboard: ClipboardState,
    pub ui: UIState,
    pub search: SearchState,
    pub operation_history: OperationHistory,
    pub cache: DirectoryCache,
    pub loading: bool,
    pub file_watcher: FileSystemWatcher,
    pub settings: crate::ui::settings::Settings,
    pub multi_pane: Option<MultiPaneView>,
    pub use_multi_pane: bool,
    pub gitignore_filter: GitIgnoreFilter,
    pub show_ignored_files: bool,
    pub git_repo_info: Option<crate::fs::git::GitRepoInfo>,
    pub git_manager: Arc<GitStatusManager>,
    pub delete_dialog: DeleteDialog,
    pub overwrite_dialog: OverwriteDialog,
    pub pending_delete: bool,
    pub pending_paste: bool,
    pub pending_operations: PendingOperations,
    pub progress: crate::ui::progress_dialog::ProgressState,
}

impl AppState {
    pub fn new() -> Self {
        let current_path = std::env::current_dir()
            .unwrap_or_else(|_| {
                #[cfg(windows)]
                {
                    PathBuf::from("C:\\")
                }
                #[cfg(not(windows))]
                {
                    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
                }
            });

        Self {
            navigation: NavigationState::new(current_path.clone()),
            entries: Vec::new(),
            selected_indices: Vec::new(),
            clipboard: ClipboardState::new(),
            ui: UIState::new(),
            search: SearchState::new(),
            operation_history: OperationHistory::new(),
            cache: DirectoryCache::new(Duration::from_secs(30)),
            loading: false,
            file_watcher: FileSystemWatcher::new(),
            settings: crate::ui::settings::Settings::load(),
            multi_pane: Some(MultiPaneView::new()),
            use_multi_pane: false,
            gitignore_filter: GitIgnoreFilter::new(),
            show_ignored_files: crate::ui::settings::Settings::load().show_ignored_files,
            git_repo_info: None,
            git_manager: Arc::new(GitStatusManager::new()),
            delete_dialog: DeleteDialog::new(),
            overwrite_dialog: OverwriteDialog::new(),
            pending_delete: false,
            pending_paste: false,
            pending_operations: PendingOperations::new(),
            progress: crate::ui::progress_dialog::ProgressState::new(),
        }
    }

    pub fn current_path(&self) -> &PathBuf {
        &self.navigation.current_path
    }

    pub fn current_path_mut(&mut self) -> &mut PathBuf {
        &mut self.navigation.current_path
    }

    pub async fn navigate_to(&mut self, path: PathBuf) {
        self.ui.show_home = false;
        if !path.exists() || !path.is_dir() {
            self.ui.set_error(format!("Path does not exist or is not a directory: {}", path.display()));
            return;
        }

        self.navigation.update_path(path.clone());
        self.selected_indices.clear();
        self.loading = true;
        self.ui.clear_error();

        self.file_watcher.unwatch();
        if let Err(e) = self.file_watcher.watch(self.navigation.current_path.clone()) {
            self.ui.set_error(format!("Failed to watch directory: {}", e));
        }

        if self.settings.show_git_status {
            self.git_repo_info = self.git_manager.get_repo_info(&path);
        } else {
            self.git_repo_info = None;
        }
        
        if let Some(cached) = self.cache.get(&path).await {
            self.entries = cached.as_ref().clone();
            self.loading = false;
        } else {
            let show_git_status = self.settings.show_git_status;
            match list_directory(&path, Some(&*self.git_manager), show_git_status).await {
                Ok(mut entries) => {
                    if !self.show_ignored_files {
                        if !self.gitignore_filter.is_cached_for(&path) {
                            self.gitignore_filter.update_for_directory(&path);
                        }
                        entries.retain(|e| !self.gitignore_filter.should_hide(&e.path, &path));
                    }
                    let entries_clone = entries.clone();
                    self.entries = entries;
                    self.cache.set(path, entries_clone).await;
                    self.loading = false;
                }
                Err(e) => {
                    self.ui.set_error(format!("Failed to list directory: {}", e));
                    self.entries.clear();
                    self.loading = false;
                }
            }
        }

        let path_str = self.navigation.current_path.to_string_lossy().to_string();
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
            true
        } else {
            false
        }
    }

    pub fn can_go_back(&self) -> bool {
        self.navigation.can_go_back()
    }

    pub fn can_go_forward(&self) -> bool {
        self.navigation.can_go_forward()
    }

    pub async fn go_back(&mut self) {
        if let Some(path) = self.navigation.go_back_index() {
            self.navigate_to(path).await;
        }
    }

    pub async fn go_forward(&mut self) {
        if let Some(path) = self.navigation.go_forward_index() {
            self.navigate_to(path).await;
        }
    }

    pub async fn refresh(&mut self) {
        self.cache.invalidate(&self.navigation.current_path).await;
        let path = self.navigation.current_path.clone();
        self.navigate_to(path).await;
    }

    pub async fn perform_search(&mut self) {
        if self.search.options.query.is_empty() {
            self.search.is_searching = false;
            self.search.results.clear();
            return;
        }

        self.search.is_searching = true;
        self.loading = true;

        let search_path = if self.search.options.global {
            PathBuf::from("C:\\")
        } else {
            self.navigation.current_path.clone()
        };

        match crate::fs::search::search_directory(&search_path, &self.search.options).await {
            Ok(results) => {
                self.search.results = results;
                self.loading = false;
            }
            Err(e) => {
                self.ui.set_error(format!("Search failed: {}", e));
                self.search.results.clear();
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
        let files: Vec<PathBuf> = self.get_selected_entries()
            .iter()
            .map(|e| e.path.clone())
            .collect();
        self.clipboard.set(files.clone(), ClipboardOperation::Copy);
        
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let paths: Vec<String> = files.iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            let _ = clipboard.set_text(paths.join("\n"));
        }
    }

    pub fn cut_selected(&mut self) {
        let files: Vec<PathBuf> = self.get_selected_entries()
            .iter()
            .map(|e| e.path.clone())
            .collect();
        self.clipboard.set(files.clone(), ClipboardOperation::Cut);
        
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let paths: Vec<String> = files.iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            let _ = clipboard.set_text(paths.join("\n"));
        }
    }

    pub fn has_clipboard_content(&self) -> bool {
        self.clipboard.has_content()
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

    pub async fn execute_delete(&mut self) -> Result<(), crate::fs::error::FileOperationError> {
        let selected = self.get_selected_entries();
        if selected.is_empty() {
            return Err(crate::fs::error::FileOperationError::InvalidInput("No files selected".to_string()));
        }

        let mut records = Vec::new();
        for entry in &selected {
            let was_dir = entry.is_dir;
            let path = entry.path.clone();
            
            let trash_path = if was_dir {
                crate::fs::operations::delete_directory(&path).await?
            } else {
                crate::fs::operations::delete_file(&path).await?
            };
            
            records.push(OperationRecord {
                op_type: OperationType::Delete {
                    path: path.clone(),
                    was_dir,
                    trash_path: Some(trash_path),
                },
                timestamp: std::time::SystemTime::now(),
            });
        }

        for record in records {
            self.operation_history.add(record);
        }

        self.selected_indices.clear();
        self.refresh().await;
        Ok(())
    }

    pub async fn undo(&mut self) -> Result<(), crate::fs::error::FileOperationError> {
        let record = match self.operation_history.undo() {
            Some(r) => r,
            None => return Err(crate::fs::error::FileOperationError::InvalidInput("Nothing to undo".to_string())),
        };
        
        match &record.op_type {
            OperationType::Delete { path, was_dir: _, trash_path: _ } => {
                // The trash crate doesn't provide a restore function, and we don't have the actual trash location
                // For now, we'll return an error indicating undo is not fully supported for deletes
                // In a full implementation, we'd need to use platform-specific APIs to restore from trash
                // or store files before deletion to enable proper undo
                return Err(crate::fs::error::FileOperationError::Custom(
                    format!("Cannot undo delete - file '{}' was moved to trash but cannot be automatically restored. Please restore it manually from the trash.", path.display())
                ));
            }
            OperationType::Move { src, dst } => {
                if dst.is_dir() {
                    crate::fs::operations::move_directory(dst, src, None).await?;
                } else {
                    crate::fs::operations::move_file(dst, src, None).await?;
                }
            }
            OperationType::Copy { src: _, dst } => {
                if dst.is_dir() {
                    let _ = crate::fs::operations::delete_directory(dst).await?;
                } else {
                    let _ = crate::fs::operations::delete_file(dst).await?;
                }
            }
            OperationType::Rename { old_path, new_path } => {
                tokio::fs::rename(new_path, old_path).await
                    .map_err(|e| crate::fs::error::FileOperationError::Io(e))?;
            }
        }

        self.refresh().await;
        Ok(())
    }

    pub fn can_undo(&self) -> bool {
        self.operation_history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.operation_history.can_redo()
    }

    pub async fn redo(&mut self) -> Result<(), crate::fs::error::FileOperationError> {
        let record = match self.operation_history.redo() {
            Some(r) => r,
            None => return Err(crate::fs::error::FileOperationError::InvalidInput("Nothing to redo".to_string())),
        };

        match &record.op_type {
            OperationType::Delete { path, was_dir, trash_path: _ } => {
                if *was_dir {
                    let _ = crate::fs::operations::delete_directory(path).await?;
                } else {
                    let _ = crate::fs::operations::delete_file(path).await?;
                }
            }
            OperationType::Move { src, dst } => {
                if src.is_dir() {
                    crate::fs::operations::move_directory(src, dst, None).await?;
                } else {
                    crate::fs::operations::move_file(src, dst, None).await?;
                }
            }
            OperationType::Copy { src, dst } => {
                if src.is_dir() {
                    crate::fs::operations::copy_directory(src, dst, None).await?;
                } else {
                    crate::fs::operations::copy_file(src, dst, None).await?;
                }
            }
            OperationType::Rename { old_path, new_path } => {
                tokio::fs::rename(old_path, new_path).await
                    .map_err(|e| crate::fs::error::FileOperationError::Io(e))?;
            }
        }

        self.refresh().await;
        Ok(())
    }

    pub fn request_paste(&mut self) {
        if !self.has_clipboard_content() {
            return;
        }
        self.pending_paste = true;
    }

    async fn paste_files_internal(&mut self, overwrite: bool) -> Result<(), crate::fs::error::FileOperationError> {
        if !self.has_clipboard_content() {
            return Err(crate::fs::error::FileOperationError::InvalidInput("No files in clipboard".to_string()));
        }

        let operation = self.clipboard.operation.ok_or_else(|| {
            crate::fs::error::FileOperationError::InvalidInput("No clipboard operation".to_string())
        })?;
        let files_to_paste = self.clipboard.files.clone();
        let dest_dir = self.navigation.current_path.clone();

        let total_size: u64 = files_to_paste.iter()
            .filter_map(|p| {
                if p.is_file() {
                    std::fs::metadata(p).ok().map(|m| m.len())
                } else {
                    None
                }
            })
            .sum();

        if total_size > 1024 * 1024 {
            let op_name = match operation {
                ClipboardOperation::Copy => "Copying",
                ClipboardOperation::Cut => "Moving",
            };
            self.progress.start(format!("{} files", op_name), total_size);
        }

        use std::sync::atomic::{AtomicU64, Ordering};
        let current_bytes = Arc::new(AtomicU64::new(0));

        for src_path in files_to_paste {
            if self.progress.cancelled {
                self.progress.finish();
                return Err(crate::fs::error::FileOperationError::Cancelled);
            }

            if !src_path.exists() {
                continue;
            }

            let file_name = src_path.file_name()
                .ok_or_else(|| crate::fs::error::FileOperationError::InvalidInput("Invalid file name".to_string()))?;
            let dest_path = dest_dir.join(file_name);

            if dest_path.exists() && !overwrite {
                let file_name_str = file_name.to_string_lossy().to_string();
                self.overwrite_dialog.show(
                    file_name_str,
                    dest_path.clone(),
                    src_path.clone(),
                );
                self.progress.finish();
                return Err(crate::fs::error::FileOperationError::Cancelled);
            }

            if dest_path.exists() && overwrite {
                if dest_path.is_dir() {
                    let _ = tokio::fs::remove_dir_all(&dest_path).await;
                } else {
                    let _ = tokio::fs::remove_file(&dest_path).await;
                }
            }

            if src_path.is_dir() {
                let progress_cb: Option<crate::fs::operations::ProgressCallback> = if total_size > 1024 * 1024 {
                    let current_bytes_clone = Arc::clone(&current_bytes);
                    Some(Box::new(move |current, _total| {
                        current_bytes_clone.store(current, Ordering::Relaxed);
                    }))
                } else {
                    None
                };
                
                match operation {
                    ClipboardOperation::Copy => {
                        crate::fs::operations::copy_directory(&src_path, &dest_path, progress_cb).await?;
                    }
                    ClipboardOperation::Cut => {
                        crate::fs::operations::move_directory(&src_path, &dest_path, progress_cb).await?;
                    }
                }
            } else {
                let progress_cb: Option<crate::fs::operations::ProgressCallback> = if total_size > 1024 * 1024 {
                    let current_bytes_clone = Arc::clone(&current_bytes);
                    Some(Box::new(move |current, _total| {
                        current_bytes_clone.store(current, Ordering::Relaxed);
                    }))
                } else {
                    None
                };
                
                match operation {
                    ClipboardOperation::Copy => {
                        crate::fs::operations::copy_file(&src_path, &dest_path, progress_cb).await?;
                    }
                    ClipboardOperation::Cut => {
                        crate::fs::operations::move_file(&src_path, &dest_path, progress_cb).await?;
                    }
                }
            }
            
            if total_size > 1024 * 1024 {
                self.progress.update(current_bytes.load(Ordering::Relaxed));
            }
        }

        self.progress.finish();

        if operation == ClipboardOperation::Cut {
            self.clipboard.clear();
        }

        self.refresh().await;
        Ok(())
    }

    pub async fn paste_clipboard(&mut self) -> Result<(), crate::fs::error::FileOperationError> {
        self.paste_files_internal(false).await
    }

    pub async fn execute_paste_with_overwrite(&mut self, overwrite: bool) -> Result<(), crate::fs::error::FileOperationError> {
        if !overwrite {
            return Ok(());
        }
        self.paste_files_internal(true).await
    }
}
