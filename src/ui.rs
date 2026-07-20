use layer_shika::calloop::channel;
use layer_shika::slint_interpreter::{ComponentInstance, Value};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use crate::cards::wallpaper::WallpaperManager;
use crate::DaemonMsg;

const COMMIT_DEBOUNCE_MS: u64 = 350;

pub fn find_ui_file() -> std::path::PathBuf {
    let dev = std::path::PathBuf::from("ui/main_card.slint");
    if dev.exists() {
        return dev;
    }
    if let Ok(data_home) = std::env::var("XDG_DATA_HOME") {
        let p = std::path::PathBuf::from(data_home).join("sierra_launcher/ui/main_card.slint");
        if p.exists() {
            return p;
        }
    } else if let Ok(home) = std::env::var("HOME") {
        let p = std::path::PathBuf::from(home).join(".local/share/sierra_launcher/ui/main_card.slint");
        if p.exists() {
            return p;
        }
    }
    let p = std::path::PathBuf::from("/usr/local/share/sierra_launcher/ui/main_card.slint");
    if p.exists() {
        return p;
    }
    std::path::PathBuf::from("/usr/share/sierra_launcher/ui/main_card.slint")
}

pub fn push_wallpaper_state(instance: &ComponentInstance, mgr: &WallpaperManager) {
    let _ = instance.set_property("wallpaper-image", Value::Image(mgr.current_image()));
    let _ = instance.set_property(
        "wallpaper-image-blurred",
        Value::Image(mgr.current_image_blurred()),
    );
    let _ = instance.set_property("wallpaper-prev-image", Value::Image(mgr.prev_image()));
    let _ = instance.set_property("wallpaper-next-image", Value::Image(mgr.next_image()));
    let _ = instance.set_property(
        "wallpaper-prev-prev-image",
        Value::Image(mgr.prev_prev_image()),
    );
    let _ = instance.set_property(
        "wallpaper-next-next-image",
        Value::Image(mgr.next_next_image()),
    );
    let _ = instance.set_property("can-select-prev", Value::Bool(mgr.can_select_prev()));
    let _ = instance.set_property("can-select-next", Value::Bool(mgr.can_select_next()));
    let _ = instance.set_property(
        "wallpaper-current-index",
        Value::Number(mgr.current_index() as f64),
    );
    let _ = instance.set_property(
        "wallpaper-total-count",
        Value::Number(mgr.total_count() as f64),
    );
}

pub fn kick_loads(
    manager: &std::rc::Rc<std::cell::RefCell<WallpaperManager>>,
    on_loaded: impl Fn() + Send + Sync + Clone + 'static,
) {
    manager.borrow().ensure_window_loaded(2, on_loaded);
}

pub fn schedule_commit(commit_gen: &Arc<AtomicU64>, sender: &channel::Sender<DaemonMsg>) {
    let my_gen = commit_gen.fetch_add(1, Ordering::SeqCst) + 1;
    let sender = sender.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(COMMIT_DEBOUNCE_MS));
        let _ = sender.send(DaemonMsg::CommitWallpaper(my_gen));
    });
}