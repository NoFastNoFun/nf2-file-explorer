use std::path::{Path, PathBuf};
use ignore::WalkBuilder;
use std::collections::{HashSet, HashMap};

pub struct GitIgnoreFilter {
    ignored_paths: HashSet<PathBuf>,
    cached_directories: HashMap<PathBuf, HashSet<PathBuf>>,
    current_directory: Option<PathBuf>,
}

impl GitIgnoreFilter {
    pub fn new() -> Self {
        Self {
            ignored_paths: HashSet::new(),
            cached_directories: HashMap::new(),
            current_directory: None,
        }
    }

    pub fn is_ignored(&self, path: &Path) -> bool {
        self.ignored_paths.contains(path)
    }

    pub fn is_cached_for(&self, directory: &Path) -> bool {
        self.current_directory.as_ref().map_or(false, |d| d == directory)
    }

    pub fn update_for_directory(&mut self, root: &Path) {
        if self.is_cached_for(root) {
            return;
        }

        if let Some(cached) = self.cached_directories.get(root) {
            self.ignored_paths = cached.clone();
            self.current_directory = Some(root.to_path_buf());
            return;
        }

        let mut ignored = HashSet::new();
        
        let walker = WalkBuilder::new(root)
            .hidden(false)
            .git_ignore(true)
            .git_exclude(true)
            .max_depth(Some(1))
            .build();
        
        for result in walker {
            match result {
                Ok(entry) => {
                    if entry.path().is_file() && entry.depth() > 0 {
                        if entry.metadata().is_err() {
                            ignored.insert(entry.path().to_path_buf());
                        }
                    }
                }
                Err(_) => {}
            }
        }

        self.cached_directories.insert(root.to_path_buf(), ignored.clone());
        self.ignored_paths = ignored;
        self.current_directory = Some(root.to_path_buf());
    }

    pub fn should_hide(&self, path: &Path, _root: &Path) -> bool {
        self.ignored_paths.contains(path)
    }
}

impl Default for GitIgnoreFilter {
    fn default() -> Self {
        Self::new()
    }
}

