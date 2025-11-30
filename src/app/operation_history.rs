use std::collections::VecDeque;
use std::path::PathBuf;

#[derive(Clone)]
pub enum OperationType {
    Copy { src: PathBuf, dst: PathBuf },
    Move { src: PathBuf, dst: PathBuf },
    Delete { path: PathBuf, was_dir: bool, trash_path: Option<PathBuf> },
    Rename { old_path: PathBuf, new_path: PathBuf },
}

#[derive(Clone)]
pub struct OperationRecord {
    pub op_type: OperationType,
    pub timestamp: std::time::SystemTime,
}

pub struct OperationHistory {
    pub history: VecDeque<OperationRecord>,
    pub undo_index: isize,
    pub redo_stack: VecDeque<OperationRecord>,
}

impl OperationHistory {
    pub fn new() -> Self {
        Self {
            history: VecDeque::new(),
            undo_index: -1,
            redo_stack: VecDeque::new(),
        }
    }

    pub fn add(&mut self, record: OperationRecord) {
        while self.undo_index >= 0 && (self.undo_index as usize) < self.history.len() {
            self.history.pop_back();
        }
        self.redo_stack.clear();
        self.history.push_back(record);
        self.undo_index = self.history.len() as isize - 1;
        
        if self.history.len() > 50 {
            self.history.pop_front();
            self.undo_index -= 1;
        }
    }

    pub fn can_undo(&self) -> bool {
        self.undo_index >= 0 && (self.undo_index as usize) < self.history.len()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn get_undo_record(&self) -> Option<&OperationRecord> {
        if self.can_undo() {
            self.history.get(self.undo_index as usize)
        } else {
            None
        }
    }

    pub fn undo(&mut self) -> Option<OperationRecord> {
        if let Some(record) = self.get_undo_record().cloned() {
            self.redo_stack.push_back(record.clone());
            self.undo_index -= 1;
            Some(record)
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<OperationRecord> {
        if let Some(record) = self.redo_stack.pop_back() {
            self.undo_index += 1;
            if (self.undo_index as usize) >= self.history.len() {
                self.history.push_back(record.clone());
            } else {
                self.history[self.undo_index as usize] = record.clone();
            }
            Some(record)
        } else {
            None
        }
    }
}


