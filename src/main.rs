mod cards;
mod ipc;
mod theme;
mod themer;
use crate::themer::notify::apply_theme;
use cards::wallpaper::WallpaperManager;
use layer_shika::prelude::*;
use layer_shika::slint_interpreter::{ComponentInstance, Value};
use layer_shika_adapters::AppState;
use slint::ComponentHandle;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::thread;

const ISLAND: &str = "Island";
const SHOWN_WIDTH: u32 = 420;
const SHOWN_HEIGHT: u32 = 630;

pub enum DaemonMsg {
    ReloadTheme,
    Toggle,
    WallpaperLoaded,
}

fn find_ui_file() -> PathBuf {
    let dev = PathBuf::from("ui/main_card.slint");
    if dev.exists() {
        return dev;
    }
    if let Ok(data_home) = std::env::var("XDG_DATA_HOME") {
        let p = PathBuf::from(data_home).join("sierra_launcher/ui/main_card.slint");
        if p.exists() {
            return p;
        }
    } else if let Ok(home) = std::env::var("HOME") {
        let p = PathBuf::from(home).join(".local/share/sierra_launcher/ui/main_card.slint");
        if p.exists() {
            return p;
        }
    }
    let p = PathBuf::from("/usr/local/share/sierra_launcher/ui/main_card.slint");
    if p.exists() {
        return p;
    }
    PathBuf::from("/usr/share/sierra_launcher/ui/main_card.slint")
}

fn push_wallpaper_state(instance: &ComponentInstance, mgr: &WallpaperManager) {
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
}

fn kick_loads(
    manager: &Rc<RefCell<WallpaperManager>>,
    on_loaded: impl Fn() + Send + Sync + Clone + 'static,
) {
    manager.borrow().ensure_window_loaded(2, on_loaded);
}

fn main() -> layer_shika::Result<()> {
    let socket_path = ipc::socket_path();
    if ipc::notify_running_instance(&socket_path) {
        return Ok(());
    }
    let listener = ipc::bind_listener(&socket_path).expect("Failed to bind IPC socket");
    let ui = find_ui_file();
    let mut shell = Shell::from_file(ui.to_str().unwrap())
        .surface(ISLAND)
        .width(SHOWN_WIDTH)
        .height(SHOWN_HEIGHT)
        .anchor(AnchorEdges::empty().with_bottom())
        .keyboard_interactivity(KeyboardInteractivity::Exclusive)
        .layer(Layer::Overlay)
        .exclusive_zone(0)
        .build()?;

    let manager = Rc::new(RefCell::new(WallpaperManager::load()));

    let loop_handle = shell.event_loop_handle();
    let (_token, sender) = {
        let manager_for_channel = manager.clone();
        loop_handle.add_channel::<DaemonMsg, _>(move |msg, app_state: &mut AppState| match msg {
            DaemonMsg::ReloadTheme => {
                let theme = theme::Theme::load();
                for surface in app_state.surfaces_by_name_mut(ISLAND) {
                    apply_theme(surface.component_instance(), &theme);
                }
                for surface in app_state.all_outputs() {
                    let _ = surface.render_frame_if_dirty();
                }
            }
            DaemonMsg::Toggle => {
                std::process::exit(0);
            }
            DaemonMsg::WallpaperLoaded => {
                let mgr = manager_for_channel.borrow();
                for surface in app_state.surfaces_by_name_mut(ISLAND) {
                    push_wallpaper_state(surface.component_instance(), &mgr);
                }
                for surface in app_state.all_outputs() {
                    let _ = surface.render_frame_if_dirty();
                }
            }
        })?
    };

    {
        let esc_sender = sender.clone();
        let manager_init = manager.clone();
        shell.with_component(ISLAND, move |instance| {
            let theme = theme::Theme::load();
            apply_theme(instance, &theme);

            push_wallpaper_state(instance, &manager_init.borrow());
            {
                let sender = esc_sender.clone();
                kick_loads(&manager_init, move || {
                    let _ = sender.send(DaemonMsg::WallpaperLoaded);
                });
            }

            let weak_prev = instance.as_weak();
            let manager_prev = manager_init.clone();
            let sender_prev = esc_sender.clone();
            let _ = instance.set_callback("request_select_prev", move |_args: &[Value]| {
                manager_prev.borrow_mut().select_prev();
                if let Some(inst) = weak_prev.upgrade() {
                    push_wallpaper_state(&inst, &manager_prev.borrow());
                }
                {
                    let sender = sender_prev.clone();
                    kick_loads(&manager_prev, move || {
                        let _ = sender.send(DaemonMsg::WallpaperLoaded);
                    });
                }
                Value::Void
            });

            let weak_next = instance.as_weak();
            let manager_next = manager_init.clone();
            let sender_next = esc_sender.clone();
            let _ = instance.set_callback("request_select_next", move |_args: &[Value]| {
                manager_next.borrow_mut().select_next();
                if let Some(inst) = weak_next.upgrade() {
                    push_wallpaper_state(&inst, &manager_next.borrow());
                }
                {
                    let sender = sender_next.clone();
                    kick_loads(&manager_next, move || {
                        let _ = sender.send(DaemonMsg::WallpaperLoaded);
                    });
                }
                Value::Void
            });

            let inner_sender = esc_sender.clone();
            let _ = instance.set_callback("request_hide", move |_args: &[Value]| {
                let _ = inner_sender.send(DaemonMsg::Toggle);
                Value::Void
            });
        });
    }

    thread::spawn(move || {
        ipc::serve(listener, move || {
            let _ = sender.send(DaemonMsg::Toggle);
        });
    });
    shell.run()?;
    Ok(())
}
