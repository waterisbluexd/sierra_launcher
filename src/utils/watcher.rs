use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};

pub struct ColorWatcher {
    _watcher: RecommendedWatcher,
    receiver: Receiver<notify::Result<Event>>,
}

impl ColorWatcher {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (tx, rx) = channel();
        
        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default(),
        )?;

        let home = std::env::var("HOME")?;
        let wal_path = PathBuf::from(home).join(".cache/wal");
        
        watcher.watch(&wal_path, RecursiveMode::NonRecursive)?;

        Ok(Self {
            _watcher: watcher,
            receiver: rx,
        })
    }

    pub fn check_for_changes(&self) -> bool {
        while let Ok(Ok(event)) = self.receiver.try_recv() {
            if let notify::EventKind::Modify(_) | notify::EventKind::Create(_) = event.kind {
                for path in event.paths {
                    if path.file_name().and_then(|n| n.to_str()) == Some("colors.json") {
                        return true;
                    }
                }
            }
        }
        false
    }
}