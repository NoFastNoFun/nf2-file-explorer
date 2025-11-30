use std::path::PathBuf;

pub enum PendingOperation {
    Navigate { path: PathBuf },
    Refresh,
    Paste,
    Delete,
    Undo,
    Redo,
    ComputeHash { path: PathBuf, hash_type: crate::fs::hash::HashType },
}

pub struct PendingOperations {
    pub current: Option<PendingOperation>,
}

impl PendingOperations {
    pub fn new() -> Self {
        Self { current: None }
    }

    pub fn is_busy(&self) -> bool {
        self.current.is_some()
    }

    pub fn set(&mut self, op: PendingOperation) {
        self.current = Some(op);
    }

    pub fn clear(&mut self) {
        self.current = None;
    }
}
