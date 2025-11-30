use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq)]
pub enum ClipboardOperation {
    Copy,
    Cut,
}

pub struct ClipboardState {
    pub files: Vec<PathBuf>,
    pub operation: Option<ClipboardOperation>,
}

impl ClipboardState {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            operation: None,
        }
    }

    pub fn has_content(&self) -> bool {
        !self.files.is_empty() && self.operation.is_some()
    }

    pub fn clear(&mut self) {
        self.files.clear();
        self.operation = None;
    }

    pub fn set(&mut self, files: Vec<PathBuf>, operation: ClipboardOperation) {
        self.files = files;
        self.operation = Some(operation);
    }
}


