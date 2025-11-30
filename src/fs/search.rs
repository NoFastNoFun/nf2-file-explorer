use std::path::{Path, PathBuf};
use crate::fs::directory::FileEntry;

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub query: String,
    pub recursive: bool,
    pub match_case: bool,
    pub file_only: bool,
    pub dir_only: bool,
    pub global: bool,
    pub use_regex: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            query: String::new(),
            recursive: false,
            match_case: false,
            file_only: false,
            dir_only: false,
            global: false,
            use_regex: false,
        }
    }
}

pub async fn search_directory(
    root: &Path,
    options: &SearchOptions,
) -> Result<Vec<FileEntry>, std::io::Error> {
    let mut results = Vec::new();
    
    let regex_pattern = if options.use_regex {
        let pattern = if options.match_case {
            options.query.clone()
        } else {
            format!("(?i){}", options.query)
        };
        match regex::Regex::new(&pattern) {
            Ok(re) => Some(re),
            Err(_) => return Ok(Vec::new()),
        }
    } else {
        None
    };
    
    let query = if options.match_case {
        options.query.clone()
    } else {
        options.query.to_lowercase()
    };

    if options.global {
        let mut all_results = Vec::new();
        for letter in b'A'..=b'Z' {
            let drive = format!("{}:\\", letter as char);
            let drive_path = PathBuf::from(&drive);
            if drive_path.exists() {
                let walker = walkdir::WalkDir::new(&drive_path);
                for entry in walker {
                    let entry = match entry {
                        Ok(e) => e,
                        Err(_) => continue,
                    };
                    let path = entry.path();
                    let metadata = match entry.metadata() {
                        Ok(m) => m,
                        Err(_) => continue,
                    };

                    if metadata.is_dir() && options.file_only {
                        continue;
                    }
                    if metadata.is_file() && options.dir_only {
                        continue;
                    }

                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");

                    let search_name = if options.match_case {
                        name.to_string()
                    } else {
                        name.to_lowercase()
                    };

                    let matches = if let Some(ref re) = regex_pattern {
                        re.is_match(&search_name)
                    } else {
                        search_name.contains(&query)
                    };

                    if matches {
                        let file_entry = FileEntry {
                            name: name.to_string(),
                            path: path.to_path_buf(),
                            is_dir: metadata.is_dir(),
                            size: metadata.len(),
                            modified: metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                            extension: path
                                .extension()
                                .and_then(|e| e.to_str())
                                .map(|s| s.to_string()),
                            git_status: None,
                            is_symlink: false,
                            is_hardlink: false,
                            link_target: None,
                        };
                        all_results.push(file_entry);
                    }
                }
            }
        }
        return Ok(all_results);
    }

    if options.recursive {
        let walker = walkdir::WalkDir::new(root);
        for entry in walker {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry.metadata()?;

            if metadata.is_dir() && options.file_only {
                continue;
            }
            if metadata.is_file() && options.dir_only {
                continue;
            }

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            let search_name = if options.match_case {
                name.to_string()
            } else {
                name.to_lowercase()
            };

            let matches = if let Some(ref re) = regex_pattern {
                re.is_match(&search_name)
            } else {
                search_name.contains(&query)
            };

            if matches {
                let file_entry = FileEntry {
                    name: name.to_string(),
                    path: path.to_path_buf(),
                    is_dir: metadata.is_dir(),
                    size: metadata.len(),
                    modified: metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                    extension: path
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|s| s.to_string()),
                    git_status: None,
                    is_symlink: false,
                    is_hardlink: false,
                    link_target: None,
                };
                results.push(file_entry);
            }
        }
    } else {
        let entries = crate::fs::directory::list_directory(root, None, false).await?;
        for entry in entries {
            let search_name = if options.match_case {
                entry.name.clone()
            } else {
                entry.name.to_lowercase()
            };

            let matches = if let Some(ref re) = regex_pattern {
                re.is_match(&search_name)
            } else {
                search_name.contains(&query)
            };

            if matches {
                if options.file_only && entry.is_dir {
                    continue;
                }
                if options.dir_only && !entry.is_dir {
                    continue;
                }
                results.push(entry);
            }
        }
    }

    Ok(results)
}

