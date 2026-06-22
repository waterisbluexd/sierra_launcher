use chrono::Local;

pub fn current_time() -> String {
    Local::now().format("%H:%M").to_string()
}

pub fn current_date() -> String {
    Local::now().format("%d %b, %A").to_string()
}
