use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use git2::{Repository, Status, StatusOptions};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GitFileStatus {
    Clean,
    Modified,
    Staged,
    Untracked,
    Ignored,
}

#[derive(Clone)]
pub struct GitRepoInfo {
    pub branch: String,
    pub root: PathBuf,
}

pub struct GitStatusManager {
    repo_cache: Arc<RwLock<HashMap<PathBuf, Arc<Repository>>>>,
    repo_root_cache: Arc<RwLock<HashMap<PathBuf, Option<PathBuf>>>>,
    repo_info_cache: Arc<RwLock<HashMap<PathBuf, Option<GitRepoInfo>>>>,
}

impl GitStatusManager {
    pub fn new() -> Self {
        Self {
            repo_cache: Arc::new(RwLock::new(HashMap::new())),
            repo_root_cache: Arc::new(RwLock::new(HashMap::new())),
            repo_info_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn find_git_repo(&self, path: &Path) -> Option<PathBuf> {
        let cache = match self.repo_root_cache.read() {
            Ok(c) => c,
            Err(_) => return None,
        };
        if let Some(cached) = cache.get(path) {
            return cached.clone();
        }
        drop(cache);

        let result = find_git_repo_internal(path);
        
        let mut cache = match self.repo_root_cache.write() {
            Ok(c) => c,
            Err(_) => return result,
        };
        cache.insert(path.to_path_buf(), result.clone());
        result
    }

    pub fn get_repository(&self, repo_path: &Path) -> Option<Arc<Repository>> {
        let cache = match self.repo_cache.read() {
            Ok(c) => c,
            Err(_) => return None,
        };
        if let Some(repo) = cache.get(repo_path) {
            return Some(Arc::clone(repo));
        }
        drop(cache);

        let repo = match Repository::open(repo_path) {
            Ok(r) => Arc::new(r),
            Err(_) => return None,
        };

        let mut cache = match self.repo_cache.write() {
            Ok(c) => c,
            Err(_) => return Some(repo),
        };
        cache.insert(repo_path.to_path_buf(), Arc::clone(&repo));
        Some(repo)
    }

    pub fn batch_get_git_status(&self, entries: &mut [crate::fs::directory::FileEntry], directory: &Path) {
        let repo_path = match self.find_git_repo(directory) {
            Some(p) => p,
            None => return,
        };

        let repo = match self.get_repository(&repo_path) {
            Some(r) => r,
            None => return,
        };

        let mut status_map = HashMap::new();
        if let Ok(statuses) = repo.statuses(Some(StatusOptions::default().include_ignored(true))) {
            for entry in statuses.iter() {
                if let Some(path) = entry.path() {
                    let full_path = repo_path.join(path);
                    let status = entry.status();
                    let git_status = status_to_git_file_status(status);
                    status_map.insert(full_path, git_status);
                }
            }
        }

        for entry in entries.iter_mut() {
            if let Some(status) = status_map.get(&entry.path) {
                entry.git_status = Some(*status);
            }
        }
    }

    pub fn get_repo_info(&self, path: &Path) -> Option<GitRepoInfo> {
        let cache = match self.repo_info_cache.read() {
            Ok(c) => c,
            Err(_) => return None,
        };
        if let Some(cached) = cache.get(path) {
            return cached.as_ref().cloned();
        }
        drop(cache);

        let repo_path = match self.find_git_repo(path) {
            Some(p) => p,
            None => {
                if let Ok(mut cache) = self.repo_info_cache.write() {
                    cache.insert(path.to_path_buf(), None);
                }
                return None;
            }
        };

        let branch = match self.get_repository(&repo_path) {
            Some(repo) => {
                if let Ok(head) = repo.head() {
                    head.shorthand().map(|s| s.to_string())
                } else {
                    None
                }
            }
            None => None,
        };

        let result = branch.map(|b| GitRepoInfo {
            branch: b,
            root: repo_path.clone(),
        });

        if let Ok(mut cache) = self.repo_info_cache.write() {
            cache.insert(path.to_path_buf(), result.clone());
        }
        result
    }

    pub fn clear_cache(&self) {
        if let Ok(mut repo_cache) = self.repo_cache.write() {
            repo_cache.clear();
        }
        if let Ok(mut root_cache) = self.repo_root_cache.write() {
            root_cache.clear();
        }
        if let Ok(mut info_cache) = self.repo_info_cache.write() {
            info_cache.clear();
        }
    }

    pub fn invalidate_path(&self, path: &Path) {
        if let Ok(mut root_cache) = self.repo_root_cache.write() {
            root_cache.remove(path);
        }
        if let Ok(mut info_cache) = self.repo_info_cache.write() {
            info_cache.remove(path);
        }
    }
}

fn find_git_repo_internal(path: &Path) -> Option<PathBuf> {
    let mut current = path.to_path_buf();
    loop {
        let git_dir = current.join(".git");
        if git_dir.exists() {
            return Some(current);
        }
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }
    None
}

fn status_to_git_file_status(status: Status) -> GitFileStatus {
    if status.contains(Status::IGNORED) {
        return GitFileStatus::Ignored;
    }
    if status.contains(Status::INDEX_NEW) || status.contains(Status::INDEX_MODIFIED) {
        return GitFileStatus::Staged;
    }
    if status.contains(Status::WT_MODIFIED) || status.contains(Status::WT_NEW) {
        return GitFileStatus::Modified;
    }
    if status.contains(Status::WT_NEW) {
        return GitFileStatus::Untracked;
    }
    GitFileStatus::Clean
}

pub fn find_git_repo(path: &Path) -> Option<PathBuf> {
    find_git_repo_internal(path)
}

pub fn get_git_status(path: &Path) -> Option<GitFileStatus> {
    if let Some(repo_path) = find_git_repo(path) {
        if let Ok(repo) = Repository::open(&repo_path) {
            if let Ok(status) = repo.status_file(path) {
                return Some(status_to_git_file_status(status));
            }
        }
    }
    None
}

pub fn get_current_branch(repo_path: &Path) -> Option<String> {
    if let Ok(repo) = Repository::open(repo_path) {
        if let Ok(head) = repo.head() {
            if let Some(name) = head.shorthand() {
                return Some(name.to_string());
            }
        }
    }
    None
}

pub fn get_repo_info(path: &Path) -> Option<GitRepoInfo> {
    if let Some(repo_path) = find_git_repo(path) {
        if let Some(branch) = get_current_branch(&repo_path) {
            return Some(GitRepoInfo {
                branch,
                root: repo_path,
            });
        }
    }
    None
}

