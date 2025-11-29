# NF2's File Explorer

A modern, feature-rich file explorer application built with Rust and egui.

## Features

### Core Functionality
- **File Operations**: Copy, Cut, Paste, Delete with confirmation dialogs
- **Context Menu**: Right-click menu for quick access to file operations
- **Keyboard Shortcuts**: Full keyboard support (Ctrl+C, Ctrl+X, Ctrl+V, Delete, F2, F5, Ctrl+A, Escape, Ctrl+Z)
- **Clipboard Integration**: System clipboard support for copy/cut operations
- **File System Watching**: Auto-refresh when files change externally

### Navigation
- **Breadcrumb Navigation**: Clickable path segments for easy navigation
- **Tree View**: Collapsible directory tree on the left sidebar
- **Recent Folders**: Quick access to recently visited directories
- **Home Page**: Default landing page with special folders, favorites, and recent items

### Visual Features
- **Colored File Icons**: Rich, colorful icons for different file types
- **File Preview**: Preview pane for images and text files
- **Multiple View Modes**: List, Grid, and Details views (settings)
- **Dark/Light Theme**: Theme support (settings)

### Advanced Features
- **Search**: Debounced search with filters (recursive, case-sensitive, file/folder only)
- **Favorites**: Bookmark frequently used directories
- **Settings Persistence**: Preferences saved to disk
- **Undo/Redo**: Operation history with undo support (Ctrl+Z)
- **Properties Panel**: View detailed file information

## Requirements

- Rust 1.70 or later
- Windows (currently optimized for Windows, but can be adapted for other platforms)

## Building

```bash
# Clone the repository
git clone <repository-url>
cd file-explorer

# Build in release mode
cargo build --release

# Run the application
cargo run --release
```

## Usage

### Basic Navigation
- **Single Click** on tree view folders: Expand/collapse directory
- **Double Click** on tree view folders: Navigate to directory
- **Double Click** on file list items: Open folder or view file properties
- **Home Button (🏠)**: Return to home page

### File Operations
- **Copy**: Select files and press Ctrl+C or use toolbar button
- **Cut**: Select files and press Ctrl+X or use toolbar button
- **Paste**: Press Ctrl+V or use toolbar button
- **Delete**: Select files and press Delete or use toolbar button
- **Rename**: Press F2 or right-click and select Rename

### Keyboard Shortcuts
- `Ctrl+C` - Copy selected files
- `Ctrl+X` - Cut selected files
- `Ctrl+V` - Paste files
- `Delete` - Delete selected files
- `F2` - Rename selected file
- `F5` - Refresh current directory
- `Ctrl+A` - Select all files
- `Escape` - Clear selection
- `Ctrl+Z` - Undo last operation

### Context Menu
Right-click on any file or folder to access:
- Copy
- Cut
- Paste
- Delete
- Rename
- Properties
- Add to Favorites (folders only)

## Project Structure

```
file-explorer/
├── src/
│   ├── main.rs           # Application entry point
│   ├── app.rs            # Application state management
│   ├── fs/               # File system operations
│   │   ├── directory.rs  # Directory listing and caching
│   │   ├── operations.rs # File operations (copy, move, delete)
│   │   ├── search.rs     # Search functionality
│   │   ├── watcher.rs    # File system watching
│   │   └── error.rs      # Error types
│   ├── ui/               # User interface components
│   │   ├── file_list.rs  # Main file list view
│   │   ├── tree_view.rs  # Directory tree view
│   │   ├── toolbar.rs    # Top toolbar
│   │   ├── status_bar.rs # Bottom status bar
│   │   ├── search_bar.rs # Search bar
│   │   ├── properties.rs # File properties panel
│   │   ├── preview.rs    # File preview pane
│   │   ├── context_menu.rs # Right-click menu
│   │   ├── dialogs.rs    # Confirmation dialogs
│   │   ├── breadcrumb.rs # Breadcrumb navigation
│   │   ├── icons.rs      # File type icons
│   │   ├── settings.rs   # Settings window
│   │   └── home.rs       # Home page
│   └── utils/            # Utility functions
└── Cargo.toml            # Rust project configuration
```

## Configuration

Settings are stored in:
- **Windows**: `%APPDATA%\file-explorer\settings.toml`

Settings include:
- Show hidden files
- Default view mode
- Theme preference
- Cache TTL
- Recent folders
- Recent files
- Favorites

## Dependencies

- **eframe/egui**: GUI framework
- **tokio**: Async runtime
- **notify**: File system watching
- **arboard**: Clipboard operations
- **serde/toml**: Settings persistence
- **image**: Image preview support
- **regex**: Search functionality
- **thiserror**: Error handling

## License

[Add your license here]

## Contributing

[Add contribution guidelines here]

## Author

NF2

