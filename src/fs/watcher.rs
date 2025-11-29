use std::path::PathBuf;
use tokio::sync::mpsc;
use notify::{Watcher, RecommendedWatcher, RecursiveMode, Event, EventKind};
use std::time::{Duration, Instant};

pub struct FileSystemWatcher {
    watcher: Option<RecommendedWatcher>,
    event_tx: mpsc::UnboundedSender<PathBuf>,
    event_rx: mpsc::UnboundedReceiver<PathBuf>,
    last_event: Option<Instant>,
    debounce_duration: Duration,
}

impl FileSystemWatcher {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            watcher: None,
            event_tx: tx,
            event_rx: rx,
            last_event: None,
            debounce_duration: Duration::from_millis(500),
        }
    }

    pub fn watch(&mut self, path: PathBuf) -> Result<(), notify::Error> {
        let event_tx = self.event_tx.clone();
        
        let mut watcher = notify::recommended_watcher(move |result: Result<Event, notify::Error>| {
            if let Ok(event) = result {
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                        for path in event.paths {
                            let _ = event_tx.send(path);
                        }
                    }
                    _ => {}
                }
            }
        })?;

        watcher.watch(&path, RecursiveMode::NonRecursive)?;
        self.watcher = Some(watcher);
        Ok(())
    }

    pub fn unwatch(&mut self) {
        self.watcher = None;
    }

    pub fn check_events(&mut self) -> Option<PathBuf> {
        let mut latest_path = None;
        let mut has_events = false;

        while let Ok(path) = self.event_rx.try_recv() {
            latest_path = Some(path);
            has_events = true;
        }

        if has_events {
            let now = Instant::now();
            if let Some(last) = self.last_event {
                if now.duration_since(last) < self.debounce_duration {
                    return None;
                }
            }
            self.last_event = Some(now);
            latest_path
        } else {
            None
        }
    }
}

impl Default for FileSystemWatcher {
    fn default() -> Self {
        Self::new()
    }
}

