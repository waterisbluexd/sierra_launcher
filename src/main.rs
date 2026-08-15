mod cards;
mod ipc;
mod theme;
mod themer;
mod ui;
use cards::wallpaper::WallpaperManager;
use layer_shika::calloop::channel::{self, Event};
use layer_shika::calloop::{TimeoutAction, Timer};
use layer_shika::prelude::*;
use layer_shika::slint_interpreter::Value;
use layer_shika_adapters::AppState;
use slint::ComponentHandle;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

const ISLAND: &str = "Island";
const SHOWN_WIDTH: u32 = 420;
const SHOWN_HEIGHT: u32 = 630;

pub enum DaemonMsg {
    ReloadTheme,
    Toggle,
    WallpaperLoaded,
    CommitWallpaper(u64),
    WeatherUpdate,
}

fn main() -> layer_shika::Result<()> {
    let socket_path = ipc::socket_path();
    if ipc::notify_running_instance(&socket_path) {
        return Ok(());
    }
    let listener = ipc::bind_listener(&socket_path).expect("Failed to bind IPC socket");
    let ui = ui::find_ui_file();
    let mut shell = Shell::from_file(ui.to_str().unwrap())
        .surface(ISLAND)
        .width(SHOWN_WIDTH)
        .height(SHOWN_HEIGHT)
        .anchor(AnchorEdges::empty().with_bottom())
        .keyboard_interactivity(KeyboardInteractivity::Exclusive)
        .layer(Layer::Overlay)
        .exclusive_zone(0)
        .build()?;

    let (sender, rx) = channel::channel::<DaemonMsg>();

    let manager = Rc::new(RefCell::new(WallpaperManager::load({
        let sender = sender.clone();
        move || {
            let _ = sender.send(DaemonMsg::WallpaperLoaded);
        }
    })));

    let commit_gen = Arc::new(AtomicU64::new(0));
    let weather_state = Arc::new(Mutex::new(cards::weather::WeatherState::default()));

    {
        shell
            .event_loop_handle()
            .insert_source(rx, {
                let manager_for_channel = manager.clone();
                let commit_gen_for_channel = commit_gen.clone();
                let weather_state_for_channel = weather_state.clone();
                move |event, _, app_state: &mut AppState| {
                    let msg = match event {
                        Event::Msg(m) => m,
                        Event::Closed => return,
                    };
                    match msg {
                        DaemonMsg::ReloadTheme => {
                            let theme = theme::Theme::load();
                            for surface in app_state.surfaces_by_name_mut(ISLAND) {
                                themer::notify::apply_theme(surface.component_instance(), &theme);
                            }
                            for surface in app_state.all_outputs() {
                                let _ = surface.render_frame_if_dirty();
                                surface.commit_surface();
                            }
                        }
                        DaemonMsg::Toggle => std::process::exit(0),
                        DaemonMsg::WallpaperLoaded => {
                            let mgr = manager_for_channel.borrow();
                            for surface in app_state.surfaces_by_name_mut(ISLAND) {
                                ui::push_wallpaper_state(surface.component_instance(), &mgr);
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
                        DaemonMsg::WeatherUpdate => {
                            let state = weather_state_for_channel.lock().unwrap();
                            for surface in app_state.surfaces_by_name_mut(ISLAND) {
                                let instance = surface.component_instance();
                                for (name, val) in [
                                    ("is-rainy", state.is_rainy),
                                    ("is-cloudy", state.is_cloudy),
                                    ("is-clear", state.is_clear),
                                    ("is-day", state.is_day),
                                ] {
                                    if let Err(e) = instance.set_property(name, Value::Bool(val)) {
                                        eprintln!("[ui] failed to set {name}: {e:?}");
                                    }
                                }
                            }
                            for surface in app_state.all_outputs() {
                                let _ = surface.render_frame_if_dirty();
                                surface.commit_surface();
                            }
                        }
                    }
                }
            })
            .expect("Failed to insert channel source");
    }

    shell
        .event_loop_handle()
        .insert_source(
            Timer::from_duration(Duration::from_millis(1000)),
            move |_deadline, _metadata, app_state: &mut AppState| {
            let time_str = cards::clock::current_time();
            let date_str = cards::clock::current_date();
            let greeting_str = cards::clock::user_greeting();
            for surface in app_state.surfaces_by_name_mut(ISLAND) {
                let instance = surface.component_instance();
                let _ = instance
                    .set_property("current-time", Value::String(time_str.clone().into()));
                let _ = instance
                    .set_property("current-date", Value::String(date_str.clone().into()));
                let _ = instance
                    .set_property("current-greeting", Value::String(greeting_str.clone().into()));
            }
            for surface in app_state.all_outputs() {
                let _ = surface.render_frame_if_dirty();
                surface.commit_surface();
            }
                TimeoutAction::ToDuration(Duration::from_millis(1000))
            },
        )
        .expect("Failed to insert clock timer");

    themer::notify::start_watcher(sender.clone());

    let theme = theme::Theme::load();
    let time_str = cards::clock::current_time();
    let date_str = cards::clock::current_date();
    let greeting_str = cards::clock::user_greeting();

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
            themer::notify::apply_theme(instance, &theme);

            ui::push_wallpaper_state(instance, &manager_init.borrow());

            let _ = instance.set_property(
                "current-time",
                Value::String(time_str.clone().into()),
            );
            let _ = instance.set_property(
                "current-date",
                Value::String(date_str.clone().into()),
            );
            let _ = instance.set_property(
                "current-greeting",
                Value::String(greeting_str.clone().into()),
            );

            {
                let sender_inner = esc_sender.clone();
                ui::kick_loads(&manager_init, move || {
                    let _ = sender_inner.send(DaemonMsg::WallpaperLoaded);
                });
            }

            let weak_prev = instance.as_weak();
            let manager_prev = manager_init.clone();
            let sender_prev = esc_sender.clone();
            let commit_gen_prev = commit_gen_inner.clone();
            let _ = instance.set_callback("request_select_prev", move |_args: &[Value]| {
                manager_prev.borrow_mut().select_prev();
                if let Some(inst) = weak_prev.upgrade() {
                    ui::push_wallpaper_state(&inst, &manager_prev.borrow());
                }
                {
                    let sender_inner = sender_prev.clone();
                    ui::kick_loads(&manager_prev, move || {
                        let _ = sender_inner.send(DaemonMsg::WallpaperLoaded);
                    });
                }
                ui::schedule_commit(&commit_gen_prev, &sender_prev);
                Value::Void
            });

            let weak_next = instance.as_weak();
            let manager_next = manager_init.clone();
            let sender_next = esc_sender.clone();
            let commit_gen_next = commit_gen_inner.clone();
            let _ = instance.set_callback("request_select_next", move |_args: &[Value]| {
                manager_next.borrow_mut().select_next();
                if let Some(inst) = weak_next.upgrade() {
                    ui::push_wallpaper_state(&inst, &manager_next.borrow());
                }
                {
                    let sender_inner = sender_next.clone();
                    ui::kick_loads(&manager_next, move || {
                        let _ = sender_inner.send(DaemonMsg::WallpaperLoaded);
                    });
                }
                ui::schedule_commit(&commit_gen_next, &sender_next);
                Value::Void
            });

            let inner_sender = esc_sender.clone();
            let _ = instance.set_callback("request_hide", move |_args: &[Value]| {
                let _ = inner_sender.send(DaemonMsg::Toggle);
                Value::Void
            });

            cards::searchbar::wire_search_callbacks(
                instance,
                |_text| {},
                |_text| {},
            );
        });
    }

    {
        shell
            .event_loop_handle()
            .insert_source(
                Timer::from_duration(Duration::from_millis(50)),
                move |_deadline, _metadata, app_state: &mut AppState| {
                    for surface in app_state.surfaces_by_name_mut(ISLAND) {
                        let _ = surface.component_instance().invoke("focus_search", &[]);
                    }
                    for surface in app_state.all_outputs() {
                        let _ = surface.render_frame_if_dirty();
                        surface.commit_surface();
                    }
                    TimeoutAction::Drop
                },
            )
            .expect("Failed to insert focus timer");
    }

    {
        let sender = sender.clone();
        let weather_state = weather_state.clone();
        thread::spawn(move || {
            cards::weather::update_weather(&mut weather_state.lock().unwrap());
            let _ = sender.send(DaemonMsg::WeatherUpdate);
            loop {
                thread::sleep(Duration::from_secs(600));
                cards::weather::update_weather(&mut weather_state.lock().unwrap());
                let _ = sender.send(DaemonMsg::WeatherUpdate);
            }
        });
    }

    let sender_ipc = sender.clone();
    thread::spawn(move || {
        ipc::serve(listener, move || {
            let _ = sender_ipc.send(DaemonMsg::Toggle);
        });
    });
    shell.run()?;
    Ok(())
}
