use layer_shika::slint_interpreter::{ComponentInstance, Value};

pub fn wire_search_callbacks(
    instance: &ComponentInstance,
    on_edited: impl Fn(String) + 'static,
    on_accepted: impl Fn(String) + 'static,
) {
    let _ = instance.set_callback("search_edited", move |args: &[Value]| {
        if let Some(Value::String(s)) = args.first() {
            on_edited(s.to_string());
        }
        Value::Void
    });

    let _ = instance.set_callback("search_accepted", move |args: &[Value]| {
        if let Some(Value::String(s)) = args.first() {
            on_accepted(s.to_string());
        }
        Value::Void
    });
}
