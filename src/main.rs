mod ipc;
mod theme;
mod themer;

use crate::themer::notify::apply_theme;
use layer_shika::prelude::*;
use layer_shika::slint_interpreter::Value;
use layer_shika_adapters::AppState;
use std::path::PathBuf;
use std::thread;

const ISLAND: &str = "Island";
const SHOWN_WIDTH: u32 = 486;
const SHOWN_HEIGHT: u32 = 714;

pub enum DaemonMsg {
    ReloadTheme,
    Toggle,
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

    let loop_handle = shell.event_loop_handle();

    let (_token, sender) = {
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
        })?
    };

    {
        let esc_sender = sender.clone();
        shell.with_component(ISLAND, move |instance| {
            let theme = theme::Theme::load();
            apply_theme(instance, &theme);

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
