use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use chrono::{DateTime, Local};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub modified: SystemTime,
    pub extension: Option<String>,
}

impl FileEntry {
    pub fn format_size(&self) -> String {
        if self.is_dir {
            return String::new();
        }
        format_bytes(self.size)
    }

    pub fn format_modified(&self) -> String {
        let dt: DateTime<Local> = self.modified.into();
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    pub fn file_type(&self) -> String {
        if self.is_dir {
            "Folder".to_string()
        } else if let Some(ext) = &self.extension {
            ext.to_uppercase()
        } else {
            "File".to_string()
        }
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

#[derive(Clone)]
struct CacheEntry {
    entries: Vec<FileEntry>,
    timestamp: SystemTime,
}

pub struct DirectoryCache {
    cache: Arc<RwLock<HashMap<PathBuf, CacheEntry>>>,
    ttl: Duration,
}

impl DirectoryCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    pub async fn get(&self, path: &Path) -> Option<Vec<FileEntry>> {
        let cache = self.cache.read().await;
        if let Some(entry) = cache.get(path) {
            if entry.timestamp.elapsed().unwrap_or(Duration::MAX) < self.ttl {
                return Some(entry.entries.clone());
            }
        }
        None
    }

    pub async fn set(&self, path: PathBuf, entries: Vec<FileEntry>) {
        let mut cache = self.cache.write().await;
        cache.insert(
            path,
            CacheEntry {
                entries,
                timestamp: SystemTime::now(),
            },
        );
    }

    pub async fn invalidate(&self, path: &Path) {
        let mut cache = self.cache.write().await;
        cache.remove(path);
    }

    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
}

pub async fn list_directory(path: &Path) -> Result<Vec<FileEntry>, std::io::Error> {
    let mut entries = Vec::new();

    let mut dir_entries = tokio::fs::read_dir(path).await?;

    while let Some(entry) = dir_entries.next_entry().await? {
        let entry_path = entry.path();
        let metadata = entry.metadata().await?;

        let name = entry_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let extension = entry_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_string());

        let file_entry = FileEntry {
            name,
            path: entry_path.clone(),
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            extension,
        };

        entries.push(file_entry);
    }

    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });

    Ok(entries)
}

