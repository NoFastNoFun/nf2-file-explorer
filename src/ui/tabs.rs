use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    pub id: String,
    pub path: PathBuf,
    pub title: String,
}

impl Tab {
    pub fn new(path: PathBuf) -> Self {
        let title = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        
        let id = format!("tab_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        
        Self {
            id,
            path,
            title,
        }
    }

    pub fn update_title(&mut self) {
        self.title = self.path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.path.to_string_lossy().to_string());
    }
}

pub fn save_tabs(tabs: &[Tab]) -> Result<(), Box<dyn std::error::Error>> {
    let tabs_path = tabs_path();
    if let Some(parent) = tabs_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let paths: Vec<String> = tabs.iter().map(|t| t.path.to_string_lossy().to_string()).collect();
    let content = toml::to_string(&paths)?;
    fs::write(&tabs_path, content)?;
    Ok(())
}

pub fn load_tabs() -> Vec<PathBuf> {
    let tabs_path = tabs_path();
    if let Ok(content) = fs::read_to_string(&tabs_path) {
        if let Ok(paths) = toml::from_str::<Vec<String>>(&content) {
            return paths.into_iter().map(PathBuf::from).collect();
        }
    }
    Vec::new()
}

fn tabs_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("file-explorer");
    path.push("tabs.toml");
    path
}

