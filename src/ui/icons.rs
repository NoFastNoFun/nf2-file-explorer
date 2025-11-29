use egui::{Color32, RichText};
use crate::fs::directory::FileEntry;

pub fn get_file_icon(entry: &FileEntry) -> (&'static str, Color32) {
    if entry.is_dir {
        return ("📁", Color32::from_rgb(100, 150, 255));
    }

    let ext = entry.extension.as_deref().unwrap_or("").to_lowercase();
    
    match ext.as_str() {
        // Images
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" | "ico" | "tiff" | "tif" => {
            ("🖼️", Color32::from_rgb(100, 200, 255))
        }
        // Documents
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "odt" | "ods" | "odp" => {
            ("📄", Color32::from_rgb(255, 100, 100))
        }
        // Text files
        "txt" | "md" | "rtf" | "log" => {
            ("📝", Color32::from_rgb(200, 200, 200))
        }
        // Code files
        "rs" | "py" | "js" | "ts" | "java" | "cpp" | "c" | "h" | "hpp" | "cs" | "go" | "rb" | "php" | "swift" | "kt" => {
            ("💻", Color32::from_rgb(255, 200, 100))
        }
        // Archives
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" => {
            ("📦", Color32::from_rgb(200, 150, 100))
        }
        // Audio
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" => {
            ("🎵", Color32::from_rgb(100, 255, 150))
        }
        // Video
        "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" | "m4v" => {
            ("🎬", Color32::from_rgb(200, 100, 255))
        }
        // Executables
        "exe" | "msi" | "bat" | "cmd" | "sh" | "app" => {
            ("⚙️", Color32::from_rgb(150, 255, 150))
        }
        // Web files
        "html" | "htm" | "css" | "jsx" | "tsx" | "json" | "xml" => {
            ("🌐", Color32::from_rgb(100, 200, 255))
        }
        _ => {
            ("📄", Color32::from_rgb(180, 180, 180))
        }
    }
}

pub fn render_file_icon(entry: &FileEntry) -> RichText {
    let (icon, color) = get_file_icon(entry);
    RichText::new(icon).color(color)
}

