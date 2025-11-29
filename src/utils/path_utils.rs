use std::path::{Path, PathBuf};

pub fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::Prefix(prefix) => {
                normalized.push(prefix.as_os_str());
            }
            std::path::Component::RootDir => {
                normalized.push("/");
            }
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::Normal(name) => {
                normalized.push(name);
            }
        }
    }
    normalized
}

pub fn get_parent_path(path: &Path) -> Option<PathBuf> {
    path.parent().map(|p| p.to_path_buf())
}

pub fn format_path_display(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

