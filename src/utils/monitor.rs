//! Clipboard monitoring using simple polling with pause/resume support.

use super::data;
use super::item::ClipboardContent;
use arboard::Clipboard;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use tracing::{debug, error, info};

// Global pause flag
static MONITORING_PAUSED: AtomicBool = AtomicBool::new(false);

/// Pause clipboard monitoring (used when copying from history)
pub fn pause_monitoring() {
    MONITORING_PAUSED.store(true, Ordering::Relaxed);
    debug!("Clipboard monitoring paused");
}

/// Resume clipboard monitoring
pub fn resume_monitoring() {
    MONITORING_PAUSED.store(false, Ordering::Relaxed);
    debug!("Clipboard monitoring resumed");
}

/// Start monitoring clipboard changes in a background thread.
pub fn start_monitor() -> Arc<AtomicBool> {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    thread::spawn(move || {
        info!("Starting clipboard monitor (polling mode)");
        
        let mut last_text = String::new();
        
        loop {
            if !running_clone.load(Ordering::Relaxed) {
                info!("Clipboard monitor stopped");
                break;
            }
            
            // Poll clipboard every 500ms
            thread::sleep(Duration::from_millis(500));
            
            // Skip if monitoring is paused
            if MONITORING_PAUSED.load(Ordering::Relaxed) {
                continue;
            }
            
            // Try to read clipboard
            match Clipboard::new() {
                Ok(mut clipboard) => {
                    if let Ok(text) = clipboard.get_text() {
                        if !text.is_empty() && text != last_text {
                            debug!("New clipboard content detected: {} chars", text.len());
                            data::add_item(ClipboardContent::Text(text.clone()));
                            last_text = text;
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to create clipboard: {}", e);
                    thread::sleep(Duration::from_secs(1));
                }
            }
        }
    });

    running
}