mod ipc;
mod theme;
mod themer;

use crate::themer::notify::apply_theme;
use layer_shika::prelude::*;
use layer_shika::slint_interpreter::Value;
use layer_shika_adapters::AppState;
use std::cell::Cell;
use std::rc::Rc;
use std::thread;

const ISLAND: &str = "Island";
const SHOWN_WIDTH: u32 = 486;
const SHOWN_HEIGHT: u32 = 714;

pub enum DaemonMsg {
    ReloadTheme,
    ToggleVisibility,
}

fn main() -> layer_shika::Result<()> {
    let socket_path = ipc::socket_path();
    if ipc::notify_running_instance(&socket_path) {
        return Ok(());
    }

    let listener = ipc::bind_listener(&socket_path).expect("Failed to bind IPC");
    let shown = Rc::new(Cell::new(false));

    let mut shell = Shell::from_file("ui/main_card.slint")
        .surface(ISLAND)
        .width(SHOWN_WIDTH)
        .height(SHOWN_HEIGHT)
        .anchor(AnchorEdges::empty().with_bottom())
        .exclusive_zone(0)
        .margin(2)
        .build()?;

    let control = shell.control();
    let loop_handle = shell.event_loop_handle();

    let (_token, sender) = {
        let control = control.clone();
        let shown = shown.clone();

        loop_handle.add_channel::<DaemonMsg, _>(move |msg, app_state: &mut AppState| match msg {
            DaemonMsg::ReloadTheme => {
                let theme = theme::Theme::load();
                for surface in app_state.surfaces_by_name_mut(ISLAND) {
                    // Call apply_theme directly
                    apply_theme(surface.component_instance(), &theme);
                }
                for surface in app_state.all_outputs() {
                    let _ = surface.render_frame_if_dirty();
                }
            }
            DaemonMsg::ToggleVisibility => {
                if shown.get() {
                    hide(&control);
                    shown.set(false);
                } else {
                    show(&control);
                    shown.set(true);
                }
            }
        })?
    };

    {
        let slint_sender = sender.clone();
        shell.with_component(ISLAND, move |instance| {
            let theme = theme::Theme::load();
            // Call apply_theme directly
            apply_theme(instance, &theme);

            let inner_sender = slint_sender.clone();

            let _ = instance.set_callback("request_hide", move |_args: &[Value]| {
                let _ = inner_sender.send(DaemonMsg::ToggleVisibility);
                Value::Void
            });
        });
    }

    thread::spawn(move || {
        ipc::serve(listener, move || {
            let _ = sender.send(DaemonMsg::ToggleVisibility);
        });
    });

    shell.run()?;
    Ok(())
}

fn show(control: &ShellControl) {
    let _ = control
        .surface(ISLAND)
        .configure()
        .size(SHOWN_WIDTH, SHOWN_HEIGHT)
        .keyboard_interactivity(KeyboardInteractivity::Exclusive)
        .apply();
}

fn hide(control: &ShellControl) {
    let _ = control
        .surface(ISLAND)
        .configure()
        .size(1, 1)
        .keyboard_interactivity(KeyboardInteractivity::None)
        .apply();
}
