use std::path::PathBuf;
use std::collections::VecDeque;

pub struct NavigationState {
    pub current_path: PathBuf,
    pub history: VecDeque<PathBuf>,
    pub nav_history_index: usize,
    pub path_input: String,
}

impl NavigationState {
    pub fn new(initial_path: PathBuf) -> Self {
        let mut history = VecDeque::new();
        history.push_back(initial_path.clone());
        Self {
            current_path: initial_path.clone(),
            history,
            nav_history_index: 0,
            path_input: initial_path.to_string_lossy().to_string(),
        }
    }

    pub fn can_go_back(&self) -> bool {
        self.nav_history_index > 0
    }

    pub fn can_go_forward(&self) -> bool {
        self.nav_history_index < self.history.len() - 1
    }

    pub fn update_path(&mut self, path: PathBuf) {
        self.current_path = path.clone();
        self.path_input = self.current_path.to_string_lossy().to_string();
        
        if let Some(last) = self.history.back() {
            if last != &self.current_path {
                self.history.push_back(self.current_path.clone());
                self.nav_history_index = self.history.len() - 1;
            }
        }
    }

    pub fn go_back_index(&mut self) -> Option<PathBuf> {
        if self.can_go_back() {
            self.nav_history_index -= 1;
            self.history.get(self.nav_history_index).cloned()
        } else {
            None
        }
    }

    pub fn go_forward_index(&mut self) -> Option<PathBuf> {
        if self.can_go_forward() {
            self.nav_history_index += 1;
            self.history.get(self.nav_history_index).cloned()
        } else {
            None
        }
    }
}


