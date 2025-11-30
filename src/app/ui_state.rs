use std::path::PathBuf;
use crate::ui::preview::PreviewMode;

pub struct UIState {
    pub show_properties: bool,
    pub properties_path: Option<PathBuf>,
    pub show_preview: bool,
    pub preview_path: Option<PathBuf>,
    pub preview_mode: PreviewMode,
    pub show_settings: bool,
    pub show_home: bool,
    pub show_command_dialog: bool,
    pub command_dialog_path: Option<PathBuf>,
    pub command_input: String,
    pub show_hash_dialog: bool,
    pub hash_dialog_path: Option<PathBuf>,
    pub computing_hash: bool,
    pub hash_result: Option<String>,
    pub hash_receiver: Option<tokio::sync::mpsc::UnboundedReceiver<Result<String, String>>>,
    pub show_create_link_dialog: bool,
    pub link_source_path: Option<PathBuf>,
    pub link_is_symlink: bool,
    pub link_target_name: String,
    pub rename_request: Option<usize>,
    pub filter_text: String,
    pub error_message: Option<String>,
    pub error_dismiss_timer: Option<std::time::Instant>,
}

impl UIState {
    pub fn new() -> Self {
        Self {
            show_properties: false,
            properties_path: None,
            show_preview: false,
            preview_path: None,
            preview_mode: PreviewMode::Raw,
            show_settings: false,
            show_home: true,
            show_command_dialog: false,
            command_dialog_path: None,
            command_input: String::new(),
            show_hash_dialog: false,
            hash_dialog_path: None,
            computing_hash: false,
            hash_result: None,
            hash_receiver: None,
            show_create_link_dialog: false,
            link_source_path: None,
            link_is_symlink: true,
            link_target_name: String::new(),
            rename_request: None,
            filter_text: String::new(),
            error_message: None,
            error_dismiss_timer: None,
        }
    }

    pub fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.error_dismiss_timer = Some(std::time::Instant::now());
    }

    pub fn clear_error(&mut self) {
        self.error_message = None;
        self.error_dismiss_timer = None;
    }

    pub fn should_auto_dismiss_error(&self) -> bool {
        if let Some(timer) = self.error_dismiss_timer {
            timer.elapsed().as_secs() >= 5
        } else {
            false
        }
    }
}

