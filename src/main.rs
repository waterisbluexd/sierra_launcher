mod cards;
mod ipc;
mod theme;
mod themer;
use crate::themer::notify::apply_theme;
use cards::wallpaper::WallpaperManager;
use layer_shika::calloop::channel::{self, Event};
use layer_shika::calloop::{TimeoutAction, Timer};
use layer_shika::prelude::*;
use layer_shika::slint_interpreter::{ComponentInstance, Value};
use layer_shika_adapters::AppState;
use slint::ComponentHandle;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;
use tracing::instrument;

const ISLAND: &str = "Island";
const SHOWN_WIDTH: u32 = 420;
const SHOWN_HEIGHT: u32 = 630;
const COMMIT_DEBOUNCE_MS: u64 = 350;

pub enum DaemonMsg {
    ReloadTheme,
    Toggle,
    WallpaperLoaded,
    CommitWallpaper(u64),
}

#[instrument]
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

#[instrument(skip(instance, mgr))]
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
    let _ = instance.set_property(
        "wallpaper-current-index",
        Value::Number(mgr.current_index() as f64),
    );
    let _ = instance.set_property(
        "wallpaper-total-count",
        Value::Number(mgr.total_count() as f64),
    );
}

#[instrument(skip(manager, on_loaded))]
fn kick_loads(
    manager: &Rc<RefCell<WallpaperManager>>,
    on_loaded: impl Fn() + Send + Sync + Clone + 'static,
) {
    manager.borrow().ensure_window_loaded(2, on_loaded);
}

#[instrument(skip(commit_gen, sender))]
fn schedule_commit(commit_gen: &Arc<AtomicU64>, sender: &channel::Sender<DaemonMsg>) {
    let my_gen = commit_gen.fetch_add(1, Ordering::SeqCst) + 1;
    let sender = sender.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(COMMIT_DEBOUNCE_MS));
        let _ = sender.send(DaemonMsg::CommitWallpaper(my_gen));
    });
}

#[instrument]
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
    let commit_gen = Arc::new(AtomicU64::new(0));
    let focused = Rc::new(RefCell::new(false));

    let (sender, rx) = channel::channel::<DaemonMsg>();

    {
        let manager_for_channel = manager.clone();
        let commit_gen_for_channel = commit_gen.clone();
        shell
            .event_loop_handle()
            .insert_source(rx, move |event, _, app_state: &mut AppState| {
                let msg = match event {
                    Event::Msg(m) => m,
                    Event::Closed => return,
                };
                match msg {
                    DaemonMsg::ReloadTheme => {
                        let theme = theme::Theme::load();
                        for surface in app_state.surfaces_by_name_mut(ISLAND) {
                            apply_theme(surface.component_instance(), &theme);
                        }
                        for surface in app_state.all_outputs() {
                            let _ = surface.render_frame_if_dirty();
                            surface.commit_surface();
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
                            surface.commit_surface();
                        }
                    }
                    DaemonMsg::CommitWallpaper(gen_id) => {
                        if commit_gen_for_channel.load(Ordering::SeqCst) == gen_id {
                            manager_for_channel.borrow().set_current_as_wallpaper();
                        }
                    }
                }
            })
            .expect("Failed to insert channel source");
    }

    shell
        .event_loop_handle()
        .insert_source(
            Timer::from_duration(Duration::from_millis(16)),
            move |_deadline, _metadata, app_state: &mut AppState| {
                let focused = focused.clone();
                if !*focused.borrow() {
                    for surface in app_state.surfaces_by_name_mut(ISLAND) {
                        let _ = surface.component_instance().invoke("focus_search", &[]);
                    }
                    *focused.borrow_mut() = true;
                }
                for surface in app_state.all_outputs() {
                    let _ = surface.render_frame_if_dirty();
                    surface.commit_surface();
                }
                TimeoutAction::ToDuration(Duration::from_millis(16))
            },
        )
        .expect("Failed to insert render-pump timer");

    shell
        .event_loop_handle()
        .insert_source(
            Timer::from_duration(Duration::from_millis(1000)),
            move |_deadline, _metadata, app_state: &mut AppState| {
                let time_str = cards::clock::current_time();
                let date_str = cards::clock::current_date();
                for surface in app_state.surfaces_by_name_mut(ISLAND) {
                    let instance = surface.component_instance();
                    let _ = instance
                        .set_property("current-time", Value::String(time_str.clone().into()));
                    let _ = instance
                        .set_property("current-date", Value::String(date_str.clone().into()));
                }
                TimeoutAction::ToDuration(Duration::from_millis(1000))
            },
        )
        .expect("Failed to insert clock timer");

    themer::notify::start_watcher(sender.clone());

    {
        let sender = sender.clone();
        let manager_preload = manager.clone();
        shell
            .event_loop_handle()
            .insert_source(
                Timer::from_duration(Duration::from_millis(150)),
                move |_deadline, _metadata, _app_state: &mut AppState| {
                    let sender_inner = sender.clone();
                    manager_preload.borrow().spawn_full_preload(move || {
                        let _ = sender_inner.send(DaemonMsg::WallpaperLoaded);
                    });
                    TimeoutAction::Drop
                },
            )
            .expect("Failed to insert preload timer");
    }

    {
        let esc_sender = sender.clone();
        let manager_init = manager.clone();
        let commit_gen_inner = commit_gen.clone();
        shell.with_component(ISLAND, move |instance| {
            let theme = theme::Theme::load();
            apply_theme(instance, &theme);

            push_wallpaper_state(instance, &manager_init.borrow());

            // NEW: seed the clock immediately so it's never on defaults
            let _ = instance.set_property(
                "current-time",
                Value::String(cards::clock::current_time().into()),
            );
            let _ = instance.set_property(
                "current-date",
                Value::String(cards::clock::current_date().into()),
            );

            {
                let sender = esc_sender.clone();
                kick_loads(&manager_init, move || {
                    let _ = sender.send(DaemonMsg::WallpaperLoaded);
                });
            }

            let weak_prev = instance.as_weak();
            let manager_prev = manager_init.clone();
            let sender_prev = esc_sender.clone();
            let commit_gen_prev = commit_gen_inner.clone();
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
                schedule_commit(&commit_gen_prev, &sender_prev);
                Value::Void
            });

            let weak_next = instance.as_weak();
            let manager_next = manager_init.clone();
            let sender_next = esc_sender.clone();
            let commit_gen_next = commit_gen_inner.clone();
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
                schedule_commit(&commit_gen_next, &sender_next);
                Value::Void
            });

            let inner_sender = esc_sender.clone();
            let _ = instance.set_callback("request_hide", move |_args: &[Value]| {
                let _ = inner_sender.send(DaemonMsg::Toggle);
                Value::Void
            });

            cards::searchbar::wire_search_callbacks(
                instance,
                |text| {
                    println!("search edited: {text}");
                },
                |text| {
                    println!("search accepted: {text}");
                },
            );
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
