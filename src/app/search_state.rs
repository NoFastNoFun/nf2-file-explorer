use crate::fs::directory::FileEntry;
use crate::fs::search::SearchOptions;
use std::time::Instant;

pub struct SearchState {
    pub options: SearchOptions,
    pub results: Vec<FileEntry>,
    pub is_searching: bool,
    pub debounce_timer: Option<Instant>,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            options: SearchOptions::default(),
            results: Vec::new(),
            is_searching: false,
            debounce_timer: None,
        }
    }

    pub fn clear(&mut self) {
        self.results.clear();
        self.is_searching = false;
        self.debounce_timer = None;
    }
}


